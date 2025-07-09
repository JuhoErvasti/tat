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
    lines: usize,
    scroll_offset_v: usize,
    scroll_offset_h: usize,
    available_rows: usize,
}

impl TatNavigableParagraph {
    pub fn new(text: String) -> Self {
        let lines = TatNavigableParagraph::count_lines(&text);
        Self {
            text,
            lines,
            scroll_offset_v: 0,
            scroll_offset_h: 0,
            available_rows: 0,
        }
    }

    pub fn paragraph(&self) -> Paragraph {
        Paragraph::new(self.text.clone())
        .scroll((self.scroll_offset_v as u16, self.scroll_offset_h as u16))
        // .wrap(Wrap { trim: false })
    }

    pub fn scroll_state(&self) -> ScrollbarState {
        ScrollbarState::new(self.last_scrollable_line())
        .position(self.scroll_offset_v)
    }

    pub fn nav_h(&mut self, conf: TatNavHorizontal) {
        match conf {
            TatNavHorizontal::Home => self.scroll_offset_h = 0,
            TatNavHorizontal::End => self.scroll_offset_h = 1000, // FIXME: go to actual end
            TatNavHorizontal::RightOne => self.scroll_offset_h +=1,
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
        if total_rows >= self.lines() as i64 {
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
