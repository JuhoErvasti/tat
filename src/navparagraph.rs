use std::usize;

use cli_log::debug;
use ratatui::widgets::{
    Paragraph,
    ScrollbarState,
};

use crate::types::TatNavJump;



/// Struct for displaying a text paragraph and allowing scrolling of the text
pub struct TatNavigableParagraph {
    text: String,
    lines: usize,
    pub scroll_offset: usize, // TODO: make non-pub
}

impl TatNavigableParagraph {
    pub fn new(text: String) -> Self {
        let lines = TatNavigableParagraph::count_lines(&text);
        Self {
            text,
            lines,
            scroll_offset: 0,
        }
    }

    pub fn paragraph(&self) -> Paragraph {
        Paragraph::new(self.text.clone())
        .scroll((self.scroll_offset as u16, 0))
    }

    pub fn scroll_state(&self) -> ScrollbarState {
        ScrollbarState::new(self.lines - 1)
        .position(self.scroll_offset)
    }

    pub fn jump(&mut self, conf: TatNavJump) {
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
                jump_by(25 );
            },
            TatNavJump::UpHalfParagraph => {
                jump_by(-25);
            },
            TatNavJump::DownParagraph => {
                jump_by(50);
            },
            TatNavJump::UpParagraph => {
                jump_by(-50);
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
        self.lines - 1
    }

    fn count_lines(text: &str) -> usize {
        let mut count = 0;
        for _ in text.lines() {
            count +=1;
        }

        count
    }
}
