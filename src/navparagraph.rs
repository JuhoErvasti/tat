use std::usize;

use cli_log::debug;
use ratatui::widgets::{
    Paragraph,
    ScrollbarState, Wrap,
};

use crate::types::TatNavJump;



/// Struct for displaying a text paragraph and allowing scrolling of the text
pub struct TatNavigableParagraph {
    text: String,
    lines: usize,
    scroll_offset: usize,
    available_rows: usize,
}

impl TatNavigableParagraph {
    pub fn new(text: String) -> Self {
        let lines = TatNavigableParagraph::count_lines(&text);
        Self {
            text,
            lines,
            scroll_offset: 0,
            available_rows: 0,
        }
    }

    pub fn paragraph(&self) -> Paragraph {
        Paragraph::new(self.text.clone())
        .scroll((self.scroll_offset as u16, 0))
        .wrap(Wrap { trim: false })
    }

    pub fn scroll_state(&self) -> ScrollbarState {
        ScrollbarState::new(self.last_scrollable_line())
        .position(self.scroll_offset)
    }

    pub fn jump(&mut self, conf: TatNavJump) {
        let total_rows = self.available_rows as i64;
        if total_rows >= self.lines() as i64 {
            return;
        }

        let mut jump_by = |amount: i64| {
            let mut new_offset = self.scroll_offset as i64 + amount;

            if new_offset < 0 {
                new_offset = 0;
            }

            if new_offset > self.last_scrollable_line() as i64 {
                new_offset = self.last_scrollable_line() as i64
            }

            self.scroll_offset = new_offset as usize;
        };

        match conf {
            TatNavJump::First => self.scroll_offset = 0,
            TatNavJump::Last => self.scroll_offset = self.last_scrollable_line(),
            TatNavJump::DownOne => {
                jump_by(1);
            },
            TatNavJump::UpOne => {
                jump_by(-1);
            },
            TatNavJump::DownHalfParagraph => {
                jump_by(total_rows / 2 );
            },
            TatNavJump::UpHalfParagraph => {
                jump_by(-(total_rows / 2));
            },
            TatNavJump::DownParagraph => {
                jump_by(total_rows);
            },
            TatNavJump::UpParagraph => {
                jump_by(-total_rows);
            },
            TatNavJump::Specific(row) => {
                panic!("Not implemented! Cannot jump to row {}", row);
            },
        }
    }

    pub fn lines(&self) -> usize {
        self.lines
    }

    fn last_scrollable_line(&self) -> usize {
        self.lines - self.available_rows
    }

    fn count_lines(text: &str) -> usize {
        let mut count = 0;
        for _ in text.lines() {
            count +=1;
        }

        count
    }

    pub fn available_rows(&self) -> usize {
        self.available_rows
    }

    pub fn set_available_rows(&mut self, available_rows: usize) {
        self.available_rows = available_rows;
    }
}
