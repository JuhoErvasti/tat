use std::{
    env::temp_dir, fmt::format, fs::File, io::{
        BufRead,
        Result,
    }, usize
};

use cli_clipboard::{ClipboardContext, ClipboardProvider};

use cli_log::{debug, error};
use crossterm::event::{
    self,
    Event,
    KeyCode,
    KeyEvent,
    KeyEventKind,
    KeyModifiers,
};
use gdal::{
    Dataset,
    Metadata,
};
use ratatui::{
    layout::{
        Constraint,
        Flex,
        Layout,
        Margin,
        Rect,
    },
    style::{
        Style,
        Stylize,
    },
    symbols::{
        self,
        scrollbar::{DOUBLE_HORIZONTAL, DOUBLE_VERTICAL},
    },
    text::Line,
    widgets::{
        Block,
        BorderType,
        Borders,
        Clear,
        ListState,
        Paragraph,
        Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    DefaultTerminal,
    Frame,
};

use crate::{layerlist::TatLayerList, navparagraph::TatNavigableParagraph, shared::{self, HELP_TEXT_TABLE}, types::{TatNavHorizontal, TatNavVertical, TatPopUpType, TatPopup}};
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

pub enum TatLayerSelectFocusedBlock {
    LayerList,
    LayerInfo,
}

// TODO: there's a bug with TatNavigableParagraph when the text wraps it doesn't
// correctly update its available rows, leading to scrolling issues

// TODO: a lot of these probably don't have to be public

/// This holds the program's state.
pub struct Tat {
    current_menu: TatMenu,
    quit: bool,
    modal_popup: Option<TatPopup>,
    table: TatTable,
    layerlist: TatLayerList,
    focused_block: TatLayerSelectFocusedBlock,
    clip: Option<ClipboardContext>,
    table_area: Rect,
}

impl Tat {
    pub fn new(ds: &'static Dataset) -> Self {
        let mut ls = ListState::default();
        ls.select_first();

        let clip_res = ClipboardContext::new();
        let clip = match clip_res {
            Ok(_clip) => Some(_clip),
            Err(_) => None
        };

        Self {
            current_menu: TatMenu::LayerSelect,
            quit: false,
            modal_popup: None,
            table: TatTable::new(),
            layerlist: TatLayerList::new(&ds),
            focused_block: TatLayerSelectFocusedBlock::LayerList,
            clip,
            table_area: Rect::default(),
        }
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.quit {
            terminal.draw(|frame| {
                self.draw(frame);
            })?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }

        Ok(())
    }

    /// Returns visible columns and rows of a bordered text area which
    /// may or may not have horizontal and/or vertical scrollbars.
    fn text_area_dimensions(rect: &Rect, max_cols: i64, max_rows: i64) -> (usize, bool, usize, bool) {
        // TODO: there's gotta be a better way to do this
        let original_cols = rect.width as i64;
        let original_rows = rect.height as i64;

        let mut visible_cols = 0;
        let mut visible_rows = 0;

        let mut has_v_scroll = false;
        let mut has_h_scroll = false;

        if original_cols >= 2 {
            visible_cols = original_cols - 2;
        }

        if max_cols > visible_cols {
            has_h_scroll = true;
        }

        if original_rows >= if has_h_scroll { 3 } else { 2 } {
            visible_rows = original_rows - if has_h_scroll { 3 } else { 2 };
        }

        if max_rows > visible_rows {
            has_v_scroll = true;
        }

        if has_v_scroll {
            if original_cols >= 3 {
                visible_cols = original_cols - 3;
            }
        }

        (visible_cols as usize, has_h_scroll, visible_rows as usize, has_v_scroll)
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.table_area = frame.area();

        match self.current_menu {
            TatMenu::LayerSelect => self.render_layer_select(frame.area(), frame),
            TatMenu::TableView => self.render_table_view(frame),
        }

        self.draw_popup(frame);
    }

    fn draw_popup(&mut self, frame: &mut Frame) {
        if let Some(popup) = &mut self.modal_popup {
            let cleared_area = Tat::popup_area(frame.area(), 70, 70);
            let popup_area = cleared_area.inner(
                Margin { horizontal: 1, vertical: 1 }
            );

            let (
                visible_cols,
                has_h_scrollbar,
                visible_rows,
                has_v_scrollbar,
            ) = Tat::text_area_dimensions(
                &popup_area,
                popup.max_line_len() as i64,
                popup.total_lines() as i64,
            );

            popup.set_available_rows(visible_rows as usize);
            popup.set_available_cols(visible_cols as usize);

            let block = popup.paragraph()
                .block(
                    Block::default()
                        .title(Line::raw(popup.title()).bold().underlined().centered())
                        .borders(Borders::ALL)
                        .border_style(crate::shared::palette::DEFAULT.highlighted_style())
                        .border_type(BorderType::Rounded)
                        .title_bottom(Line::raw(crate::shared::POPUP_HINT).centered())
                );

            frame.render_widget(Clear, cleared_area);
            frame.render_widget(block, popup_area);

            if has_v_scrollbar {
                let scrollbar = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some(DOUBLE_VERTICAL.begin))
                    .style(crate::shared::palette::DEFAULT.highlighted_style())
                    .end_symbol(Some(DOUBLE_VERTICAL.end));

                let scrollbar_area = popup_area.inner(Margin { horizontal: 1, vertical: 1 });

                if !scrollbar_area.is_empty() {
                    frame.render_stateful_widget(
                        scrollbar,
                        scrollbar_area,
                        &mut popup.scroll_state_v(),
                    );
                }
            }

            if has_h_scrollbar {
                let scrollbar = Scrollbar::default()
                    .orientation(ScrollbarOrientation::HorizontalBottom)
                    .begin_symbol(Some(DOUBLE_HORIZONTAL.begin))
                    .style(crate::shared::palette::DEFAULT.highlighted_style())
                    .end_symbol(Some(DOUBLE_HORIZONTAL.end));

                let scrollbar_area = popup_area.inner(Margin { horizontal: 2, vertical: 1 });

                if !scrollbar_area.is_empty() {
                    frame.render_stateful_widget(
                        scrollbar,
                        scrollbar_area,
                        &mut popup.scroll_state_h(),
                    );
                }
            }
        }
    }

    fn close(&mut self) {
        self.quit = true;
    }

    fn copy_table_value_to_clipboard(&mut self) {
        if let Some(clip) = self.clip.as_mut() {
            if let Some(text_to_copy) = self.table.selected_value() {
                match clip.set_contents(text_to_copy) {
                    Ok(()) => return,
                    Err(e) => error!("Could not set clipboard contents: {}", e.to_string()),
                }
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        let ctrl_down: bool = key.modifiers.contains(KeyModifiers::CONTROL);
        let in_layer_list: bool = matches!(self.current_menu, TatMenu::LayerSelect) && matches!(self.focused_block, TatLayerSelectFocusedBlock::LayerList);
        let in_table: bool = matches!(self.current_menu, TatMenu::TableView);
        let in_layer_select: bool = matches!(self.current_menu, TatMenu::LayerSelect);
        let popup_open: bool = self.has_popup();

        match key.code {
            KeyCode::Char('q') if ctrl_down => self.close(),
            KeyCode::Char('q') | KeyCode::Esc => self.previous_menu(),
            KeyCode::Char('g') => self.delegate_nav_v(TatNavVertical::First),
            KeyCode::Char('G') => self.delegate_nav_v(TatNavVertical::Last),
            KeyCode::Char('k') | KeyCode::Up => self.delegate_nav_v(TatNavVertical::UpOne),
            KeyCode::Char('j') | KeyCode::Down => self.delegate_nav_v(TatNavVertical::DownOne),
            KeyCode::Char('d') if ctrl_down => self.delegate_nav_v(TatNavVertical::DownHalfParagraph),
            KeyCode::Char('u') if ctrl_down => self.delegate_nav_v(TatNavVertical::UpHalfParagraph),
            KeyCode::Char('f') if ctrl_down => self.delegate_nav_v(TatNavVertical::DownParagraph),
            KeyCode::Char('b') if ctrl_down => self.delegate_nav_v(TatNavVertical::UpParagraph),
            KeyCode::PageDown => self.delegate_nav_v(TatNavVertical::DownParagraph),
            KeyCode::PageUp => self.delegate_nav_v(TatNavVertical::UpParagraph),
            KeyCode::Char('h') | KeyCode::Left => self.delegate_nav_h(TatNavHorizontal::LeftOne),
            KeyCode::Char('l') | KeyCode::Right => self.delegate_nav_h(TatNavHorizontal::RightOne),
            KeyCode::Char('0') | KeyCode::Home => self.delegate_nav_h(TatNavHorizontal::Home),
            KeyCode::Char('$') | KeyCode::End => self.delegate_nav_h(TatNavHorizontal::End),
            KeyCode::Char('c') if ctrl_down && in_table => self.copy_table_value_to_clipboard(),
            KeyCode::Char('y') if in_table => self.copy_table_value_to_clipboard(),
            KeyCode::Char('L') =>  {
                if !popup_open {
                    self.show_gdal_log();
                }
            },
            KeyCode::Char('D') =>  {
                if !popup_open {
                    self.show_debug_log();
                }
            },
            KeyCode::Char('?') =>  {
                if !popup_open {
                    self.show_help();
                }
            },
            KeyCode::Enter => {
                match self.current_menu {
                    TatMenu::LayerSelect => {
                        if !in_table && in_layer_list && !popup_open {
                            self.open_table();
                        }
                    },
                    TatMenu::TableView => {
                        self.show_full_value_popup();
                    },
                }
            },
            KeyCode::Tab | KeyCode::BackTab => {
                if in_layer_select && !popup_open {
                    self.cycle_block_selection();
                }
            }
            _ => {},
        }
    }

    fn show_full_value_popup(&mut self) {
        let value = if let Some(_value) = self.table.selected_value() {
            _value
        } else {
            crate::shared::MISSING_VALUE.to_string()
        };

        let p = TatNavigableParagraph::new(
            format!(
                "{}",
                value,
            )
        );

        let title = format!(
                " Feature {} - Value of \"{}\" ",
                self.table.selected_fid(),
                self.table.current_column_name(),
            );

        self.modal_popup = Some(
            TatPopup::new(
                title,
                p,
                TatPopUpType::FullValue,
            )
        )
    }

    fn show_debug_log(&mut self) {
        let mut text = String::from("");
        match File::open("tat.log") {
            Ok(file) => {
                let lines = std::io::BufReader::new(file).lines();

                for line in lines.map_while(Result::ok) {
                    text = format!("{}\n{}", text, line);
                }
            },
            Err(e) => {
                error!("Could not open file: {}", e.to_string());
                text = format!("Could not open file: {}", e.to_string());
            },
        };

        let p = TatNavigableParagraph::new(text);
        self.modal_popup = Some(
            TatPopup::new(
                crate::shared::TITLE_DEBUG_LOG.to_string(),
                p,
                TatPopUpType::DebugLog,
            )
        )
    }

    fn show_gdal_log(&mut self) {
        let file = match File::open(format!("{}/tat_gdal.log", temp_dir().display())) {
            Ok(file) => file,
            Err(e) => {
                error!("Could not open file: {}", e.to_string());
                return;
            },
        };

        let lines = std::io::BufReader::new(file).lines();
        let mut text = String::from("");

        for line in lines.map_while(Result::ok) {
            text = format!("{}\n{}", text, line);
        }

        let p = TatNavigableParagraph::new(text);
        self.modal_popup = Some(
            TatPopup::new(
                crate::shared::TITLE_GDAL_LOG.to_string(),
                p,
                TatPopUpType::GdalLog,
            )
        )
    }

    fn show_help(&mut self) {
        let help_text = match self.current_menu {
            TatMenu::TableView => crate::shared::HELP_TEXT_TABLE,
            TatMenu::LayerSelect => crate::shared::HELP_TEXT_LAYERSELECT,
        }.to_string();

        let p = TatNavigableParagraph::new(help_text);
        self.modal_popup = Some(
            TatPopup::new(
                crate::shared::TITLE_HELP.to_string(),
                p,
                TatPopUpType::Help,
            )
        )
    }

    fn cycle_block_selection(&mut self) {
        self.focused_block = match self.focused_block {
            TatLayerSelectFocusedBlock::LayerList => TatLayerSelectFocusedBlock::LayerInfo,
            TatLayerSelectFocusedBlock::LayerInfo => TatLayerSelectFocusedBlock::LayerList,
        }
    }

    fn open_table(&mut self) {
        if let Some(layer) = self.layerlist.current_layer() {
            self.table.set_layer(layer.clone());
        }

        let (fid_col_area, table_rect) = self.table_rects();
        self.table.set_rects(table_rect, fid_col_area);
        self.current_menu = TatMenu::TableView;
    }

    fn close_popup(&mut self) {
        self.modal_popup = None;
    }

    fn has_popup(&self) -> bool {
        self.modal_popup.is_some()
    }

    fn previous_menu(&mut self) {
        if self.has_popup() {
            self.close_popup();
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

    fn table_rects(&self) -> (Rect, Rect) {
        let [fid_col_area, table_rect] = Layout::horizontal([
            Constraint::Length(11),
            Constraint::Fill(1),
        ])
        .areas(self.table_area);

        (fid_col_area, table_rect)
    }

    fn delegate_nav_v(&mut self, conf: TatNavVertical) {
        if let Some(pop) = &mut self.modal_popup {
            pop.nav_v(conf);
            return;
        }

        match self.current_menu {
            TatMenu::LayerSelect => {
                match self.focused_block {
                    TatLayerSelectFocusedBlock::LayerList => {
                        self.layerlist.nav(conf);
                    },
                    TatLayerSelectFocusedBlock::LayerInfo => self.layerlist.current_layer_info().nav_v(conf),
                }
            },
            TatMenu::TableView => self.table.nav_v(conf),
        }
    }

    fn delegate_nav_h(&mut self, conf: TatNavHorizontal) {
        if let Some(pop) = &mut self.modal_popup {
            pop.nav_h(conf);
            return;
        }

        match self.current_menu {
            TatMenu::LayerSelect => {
                match self.focused_block {
                    TatLayerSelectFocusedBlock::LayerList => return,
                    TatLayerSelectFocusedBlock::LayerInfo => self.layerlist.current_layer_info().nav_h(conf),
                }
            },
            TatMenu::TableView => self.table.nav_h(conf),
        }
    }

    fn render_header(area: Rect, frame: &mut Frame) {
        frame.render_widget(
            Paragraph::new(crate::shared::TITLE_PROGRAM)
                .bold()
                .centered()
                .fg(crate::shared::palette::DEFAULT.default_fg),
            area,
        );
    }

    fn render_dataset_info(&mut self, area: Rect, frame: &mut Frame) {
        let block = Block::new()
            .fg(crate::shared::palette::DEFAULT.default_fg)
            .title_top(Line::raw(crate::shared::TITLE_DATASET_INFO).underlined().bold())
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_set(symbols::border::ROUNDED)
            .title_top(Line::raw(crate::shared::SHOW_HELP).centered());

        frame.render_widget(
            Paragraph::new(self.layerlist.dataset_info_text())
                .fg(crate::shared::palette::DEFAULT.default_fg)
                .block(block),
            area
        );

    }

    fn render_table_view(&mut self, frame: &mut Frame) {
        let (fid_col_area, table_rect) = self.table_rects();
        self.table.set_rects(table_rect, fid_col_area);
        self.table.render(frame);
    }

    fn render_layer_select(&mut self, area: Rect, frame: &mut Frame) {
        let [header_area, dataset_area, layer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(4),
            Constraint::Fill(1),
        ])
        .areas(area);

        let [list_area, info_area] =
            Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Fill(4),
        ]).areas(layer_area);

        Tat::render_header(header_area, frame);
        self.render_dataset_info(dataset_area, frame);
        self.layerlist.render(list_area, frame, matches!(self.focused_block, TatLayerSelectFocusedBlock::LayerList) && !self.has_popup());
        self.render_layer_info(info_area, frame,  matches!(self.focused_block, TatLayerSelectFocusedBlock::LayerInfo));
    }


    fn render_layer_info(&mut self, area: Rect, frame: &mut Frame, selected: bool) {
        let border_style = if selected && !self.has_popup() {
            crate::shared::palette::DEFAULT.highlighted_style()
        } else {
            crate::shared::palette::DEFAULT.default_style()
        };


        let block = Block::bordered()
            .title(Line::raw(crate::shared::TITLE_LAYER_INFO).bold().underlined())
            .fg(crate::shared::palette::DEFAULT.default_fg)
            .border_set(LAYER_INFO_BORDER)
            .border_style(border_style);

        let info = self.layerlist.current_layer_info();

        frame.render_widget(
            info.paragraph().block(block),
            area,
        );

        let (
            visible_cols,
            has_h_scrollbar,
            visible_rows,
            has_v_scrollbar,
        ) = Tat::text_area_dimensions(
            &area,
            info.max_line_len() as i64,
            info.total_lines() as i64,
        );

        info.set_available_rows(visible_rows as usize);
        info.set_available_cols(visible_cols as usize);

        if has_v_scrollbar {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .style(border_style)
                .begin_symbol(Some(DOUBLE_VERTICAL.begin))
                .end_symbol(Some(DOUBLE_VERTICAL.end));

            let scrollbar_area = area.inner(Margin { horizontal: 1, vertical: 1 });

            if !scrollbar_area.is_empty() {
                frame.render_stateful_widget(
                    scrollbar,
                    scrollbar_area,
                    &mut info.scroll_state_v(),
                );
            }
        }

        if has_h_scrollbar {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::HorizontalBottom)
                .begin_symbol(Some(DOUBLE_HORIZONTAL.begin))
                .style(crate::shared::palette::DEFAULT.highlighted_style())
                .end_symbol(Some(DOUBLE_HORIZONTAL.end));

            let scrollbar_area = area.inner(Margin { horizontal: 1, vertical: 1 });

            if !scrollbar_area.is_empty() {
                frame.render_stateful_widget(
                    scrollbar,
                    scrollbar_area,
                    &mut info.scroll_state_h(),
                );
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
}
