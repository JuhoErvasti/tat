use std::num::IntErrorKind;

use crossterm::event::KeyCode;
use ratatui::{layout::{Position, Rect}, Frame};

/// Result of completing the input dialog
#[derive(Debug, PartialEq)]
pub enum TatNumberInputResult {
    RejectedKey,
    AcceptedKey,
    Close,
    Accept(i64),
}

/// A widget for getting a number value from the user. Number must be a positive integer.
#[derive(PartialEq, Debug)]
pub struct TatNumberInput {
    string: String,
    cursor_pos: u16,
}

impl TatNumberInput {
    /// Constructs a new widget.
    pub fn new() -> Self {
        Self {
            string: "".to_string(),
            cursor_pos: 0,
        }
    }

    /// Renders the current state of the widget.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(&self.string, area);
        frame.set_cursor_position(Position {
            x: area.x + self.cursor_pos as u16,
            y: area.y,
        });
    }

    /// Handles the incoming key code, rejecting any unaccepted keys
    pub fn key_press(&mut self, key: KeyCode, ctrl_down: bool) -> TatNumberInputResult {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => TatNumberInputResult::Close,
            KeyCode::Home => {
                self.cursor_pos = 0;

                TatNumberInputResult::AcceptedKey
            },
            KeyCode::Left if ctrl_down => {
                self.cursor_pos = 0;

                TatNumberInputResult::AcceptedKey
            },
            KeyCode::End => {
                self.cursor_pos = self.string.len() as u16;

                TatNumberInputResult::AcceptedKey
            },
            KeyCode::Right if ctrl_down => {
                self.cursor_pos = self.string.len() as u16;

                TatNumberInputResult::AcceptedKey
            },
            KeyCode::Backspace => {
                self.handle_backspace();

                TatNumberInputResult::AcceptedKey
            },
            KeyCode::Delete => {
                self.handle_delete();

                TatNumberInputResult::AcceptedKey
            },
            KeyCode::Left => {
                self.handle_left();

                TatNumberInputResult::AcceptedKey
            },
            KeyCode::Right => {
                self.handle_right();

                TatNumberInputResult::AcceptedKey
            },
            KeyCode::Enter => {
                let value = match self.string.parse::<i64>() {
                    Ok(value) => value,
                    Err(e) => {
                        match e.kind() {
                            IntErrorKind::Empty => return TatNumberInputResult::Close,
                            IntErrorKind::PosOverflow => i64::MAX,
                            // should never happen due to filtering of acceptable chars
                            _ => panic!(),
                        }
                    },
                };

                TatNumberInputResult::Accept(value)
            }
            KeyCode::Char('0'..='9') => {
                let char = match key {
                    KeyCode::Char(ch) => ch,
                    _ => panic!(),
                };

                self.handle_char(char);

                TatNumberInputResult::AcceptedKey
            },
            _ => TatNumberInputResult::RejectedKey,
        }
    }

    /// Handles when user wants to move the cursor to the left
    fn handle_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    /// Handles when user wants to move the cursor to the right
    fn handle_right(&mut self) {
        if self.cursor_pos < self.string.len() as u16 {
            self.cursor_pos += 1;
        }
    }

    /// Handles when user wants to delete the number before the cursor
    fn handle_backspace(&mut self) {
        if self.string.is_empty() || self.cursor_pos == 0 {
            return;
        }

        self.string.remove(self.cursor_pos as usize - 1);
        self.cursor_pos -= 1;
    }

    /// Handles when user wants to delete the number after the cursor
    fn handle_delete(&mut self) {
        if self.string.is_empty() || self.cursor_pos == self.string.len() as u16 {
            return;
        }

        self.string.remove(self.cursor_pos as usize);
    }

    /// Handles when user has typed in a character in 0..9
    fn handle_char(&mut self, ch: char) {
        if self.cursor_pos == 0 && ch == '0' {
            return;
        }

        if self.string.len() >= u16::MAX as usize {
            return;
        }

        if self.cursor_pos == self.string.len() as u16 {
            self.string.push(ch);
        } else {
            self.string.insert(self.cursor_pos as usize, ch);
        }
        self.cursor_pos += 1;
    }

    /// Resets the widget
    #[allow(dead_code)]
    fn reset(&mut self) {
        self.string.clear();
        self.cursor_pos = 0;
    }
}

#[cfg(test)]
mod test {
    #[allow(unused)]
    use super::*;

    #[test]
    fn test_new() {
        let ni = TatNumberInput::new();
        assert_eq!(ni.cursor_pos, 0);
        assert!(ni.string.is_empty());
    }

    #[test]
    fn test_key_press_chars() {
        let mut ni = TatNumberInput::new();

        for _char in 0..u8::MAX {
            let char = _char as char;

            if char == 'q' || (char >= '0' && char <= '9') {
                continue
            }

            let res = ni.key_press(KeyCode::Char(char), false);
            assert_eq!(res, TatNumberInputResult::RejectedKey);
        }

        {
            let res = ni.key_press(KeyCode::Char('q'), false);
            assert_eq!(res, TatNumberInputResult::Close);
            ni.reset();
        }

        {
            let res = ni.key_press(KeyCode::Char('0'), false);
            assert_eq!(res, TatNumberInputResult::AcceptedKey);
            assert!(ni.string.is_empty()); // should empty because starting number with 0 not
            // allowed
            ni.reset();
        }

        for char in '1'..='9' {
            let res = ni.key_press(KeyCode::Char(char), false);
            assert_eq!(res, TatNumberInputResult::AcceptedKey);
            assert_eq!(ni.string, char.to_string());
            ni.reset();
        }
    }

