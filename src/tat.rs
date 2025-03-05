use core::panic;
use std::{cmp::max, io::Result};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use gdal::{vector::{field_type_to_name, geometry_type_to_name, Defn, Layer, LayerAccess}, Dataset, Metadata};
use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Rect}, style::{palette::tailwind, Style, Stylize}, symbols, text::Line, widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph, Row, StatefulWidget, Table, TableState, Widget}, DefaultTerminal};

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
    top_fid: u64,
    visible_rows: u16,
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
            top_fid: 1,
            visible_rows: 0,
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
            KeyCode::Char('c') => self.close(),
            KeyCode::Char('q') | KeyCode::Esc => self.previous_menu(),
            KeyCode::Char('h') | KeyCode::Left => self.nav_left(),
            KeyCode::Char('l') | KeyCode::Right => self.nav_right(),
            KeyCode::Char('k') | KeyCode::Up => self.nav_up(),
            KeyCode::Char('j') | KeyCode::Down => self.nav_down(),
            KeyCode::Char('g') => self.nav_first(),
            KeyCode::Char('G') => self.nav_last(),
            KeyCode::Char('f') if ctrl_down => self.nav_jump_forward(),
            KeyCode::Char('b') if ctrl_down => self.nav_jump_back(),
            KeyCode::Char('d') if ctrl_down => self.nav_jump_down(),
            KeyCode::Char('u') if ctrl_down => self.nav_jump_up(),
            KeyCode::Enter => self.current_menu = Menu::Table,
            _ => {},
        }
    }

    fn set_top_fid(&mut self, fid: i64) {
        let max_top_fid: i64 = self.selected_layer().feature_count() as i64 - self.visible_rows as i64 + 1;

        if fid >= max_top_fid {
            self.top_fid = max_top_fid as u64;
            return;
        }

        if fid <= 1 {
            self.top_fid = 1;
            return;
        }

        self.top_fid = fid as u64;
    }

    fn bottom_fid(&self) -> u64 {
        return self.top_fid + self.visible_rows as u64 - 1;
    }

    fn reset_table(&mut self) {
        self.top_fid = 1;
        self.table_state.select_column(None);
        self.table_state.select_first();
    }

    fn nav_left(&mut self) {
        match self.current_menu {
            Menu::LayerSelect => (),
            Menu::Table => {
                if let Some(col) = self.table_state.selected_column() {
                    if col == 0 {
                        self.table_state.select_column(None);
                        return;
                    }
                }
                self.table_state.select_previous_column()
            }
        }
    }

    fn nav_right(&mut self) {
        match self.current_menu {
            Menu::LayerSelect => (),
            Menu::Table => {
                if let Some(col) = self.table_state.selected_column() {
                    // TODO: take FID into account
                    let cols =  self.selected_layer().defn().fields().count();

                    if col == cols - 1 {
                        self.table_state.select_column(None);
                        return;
                    }
                }
                self.table_state.select_next_column();
            }
        }
    }

    fn previous_menu(&mut self) {
        match self.current_menu {
            Menu::Table => {
                self.reset_table();
                self.current_menu = Menu::LayerSelect;
            },
            Menu::LayerSelect => self.close(),
        }
    }

    fn nav_jump_forward(&mut self) {
        let jump_amount = 50;
        match self.current_menu {
            Menu::LayerSelect => self.list_state.scroll_down_by(jump_amount),
            Menu::Table => {
                if let Some(selected) = self.table_state.selected() {
                    if selected as u16 + jump_amount > self.visible_rows {
                        self.set_top_fid(self.top_fid as i64 + jump_amount as i64);
                    } else {
                        self.table_state.scroll_down_by(jump_amount);
                    }
                }
            },
        }
    }

    fn nav_jump_back(&mut self) {
        let jump_amount = 50;
        match self.current_menu {
            Menu::LayerSelect => self.list_state.scroll_up_by(jump_amount),
            Menu::Table => {
                if let Some(selected) = self.table_state.selected() {
                    if (selected as i16 - jump_amount as i16) < 0 {
                        self.set_top_fid(self.top_fid as i64 - jump_amount as i64);
                    } else {
                        self.table_state.scroll_up_by(jump_amount);
                    }
                }
            },
        }
    }

    fn nav_jump_up(&mut self) {
        let jump_amount = 25;
        match self.current_menu {
            Menu::LayerSelect => self.list_state.scroll_up_by(jump_amount),
            Menu::Table => {
                if let Some(selected) = self.table_state.selected() {
                    if (selected as i16 - jump_amount as i16) < 0 {
                        self.set_top_fid(self.top_fid as i64 - jump_amount as i64);
                    } else {
                        self.table_state.scroll_up_by(jump_amount);
                    }
                }
            },
        }
    }

    fn nav_jump_down(&mut self) {
        let jump_amount = 25;
        match self.current_menu {
            Menu::LayerSelect => self.list_state.scroll_down_by(jump_amount),
            Menu::Table => {
                if let Some(selected) = self.table_state.selected() {
                    if selected as u16 + jump_amount > self.visible_rows {
                        self.set_top_fid(self.top_fid as i64 + jump_amount as i64);
                    } else {
                        self.table_state.scroll_down_by(jump_amount);
                    }
                }
            },
        }
    }

    fn nav_first(&mut self) {
        match self.current_menu {
            Menu::LayerSelect => self.list_state.select_first(),
            Menu::Table => {
                self.set_top_fid(1);
                self.table_state.select_first();
            }
        }
    }

    fn nav_last(&mut self) {
        match self.current_menu {
            Menu::LayerSelect => self.list_state.select_last(),
            Menu::Table => {
                self.set_top_fid(self.selected_layer().feature_count() as i64 - self.visible_rows as i64 + 1);
                self.table_state.select_last();
            }
        }
    }

    fn nav_up(& mut self) {
        match self.current_menu {
            Menu::LayerSelect => self.list_state.select_previous(),
            Menu::Table => {
                if let Some(selected) = self.table_state.selected() {
                    if selected == 0 {
                        self.set_top_fid(self.top_fid as i64 - 1);
                    } else {
                        self.table_state.select_previous();
                    }
                }
            }
        }
    }

    fn nav_down(& mut self) {
        match self.current_menu {
            Menu::LayerSelect => self.list_state.select_next(),
            Menu::Table => {
                if let Some(selected) = self.table_state.selected() {
                    if selected + 1 == self.visible_rows as usize {
                        self.set_top_fid(self.top_fid as i64 + 1);
                    } else {
                        self.table_state.select_next();
                    }
                }
            }
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
        let [table_area, footer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        self.visible_rows = table_area.height - 4;

        let layer = self.selected_layer();
        let defn = layer.defn();

        self.render_footer(footer_area, buf);

        let block = Block::new()
            .title(Line::raw(format!(" {} (debug - visible_rows: {}, bottom_fid: {}, top_fid: {} ", layer.name(), self.visible_rows, self.bottom_fid(), self.top_fid)).centered().bold().underlined())
            .borders(Borders::ALL)
            .padding(Padding::top(1))
            .border_set(symbols::border::ROUNDED);

        let total_fields = defn.fields().count();
        let mut header_items: Vec<String> = vec![
            String::from("fid")
        ];

        for field in layer.defn().fields() {
            header_items.push(field.name());
        }


        let mut rows: Vec<Row> = [].to_vec();
        let mut widths = [].to_vec();

        for _ in 0..total_fields {
            widths.push(Constraint::Fill(1));
        }

        for i in self.top_fid..self.bottom_fid() + 1 {
            // TODO: no unwraps etc etc.
            let feature = match layer.feature((i) as u64) {
                Some(f) => f,
                None => break,
            };

            let mut row_items: Vec<String> = [
                format!("{i}")
            ].to_vec();

            for i in 0..total_fields {
                let str_opt = match feature.field_as_string(i as i32) {
                    Ok(str_opt) => str_opt,
                    // TODO: should the GdalError here be handled differently?
                    Err(_) => Some(String::from("NULL")),
                };

                if let Some(str) = str_opt {
                    row_items.push(str);
                } else {
                    row_items.push(String::from("NULL"));
                }

            }

            rows.push(Row::new(row_items));
        }

        let header = Row::new(header_items);

        // hs = highlight style
        let col_hs = Style::default()
        .fg(tailwind::SLATE.c500);
        let row_hs = Style::default()
        .fg(tailwind::SLATE.c500);
        let cell_hs = Style::default()
        .fg(tailwind::SLATE.c950)
        .bg(tailwind::SLATE.c400);

        let table = Table::new(rows, widths)
            .header(header.underlined())
            .block(block)
            .column_highlight_style(col_hs)
            .row_highlight_style(row_hs)
            .cell_highlight_style(cell_hs);

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
                Constraint::Length(1),
            ])
            .areas(area);

            Tat::render_header(header_area, buf);
            self.render_dataset_info(dataset_area, buf);
            self.render_footer(footer_area, buf);
        } else {
            let [header_area, dataset_area, layer_area, footer_area] = Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Fill(3),
                Constraint::Length(1),
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
            self.render_footer(footer_area, buf);
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

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        match self.current_menu {
            Menu::LayerSelect => {
                Paragraph::new("<up, k> <down, j>: browse layers | <enter> open layer table | <q, ESC, ctrl+c> quit program")
                .centered()
                .render(area, buf);
            }
            Menu::Table => {
                Paragraph::new("<left, h> <down, j> <up, k> <right, l>: browse table | <q, esc> return to layer selection | <ctrl+c> quit program")
                .centered()
                .render(area, buf);
            }
        }
    }
}
