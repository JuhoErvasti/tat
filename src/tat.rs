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

use crate::{layerlist::TatLayerList, types::TatNavJump};
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

/// This holds the program's state.
pub struct Tat {
    pub current_menu: TatMenu,
    pub quit: bool,
    log_visible: bool,
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
            log_visible: false,
            table: TatTable::new(),
            layerlist: TatLayerList::new(&ds),
            focused_block: TatLayerSelectFocusedBlock::LayerList,
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
        let in_layer_list: bool = matches!(self.current_menu, TatMenu::LayerSelect) && matches!(self.focused_block, TatLayerSelectFocusedBlock::LayerList);
        let in_table: bool = matches!(self.current_menu, TatMenu::TableView);
        let in_layer_select: bool = matches!(self.current_menu, TatMenu::LayerSelect);

        match key.code {
            KeyCode::Char('c') => self.close(),
            KeyCode::Char('q') | KeyCode::Esc => self.previous_menu(),
            KeyCode::Char('h') | KeyCode::Left => self.nav_left(),
            KeyCode::Char('l') | KeyCode::Right => self.nav_right(),
            KeyCode::Char('g') => self.delegate_jump(TatNavJump::First),
            KeyCode::Char('G') => self.delegate_jump(TatNavJump::Last),
            KeyCode::Char('k') | KeyCode::Up => self.delegate_jump(TatNavJump::UpOne),
            KeyCode::Char('j') | KeyCode::Down => self.delegate_jump(TatNavJump::DownOne),
            KeyCode::Char('d') if ctrl_down => self.delegate_jump(TatNavJump::DownHalfParagraph),
            KeyCode::Char('u') if ctrl_down => self.delegate_jump(TatNavJump::UpHalfParagraph),
            KeyCode::Char('f') if ctrl_down => self.delegate_jump(TatNavJump::DownParagraph),
            KeyCode::Char('b') if ctrl_down => self.delegate_jump(TatNavJump::UpParagraph),
            KeyCode::Char('L') => self.log_visible = !self.log_visible,
            KeyCode::Char('0') => {
                if in_table {
                    self.table.jump_first_column();
                }
            }
            KeyCode::Char('$') => {
                if in_table {
                    self.table.jump_last_column();
                }
            }
            KeyCode::Enter => {
                if !in_table && in_layer_list {
                    self.open_table();
                }
            },
            KeyCode::Tab | KeyCode::BackTab => {
                if in_layer_select {
                    self.cycle_block_selection();
                }
            }
            _ => {},
        }
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

    fn delegate_jump(&mut self, conf: TatNavJump) {
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
            Paragraph::new("Terminal Attribute Table")
                .bold()
                .centered()
                .fg(crate::shared::palette::DEFAULT.default_fg),
            area,
        );
    }

    fn render_dataset_info(&mut self, area: Rect, frame: &mut Frame) {
        let block = Block::new()
            .fg(crate::shared::palette::DEFAULT.default_fg)
            .title_top(Line::raw(" Dataset ").left_aligned().underlined().bold())
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_set(symbols::border::ROUNDED)
            .title_top(Line::raw(crate::shared::SHOW_HELP).centered());
        // TODO: actually implement this help menu, something like this:
        //     let text = match self.current_menu {
        //         TatMenu::LayerSelect => {
        //             "<up, k> <down, j>: browse layers | <enter> open layer table | <q, ESC, ctrl+c> quit program"
        //         },
        //         TatMenu::TableView => {
        //             "<left, h> <down, j> <up, k> <right, l>: browse table | <q, esc> return to layer selection | <ctrl+c> quit program"
        //         }
        //     };

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
        self.layerlist.render(list_area, frame, matches!(self.focused_block, TatLayerSelectFocusedBlock::LayerList));
        // frame.render_widget(&mut self.layerlist, list_area);
        self.render_layer_info(info_area, frame,  matches!(self.focused_block, TatLayerSelectFocusedBlock::LayerInfo));
    }


    fn render_layer_info(&mut self, area: Rect, frame: &mut Frame, selected: bool) {
        // TODO: better paletting system
        let border_style = if selected {
            crate::shared::palette::DEFAULT.highlighted_style()
        } else {
            crate::shared::palette::DEFAULT.default_style()
        };


        let block = Block::bordered()
            .title(Line::raw(" Layer Information ").underlined().bold())
            .fg(crate::shared::palette::DEFAULT.default_fg)
            .border_set(LAYER_INFO_BORDER)
            .border_style(border_style);

        let info = self.layerlist.current_layer_info();

        frame.render_widget(
            info.paragraph().block(block),
            area,
        );

        let max_visible_rows = area.height - 2; // account for borders
        info.set_available_rows(max_visible_rows as usize);

        if info.lines() > max_visible_rows as usize {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .style(border_style)
                .begin_symbol(Some(DOUBLE_VERTICAL.begin))
                .end_symbol(Some(DOUBLE_VERTICAL.end));

            frame.render_stateful_widget(
                scrollbar,
                area.inner(
                Margin { horizontal: 1, vertical: 1 }),
                &mut info.scroll_state(),
            );
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
                    .title("GDAL Log")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
            );

        let area = Tat::popup_area(area, 60, 60);

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
    }
}
