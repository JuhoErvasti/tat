use std::{any::Any, io::Result};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use gdal::{vector::{field_type_to_name, geometry_type_to_name, Defn, Layer, LayerAccess}, Dataset, Metadata};
use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Rect}, style::Stylize, symbols, text::Line, widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, Row, StatefulWidget, Table, TableState, Widget}, DefaultTerminal};

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
    table_state: TableState,
}

impl Widget for &mut Tat {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        match self.current_menu {
            Menu::LayerSelect => self.render_layer_select(area, buf),
            Menu::Table => self.render_table(area, buf),
        }
    }
}

impl Tat {
    pub fn new(ds: Dataset) -> Self {
        Self {
            current_menu: Menu::LayerSelect,
            quit: false,
            dataset: ds,
            list_state: ListState::default(),
            table_state: TableState::default(),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.list_state.select_first();
        self.table_state.select_first();
        while !self.quit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    fn close(&mut self) {
        self.quit = true;
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        let ctrl_down: bool = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.previous_menu(),
            KeyCode::Char('k') | KeyCode::Up => self.nav_up(),
            KeyCode::Char('j') | KeyCode::Down => self.nav_down(),
            KeyCode::Char('g') => self.nav_first(),
            KeyCode::Char('G') => self.nav_last(),
            KeyCode::Char('d') if ctrl_down => self.nav_jump_down(),
            KeyCode::Char('u') if ctrl_down => self.nav_jump_up(),
            KeyCode::Char('f') if ctrl_down => self.nav_jump_forward(),
            KeyCode::Char('b') if ctrl_down => self.nav_jump_back(),
            KeyCode::Enter => self.current_menu = Menu::Table,
            _ => {},
        }
    }

    fn previous_menu(&mut self) {
        match self.current_menu {
            Menu::Table => self.current_menu = Menu::LayerSelect,
            Menu::LayerSelect => self.close(),
        }
    }

    fn nav_jump_forward(&mut self) {
        match self.current_menu {
            Menu::LayerSelect => self.list_state.scroll_down_by(50),
            Menu::Table => self.table_state.scroll_down_by(50),
        }
    }

    fn nav_jump_back(&mut self) {
        match self.current_menu {
            Menu::LayerSelect => self.list_state.scroll_up_by(50),
            Menu::Table => self.table_state.scroll_up_by(50),
        }
    }

    fn nav_jump_up(&mut self) {
        match self.current_menu {
            Menu::LayerSelect => self.list_state.scroll_up_by(25),
            Menu::Table => self.table_state.scroll_up_by(25),
        }
    }

    fn nav_jump_down(&mut self) {
        match self.current_menu {
            Menu::LayerSelect => self.list_state.scroll_down_by(25),
            Menu::Table => self.table_state.scroll_down_by(25),
        }
    }

    fn nav_first(&mut self) {
        match self.current_menu {
            Menu::LayerSelect => self.list_state.select_first(),
            Menu::Table => self.table_state.select_first(),
        }
    }

    fn nav_last(&mut self) {
        match self.current_menu {
            Menu::LayerSelect => self.list_state.select_last(),
            Menu::Table => self.table_state.select_last(),
        }
    }

    fn nav_up(& mut self) {
        match self.current_menu {
            Menu::LayerSelect => self.list_state.select_previous(),
            Menu::Table => self.table_state.select_previous(),
        }
    }

    fn nav_down(& mut self) {
        match self.current_menu {
            Menu::LayerSelect => self.list_state.select_next(),
            Menu::Table => self.table_state.select_next(),
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
            .title(Line::raw(" Dataset ").underlined().bold())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        let mut text = vec![
            // TODO: don't unwrap()
            Line::from(format!("- URI: \"{}\"", self.dataset.description().unwrap())),
            Line::from(format!("- Driver: {} ({})", self.dataset.driver().long_name(), self.dataset.driver().short_name())),
        ];

        if self.dataset.metadata().count() > 0 {
            text.push(Line::from("Metadata:"));
        }

        for domain in self.dataset.metadata_domains() {
            if self.dataset.metadata_domain(&domain).into_iter().len() == 0 {
                continue;
            }

            let display_str: &str = if domain.is_empty() {
                "Default"
            } else {
                &domain
            };

            text.push(
                Line::from(format!("  {} Domain:", display_str))
            );

            self.dataset.metadata_domain(&domain).into_iter().for_each(|values| {
                for value in values {
                    text.push(
                        Line::from(format!("    {}", value))
                    );
                }
            });
        }

        Paragraph::new(text)
            .block(block)
            .render(area, buf);
    }

    fn render_layer_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw(" Layers ").underlined().bold())
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

    fn render_table(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let [header_area, table_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        let mut layer = self.selected_layer();

        Tat::render_header(header_area, buf);
        Tat::render_footer(footer_area, buf);

        let block = Block::new()
            .title(format!(" {} ", layer.name()).bold().underlined())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        let mut rows: Vec<Row> = [].to_vec();
        let widths = vec![Constraint::Fill(1)];

        for feature in layer.features() {
            rows.push(Row::new([format!("{}", feature.fid().unwrap())]));
        }

        let table = Table::new(rows, widths)
            .block(block)
            .highlight_symbol(">");

        StatefulWidget::render(table, table_area, buf, &mut self.table_state);
    }

    fn selected_layer(&self) -> Layer {
        // TODO: don't use the unwraps
        self.dataset.layer(self.list_state.selected().unwrap()).unwrap()
    }

    fn render_layer_select(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        if self.dataset.layer_count() == 0 {
            let [header_area, dataset_area, footer_area] = Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(2),
            ])
            .areas(area);

            Tat::render_header(header_area, buf);
            self.render_dataset_info(dataset_area, buf);
            Tat::render_footer(footer_area, buf);
        } else {
            let [header_area, dataset_area, layer_area, footer_area] = Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Fill(3),
                Constraint::Length(2),
            ])
            .areas(area);

