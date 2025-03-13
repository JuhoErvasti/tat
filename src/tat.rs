use std::{fs::File, io::{BufRead, Result}};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use gdal::{vector::{field_type_to_name, geometry_type_to_name, Defn, Layer, LayerAccess}, Dataset, Metadata};
use ratatui::{buffer::Buffer, layout::{Constraint, Flex, Layout, Margin, Rect}, style::{palette::tailwind, Style, Stylize}, symbols, text::Line, widgets::{Block, BorderType, Borders, Clear, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget}, DefaultTerminal, Frame};

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
    first_column: u64,
    visible_columns: u64,
    log_visible: bool,
    layer_index: Vec<u64>,
    vert_scroll_state: ScrollbarState,
    horz_scroll_state: ScrollbarState,
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
            first_column: 0,
            visible_columns: 0,
            log_visible: false,
            layer_index: Vec::new(),
            vert_scroll_state: ScrollbarState::default(),
            horz_scroll_state: ScrollbarState::default(),
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.list_state.select_first();
        self.table_state.select_first();
        self.table_state.select_first_column();

        while !self.quit {
            terminal.draw(|frame| {
                frame.render_widget(&mut self, frame.area());
                if self.log_visible {
                    self.draw_log(frame.area(), frame);
                }
            })?;
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
            KeyCode::Char('0') => self.jump_first_column(),
            KeyCode::Char('$') => self.jump_last_column(),
            KeyCode::Char('f') if ctrl_down => self.nav_jump_forward(),
            KeyCode::Char('b') if ctrl_down => self.nav_jump_back(),
            KeyCode::Char('d') if ctrl_down => self.nav_jump_down(),
            KeyCode::Char('u') if ctrl_down => self.nav_jump_up(),
            KeyCode::Char('L') => self.log_visible = !self.log_visible,
            KeyCode::Enter => self.open_table(),
            _ => {},
        }
    }

    fn update_vert_scrollbar(&mut self) {
        self.vert_scroll_state = self.vert_scroll_state.position(self.top_fid as usize + self.table_state.selected().unwrap());
    }

    fn build_layer_index(&mut self) {
        self.layer_index.clear();

        let mut layer = self.dataset.layer(self.list_state.selected().unwrap()).unwrap();

        for feature in layer.features() {
            self.layer_index.push(feature.fid().unwrap());
        }

        self.vert_scroll_state = ScrollbarState::new(self.layer_index.len());
        self.horz_scroll_state = ScrollbarState::new(layer.defn().fields().count());
    }

    fn open_table(&mut self) {
        self.current_menu = Menu::Table;
        self.build_layer_index();
    }

    fn jump_first_column(&mut self) {
        self.first_column = 0;
        self.table_state.select_first_column();
    }

    fn jump_last_column(&mut self) {
        self.set_first_column(self.selected_layer().defn().fields().count() as i64 - self.visible_columns as i64);
        self.table_state.select_last_column();
    }

    fn max_top_fid(&self) -> i64 {
        self.selected_layer().feature_count() as i64 - self.visible_rows as i64 + 3
    }

    fn set_first_column(&mut self, col: i64) {
        let max_first_column: i64 = self.selected_layer().defn().fields().count() as i64 - self.visible_columns as i64;

        if col >= max_first_column {
            self.first_column = max_first_column as u64;
            return;
        }

        if col <= 0 {
            self.first_column = 0;
            return;
        }

        self.first_column = col as u64;
    }

    fn set_top_fid(&mut self, fid: i64) {
        if self.max_top_fid() <= 1 {
            self.top_fid = 1;
            return;
        }

        if fid >= self.max_top_fid() {
            self.top_fid = self.max_top_fid() as u64;
            return;
        }

        if fid <= 1 {
            self.top_fid = 1;
            return;
        }

        self.top_fid = fid as u64;
    }

    fn bottom_fid(&self) -> u64 {
        self.top_fid + self.visible_rows as u64 - 1
    }

    fn reset_table(&mut self) {
        self.top_fid = 1;
        self.first_column = 0;
        self.visible_columns = 0;
        self.table_state.select_first_column();
        self.table_state.select_first();
    }

    fn nav_left(&mut self) {
        match self.current_menu {
            Menu::LayerSelect => (),
            Menu::Table => {
                if let Some(col) = self.table_state.selected_column() {
                    if col == 0 {
                        if self.first_column == 0 {
                            let cols =  self.selected_layer().defn().fields().count();
                            self.set_first_column(cols as i64 - self.visible_columns as i64);
                            self.table_state.select_column(Some(self.first_column as usize + self.visible_columns as usize));
                        } else {
                            self.set_first_column(self.first_column as i64 - 1);
                        }
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
                    if col == self.visible_columns as usize {
                        let cols =  self.selected_layer().defn().fields().count();
                        if self.first_column as usize + col == cols {
                            self.set_first_column(0);
                            self.table_state.select_column(Some(0));
                        } else {
                            self.set_first_column(self.first_column as i64 + 1);
                        }
                        return;
                    }
                }
                self.table_state.select_next_column();
                self.horz_scroll_state = self.horz_scroll_state.position(self.table_state.selected_column().unwrap());
            }
        }
    }

    fn previous_menu(&mut self) {
        if self.log_visible {
            self.log_visible = false;
            return;
        }

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
                self.update_vert_scrollbar();
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
                self.update_vert_scrollbar();
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
                self.update_vert_scrollbar();
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
                self.update_vert_scrollbar();
            },
        }
    }

    fn nav_first(&mut self) {
        match self.current_menu {
            Menu::LayerSelect => self.list_state.select_first(),
            Menu::Table => {
                self.set_top_fid(1);
                self.table_state.select_first();
                self.update_vert_scrollbar();
            }
        }
    }

    fn nav_last(&mut self) {
        match self.current_menu {
            Menu::LayerSelect => self.list_state.select_last(),
            Menu::Table => {
                self.set_top_fid(self.max_top_fid());
                self.table_state.select(Some(self.visible_rows as usize));
                self.update_vert_scrollbar();
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
                self.update_vert_scrollbar();
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
                self.update_vert_scrollbar();
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

        // This weird order of operations is to prevent self.selected_layer()
        // borrowing self.visible_columns
        // To be honest I don't really get it but I'm just going with this
        // for now. Honestly at the end of the day I might want to have
        // some kind of layer struct to handle some of this stuff more cleanly
        let total_fields = self.selected_layer().defn().fields().count();

        if total_fields * 30 < table_area.width as usize {
            self.visible_columns = total_fields as u64;
        } else {
            self.visible_columns = (table_area.width / 30) as u64;
        }

        let layer = self.selected_layer();
        let defn = layer.defn();

        self.render_footer(footer_area, buf);

        let block = Block::new()
            .title(Line::raw(format!(" {} (debug - visible_rows: {}, visible_columns: {} bottom_fid: {}, top_fid: {}, table_area_width: {})", layer.name(), self.visible_rows, self.visible_columns, self.bottom_fid(), self.top_fid, table_area.width)).centered().bold().underlined())
            .borders(Borders::ALL)
            .padding(Padding::top(1))
            .border_set(symbols::border::ROUNDED);

        let mut header_items: Vec<String> = vec![
            String::from("Feature")
        ];

        let mut field_idx = 0;
        for field in defn.fields() {
            if field_idx < self.first_column {
                field_idx += 1;
                continue;
            }

            if field_idx > self.first_column + self.visible_columns - 1 {
                break;
            }

            field_idx += 1;

            header_items.push(field.name());
        }


        let mut rows: Vec<Row> = [].to_vec();
        let mut widths = [].to_vec();

        for _ in 0..self.visible_columns + 1 {
            widths.push(Constraint::Fill(1));
        }

        for i in self.top_fid..self.bottom_fid() + 1 {
            let fid = match self.layer_index.get(i as usize - 1) {
                Some(fid) => fid,
                None => break,
            };

            let feature = match layer.feature(*fid) {
                Some(f) => f,
                None => break,
            };

            let mut row_items: Vec<String> = [
                format!("{i}")
            ].to_vec();

            for i in self.first_column..self.first_column + self.visible_columns {
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

        let vert_scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);

        let horz_scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::HorizontalBottom)
                .begin_symbol(None)
                .end_symbol(None);

        StatefulWidget::render(vert_scrollbar, table_area.inner(Margin { horizontal: 0, vertical: 1 }), buf, &mut self.vert_scroll_state);
        StatefulWidget::render(horz_scrollbar, table_area.inner(Margin { horizontal: 1, vertical: 0 }), buf, &mut self.horz_scroll_state);
        StatefulWidget::render(table, table_area.inner(Margin {horizontal: 1, vertical: 1 }), buf, &mut self.table_state);
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

    fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

    fn draw_log(&self, area: Rect, frame: &mut Frame) {
        let lines = std::io::BufReader::new(File::open("tat_gdal.log").unwrap()).lines();
        let mut text = String::from("");

        for line in lines.map_while(Result::ok) {
            text = format!("{}\n{}", text, line);
        }

        let block = Paragraph::new(text)
            .block(
                Block::default()
                    .title("GDAL Log")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
            );

        let area = Tat::popup_area(area, 60, 60);

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
    }
}
