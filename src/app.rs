use cli_log::{error, info};
use std::{
    env::temp_dir, fs::File, io::{
        BufRead,
        Result,
    }, sync::mpsc::{self, Sender}
};

use cli_clipboard::{ClipboardContext, ClipboardProvider};

use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind
};
use ratatui::{
    layout::{
        Constraint,
        Flex,
        Layout,
        Margin,
        Rect,
    }, style::
        Stylize, symbols::{
        self,
        scrollbar::{DOUBLE_HORIZONTAL, DOUBLE_VERTICAL},
    }, text::Line, widgets::{
        Block,
        BorderType,
        Borders,
        Clear,
        ListState,
        Paragraph,
        Scrollbar,
        ScrollbarOrientation,
    }, DefaultTerminal, Frame
};
use unicode_segmentation::UnicodeSegmentation;
use crate::{
    dataset::{DatasetRequest, DatasetResponse}, layerlist::TatLayerList, navparagraph::TatNavigableParagraph, numberinput::{TatNumberInput, TatNumberInputResult}, table::TableRects, types::{TatNavHorizontal, TatNavVertical}
};
use crate::table::TatTable;

const BORDER_LAYER_INFO: symbols::border::Set = symbols::border::Set {
    top_left: symbols::line::ROUNDED.horizontal_down,
    top_right: symbols::line::NORMAL.horizontal_down,
    bottom_left: symbols::line::ROUNDED.horizontal_up,
    bottom_right: symbols::line::ROUNDED.horizontal_up,
    ..symbols::border::ROUNDED
};

const BORDER_PREVIEW_TABLE: symbols::border::Set = symbols::border::Set {
    top_right: symbols::line::NORMAL.vertical_left,
    bottom_left: symbols::line::ROUNDED.horizontal_up,
    ..symbols::border::ROUNDED
};

/// Specificies the available menus of the program
#[derive(PartialEq, Debug)]
enum TatMenu {
    MainMenu,
    TableView,
}

/// Specifies which section in the main menu has the focus
#[derive(PartialEq, Debug)]
enum TatMainMenuSectionFocus {
    LayerList,
    LayerInfo,
    PreviewTable,
}

/// Custom event enum which also wraps Crossterm events
#[derive(Debug)]
pub enum TatEvent {
    Keyboard(KeyEvent),
    Mouse(MouseEvent),
    Dataset(DatasetResponse),
}

/// This is the main widget of the program, initiating the rendering and primarily handling
/// key/mouse events.
pub struct TatApp {
    current_menu: TatMenu,
    quit: bool,
    modal_popup: Option<TatNavigableParagraph>,
    table: TatTable,
    layerlist: TatLayerList,
    focused_section: TatMainMenuSectionFocus,
    clip: Option<ClipboardContext>,
    table_area: Rect,
    number_input: Option<TatNumberInput>,
    clipboard_feedback: Option<String>,
    dataset_info_text: String,
}

impl TatApp {
    /// Constructs a new object
    pub fn new(dataset_request_tx: Sender<DatasetRequest>) -> Self {
        let mut ls = ListState::default();
        ls.select_first();

        let clip_res = ClipboardContext::new();
        let clip = match clip_res {
            Ok(_clip) => Some(_clip),
            Err(_) => None
        };

        dataset_request_tx.send(DatasetRequest::DatasetInfo).unwrap();
        dataset_request_tx.send(DatasetRequest::BuildLayers).unwrap();

        // TODO: disable for now
        // crossterm::execute!(std::io::stdout(), EnableMouseCapture).unwrap();

        Self {
            current_menu: TatMenu::MainMenu,
            quit: false,
            modal_popup: None,
            layerlist: TatLayerList::new(dataset_request_tx.clone()),
            table: TatTable::new(dataset_request_tx.clone()),
            focused_section: TatMainMenuSectionFocus::LayerList,
            clip,
            table_area: Rect::default(),
            number_input: None,
            clipboard_feedback: None,
            dataset_info_text: String::default(),
        }
    }

