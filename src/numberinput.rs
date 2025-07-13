use std::num::IntErrorKind;

use crossterm::event::KeyCode;
use ratatui::{layout::{Position, Rect}, Frame};

/// Result of completing the input dialog
pub enum TatNumberInputResult {
    UnacceptedKey,
    AcceptedKey,
    Close,
    Accept(i64),
}

/// A widget for getting a number value from the user. Number must be a positive integer.
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
            KeyCode::Home | KeyCode::Left if ctrl_down => {
                self.cursor_pos = 0;

                TatNumberInputResult::AcceptedKey
            }
            KeyCode::End | KeyCode::Right if ctrl_down => {
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
            _ => TatNumberInputResult::UnacceptedKey,
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
}
