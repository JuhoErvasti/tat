use cli_log::debug;

use ratatui::{
    layout::{
        Constraint, Rect
    },
    style::Stylize,
    symbols::{self, scrollbar::{
        DOUBLE_HORIZONTAL,
        DOUBLE_VERTICAL,
    }},
    text::Line,
    widgets::{
        Block, Borders, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState
    }, Frame,
};
use gdal::{vector::LayerAccess, Dataset, Metadata};
use unicode_segmentation::UnicodeSegmentation;

use crate::types::{
    TatNavHorizontal, TatNavVertical
};
use crate::layer::TatLayer;

pub const FID_COLUMN_BORDER_FULL: symbols::border::Set = symbols::border::Set {
    bottom_right: symbols::line::HORIZONTAL_UP,
    ..symbols::border::ROUNDED
};

pub const FID_COLUMN_BORDER_PREVIEW: symbols::border::Set = symbols::border::Set {
    top_right: symbols::line::HORIZONTAL_DOWN,
    ..FID_COLUMN_BORDER_FULL
};

const MIN_COLUMN_LENGTH: i32 = 30;
const THEORETICAL_MAX_COLUMN_UTF8_BYTE_SIZE: i32 = MIN_COLUMN_LENGTH * 4;


pub type TableRects = (Rect, Rect, Rect, Rect);

/// Widget for displaying the attribute table
pub struct TatTable {
    table_state: TableState,
    gdal_ds: &'static Dataset,
    top_fid: u64,
    first_column: u64,
    v_scroll: ScrollbarState,
    h_scroll: ScrollbarState,
    layer_index: usize,
    layers: Vec<TatLayer>,
    table_rect: Rect,
    fid_col_rect: Rect,
    v_scroll_area: Rect,
    h_scroll_area: Rect,
}

impl TatTable {
    pub fn new(ds: &'static Dataset) -> Self {
        let mut ts = TableState::default();
        ts.select_first();
        ts.select_first_column();

        Self {
            table_state: ts,
            top_fid: 1,
            first_column: 0,
            v_scroll: ScrollbarState::default(),
            h_scroll: ScrollbarState::default(),
            layer_index: 0,
            table_rect: Rect::default(),
            fid_col_rect: Rect::default(),
            v_scroll_area: Rect::default(),
            h_scroll_area: Rect::default(),
            layers: TatTable::layers_from_ds(ds),
            gdal_ds: ds,
        }
    }

    pub fn layers_from_ds(ds: &'static Dataset) -> Vec<TatLayer> {
        let mut layers: Vec<TatLayer> = vec![];
        for (i, _) in ds.layers().enumerate() {
            let mut lyr = TatLayer::new(&ds, i);
            lyr.build_feature_index();
            layers.push(lyr);
        }

        layers
    }


    pub fn dataset_info_text(&self) -> String {
        format!(
            "- URI: \"{}\"\n- Driver: {} ({})",
            self.gdal_ds.description().unwrap(),
            self.gdal_ds.driver().long_name(),
            self.gdal_ds.driver().short_name(),
        )
    }

    pub fn set_layer_index(&mut self, idx: usize) {
        self.layer_index = idx;
    }

    fn layer(&self) -> &TatLayer {
        self.layers.get(self.layer_index).unwrap()
    }

    /// Returns currently selected row's fid
    fn current_row(&self) -> u64 {
        self.top_fid + self.relative_highlighted_row()
    }

    fn current_column(&self) -> u64 {
        self.first_column + self.relative_highlighted_column()
    }

    pub fn current_column_name(&self) -> &str {
        let field_idx =self.current_column();
        if let Some(field) = self.layer().fields().get(field_idx as usize) {
            return field.name();
        }

        "UNKNOWN"
    }

