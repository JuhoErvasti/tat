use std::{env::temp_dir, fs::File, io::{BufRead, Result}};

use cli_log::debug;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use gdal::{vector::{field_type_to_name, geometry_type_to_name, Defn, Layer, LayerAccess}, Dataset, Metadata};
use ratatui::{style::{palette::tailwind, Style, Stylize}, layout::{Constraint, Flex, Layout, Margin, Rect}, symbols, text::Line, widgets::{Block, BorderType, Borders, Clear, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget}, DefaultTerminal, Frame};

use crate::types::{TatDataset, TatNavJump};
use crate::table::TatTable;

pub const LAYER_LIST_BORDER: symbols::border::Set = symbols::border::Set {
    top_left: symbols::line::ROUNDED.vertical,
    top_right: symbols::line::ROUNDED.horizontal,
    bottom_left: symbols::line::ROUNDED.bottom_left,
    bottom_right: symbols::line::ROUNDED.horizontal,
    vertical_left: symbols::line::ROUNDED.vertical,
    vertical_right: " ",
    horizontal_top: symbols::line::ROUNDED.horizontal,
    horizontal_bottom: symbols::line::ROUNDED.horizontal,
};

pub const LAYER_INFO_BORDER: symbols::border::Set = symbols::border::Set {
    top_left: symbols::line::ROUNDED.horizontal_down,
    top_right: symbols::line::NORMAL.vertical_left,
    bottom_left: symbols::line::ROUNDED.horizontal_up,
    bottom_right: symbols::line::ROUNDED.bottom_right,
    vertical_left: symbols::line::ROUNDED.vertical,
    vertical_right: symbols::line::ROUNDED.vertical,
    horizontal_top: symbols::line::ROUNDED.horizontal,
    horizontal_bottom: symbols::line::ROUNDED.horizontal,
};

pub enum TatMenu {
    LayerSelect,
    TableView,
}

/// This holds the program's state.
pub struct Tat {
    pub current_menu: TatMenu,
    pub quit: bool,
    dataset: TatDataset,
    list_state: ListState,
    log_visible: bool,
    layer_index: Vec<u64>,
    table: TatTable,
}