    /// Main execution loop of the program. The state of the program is rendered along with key and
    /// mouse events being handled
    pub fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<TatEvent>) -> Result<()> {
        while !self.quit {
            // TODO: don't unwrap yada yada
            match rx.recv().unwrap() {
                TatEvent::Keyboard(key) if key.kind == KeyEventKind::Press => self.handle_key(key),
                TatEvent::Mouse(mouse) => self.handle_mouse(mouse),
                TatEvent::Dataset(dataset_response) => self.handle_dataset(dataset_response),
                _ => continue,
            };

            terminal.draw(|frame| {
                self.render(frame);
            })?;
        }

        Ok(())
    }

    pub fn handle_dataset(&mut self, response: DatasetResponse) {
        match response {
            DatasetResponse::LayerInfos(layer_info) => {
                self.layerlist.set_infos(layer_info);

                if self.layerlist.current_layer_info_paragraph().is_none() {
                    self.layerlist.nav(TatNavVertical::First);
                }
            },
            DatasetResponse::DatasetInfo(info) => {
                self.dataset_info_text = info;
            },
            DatasetResponse::LayerSchemas(tat_layer_schemas) => {
                self.table.set_layer_schemas(tat_layer_schemas);
            },
            DatasetResponse::AttributeView(view) => {
                self.table.set_attribute_view(view);
            },
            DatasetResponse::AttributeViewUpdated => {
            },
            DatasetResponse::LayersBuilt => {
            },
        }
    }

    pub fn set_table_area(&mut self, area: Rect) {
        self.table_area = area;
    }

    /// Renders the current menu and any other active pop-ups or dialogs
    pub fn render(&mut self, frame: &mut Frame) {
        self.table_area = frame.area();

        match self.current_menu {
            TatMenu::MainMenu => self.render_main_menu(frame.area(), frame),
            TatMenu::TableView => self.render_table_view(frame),
        }

        self.render_popup(frame);
        self.render_number_input(frame);
        self.render_clipboard_feedback(frame);
    }

    /// Opens the table view menu
    pub fn open_table(&mut self) {
        self.table.set_rects(self.current_table_rects(false));
        self.current_menu = TatMenu::TableView;
    }

    /// Closes the table view menu
    pub fn close_table(&mut self) {
        self.table.reset();
        self.current_menu = TatMenu::MainMenu;
    }

    /// Sets currently selected layer's index
    pub fn set_layer_index(&mut self, idx: usize) {
        self.table.set_layer_index(idx);
    }

    /// Renders the clipboard feedback message (if any)
    fn render_clipboard_feedback(&mut self, frame: &mut Frame) {
        if let Some(feedback) = self.clipboard_feedback.as_mut() {
            let cleared_area = TatApp::number_input_area(frame.area(), 50);
            let block_area = cleared_area.inner(Margin { horizontal: 1, vertical: 1 });

            let block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(crate::shared::palette::DEFAULT.highlighted_style())
                        .border_type(BorderType::Rounded)
                        .title_bottom(Line::raw( " <press any key to continue> ").centered());

            let text_area = block_area.inner(Margin { horizontal: 1, vertical: 1 });

            frame.render_widget(Clear, cleared_area);
            frame.render_widget(block, block_area);
            frame.render_widget(feedback.as_str(), text_area);
        }
    }

    /// Renders the number input dialog (if any)
    fn render_number_input(&mut self, frame: &mut Frame) {
        if let Some(number_input) = self.number_input.as_mut() {
            let cleared_area = TatApp::number_input_area(frame.area(), 50);
            let block_area = cleared_area.inner(Margin { horizontal: 1, vertical: 1 });

            let block = Block::default()
                        .title(Line::raw(" Jump To Feature ").bold().underlined().centered())
                        .borders(Borders::ALL)
                        .border_style(crate::shared::palette::DEFAULT.highlighted_style())
                        .border_type(BorderType::Rounded)
                        .title_bottom(Line::raw( " <press Enter to jump, q to cancel> ").centered());

            let input_area = block_area.inner(Margin { horizontal: 1, vertical: 1 });

            frame.render_widget(Clear, cleared_area);
            frame.render_widget(block, block_area);
            number_input.render(frame, input_area);
        }
    }

    /// Renders the current active pop-up dialog (if any)
    fn render_popup(&mut self, frame: &mut Frame) {
        if let Some(popup) = &mut self.modal_popup {
            let cleared_area = TatApp::popup_area(frame.area(), 70, 70);
            let popup_area = cleared_area.inner(
                Margin { horizontal: 1, vertical: 1 }
            );

            let (
                visible_cols,
                has_h_scrollbar,
                visible_rows,
                has_v_scrollbar,
            ) = TatApp::text_area_dimensions(
                &popup_area,
                popup.max_line_len() as i64,
                popup.total_lines() as i64,
            );

            popup.set_visible_rows(visible_rows as usize);
            popup.set_visible_cols(visible_cols as usize);

            let title: &str = if let Some(title) = popup.title() {
                title.as_str()
            } else {
                "UNTITLED"
            };


            let block = popup.paragraph()
                .block(
                    Block::default()
                        .title(Line::raw(title).bold().underlined().centered())
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

    /// Attempts to copy a value to the system keyboard
    fn copy_table_value_to_clipboard(&mut self) {
        if let Some(clip) = self.clip.as_mut() {
            if let Some(text_to_copy) = self.table.selected_value() {
                match clip.set_contents(text_to_copy.to_string()) {
                    Ok(()) => {
                        let postscript = " copied to clipboard!";
                        let max_len = 50;
                        if text_to_copy.chars().count() < max_len {
                            self.set_clipboard_feedback(format!("\"{text_to_copy}\"{postscript}"));
                        } else {
                            let graph = text_to_copy.graphemes(true);
                            let substring: String = graph.into_iter().take(max_len).collect();

                            self.set_clipboard_feedback(format!("\"{}…\"{}", &substring, postscript));
                        }
                        return;
                    }
                    Err(e) => {
                        self.clipboard_feedback = Some(format!("ERROR! Could not copy to clipboard: {}", e.to_string()));
                    }
                }
            } else {
                self.clipboard_feedback = Some(format!("NULL value NOT copied to clipboard!"));
            }
        } else {
            self.clipboard_feedback = Some("ERROR! Could not copy to clipboard: clipboard context does not exist!".to_string());
        }
    }

    /// Terminates the program
    fn close(&mut self) {
        self.quit = true;
    }

    /// Activates the clipboard feedback, which is shown in a small pop-up
    fn set_clipboard_feedback(&mut self, text: String) {
        self.clipboard_feedback = Some(text);
    }

    /// Handles incoming mouse events and delegates to other widgets
    fn handle_mouse(&mut self, event: MouseEvent) {
        match event.kind {
            MouseEventKind::ScrollUp => self.delegate_nav_v(TatNavVertical::MouseScrollUp),
            MouseEventKind::ScrollDown => self.delegate_nav_v(TatNavVertical::MouseScrollDown),
            _ => (),
        }
    }

    /// Handles incoming key events and delegates to other widgets
    fn handle_key(&mut self, key: KeyEvent) {
        let ctrl_down: bool = key.modifiers.contains(KeyModifiers::CONTROL);
        let in_layer_list: bool = matches!(self.current_menu, TatMenu::MainMenu) && matches!(self.focused_section, TatMainMenuSectionFocus::LayerList);
        let in_table: bool = matches!(self.current_menu, TatMenu::TableView);
        let in_main_menu: bool = matches!(self.current_menu, TatMenu::MainMenu);
        let in_preview_table: bool = matches!(self.focused_section, TatMainMenuSectionFocus::PreviewTable);
        let popup_open: bool = self.modal_popup.is_some();

        if in_table {
            if let Some(number_input) = & mut self.number_input {
                let res = number_input.key_press(key.code, ctrl_down);

                match res {
                    TatNumberInputResult::Close => self.number_input = None,
                    TatNumberInputResult::Accept(num) => {
                        self.table.nav_v(TatNavVertical::Specific(num));
                        self.number_input = None;
                    }
                    _ => (),
                }

                return;
            }
        }


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
            KeyCode::Char('c') if ctrl_down && in_table => {
                self.copy_table_value_to_clipboard();
                return;
            },
            KeyCode::Char('y') if in_table => {
                self.copy_table_value_to_clipboard();

                return;
            },
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
                    TatMenu::MainMenu => {
                        if !in_table && !popup_open && (in_preview_table || in_layer_list && self.layerlist.layer_index().is_some()) {
                            self.open_table();
                        }
                    },
                    TatMenu::TableView => {
                        self.show_full_value_popup();
                    },
                }
            },
            KeyCode::Tab => {
                if in_main_menu && !popup_open {
                    self.cycle_section_selection(false);
                }
            },
            KeyCode::BackTab => {
                if in_main_menu && !popup_open {
                    self.cycle_section_selection(true);
                }
            },
            KeyCode::Char(':') =>  {
                if in_table {
                    self.number_input = Some(
                        TatNumberInput::new(),
                    )
                }
            },
            _ => {},
        }

        if self.clipboard_feedback.is_some() {
            self.clipboard_feedback = None;
        }
    }

    /// Opens a pop-up which displays the full value of the selected cell in the table
    fn show_full_value_popup(&mut self) {
        let value = if let Some(_value) = self.table.selected_value() {
            _value
        } else {
            crate::shared::MISSING_VALUE.to_string()
        };

        let title = format!(
                " Feature {} - Value of \"{}\" ",
                self.table.current_row(),
                self.table.current_column_name().unwrap_or("UNKNOWN COLUMN"),
            );

        self.modal_popup = Some(
            TatNavigableParagraph::new(
                format!(
                    "{}",
                    value,
                )
            ).with_title(title)
        );
    }

    /// Opens the debug log in which all internal logging messages are written
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
                text = format!("Could not open file: {}", e.to_string());
            },
        };

        self.modal_popup = Some(
            TatNavigableParagraph::new(
                    text,
                ).with_title(crate::shared::TITLE_DEBUG_LOG.to_string()
            )
        );
    }

    /// Opens the GDAL log in a pop-up in which any direct GDAL output is written
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

        self.modal_popup = Some(
            TatNavigableParagraph::new(
                    text,
                ).with_title(crate::shared::TITLE_GDAL_LOG.to_string()
            )
        );
    }

    /// Shows the help pop-up dialog according to the current menu
    fn show_help(&mut self) {
        let help_text = match self.current_menu {
            TatMenu::TableView => crate::shared::HELP_TEXT_TABLE,
            TatMenu::MainMenu => crate::shared::HELP_TEXT_MAINMENU,
        }.to_string();

        self.modal_popup = Some(
            TatNavigableParagraph::new(
                    help_text,
                ).with_title(crate::shared::TITLE_HELP.to_string()
            )
        );
    }

    /// Cycles through the available sections in the main menu (non-looping)
    fn cycle_section_selection(&mut self, back: bool) {
        self.focused_section = match self.focused_section {
            TatMainMenuSectionFocus::LayerList if back => return,
            TatMainMenuSectionFocus::LayerInfo if back => TatMainMenuSectionFocus::LayerList,
            TatMainMenuSectionFocus::PreviewTable if back => TatMainMenuSectionFocus::LayerInfo,

            TatMainMenuSectionFocus::LayerList => TatMainMenuSectionFocus::LayerInfo,
            TatMainMenuSectionFocus::LayerInfo => TatMainMenuSectionFocus::PreviewTable,
            TatMainMenuSectionFocus::PreviewTable => return,
        }
    }

    /// Closes any active pop-up dialog
    fn close_popup(&mut self) {
        self.modal_popup = None;
    }

    /// Goes back in menus, also closing pop-ups and ultimately terminating program if in the main
    /// menu
    fn previous_menu(&mut self) {
        if self.modal_popup.is_some() {
            self.close_popup();
            return;
        }

        if self.clipboard_feedback.is_some() {
            return;
        }

        match self.current_menu {
            TatMenu::TableView => {
                self.table.reset();
                self.current_menu = TatMenu::MainMenu;
            },
            TatMenu::MainMenu => self.close(),
        }
    }

    /// Delegates a vertical navigation event to the active widget
    fn delegate_nav_v(&mut self, conf: TatNavVertical) {
        if let Some(pop) = &mut self.modal_popup {
            pop.nav_v(conf);
            return;
        }

        match self.current_menu {
            TatMenu::MainMenu => {
                match self.focused_section {
                    TatMainMenuSectionFocus::LayerList => {
                        self.layerlist.nav(conf);

                        if let Some(lyr_i) = self.layerlist.layer_index() {
                            self.table.set_layer_index(lyr_i);
                        }
                        self.table.reset();
                    },
                    TatMainMenuSectionFocus::LayerInfo => {
                        if let Some(para) = self.layerlist.current_layer_info_paragraph() {
                            para.nav_v(conf);
                        }
                    },
                    TatMainMenuSectionFocus::PreviewTable => {
                        self.table.nav_v(conf);
                    }
                }
            },
            TatMenu::TableView => self.table.nav_v(conf),
        }
    }

    /// Delegates a horizontal navigation event to the active widget
    fn delegate_nav_h(&mut self, conf: TatNavHorizontal) {
        if let Some(pop) = &mut self.modal_popup {
            pop.nav_h(conf);
            return;
        }

        match self.current_menu {
            TatMenu::MainMenu => {
                match self.focused_section {
                    TatMainMenuSectionFocus::LayerList => return,
                    TatMainMenuSectionFocus::LayerInfo => {
                        if let Some(para) = self.layerlist.current_layer_info_paragraph() {
                            para.nav_h(conf);
                        }
                    },
                    TatMainMenuSectionFocus::PreviewTable => {
                        self.table.nav_h(conf);
                    }
                }
            },
            TatMenu::TableView => self.table.nav_h(conf),
        }
    }

    /// Renders all the sections of the main menu
    fn render_main_menu(&mut self, area: Rect, frame: &mut Frame) {
        let (header_area, dataset_area, list_area, info_area, preview_table_area) = TatApp::main_menu_areas(&area);

        TatApp::render_title(header_area, frame);
        self.render_dataset_info(dataset_area, frame);
        self.layerlist.render(list_area, frame, matches!(self.focused_section, TatMainMenuSectionFocus::LayerList) && self.modal_popup.is_none());
        self.render_layer_info(info_area, frame,  matches!(self.focused_section, TatMainMenuSectionFocus::LayerInfo));

        self.table_area = preview_table_area;
        self.table.set_rects(self.current_table_rects(true));

        let block = Block::new()
            .title(
                Line::raw(
                    " Preview Table ",
                ).bold().underlined().left_aligned().fg(
                    if matches!(self.focused_section, TatMainMenuSectionFocus::PreviewTable) {crate::shared::palette::DEFAULT.highlighted_fg} else {crate::shared::palette::DEFAULT.default_fg}
                    ),
                )
            .border_style(if matches!(self.focused_section, TatMainMenuSectionFocus::PreviewTable) {crate::shared::palette::DEFAULT.highlighted_style()} else {crate::shared::palette::DEFAULT.default_style()})
            .title_bottom(Line::raw(" <Enter> to open full table ").centered())
            .border_set(BORDER_PREVIEW_TABLE)
            .borders(Borders::BOTTOM | Borders::RIGHT | Borders::TOP);

        self.table.render_preview(frame);

        frame.render_widget(block, preview_table_area);
    }

    /// Renders the title of the program
    fn render_title(area: Rect, frame: &mut Frame) {
        frame.render_widget(
            Paragraph::new(crate::shared::TITLE_PROGRAM)
                .bold()
                .centered()
                .fg(crate::shared::palette::DEFAULT.default_fg),
            area,
        );
    }

    /// Renders the dataset information
    fn render_dataset_info(&mut self, area: Rect, frame: &mut Frame) {
        let block = Block::new()
            .fg(crate::shared::palette::DEFAULT.default_fg)
            .title_top(Line::raw(crate::shared::TITLE_DATASET_INFO).underlined().bold())
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_set(symbols::border::ROUNDED)
            .title_top(Line::raw(crate::shared::SHOW_HELP).centered());

        frame.render_widget(
            Paragraph::new(self.dataset_info_text.as_str())
                .fg(crate::shared::palette::DEFAULT.default_fg)
                .block(block),
            area
        );

    }

    /// Returns the table view Menu
    fn render_table_view(&mut self, frame: &mut Frame) {
        self.table.set_rects(self.current_table_rects(false));

        self.table.render(frame);
    }

    /// Renders the layer information section
    fn render_layer_info(&mut self, area: Rect, frame: &mut Frame, selected: bool) {
        let border_style = if selected && self.modal_popup.is_none() {
            crate::shared::palette::DEFAULT.highlighted_style()
        } else {
            crate::shared::palette::DEFAULT.default_style()
        };


        let block = Block::bordered()
            .title(Line::raw(crate::shared::TITLE_LAYER_INFO).bold().underlined())
            .fg(crate::shared::palette::DEFAULT.default_fg)
            .border_set(BORDER_LAYER_INFO)
            .border_style(border_style);

        if let Some(info) = self.layerlist.current_layer_info_paragraph() {
            frame.render_widget(
                info.paragraph().block(block),
                area,
            );

            let (
                visible_cols,
                has_h_scrollbar,
                visible_rows,
                has_v_scrollbar,
            ) = TatApp::text_area_dimensions(
                &area,
                info.max_line_len() as i64,
                info.total_lines() as i64,
            );

            info.set_visible_rows(visible_rows as usize);
            info.set_visible_cols(visible_cols as usize);

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
    }

    /// Returns the areas for the table either in preview or full mode
    pub fn table_rects(rect: Rect, preview: bool) -> TableRects {
        let rects = if preview {
            let [_, table_rect_temp] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1),
            ]).areas(rect);

            let [fid_col_area, mut table_rect] = Layout::horizontal([
                Constraint::Length(11),
                Constraint::Fill(1),
            ])
            .areas(table_rect_temp);

            table_rect.height += 1;

            (table_rect, fid_col_area, Rect::default(), Rect::default())
        } else {
            let [table_rect_temp, scroll_h_area] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(1),
            ]).areas(rect);

            let [feature_col_area, table_rect, scroll_v_area] = Layout::horizontal([
                Constraint::Length(11),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .areas(table_rect_temp);

            (table_rect, feature_col_area, scroll_v_area, scroll_h_area)
        };

        rects
    }

    /// Returns the areas for the table based on the current state
    fn current_table_rects(&self, preview: bool) -> TableRects {
        TatApp::table_rects(self.table_area, preview)
    }


    /// Returns the rects for each section in the main menu
    fn main_menu_areas(area: &Rect) -> (Rect, Rect, Rect, Rect, Rect) {
        let [header_area, dataset_area, layer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(4),
            Constraint::Fill(1),
        ])
        .areas(*area);

        let [list_area, info_area, preview_table_area] =
            Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Fill(2),
                Constraint::Fill(4),
        ]).areas(layer_area);

        (header_area, dataset_area, list_area, info_area, preview_table_area)
    }

    /// Returns a flat rect in the center of the screen
    fn number_input_area(area: Rect, percent_x: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Length(5)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);

        area
    }

    /// Returns a rect in the center of the screen
    fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);

        area
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

    pub fn table(&self) -> &TatTable {
        &self.table
    }

}