    fn relative_highlighted_row(&self) -> u64 {
        // the idea is to avoid unwrapping/whatever everywhere
        // TODO: not sure how idiomatic this is in Rust, maybe reconsider
        // the approach, feels like this is just trying skirt around
        // the whole point of using Options and such, but idk
        if let Some(sel) = self.table_state.selected() {
            return sel as u64;
        } else {
            return 0;
        }
    }

    pub fn relative_highlighted_column(&self) -> u64 {
        // see above (relative_highlighted_row)
        self.table_state.selected_column().unwrap() as u64
    }

    fn update_v_scrollbar(&mut self) {
        self.v_scroll = ScrollbarState::new(self.layer().feature_count() as usize - self.visible_rows() as usize + 1);
        self.v_scroll = self.v_scroll.position(self.top_fid as usize);
    }

    fn update_h_scrollbar(&mut self) {
        self.h_scroll = ScrollbarState::new(self.layer().field_count() as usize - self.visible_columns() as usize + 1);
        self.h_scroll = self.h_scroll.position(self.first_column as usize);
    }

    pub fn nav_h(&mut self, conf: TatNavHorizontal) {
        if self.visible_columns() <= 0 {
            return;
        }

        match conf {
            TatNavHorizontal::Home => {
                self.set_first_column(0);
                self.table_state.select_first_column();
                self.update_h_scrollbar();
            },
            TatNavHorizontal::End => {
                self.set_first_column(self.layer().field_count() as i64 - self.visible_columns() as i64);
                self.table_state.select_column(Some(self.visible_columns() as usize - 1));
                self.update_h_scrollbar();
            },
            TatNavHorizontal::RightOne => {
                let relative_col = self.relative_highlighted_column();
                let real_col = self.current_column();

                if relative_col == self.visible_columns() - 1 {
                    let cols =  self.layer().field_count();

                    if real_col == cols {
                        self.update_h_scrollbar();
                        return;
                    } else {
                        self.set_first_column(self.first_column as i64 + 1);
                    }
                    self.update_h_scrollbar();
                    return;
                }
                self.table_state.select_next_column();
            }
            TatNavHorizontal::LeftOne => {
                let relative_col = self.relative_highlighted_column();
                if relative_col == 0 {
                    if self.first_column == 0 {
                        self.update_h_scrollbar();
                        return;
                    } else {
                        self.set_first_column(self.first_column as i64 - 1);
                    }
                    self.update_h_scrollbar();
                    return;
                }
                self.table_state.select_previous_column();
            }
        }
    }

    pub fn nav_v(&mut self, conf: TatNavVertical) {
        let visible_rows = self.visible_rows() as i64;
        if visible_rows <= 0 {
            return;
        }
        let mut nav_by = |amount: i64| {
            let row = self.relative_highlighted_row();

            if amount > 0 {
                if row + amount as u64 >= visible_rows as u64 {
                    self.set_top_fid(self.top_fid as i64 + amount as i64);
                } else {
                    self.table_state.scroll_down_by(amount as u16);
                }
            } else {
                let abs_amount = amount * -1;
                if (row as i16 - abs_amount as i16) < 0 {
                    self.set_top_fid(self.top_fid as i64 - abs_amount as i64);
                } else {
                    self.table_state.scroll_up_by(abs_amount as u16);
                }
            }
        };

        match conf {
            TatNavVertical::First => {
                self.set_top_fid(1);
                self.table_state.select_first();
            },
            TatNavVertical::Last => {
                let jump_to_relative = if self.all_rows_visible() {
                    if self.layer().feature_count() > 0 { self.layer().feature_count() as i64 - 1 } else { 0 }
                } else {
                    visible_rows - 1
                };

                self.set_top_fid(self.max_top_fid());
                self.table_state.select(Some(jump_to_relative as usize ));
            },
            TatNavVertical::DownOne => {
                nav_by(1);
            },
            TatNavVertical::UpOne => {
                nav_by(-1);
            },
            TatNavVertical::DownHalfParagraph => {
                nav_by(visible_rows / 2 );
            },
            TatNavVertical::UpHalfParagraph => {
                nav_by(-(visible_rows / 2));
            },
            TatNavVertical::DownParagraph => {
                nav_by(visible_rows);
            },
            TatNavVertical::UpParagraph => {
                nav_by(-(visible_rows));
            },
            TatNavVertical::MouseScrollDown => {
                nav_by(visible_rows / 3 );
            },
            TatNavVertical::MouseScrollUp => {
                nav_by(-(visible_rows / 3));
            },
            TatNavVertical::Specific(fid) => {
                if fid >= self.layer().feature_count() as i64 {
                    self.nav_v(TatNavVertical::Last);
                    return;
                }
                if self.fid_visible(fid as i64) {
                    self.table_state.select(Some(self.fid_relative_row(fid).unwrap() as usize));
                } else {
                    self.set_top_fid(fid as i64 - self.relative_highlighted_row() as i64);
                    self.table_state.select(Some(self.fid_relative_row(fid).unwrap() as usize));
                }
            },
        }

        self.update_v_scrollbar();
    }

