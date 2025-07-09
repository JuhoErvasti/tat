use std::usize;

use cli_log::debug;
use ratatui::widgets::{
    Paragraph,
    ScrollbarState, Wrap,
};

use crate::types::{TatNavHorizontal, TatNavVertical};



/// Struct for displaying a text paragraph and allowing scrolling of the text
pub struct TatNavigableParagraph {
    text: String,
    total_lines: usize,
    scroll_offset_v: usize,
    scroll_offset_h: usize,
    available_rows: usize,
    available_cols: usize,
    max_line_len: usize,
}

impl TatNavigableParagraph {
    pub fn new(text: String) -> Self {
        Self {
            total_lines: TatNavigableParagraph::count_total_lines(&text),
            max_line_len: TatNavigableParagraph::get_longest_line(&text),
            text,
            scroll_offset_v: 0,
            scroll_offset_h: 0,
            available_rows: 0,
            available_cols: 0,
        }
    }

    pub fn paragraph(&self) -> Paragraph {
        Paragraph::new(self.text.clone())
        .scroll((self.scroll_offset_v as u16, self.scroll_offset_h as u16))
        // .wrap(Wrap { trim: false })
    }

    pub fn scroll_state_v(&self) -> ScrollbarState {
        ScrollbarState::new(self.last_scrollable_row())
        .position(self.scroll_offset_v)
    }

    pub fn scroll_state_h(&self) -> ScrollbarState {
        ScrollbarState::new(self.last_scrollable_col())
        .position(self.scroll_offset_h)
    }

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

            if new_offset > self.last_scrollable_row() as i64 {
                new_offset = self.last_scrollable_row() as i64
            }

            self.scroll_offset_v = new_offset as usize;
        };

        match conf {
            TatNavVertical::First => self.scroll_offset_v = 0,
            TatNavVertical::Last => self.scroll_offset_v = self.last_scrollable_row(),
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
            TatNavVertical::DownParagraph => {
                nav_by(total_rows);
            },
            TatNavVertical::UpParagraph => {
                nav_by(-total_rows);
            },
            TatNavVertical::Specific(row) => {
                panic!("Not implemented! Cannot nav to row {}", row);
            },
        }
    }

    pub fn total_lines(&self) -> usize {
        self.total_lines
    }

    fn last_scrollable_row(&self) -> usize {
        if self.available_rows >= self.total_lines {
            return 0;
        }

        self.total_lines - self.available_rows
    }

    fn last_scrollable_col(&self) -> usize {
        if self.available_cols >= self.max_line_len {
            return 0;
        }
        self.max_line_len - self.available_cols
    }

    fn count_total_lines(text: &str) -> usize {
        let mut count = 0;
        for _ in text.lines() {
            count +=1;
        }

        count
    }

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

    pub fn available_rows(&self) -> usize {
        self.available_rows
    }

    pub fn set_available_rows(&mut self, available_rows: usize) {
        self.available_rows = available_rows;
    }

    pub fn set_available_cols(&mut self, available_cols: usize) {
        self.available_cols = available_cols;
    }

    pub fn max_line_len(&self) -> usize {
        self.max_line_len
    }
}