#[cfg(test)]
mod test {
    use std::{fs::{remove_file, OpenOptions}, path::Path};

    #[allow(unused)]
    use super::*;

    use crate::fixtures::{basic_app, table_rects, TatTestStructure, TatTestUtils};

    use crossterm::event::KeyEventState;
    use rstest::*;

    #[rstest]
    fn test_new(basic_app: (TatTestStructure, TatApp)) {
        let (test, t) = basic_app;

        assert_eq!(t.current_menu, TatMenu::MainMenu);
        assert_eq!(t.quit, false);
        assert_eq!(t.modal_popup, None);
        assert_eq!(t.focused_section, TatMainMenuSectionFocus::LayerList);
        assert!(t.table_area.is_empty());
        assert_eq!(t.number_input, None);
        assert_eq!(t.clipboard_feedback, None);
        test.terminate();
    }

    #[rstest]
    fn test_copy_table_value_to_clipboard(basic_app: (TatTestStructure, TatApp)) {
        let (test, mut t) = basic_app;

        if t.clip.is_none() {
            return;
        }

        let old_contents: String;

        {
            let clip = t.clip.as_mut().unwrap();
            old_contents = clip.get_contents().unwrap();
        }

        {
            TatTestUtils::request_attribute_view_update_mocked(0, 1, test.ds_request_tx.clone());
            TatTestUtils::wait_attribute_view_update(&test.tatevent_rx);
            t.copy_table_value_to_clipboard();
            let clip = t.clip.as_mut().unwrap();
            let contents = clip.get_contents().unwrap();
            assert_eq!(contents, "POINT (0 0)".to_string());
        }

        {
            let clip = t.clip.as_mut().unwrap();
            clip.set_contents(old_contents).unwrap();
        }
        test.terminate();
    }