    pub fn selected_fid(&self) -> u64 {
        self.top_fid + self.relative_highlighted_row()
    }

    pub fn selected_value(&self) -> Option<String> {
        self.layer().get_value_by_id(self.selected_fid(), self.current_column() as i32)
    }

    fn fid_relative_row(&self, fid: i64) -> Result<u64, &str> {
        if !self.fid_visible(fid) {
            return Err("Fid is not visible!");
        }

        Ok((fid - self.top_fid as i64) as u64)
    }

    fn fid_visible(&self, fid: i64) -> bool {
        let top = self.top_fid as i64;
        let bottom = self.bottom_fid() as i64;

        return fid >= top && fid <= bottom;
    }

    fn set_first_column(&mut self, col: i64) {
        let max_first_column: i64 = self.layer().field_count() as i64 - self.visible_columns() as i64;

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
        self.top_fid + self.visible_rows() as u64 - 1
    }

    pub fn reset(&mut self) {
        self.top_fid = 1;
        self.first_column = 0;
        self.table_state.select_first_column();
        self.table_state.select_first();
        self.table_rect = Rect::default();
    }

    fn max_top_fid(&self) -> i64 {
        self.layer().gdal_layer().feature_count() as i64 - self.visible_rows() as i64 + 1
    }

    /// Returns table based on current state
    fn get_table(&self) -> Table {
        // TODO: render the title separately in self.render()
        // ALSO THE BOTTOM LINE AND HINT TO OPEN HELP FROM ?

        // let block = Block::new()
        //     .title(
        //         Line::raw(
        //             format!(
        //                 " {} (debug - visible_rows: {}, visible_columns: {} bottom_fid: {}, top_fid: {}, area_width: {})",
        //                 layer.name(),
        //                 self.visible_rows(),
        //                 self.visible_columns(),
        //                 self.bottom_fid(),
        //                 self.top_fid,
        //                 self.table_rect.width,
        //             ),
        //         ).centered().bold().underlined(),
        //     )
        //     // .title(Line::raw(format!(" {} ", layer.name())))
        //     .borders(Borders::BOTTOM)
        //     .border_set(symbols::border::PLAIN)
        //     .padding(Padding::top(1))
        //     .fg(crate::shared::palette::DEFAULT.default_fg)
        //     .title_bottom(Line::raw(crate::shared::SHOW_HELP).centered());

        let mut header_items: Vec<String> = vec![];

        for i in self.first_column..self.first_column + self.visible_columns() {
            if let Some(field_name) = self.layer().field_name_by_id(i as i32) {
                header_items.push(field_name);
            } else {
                panic!();
            }
        }

        let mut rows: Vec<Row> = [].to_vec();
        let mut widths = [].to_vec();

        for _ in 0..self.visible_columns() {
            widths.push(Constraint::Fill(3));
        }

        // TODO: use the iterator maybe
        for i in self.top_fid..self.bottom_fid() + 1 {
            let fid = match self.layer().feature_index().get(i as usize - 1) {
                Some(fid) => fid,
                None => break,
            };

            let mut row_items: Vec<String> = vec![];

            for i in self.first_column..self.first_column + self.visible_columns() {
                if let Some(str) = self.layer().get_value_by_id(*fid, i as i32) {
                    // this is (maybe a premature (lol)) optimization fast path
                    // since str.len() is O(1) and str.chars().count() is O(n),
                    // we check first if a theoretically full 4 byte UTF-8 would overflow
                    // which would mean that the string definitely will overflow no matter
                    // what. only then we check with the actual string "length" i.e. the
                    // number of actual symbols, not unicode code points
                    let squish_contents: bool = if str.len() > THEORETICAL_MAX_COLUMN_UTF8_BYTE_SIZE as usize {
                        true
                    } else if str.chars().count() > MIN_COLUMN_LENGTH as usize {
                        true
                    } else {
                        false
                    };

                    if squish_contents {
                        let graph = str.graphemes(true);
                        let substring: String = graph.into_iter().take(MIN_COLUMN_LENGTH as usize).collect();
                        row_items.push(format!("{substring}…",));
                    } else {
                        row_items.push(str);
                    }
                } else {
                    row_items.push(String::from(crate::shared::MISSING_VALUE));
                }

            }

            rows.push(Row::new(row_items));
        }

        let header = Row::new(header_items);

        // TODO: don't just construct all the rows every time we render the table
        let table = Table::new(rows, widths)
            .header(header.underlined())
            .style(crate::shared::palette::DEFAULT.default_style())
            .column_highlight_style(
                crate::shared::palette::DEFAULT.highlighted_darker_fg()
            )
            .row_highlight_style(
                crate::shared::palette::DEFAULT.highlighted_darker_fg()
            )
            .cell_highlight_style(
                crate::shared::palette::DEFAULT.selected_style()
            )
            .column_spacing(1);

        table
    }

