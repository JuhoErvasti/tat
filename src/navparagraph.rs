use std::usize;

use ratatui::widgets::{
    Paragraph,
    ScrollbarState,
};

use crate::types::{TatNavHorizontal, TatNavVertical};



/// Struct for displaying a text paragraph and allowing scrolling of the text
pub struct TatNavigableParagraph {
    title: Option<String>,
    text: String,
    total_lines: usize,
    scroll_offset_v: usize,
    scroll_offset_h: usize,
    available_rows: usize,
    available_cols: usize,
    max_line_len: usize,
}

impl TatNavigableParagraph {
    /// Creates a new navigable paragraph
    pub fn new(text: String) -> Self {
        Self {
            title: None,
            total_lines: TatNavigableParagraph::count_total_lines(&text),
            max_line_len: TatNavigableParagraph::get_longest_line(&text),
            text,
            scroll_offset_v: 0,
            scroll_offset_h: 0,
            available_rows: 0,
            available_cols: 0,
        }
    }

    /// Sets the (optional) title of the paragraph
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);

        self
    }

    /// Returns the underlying ratatui Paragraph
    pub fn paragraph(&self) -> Paragraph {
        Paragraph::new(self.text.clone())
        .scroll((self.scroll_offset_v as u16, self.scroll_offset_h as u16))
        // .wrap(Wrap { trim: false })
    }

    /// Constructs a vertical ratatui ScrollbarState based on the stored offset
    pub fn scroll_state_v(&self) -> ScrollbarState {
        ScrollbarState::new(self.last_scrollable_line())
        .position(self.scroll_offset_v)
    }

    /// Constructs a horizontal ratatui ScrollbarState based on the stored offset
    pub fn scroll_state_h(&self) -> ScrollbarState {
        ScrollbarState::new(self.last_scrollable_col())
        .position(self.scroll_offset_h)
    }

    /// Handles horizontal navigation
    pub fn nav_h(&mut self, conf: TatNavHorizontal) {
        match conf {
            TatNavHorizontal::Home => self.scroll_offset_h = 0,
            TatNavHorizontal::End => self.scroll_offset_h = self.last_scrollable_col(),
            TatNavHorizontal::RightOne => {
                if self.scroll_offset_h >= self.last_scrollable_col() {
                    return;
                }
                self.scroll_offset_h += 1;
            },
            TatNavHorizontal::LeftOne => {
                if self.scroll_offset_h == 0 {
                    return;
                }

                self.scroll_offset_h -=1;
            }
        }
    }

    /// Handles vertical navigation
    pub fn nav_v(&mut self, conf: TatNavVertical) {
        let total_rows = self.available_rows as i64;
        if total_rows >= self.total_lines() as i64 {
            return;
        }

        let mut nav_by = |amount: i64| {
            let mut new_offset = self.scroll_offset_v as i64 + amount;

            if new_offset < 0 {
                new_offset = 0;
            }

            if new_offset > self.last_scrollable_line() as i64 {
                new_offset = self.last_scrollable_line() as i64
            }

            self.scroll_offset_v = new_offset as usize;
        };

        match conf {
            TatNavVertical::First => self.scroll_offset_v = 0,
            TatNavVertical::Last => self.scroll_offset_v = self.last_scrollable_line(),
            TatNavVertical::DownOne => {
                nav_by(1);
            },
            TatNavVertical::UpOne => {
                nav_by(-1);
            },
            TatNavVertical::DownHalfParagraph => {
                nav_by(total_rows / 2 );
            },
            TatNavVertical::UpHalfParagraph => {
                nav_by(-(total_rows / 2));
            },
            TatNavVertical::MouseScrollDown => {
                nav_by(total_rows / 3 );
            },
            TatNavVertical::MouseScrollUp => {
                nav_by(-(total_rows / 3));
            },
            TatNavVertical::DownParagraph => {
                nav_by(total_rows);
            },
            TatNavVertical::UpParagraph => {
                nav_by(-total_rows);
            },
            TatNavVertical::Specific(row) => {
                unimplemented!("Cannot nav to row {}", row);
            },
        }
    }

    /// Returns the total number of lines in the text contents
    pub fn total_lines(&self) -> usize {
        self.total_lines
    }

    /// Returns the longest line in the text contents
    pub fn max_line_len(&self) -> usize {
        self.max_line_len
    }

    /// Sets the maximum number of lines
    pub fn set_available_rows(&mut self, available_rows: usize) {
        self.available_rows = available_rows;
    }

    /// Sets the maximum number of column
    pub fn set_available_cols(&mut self, available_cols: usize) {
        self.available_cols = available_cols;
    }

    /// Returns the last line this paragraph can be scrolled to. This means the first line at which
    /// all remaining lines are also visible.
    fn last_scrollable_line(&self) -> usize {
        if self.available_rows >= self.total_lines {
            return 0;
        }

        self.total_lines - self.available_rows
    }

    /// Returns the last line this paragraph can be scrolled to. This means the first column at which
    /// all remaining column would be visible.
    fn last_scrollable_col(&self) -> usize {
        if self.available_cols >= self.max_line_len {
            return 0;
        }
        self.max_line_len - self.available_cols
    }

    /// Counts the number of total lines in a string slice
    fn count_total_lines(text: &str) -> usize {
        let mut count = 0;
        for _ in text.lines() {
            count +=1;
        }

        count
    }

    /// Counts the longest line in a string slice
    fn get_longest_line(text: &str) -> usize {
        let mut longest = 0;
        // TODO: is there a faster/more idiomatic way of doing this?
        for line in text.lines() {
            if line.chars().count() > longest {
                longest = line.len();
            }
        }

        longest
    }

    pub fn title(&self) -> Option<&String> {
        self.title.as_ref()
    }

}

mod test {
    use super::*;

    #[test]
    fn test_new() {
        // panic!();
    }

}