    #[rstest]
    fn test_handle_mouse(basic_app: (TatTestStructure, TatApp), table_rects: TableRects) {
        let (test, mut t) = basic_app;
        t.layerlist.set_available_rows(10);

        t.handle_mouse(MouseEvent { kind: MouseEventKind::ScrollDown, column: 0, row: 0, modifiers: KeyModifiers::NONE });
        assert_eq!(t.layerlist.layer_index(), Some(3));
        t.handle_mouse(MouseEvent { kind: MouseEventKind::ScrollUp, column: 0, row: 0, modifiers: KeyModifiers::NONE });
        assert_eq!(t.layerlist.layer_index(), Some(0));
        t.handle_mouse(MouseEvent { kind: MouseEventKind::ScrollDown, column: 0, row: 0, modifiers: KeyModifiers::NONE });
        t.handle_mouse(MouseEvent { kind: MouseEventKind::ScrollDown, column: 0, row: 0, modifiers: KeyModifiers::NONE });

        t.table.set_rects(table_rects);
        t.current_menu = TatMenu::TableView;

        assert_eq!(t.table.current_row(), 1);
        t.handle_mouse(MouseEvent { kind: MouseEventKind::ScrollDown, column: 0, row: 0, modifiers: KeyModifiers::NONE });
        assert_eq!(t.table.current_row(), 6);
        t.handle_mouse(MouseEvent { kind: MouseEventKind::ScrollUp, column: 0, row: 0, modifiers: KeyModifiers::NONE });
        assert_eq!(t.table.current_row(), 1);
        test.terminate();
    }