    pub fn table_rect(&self) -> Rect {
        self.table_rect
    }

    pub fn set_rects(&mut self, (table_rect, fid_col_rect, v_scroll_area, h_scroll_area): TableRects) {
        let old_fid = self.current_row();
        let first_update = self.table_rect.is_empty();

        let rect_changed = if self.table_rect != table_rect {
            true
        } else { false };

        if rect_changed {
            self.table_rect = table_rect;
            self.fid_col_rect = fid_col_rect;
            self.v_scroll_area = v_scroll_area;
            self.h_scroll_area = h_scroll_area;

            self.update_v_scrollbar();
            self.update_h_scrollbar();

            if self.bottom_fid() + self.top_fid >= self.layer().feature_count() {
                self.set_top_fid(self.max_top_fid());
            }

            if !first_update {
                self.nav_v(TatNavVertical::Specific(old_fid as i64));
            }
        }
    }

    fn visible_rows(&self) -> u64 {
        let value = if self.table_rect.height >= 4 {
            (self.table_rect.height - 4) as u64
        } else {
            0
        };

        if value > self.layer().feature_count() {
            return self.layer().feature_count();
        }

        value
    }

    fn visible_columns(&self) -> u64 {
        if self.layer().field_count() * (MIN_COLUMN_LENGTH as u64) < self.table_rect.width as u64 {
            self.layer().field_count() as u64
        } else {
            (self.table_rect.width / MIN_COLUMN_LENGTH as u16) as u64
        }
    }

    fn all_columns_visible(&self) -> bool {
        self.visible_columns() >= self.layer().field_count()
    }

    fn all_rows_visible(&self) -> bool {
        self.visible_rows() >= self.layer().feature_count()
    }

    fn show_h_scrollbar(&self) -> bool {
        !self.all_columns_visible()
    }

    fn show_v_scrollbar(&self) -> bool {
        true
    }