    #[test]
    fn test_handle_char() {
        let mut ni = TatNumberInput::new();

        ni.handle_char('0');

        // should empty because starting number with 0 not allowed
        assert!(ni.string.is_empty());
        ni.reset();
        ni.handle_char('1');
        ni.handle_char('0');
        ni.handle_char('2');
        assert_eq!(ni.string, "102");
        ni.reset();

        ni.string = "1234".to_string();
        ni.cursor_pos = 2;

        ni.handle_char('0');
        assert_eq!(ni.string, "12034");
        ni.reset();
    }

    #[test]
    fn test_handle_left() {
        let mut ni = TatNumberInput::new();

        ni.handle_left();
        assert_eq!(ni.cursor_pos, 0);

        ni.cursor_pos = 5;

        ni.handle_left();
        assert_eq!(ni.cursor_pos, 4);
    }

    #[test]
    fn test_handle_right() {
        let mut ni =  TatNumberInput::new();
        ni.string = "12".to_string();
        ni.handle_right();
        assert_eq!(ni.cursor_pos, 1);
        ni.handle_right();
        assert_eq!(ni.cursor_pos, 2);
        ni.handle_right();
        assert_eq!(ni.cursor_pos, 2);
    }

    #[test]
    fn test_handle_backspace() {
        let mut ni =  TatNumberInput::new();
        ni.handle_backspace();
        assert!(ni.string.is_empty());

        ni.string = "123456".to_string();
        ni.handle_backspace();
        assert_eq!(ni.string, "123456");

        ni.cursor_pos = 6;
        ni.handle_backspace();
        assert_eq!(ni.string, "12345");
        assert_eq!(ni.cursor_pos, 5);

        ni.cursor_pos = 3;
        ni.handle_backspace();
        assert_eq!(ni.string, "1245");
        assert_eq!(ni.cursor_pos, 2);

        ni.handle_backspace();
        assert_eq!(ni.string, "145");
        assert_eq!(ni.cursor_pos, 1);

        ni.handle_backspace();
        assert_eq!(ni.string, "45");
        assert_eq!(ni.cursor_pos, 0);

        ni.handle_backspace();
        assert_eq!(ni.string, "45");
        assert_eq!(ni.cursor_pos, 0);
    }

    #[test]
    fn test_handle_delete() {
        let mut ni =  TatNumberInput::new();
        ni.handle_delete();
        assert!(ni.string.is_empty());

        ni.string = "123456".to_string();
        ni.handle_delete();
        assert_eq!(ni.string, "23456");
        assert_eq!(ni.cursor_pos, 0);

        ni.cursor_pos = 4;
        ni.handle_delete();
        assert_eq!(ni.string, "2345");
        assert_eq!(ni.cursor_pos, 4);

        ni.handle_delete();
        assert_eq!(ni.string, "2345");
        assert_eq!(ni.cursor_pos, 4);
    }

    #[test]
    fn test_key_press() {
        let mut ni =  TatNumberInput::new();

        {
            let res = ni.key_press(KeyCode::Esc, false);
            assert_eq!(res, TatNumberInputResult::Close);

            ni.reset();
        }

        {
            ni.string = "12345".to_string();
            ni.cursor_pos = 5;
            let res = ni.key_press(KeyCode::Left, true);
            assert_eq!(ni.cursor_pos, 0);
            assert_eq!(res, TatNumberInputResult::AcceptedKey);

            ni.reset();
        }

        {
            ni.string = "12345".to_string();
            ni.cursor_pos = 5;
            let res = ni.key_press(KeyCode::Home, false);
            assert_eq!(ni.cursor_pos, 0);
            assert_eq!(res, TatNumberInputResult::AcceptedKey);

            ni.reset();
        }

        {
            ni.string = "12345".to_string();
            let res = ni.key_press(KeyCode::End, false);
            assert_eq!(ni.cursor_pos, 5);
            assert_eq!(res, TatNumberInputResult::AcceptedKey);

            ni.reset();
        }

        {
            ni.string = "12345".to_string();
            let res = ni.key_press(KeyCode::Right, true);
            assert_eq!(ni.cursor_pos, 5);
            assert_eq!(res, TatNumberInputResult::AcceptedKey);

            ni.reset();
        }

        {
            ni.string = "12345".to_string();
            let res = ni.key_press(KeyCode::Enter, false);
            assert_eq!(res, TatNumberInputResult::Accept(12345));

            ni.reset();
        }

        {
            ni.string = "100000000000000000000".to_string();
            let res = ni.key_press(KeyCode::Enter, false);
            assert_eq!(res, TatNumberInputResult::Accept(i64::MAX));

            ni.reset();
        }
    }
}
