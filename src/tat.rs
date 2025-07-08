use std::{
    env::temp_dir,
    fs::File,
    io::{
        BufRead,
        Result,
    },
};

use cli_log::debug;
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
        scrollbar::DOUBLE_VERTICAL,
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
        ScrollbarOrientation,
    },
    DefaultTerminal,
    Frame,
};

use crate::{layerlist::TatLayerList, navparagraph::TatNavigableParagraph, shared::{self, HELP_TEXT_TABLE}, types::TatNavJump};
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

// TODO: move to types.rs
pub enum TatPopUpType {
    // TODO: really not sure this is that necessary?
    Help,
    GdalLog,
}

impl TatPopUpType {
    pub fn to_title(self) -> String {
        match self {
            TatPopUpType::Help => crate::shared::TITLE_HELP.to_string(),
            TatPopUpType::GdalLog => crate::shared::TITLE_GDAL_LOG.to_string(),
        }
    }
}

pub struct TatPopup {
    paragraph: TatNavigableParagraph,
    ptype: TatPopUpType,
}

impl TatPopup {

    pub fn new(paragraph: TatNavigableParagraph, ptype: TatPopUpType) -> Self {
        Self { paragraph, ptype }
    }

    pub fn paragraph_mut(&mut self) -> &mut TatNavigableParagraph {
        &mut self.paragraph
    }

    pub fn ptype(&self) -> &TatPopUpType {
        &self.ptype
    }
}

/// This holds the program's state.
pub struct Tat {
    current_menu: TatMenu,
    quit: bool,
    modal_popup: Option<TatPopup>,
    table: TatTable,
    layerlist: TatLayerList,
    focused_block: TatLayerSelectFocusedBlock,
}