    fn render_fid_column(&mut self, frame: &mut Frame, preview: bool) {
        if self.fid_col_rect.height <= 2 {
            return;
        }

        let borders = if preview { Borders::RIGHT | Borders::BOTTOM } else { Borders::BOTTOM | Borders::RIGHT };
        let border_symbols = if preview { FID_COLUMN_BORDER_PREVIEW } else { FID_COLUMN_BORDER_FULL };

        let block = Block::new()
            .border_set(border_symbols)
            .borders(borders)
            .fg(crate::shared::palette::DEFAULT.default_fg);

        let fid_header = Line::raw(
            "Feature"
        ).bold().underlined().fg(crate::shared::palette::DEFAULT.default_fg);

        let header_area = if preview {
            Rect {
                x: self.fid_col_rect.x,
                y: self.fid_col_rect.y + 1,
                height: 1,
                width: 11,
            }
        } else { 
            Rect {
                x: self.fid_col_rect.x,
                y: self.fid_col_rect.y + 2,
                height: 1,
                width: 11,
            }
        };

        let block_rect = if preview {
            Rect {
                x: self.fid_col_rect.x,
                y: self.fid_col_rect.y,
                height: self.fid_col_rect.height + 1,
                width: self.fid_col_rect.width,
            }
        } else {
            Rect {
                x: self.fid_col_rect.x,
                y: self.fid_col_rect.y + 2,
                height: self.fid_col_rect.height - 2,
                width: self.fid_col_rect.width,
            }
        };

        frame.render_widget(block, block_rect);
        frame.render_widget(fid_header, header_area);

        for (i, fid) in (self.top_fid..=self.bottom_fid()).enumerate() {
            let line = Line::raw(
                format!(
                    "{}",
                    fid,
                ),
            ).bold().fg(crate::shared::palette::DEFAULT.default_fg);
            let rect = Rect {
                x: self.fid_col_rect.x,
                y: self.fid_col_rect.y + i as u16 + if preview { 2 } else { 3 },
                height: 1,
                width: 11,
            };

            frame.render_widget(line, rect);
        }
    }

    pub fn render_preview(&mut self, frame: &mut Frame) {
        if self.fid_col_rect.is_empty() || self.table_rect.is_empty() {
            return;
        }

        self.render_fid_column(frame, true);

        let table = self.get_table();

        let table_widget_rect = Rect {
            x: self.table_rect.x,
            y: self.table_rect.y + 1,
            width: self.table_rect.width - 1,
            height: self.table_rect.height,
        };

        frame.render_stateful_widget(
            table,
            table_widget_rect,
            &mut self.table_state.clone(),
        );
    }

    pub fn render(&mut self, frame: &mut Frame) {
        self.render_fid_column(frame, false);

        // TODO: handle potential overflows

        // TODO: If value will not fit the cell, distinguish it, for example with the … symbol
        let vert_scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some(DOUBLE_VERTICAL.begin))
                .end_symbol(Some(DOUBLE_VERTICAL.end));

        // TODO: maybe: only show scrollbars when needed

        let horz_scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::HorizontalBottom)
                .begin_symbol(Some(DOUBLE_HORIZONTAL.begin))
                .end_symbol(Some(DOUBLE_HORIZONTAL.end));

        frame.render_stateful_widget(
            vert_scrollbar,
            self.v_scroll_area,
            &mut self.v_scroll,
        );

        frame.render_stateful_widget(
            horz_scrollbar,
            self.h_scroll_area,
            &mut self.h_scroll,
        );

        let table_widget_rect = Rect {
            x: self.table_rect.x,
            y: self.table_rect.y + 2,
            width: self.table_rect.width,
            height: self.table_rect.height - 3,
        };

        let table = self.get_table();
        frame.render_stateful_widget(
            table,
            table_widget_rect,
            &mut self.table_state.clone(),
        );
    }
}