impl Tat {
    pub fn new(ds: &'static Dataset) -> Self {
        let mut ls = ListState::default();
        ls.select_first();
        Self {
            current_menu: TatMenu::LayerSelect,
            quit: false,
            dataset: TatDataset::new(&ds),
            list_state: ls,
            log_visible: false,
            layer_index: Vec::new(),
            table: TatTable::new(),
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.quit {
            terminal.draw(|frame| {
                self.draw(frame);
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

    pub fn draw(&mut self, frame: &mut Frame) {
        match self.current_menu {
            TatMenu::LayerSelect => self.render_layer_select(frame.area(), frame),
            TatMenu::TableView => self.render_table_view(frame.area(), frame),
        }
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
            KeyCode::Char('0') => self.table.jump_first_column(),
            KeyCode::Char('$') => self.table.jump_last_column(),
            KeyCode::Char('f') if ctrl_down => self.nav_jump_forward(50),
            KeyCode::Char('b') if ctrl_down => self.nav_jump_back(50),
            KeyCode::Char('d') if ctrl_down => self.nav_jump_forward(25),
            KeyCode::Char('u') if ctrl_down => self.nav_jump_back(25),
            KeyCode::Char('L') => self.log_visible = !self.log_visible,
            KeyCode::Enter => self.open_table(),
            _ => {},
        }
    }

    fn open_table(&mut self) {
        self.table.set_layer(self.dataset.layer(self.list_state.selected().unwrap()).unwrap().clone());
        self.current_menu = TatMenu::TableView;
    }

    fn nav_left(&mut self) {
        match self.current_menu {
            TatMenu::LayerSelect => (),
            TatMenu::TableView => self.table.nav_left(),
        }
    }

    fn nav_right(&mut self) {
        match self.current_menu {
            TatMenu::LayerSelect => (),
            TatMenu::TableView => self.table.nav_right(),
        }
    }

    fn previous_menu(&mut self) {
        if self.log_visible {
            self.log_visible = false;
            return;
        }

        match self.current_menu {
            TatMenu::TableView => {
                // TODO: maybe don't reset?
                self.table.reset();
                self.current_menu = TatMenu::LayerSelect;
            },
            TatMenu::LayerSelect => self.close(),
        }
    }

    fn nav_jump_forward(&mut self, amount: u16) {
        match self.current_menu {
            TatMenu::LayerSelect => self.list_state.scroll_down_by(amount),
            TatMenu::TableView => self.table.jump_row(TatNavJump::ForwardParagraph),
        }
    }

    fn nav_jump_back(&mut self, amount: u16) {
        match self.current_menu {
            TatMenu::LayerSelect => self.list_state.scroll_up_by(amount),
            TatMenu::TableView => self.table.jump_row(TatNavJump::BackParagraph),
        }
    }

    fn nav_first(&mut self) {
        match self.current_menu {
            TatMenu::LayerSelect => self.list_state.select_first(),
            TatMenu::TableView => self.table.jump_row(TatNavJump::First),
        }
    }

    fn nav_last(&mut self) {
        match self.current_menu {
            TatMenu::LayerSelect => self.list_state.select_last(),
            TatMenu::TableView => self.table.jump_row(TatNavJump::Last),
        }
    }

    fn nav_up(& mut self) {
        match self.current_menu {
            TatMenu::LayerSelect => self.list_state.select_previous(),
            TatMenu::TableView => self.table.jump_row(TatNavJump::BackOne),
        }
    }

    fn nav_down(& mut self) {
        match self.current_menu {
            TatMenu::LayerSelect => self.list_state.select_next(),
            TatMenu::TableView => self.table.jump_row(TatNavJump::ForwardOne),
        }
    }

    fn render_header(area: Rect, frame: &mut Frame) {
        frame.render_widget(
            Paragraph::new("Terminal Attribute Table")
                .bold()
                .centered(),
            area,
        );
    }

    fn render_dataset_info(&mut self, area: Rect, frame: &mut Frame) {
        let block = Block::new()
            .title(Line::raw(" Dataset ").underlined().bold())
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_set(symbols::border::ROUNDED);

        let mut text = vec![
            // TODO: don't unwrap()
            Line::from(format!("- URI: \"{}\"", self.dataset.gdal_ds.description().unwrap())),
            Line::from(format!("- Driver: {} ({})", self.dataset.gdal_ds.driver().long_name(), self.dataset.gdal_ds.driver().short_name())),
        ];

        if self.dataset.gdal_ds.metadata().count() > 0 {
            text.push(Line::from("Metadata:"));
        }

        for domain in self.dataset.gdal_ds.metadata_domains() {
            if self.dataset.gdal_ds.metadata_domain(&domain).into_iter().len() == 0 {
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

            self.dataset.gdal_ds.metadata_domain(&domain).into_iter().for_each(|values| {
                for value in values {
                    text.push(
                        Line::from(format!("    {}", value))
                    );
                }
            });
        }

        frame.render_widget(
            Paragraph::new(text)
                .block(block),
            area
        );
    }

    fn render_layer_list(&mut self, area: Rect, frame: &mut Frame) {
        let block = Block::new()
            .title(Line::raw(" Layers ").underlined().bold())
            .borders(Borders::ALL)
            .border_set(LAYER_LIST_BORDER);

        let items: Vec<ListItem> = self
            .dataset
            .gdal_ds
            .layers()
            .map(|layer_item| {
                ListItem::new(Line::raw(layer_item.name()))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }


    fn selected_layer(&self) -> Layer {
        // TODO: don't use the unwraps
        self.dataset.gdal_ds.layer(self.list_state.selected().unwrap()).unwrap()
    }

    fn render_table_view(&mut self, area: Rect, frame: &mut Frame) {
        frame.render_widget(& mut self.table, area);
    }

    fn render_layer_select(&mut self, area: Rect, frame: &mut Frame) {
        if self.dataset.gdal_ds.layer_count() == 0 {
            let [header_area, dataset_area, footer_area] = Layout::vertical([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .areas(area);

            Tat::render_header(header_area, frame);
            self.render_dataset_info(dataset_area, frame);
            self.render_footer(footer_area, frame);
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
                    Constraint::Fill(4),
            ]).areas(layer_area);

            Tat::render_header(header_area, frame);
            self.render_dataset_info(dataset_area, frame);
            self.render_layer_list(list_area, frame);
            self.render_layer_info(info_area, frame);
            self.render_footer(footer_area, frame);
        }
    }


    fn render_layer_info(&mut self, area: Rect, frame: &mut Frame) {
        let block = Block::new()
            .title(Line::raw(" Layer Information ").underlined().bold())
            .borders(Borders::ALL)
            .border_set(LAYER_INFO_BORDER);

        let mut text: Vec<Line> = [].to_vec();

        if let Some(selected) = self.list_state.selected() {
            // TODO: don't unwrap
            let layer: Layer = self.dataset.gdal_ds.layer(selected).unwrap();

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

        frame.render_widget(Paragraph::new(text).block(block), area);
    }

    fn render_footer(&self, area: Rect, frame: &mut Frame) {
        let text = match self.current_menu {
            TatMenu::LayerSelect => {
                "<up, k> <down, j>: browse layers | <enter> open layer table | <q, ESC, ctrl+c> quit program"
            },
            TatMenu::TableView => {
                "<left, h> <down, j> <up, k> <right, l>: browse table | <q, esc> return to layer selection | <ctrl+c> quit program"
            }
        };

        frame.render_widget(
            Paragraph::new(text).centered(),
            area,
        )
    }

    fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

    fn draw_log(&self, area: Rect, frame: &mut Frame) {
        let lines = std::io::BufReader::new(File::open(format!("{}/tat_gdal.log", temp_dir().display())).unwrap()).lines();
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