    #[rstest]
    fn test_show_full_value_popup(basic_app: (TatTestStructure, TatApp)) {
        let (test, mut t) = basic_app;
        TatTestUtils::request_attribute_view_update_mocked(0, 1, test.ds_request_tx.clone());
        TatTestUtils::wait_attribute_view_update(&test.tatevent_rx);
        t.show_full_value_popup();

        assert!(t.modal_popup.is_some());
        assert_eq!(t.modal_popup.as_ref().unwrap().text(), "POINT (0 0)".to_string());
        let title = t.modal_popup.unwrap().title().unwrap().clone();
        assert_eq!(title, " Feature 1 - Value of \"geom\" ".to_string());
        test.terminate();
    }

    #[rstest]
    fn test_show_gdal_log(basic_app: (TatTestStructure, TatApp)) {
        {
            use std::io::prelude::Write;

            let (test, mut t) = basic_app;


            let path = format!("{}/tat_gdal.log", temp_dir().display());

            if Path::new(&path).exists() {
                remove_file(&path).unwrap();
            }

            File::create(format!("{}/tat_gdal.log", temp_dir().display())).unwrap();

            match OpenOptions::new().write(true).open(path.clone()) {
                Ok(mut file) => {
                    match writeln!(file, "output from gdal") {
                        Ok(()) => (),
                        Err(e) => panic!("{}", e.to_string()),
                    }
                },
                Err(e) => panic!("{}", e.to_string()),
            }
            t.show_gdal_log();

            let expected = "\noutput from gdal";
            assert!(t.modal_popup.is_some());
            assert_eq!(t.modal_popup.as_ref().unwrap().text(), expected);

            test.terminate();
        }
    }

