use std::io::Result;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use gdal::Dataset;
use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, widgets::{Paragraph, Widget}, DefaultTerminal};

pub enum Menu {
    LayerSelect,
    Table,
}

/// This holds the program's state.
pub struct Tat {
    pub current_menu: Menu,
    quit: bool,
}

impl Widget for &mut Tat {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        Tat::render_header(area, buf);
    }
}

impl Tat {
    pub fn new(ds: &Dataset) -> Self {
        Self {
            current_menu: Menu::LayerSelect,
            quit: false,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.quit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            _ => {},
        }
    }

    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Terminal Attribute Table")
            .bold()
            .centered()
            .render(area, buf);
    }
}