            let [list_area, info_area] =
                Layout::horizontal([
                    Constraint::Fill(1),
                    Constraint::Fill(1),
            ]).areas(layer_area);

            Tat::render_header(header_area, buf);
            self.render_dataset_info(dataset_area, buf);
            self.render_layer_list(list_area, buf);
            self.render_layer_info(info_area, buf);
            Tat::render_footer(footer_area, buf);
        }
    }


    fn render_layer_info(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw(" Layer Information ").underlined().bold())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        let mut text: Vec<Line> = [].to_vec();

        if let Some(selected) = self.list_state.selected() {
            // TODO: don't unwrap
            let layer: Layer = self.dataset.layer(selected).unwrap();

            text.push(Line::from(format!("- Name: {}", layer.name())));

            if let Some(crs) = layer.spatial_ref() {
                // TODO: don't unwrap
                text.push(Line::from(format!("- CRS: {}:{} ({})", crs.auth_name().unwrap(), crs.auth_code().unwrap(), crs.name().unwrap())));
            }

            text.push(
                Line::from(format!("- Feature Count: {}", layer.feature_count()))
            );

            let defn: &Defn = layer.defn();

            let geom_fields_count = defn.geom_fields().count();
            let geom_fields = defn.geom_fields();

            if geom_fields_count > 0 {
                text.push(
                    Line::from("- Geometry fields:")
                );
                for geom_field in geom_fields {
                    let display_str: &str = if geom_field.name().is_empty() {
                        "ANONYMOUS"
                    } else {
                        &geom_field.name()
                    };

                    // TODO: don't unwrap etc.
                    text.push(
                        Line::from(format!("    \"{}\" - ({}, {}:{})", display_str, geometry_type_to_name(geom_field.field_type()), geom_field.spatial_ref().unwrap().auth_name().unwrap(), geom_field.spatial_ref().unwrap().auth_code().unwrap()))
                    );
                }
            }

            let fields_count = defn.fields().count();
            let fields = defn.fields();

            if fields_count > 0 {
                text.push(
                    Line::from("- Fields:")
                );

                for field in fields {
                    text.push(
                        Line::from(
                            format!("    \"{}\" - ({})", field.name(), field_type_to_name(field.field_type()))
                        )
                    );
                }
            }
        }

        Paragraph::new(text)
            .block(block)
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("some help should go here probably")
            .centered()
            .render(area, buf);
    }
}