    #[rstest]
    fn test_show_help(basic_app: (TatTestStructure, TatApp)) {
        let (test, mut t) = basic_app;
        t.show_help();

        assert!(t.modal_popup.is_some());
        assert!(t.modal_popup.as_ref().unwrap().text().starts_with("Keybinds for Main Menu"));

        let popup = t.modal_popup.as_ref().unwrap();
        let title = popup.title().unwrap();
        assert_eq!(title.as_str(), " Help ");

        t.current_menu = TatMenu::TableView;
        t.show_help();

        assert!(t.modal_popup.as_ref().unwrap().text().starts_with("Keybinds for Attribute Table"));

        test.terminate();
    }

    #[rstest]
    fn test_previous_menu(basic_app: (TatTestStructure, TatApp)) {
        let (test, mut t) = basic_app;

        t.show_help();
        assert!(t.modal_popup.is_some());
        t.previous_menu();
        assert!(t.modal_popup.is_none());

        t.current_menu = TatMenu::TableView;
        t.previous_menu();
        assert_eq!(t.current_menu, TatMenu::MainMenu);
        assert_eq!(t.quit, false);

        t.previous_menu();
        assert_eq!(t.quit, true);
        test.terminate();
    }

    #[rstest]
    fn test_delegate_nav_v(basic_app: (TatTestStructure, TatApp), table_rects: TableRects) {
        let (test, mut t) = basic_app;
        t.layerlist.set_available_rows(10);

        t.delegate_nav_v(TatNavVertical::DownParagraph);
        assert_eq!(t.layerlist.layer_index(), Some(4));

        t.table.set_rects(table_rects);
        t.current_menu = TatMenu::TableView;

        assert_eq!(t.table.current_row(), 1);
        t.delegate_nav_v(TatNavVertical::DownParagraph);
        assert_eq!(t.table.current_row(), 16);
        test.terminate();
    }

