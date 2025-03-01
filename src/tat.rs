use std::io::Result;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use gdal::{vector::LayerAccess, Dataset};
use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Rect}, style::Stylize, symbols, text::Line, widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget}, DefaultTerminal};

pub enum Menu {
    LayerSelect,
    Table,
}

/// This holds the program's state.
pub struct Tat {
    pub current_menu: Menu,
    quit: bool,
    dataset: Dataset,
    list_state: ListState,
}

impl Widget for &mut Tat {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let [header_area, dataset_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        let [list_area, info_area] =
            Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Fill(1),
        ]).areas(main_area);

        Tat::render_header(header_area, buf);
        Tat::render_dataset_info(self, dataset_area, buf);
        Tat::render_layer_list(self, list_area, buf);
        Tat::render_layer_info(self, info_area, buf);
        Tat::render_footer(footer_area, buf);
    }
}

impl Tat {
    pub fn new(ds: Dataset) -> Self {
        Self {
            current_menu: Menu::LayerSelect,
            quit: false,
            dataset: ds,
            list_state: ListState::default(),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.list_state.select_first();
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

    fn render_dataset_info(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw(" Dataset ").centered())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        block.render(area, buf);
    }

    fn render_layer_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw(" Layers ").centered())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        let items: Vec<ListItem> = self
            .dataset
            .layers()
            .map(|layer_item| {
                ListItem::new(Line::raw(layer_item.name()))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }

    fn render_layer_info(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw(" Layer Information ").centered())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        block.render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("some help should go here probably")
            .centered()
            .render(area, buf);
    }
}