impl Tat {
    pub fn new(ds: &'static Dataset) -> Self {
        let mut ls = ListState::default();
        ls.select_first();
        Self {
            current_menu: TatMenu::LayerSelect,
            quit: false,
            modal_popup: None,
            table: TatTable::new(),
            layerlist: TatLayerList::new(&ds),
            focused_block: TatLayerSelectFocusedBlock::LayerList,
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

    fn draw(&mut self, frame: &mut Frame) {
        match self.current_menu {
            TatMenu::LayerSelect => self.render_layer_select(frame.area(), frame),
            TatMenu::TableView => self.render_table_view(frame.area(), frame),
        }

        self.draw_popup(frame);
    }

    fn draw_popup(&mut self, frame: &mut Frame) {
        if let Some(popup) = &mut self.modal_popup {
            let nav_para = popup.paragraph_mut();

            let cleared_area = Tat::popup_area(frame.area(), 70, 70);
            let popup_area = cleared_area.inner(
                Margin { horizontal: 1, vertical: 1 }
            );

            let max_visible_rows: u16 = if popup_area.height >= 2 {
                popup_area.height - 2
            } else {
                0
            };

            nav_para.set_available_rows(max_visible_rows as usize);

            let block = nav_para.paragraph()
                .block(
                    Block::default()
                        .title(Line::raw(popup.ptype().to_title()).bold().underlined().centered())
                        .borders(Borders::ALL)
                        .border_style(crate::shared::palette::DEFAULT.highlighted_style())
                        .border_type(BorderType::Rounded)
                        .title_bottom(Line::raw(crate::shared::POPUP_HINT).centered())
                );

            frame.render_widget(Clear, cleared_area);
            frame.render_widget(block, popup_area);

            if nav_para.lines() > max_visible_rows as usize {
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
                        &mut nav_para.scroll_state(),
                    );
                }
            }
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
        let in_layer_list: bool = matches!(self.current_menu, TatMenu::LayerSelect) && matches!(self.focused_block, TatLayerSelectFocusedBlock::LayerList);
        let in_table: bool = matches!(self.current_menu, TatMenu::TableView);
        let in_layer_select: bool = matches!(self.current_menu, TatMenu::LayerSelect);
        let popup_open: bool = self.has_popup();

        match key.code {
            KeyCode::Char('c') if ctrl_down => self.close(),
            KeyCode::Char('q') | KeyCode::Esc => self.previous_menu(),
            KeyCode::Char('g') => self.delegate_jump(TatNavJump::First),
            KeyCode::Char('G') => self.delegate_jump(TatNavJump::Last),
            KeyCode::Char('k') | KeyCode::Up => self.delegate_jump(TatNavJump::UpOne),
            KeyCode::Char('j') | KeyCode::Down => self.delegate_jump(TatNavJump::DownOne),
            KeyCode::Char('d') if ctrl_down => self.delegate_jump(TatNavJump::DownHalfParagraph),
            KeyCode::Char('u') if ctrl_down => self.delegate_jump(TatNavJump::UpHalfParagraph),
            KeyCode::Char('f') if ctrl_down => self.delegate_jump(TatNavJump::DownParagraph),
            KeyCode::Char('b') if ctrl_down => self.delegate_jump(TatNavJump::UpParagraph),
            KeyCode::PageDown => self.delegate_jump(TatNavJump::DownParagraph),
            KeyCode::PageUp => self.delegate_jump(TatNavJump::UpParagraph),
            KeyCode::Char('L') =>  {
                if !popup_open {
                    self.show_gdal_log();
                }
            },
            KeyCode::Char('?') =>  {
                if !popup_open {
                    self.show_help();
                }
            },
            KeyCode::Char('h') | KeyCode::Left => {
                if !popup_open {
                    self.nav_left();
                }
            },
            KeyCode::Char('l') | KeyCode::Right => {
                if !popup_open {
                    self.nav_right();
                }
            }
            KeyCode::Char('0') | KeyCode::Home => {
                if in_table && !popup_open{
                    self.table.jump_first_column();
                }
            }
            KeyCode::Char('$') | KeyCode::End => {
                if in_table && !popup_open{
                    self.table.jump_last_column();
                }
            }
            KeyCode::Enter => {
                if !in_table && in_layer_list && !popup_open {
                    self.open_table();
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

    fn show_gdal_log(&mut self) {
        let lines = std::io::BufReader::new(File::open(format!("{}/tat_gdal.log", temp_dir().display())).unwrap()).lines();
        let mut text = String::from("");

        for line in lines.map_while(Result::ok) {
            text = format!("{}\n{}", text, line);
        }

        let p = TatNavigableParagraph::new(text);
        self.modal_popup = Some(
            TatPopup {
                paragraph: p,
                ptype: TatPopUpType::GdalLog,
            }
        )
    }

    fn show_help(&mut self) {
        let help_text = match self.current_menu {
            TatMenu::TableView => crate::shared::HELP_TEXT_TABLE,
            TatMenu::LayerSelect => crate::shared::HELP_TEXT_LAYERSELECT,
        }.to_string();

        let p = TatNavigableParagraph::new(help_text);
        self.modal_popup = Some(
            TatPopup {
                paragraph: p,
                ptype: TatPopUpType::Help,
            }
        )
    }

    fn cycle_block_selection(&mut self) {
        self.focused_block = match self.focused_block {
            TatLayerSelectFocusedBlock::LayerList => TatLayerSelectFocusedBlock::LayerInfo,
            TatLayerSelectFocusedBlock::LayerInfo => TatLayerSelectFocusedBlock::LayerList,
        }
    }

    fn open_table(&mut self) {
        self.table.set_layer(self.layerlist.current_layer().unwrap().clone());
        self.current_menu = TatMenu::TableView;
    }

    fn nav_left(&mut self) {
        match self.current_menu {
            TatMenu::LayerSelect => {
                if matches!(self.current_menu, TatMenu::LayerSelect) {
                    self.cycle_block_selection();
                }
            },
            TatMenu::TableView => self.table.nav_left(),
        }
    }

    fn nav_right(&mut self) {
        match self.current_menu {
            TatMenu::LayerSelect => {
                if matches!(self.current_menu, TatMenu::LayerSelect) {
                    self.cycle_block_selection();
                }
            },
            TatMenu::TableView => self.table.nav_right(),
        }
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

    fn delegate_jump(&mut self, conf: TatNavJump) {
        if let Some(pop) = &mut self.modal_popup {
            pop.paragraph_mut().jump(conf);
            return;
        }

        match self.current_menu {
            TatMenu::LayerSelect => {
                match self.focused_block {
                    TatLayerSelectFocusedBlock::LayerList => self.layerlist.jump(conf),
                    TatLayerSelectFocusedBlock::LayerInfo => self.layerlist.current_layer_info().jump(conf),
                }
            },
            TatMenu::TableView => self.table.jump_row(conf),
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

    fn render_table_view(&mut self, area: Rect, frame: &mut Frame) {
        let [table_area] = Layout::vertical([
            Constraint::Fill(1),
        ])
        .areas(area);

        frame.render_widget(& mut self.table, table_area);
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

        let max_visible_rows = if area.height >= 2 {
            area.height - 2
        } else {
            0
        };

        info.set_available_rows(max_visible_rows as usize);

        if info.lines() > max_visible_rows as usize {
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
                    &mut info.scroll_state(),
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

    fn draw_log(&self, area: Rect, frame: &mut Frame) {
        let lines = std::io::BufReader::new(File::open(format!("{}/tat_gdal.log", temp_dir().display())).unwrap()).lines();
        let mut text = String::from("");

        for line in lines.map_while(Result::ok) {
            text = format!("{}\n{}", text, line);
        }

        let block = Paragraph::new(text)
            .block(
                Block::default()
                    .title(crate::shared::TITLE_GDAL_LOG)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
            );

        let area = Tat::popup_area(area, 60, 60);

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
    }
}