    #[rstest]
    fn test_delegate_nav_h(basic_app: (TatTestStructure, TatApp), table_rects: TableRects) {
        let (test, mut t) = basic_app;
        t.table.set_rects(table_rects);
        t.current_menu = TatMenu::TableView;

        assert_eq!(t.table.current_column_name(), Some("geom"));
        t.delegate_nav_h(TatNavHorizontal::RightOne);
        assert_eq!(t.table.current_column_name(), Some("field"));
        test.terminate();
    }

    #[rstest]
    fn test_table_rect(basic_app: (TatTestStructure, TatApp)) {
        let (test, mut t) = basic_app;
        t.table_area = Rect {
            x: 0,
            y: 0,
            width: 400,
            height: 300,
        };

        {
            let (table_rect, feature_col_area, scroll_v_area, scroll_h_area) = t.current_table_rects(false);

            assert_eq!(
                table_rect,
                Rect {
                    x: 11,
                    y: 0,
                    width: 388,
                    height: 299,
                },
            );

            assert_eq!(
                feature_col_area,
                Rect {
                    x: 0,
                    y: 0,
                    width: 11,
                    height: 299,
                },
            );

            assert_eq!(
                scroll_v_area,
                Rect {
                    x: 399,
                    y: 0,
                    width: 1,
                    height: 299,
                },
            );

            assert_eq!(
                scroll_h_area,
                Rect {
                    x: 0,
                    y: 299,
                    width: 400,
                    height: 1,
                },
            );
        }

        {
            let (table_rect, feature_col_area, scroll_v_area, scroll_h_area) = t.current_table_rects(true);

            assert_eq!(
                table_rect,
                Rect {
                    x: 11,
                    y: 1,
                    width: 389,
                    height: 300,
                },
            );

            assert_eq!(
                feature_col_area,
                Rect {
                    x: 0,
                    y: 1,
                    width: 11,
                    height: 299,
                },
            );

            assert!(scroll_v_area.is_empty());
            assert!(scroll_h_area.is_empty());
        }
        test.terminate();
    }

    #[rstest]
    fn test_main_menu_areas() {
        let tat_area = Rect {
            x: 0,
            y: 0,
            width: 400,
            height: 300,
        };

        let (header_area, dataset_area, list_area, info_area, preview_table_area) = TatApp::main_menu_areas(&tat_area);

        assert_eq!(
            header_area,
            Rect {
                x: 0,
                y: 0,
                width: 400,
                height: 2,
            },
        );

        assert_eq!(
            dataset_area,
            Rect {
                x: 0,
                y: 2,
                width: 400,
                height: 4,
            },
        );

        assert_eq!(
            list_area,
            Rect {
                x: 0,
                y: 6,
                width: 57,
                height: 294,
            },
        );

        assert_eq!(
            info_area,
            Rect {
                x: 57,
                y: 6,
                width: 114,
                height: 294,
            },
        );

        assert_eq!(
            preview_table_area,
            Rect {
                x: 171,
                y: 6,
                width: 229,
                height: 294,
            },
        );
    }

    #[rstest]
    fn test_number_input_area() {
        let tat_area = Rect {
            x: 0,
            y: 0,
            width: 400,
            height: 300,
        };

        let ni_area = TatApp::number_input_area(tat_area, 30);
        assert_eq!(
            ni_area,
            Rect {
                x: 140,
                y: 148,
                width: 120,
                height: 5,
            },
        );
    }

    #[rstest]
    fn test_popup_area() {
        let tat_area = Rect {
            x: 0,
            y: 0,
            width: 400,
            height: 300,
        };

        let popup_area = TatApp::popup_area(tat_area, 70, 70);
        assert_eq!(
            popup_area,
            Rect {
                x: 60,
                y: 45,
                width: 280,
                height: 210,
            },
        );
    }

    // use insta::assert_snapshot;
    // use ratatui::{backend::TestBackend, Terminal};
    //
    // #[rstest]
    // fn test_render_main_menu(basic_app: (TatTestStructure, TatApp)) {
    //     let (test, mut t) = basic_app;
    //     let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
    //
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Tab, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Char('l'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Right, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Right, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Tab, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Right, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('?'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::BackTab, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::BackTab, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Char('j'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('j'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('j'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('j'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Tab, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Tab, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::End, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Home, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::BackTab, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('l'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('h'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('$'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('0'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('?'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Right, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Down, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Tab, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Tab, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!(terminal.backend());
    //     test.terminate();
    // }
    //
    // #[rstest]
    // fn test_render_table(basic_app: (TatTestStructure, TatApp)) {
    //     let (test, mut t) = basic_app;
    //     let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
    //
    //     t.open_table();
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("open_table", terminal.backend());
    //
    //     t.show_full_value_popup();
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("show_value_popup", terminal.backend());
    //
    //     if t.clip.is_some() {
    //         t.copy_table_value_to_clipboard();
    //         terminal.draw(|frame| {t.render(frame)}).unwrap();
    //         assert_snapshot!("copied_to_clipboard", terminal.backend());
    //
    //         t.handle_key(KeyEvent { code: KeyCode::Char('b'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //         terminal.draw(|frame| {t.render(frame)}).unwrap();
    //         assert_snapshot!("any_key_closed_clipboard_feedback", terminal.backend());
    //     }
    //
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("close_value_popup", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("back_to_main_menu", terminal.backend());
    //
    //     t.open_table();
    //     t.show_help();
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("open_table_again", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Char('G'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("select_last_layer", terminal.backend());
    //
    //     t.open_table();
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("open_table_with_last_layer", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('G'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("goto_last_feature", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('g'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("goto_first_feature", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char(':'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("open_jump_to_line", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('5'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("type_5_to_jump_to_line", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('2'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("type_2_to_jump_to_line", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("execute_jump_to_line", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char(':'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("close_jump_to_line", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char(':'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Char('0'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("try_enter_0_as_first_in_jump_to_line", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('9'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("type_9_to_jump_to_line", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Char('8'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Char('7'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Char('6'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Char('5'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Char('4'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("long_number_in_jump_to_line", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Backspace, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("backspace_in_jump_to_line", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Left, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Left, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Delete, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("delete_in_jump_to_line", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Home, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Delete, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("home_in_jump_to_line", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::End, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Backspace, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("end_in_jump_to_line", terminal.backend());
    //
    //     t.handle_key(KeyEvent { code: KeyCode::Home, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     t.handle_key(KeyEvent { code: KeyCode::Char('5'), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press, state: KeyEventState::NONE });
    //     terminal.draw(|frame| {t.render(frame)}).unwrap();
    //     assert_snapshot!("enter_in_middle_in_jump_to_line", terminal.backend());
    //     test.terminate();
    // }
}
