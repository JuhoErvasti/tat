use cli_log::debug;

use ratatui::{
    layout::{
        Constraint, Layout, Margin, Rect
    },
    style::Stylize,
    symbols::scrollbar::{
        DOUBLE_HORIZONTAL,
        DOUBLE_VERTICAL,
    },
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget
    }, Frame,
};
use gdal::vector::LayerAccess;

use crate::types::{
    TatLayer, TatNavHorizontal, TatNavVertical
};

/// Widget for displaying the attribute table
pub struct TatTable {
    table_state: TableState,
    top_fid: u64,
    first_column: u64,
    v_scroll: ScrollbarState,
    h_scroll: ScrollbarState,
    layer: Option<TatLayer>,
    table_rect: Rect,
    fid_col_rect: Rect,
}

impl TatTable {
    pub fn new() -> Self {
        let mut ts = TableState::default();
        ts.select_first();
        ts.select_first_column();

        Self {
            table_state: ts,
            top_fid: 1,
            first_column: 0,
            v_scroll: ScrollbarState::default(),
            h_scroll: ScrollbarState::default(),
            layer: Option::None,
            table_rect: Rect::default(),
            fid_col_rect: Rect::default(),
        }
    }

    pub fn set_layer(&mut self, layer: TatLayer) {
        self.v_scroll = ScrollbarState::new(layer.feature_count() as usize);
        self.h_scroll = ScrollbarState::new(layer.field_count() as usize);
        self.layer = Some(layer);
    }

    fn current_row(&self) -> u64 {
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

    fn current_column(&self) -> u64 {
        self.first_column + self.relative_highlighted_column()
    }

    pub fn current_column_name(&self) -> &str {
        if let Some(layer) = self.layer.as_ref() {
            let field_idx =self.current_column();
            if let Some(field) = layer.fields().get(field_idx as usize) {
                return field.name();
            }
        }

        "UNKNOWN"
    }

    pub fn relative_highlighted_column(&self) -> u64 {
        // see above (current_row)
        self.table_state.selected_column().unwrap() as u64
    }


    fn update_v_scrollbar(&mut self) {
        self.v_scroll = self.v_scroll.position((self.top_fid + self.current_row() - 1) as usize);
    }

    fn update_h_scrollbar(&mut self) {
        self.h_scroll = self.h_scroll.position(self.first_column as usize);
    }

    pub fn nav_h(&mut self, conf: TatNavHorizontal) {
        match conf {
            TatNavHorizontal::Home => {
                self.set_first_column(0);
                self.table_state.select_first_column();
                self.update_h_scrollbar();
            },
            TatNavHorizontal::End => {
                self.set_first_column(self.layer.as_ref().unwrap().field_count() as i64 - self.visible_columns() as i64);
                self.table_state.select_column(Some(self.visible_columns() as usize - 1));
                self.update_h_scrollbar();
            },
            TatNavHorizontal::RightOne => {
                let relative_col = self.relative_highlighted_column();
                let real_col = self.current_column();

                if relative_col == self.visible_columns() - 1 {
                    let cols =  self.layer.as_ref().unwrap().field_count();

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
        let mut nav_by = |amount: i64| {
            let row = self.current_row();

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
                if let Some(layer) = self.layer.as_ref() {
                    let jump_to_relative = if self.all_rows_visible() {
                        if layer.feature_count() > 0 { layer.feature_count() as i64 - 1 } else { 0 }
                    } else {
                        visible_rows - 1
                    };

                    self.set_top_fid(self.max_top_fid());
                    self.table_state.select(Some(jump_to_relative as usize ));
                }
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
            TatNavVertical::Specific(row) => {
                panic!("Not implemented! Cannot nav to row {}", row);
            },
        }

        self.update_v_scrollbar();
    }

    pub fn selected_fid(&self) -> u64 {
        self.top_fid + self.current_row()
    }

    pub fn selected_value(&self) -> Option<String> {
        if let Some(layer) = self.layer.as_ref() {
            layer.get_value(self.selected_fid(), self.current_column_name())
        } else {
            None
        }
    }

    fn set_first_column(&mut self, col: i64) {
        let max_first_column: i64 = self.layer.as_ref().unwrap().field_count() as i64 - self.visible_columns() as i64;

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
        self.layer.as_ref().unwrap().gdal_layer().feature_count() as i64 - self.visible_rows() as i64 + 1
    }

    fn current_layer(&self) -> TatLayer {
        if let Some(lyr) = &self.layer {
            lyr.clone()
        } else {
            panic!();
        }
    }

    /// Returns table based on current state
    fn get_table(&self) -> Table {
        let layer = self.current_layer();
        let gdal_layer = layer.gdal_layer();

        let block = Block::new()
            .title(
                Line::raw(
                    format!(
                        " {} (debug - visible_rows: {}, visible_columns: {} bottom_fid: {}, top_fid: {}, area_width: {})",
                        layer.name(),
                        self.visible_rows(),
                        self.visible_columns(),
                        self.bottom_fid(),
                        self.top_fid,
                        self.table_rect.width,
                    ),
                ).centered().bold().underlined(),
            )
            // .title(Line::raw(format!(" {} ", layer.name())))
            .borders(Borders::BOTTOM)
            .padding(Padding::top(1))
            .fg(crate::shared::palette::DEFAULT.default_fg)
            .title_bottom(Line::raw(crate::shared::SHOW_HELP).centered());

        let mut header_items: Vec<String> = vec![];

        let mut field_idx = 0;
        for field in layer.gdal_layer().defn().fields() {
            if self.visible_columns() == 0 {
                break;
            }

            if field_idx < self.first_column {
                field_idx += 1;
                continue;
            }

            if field_idx > self.first_column + self.visible_columns() - 1 {
                break;
            }

            field_idx += 1;

            header_items.push(field.name());
        }

        let mut rows: Vec<Row> = [].to_vec();
        let mut widths = [].to_vec();

        for _ in 0..self.visible_columns() {
            widths.push(Constraint::Fill(3));
        }

        for i in self.top_fid..self.bottom_fid() + 1 {
            let fid = match layer.feature_index().get(i as usize - 1) {
                Some(fid) => fid,
                None => break,
            };


            let feature = match gdal_layer.feature(*fid) {
                Some(f) => f,
                None => break,
            };

            let mut row_items: Vec<String> = vec![];

            for i in self.first_column..self.first_column + self.visible_columns() {
                let str_opt = match feature.field_as_string(i as i32) {
                    Ok(str_opt) => str_opt,
                    // TODO: should the GdalError here be handled differently?
                    Err(_) => Some(String::from(crate::shared::MISSING_VALUE)),
                };

                if let Some(str) = str_opt {
                    row_items.push(str);
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
            .block(block)
            .column_highlight_style(crate::shared::palette::DEFAULT.highlighted_darker_fg())
            .row_highlight_style(crate::shared::palette::DEFAULT.highlighted_darker_fg())
            .cell_highlight_style(crate::shared::palette::DEFAULT.selected_style())
            .column_spacing(1);

        table
    }

    pub fn table_rect(&self) -> Rect {
        self.table_rect
    }

    pub fn set_rects(&mut self, table_rect: Rect, fid_col_rect: Rect) {
        self.table_rect = table_rect;
        self.fid_col_rect = fid_col_rect;
        self.h_scroll = ScrollbarState::new(self.layer.as_ref().unwrap().field_count() as usize - self.visible_columns() as usize + 1);
    }

    fn visible_rows(&self) -> u64 {
        let value = if self.table_rect.height >= 5 {
            (self.table_rect.height - 5) as u64
        } else {
            0
        };

        if let Some(layer) = self.layer.as_ref() {
            if value > layer.feature_count() {
                return layer.feature_count();
            }
        }

        value
    }

    fn visible_columns(&self) -> u64 {
        if self.current_layer().field_count() * 30 < self.table_rect.width as u64 {
            self.current_layer().field_count() as u64
        } else {
            (self.table_rect.width / 30) as u64
        }
    }

    fn all_columns_visible(&self) -> bool {
        self.visible_columns() >= self.current_layer().field_count()
    }

    fn all_rows_visible(&self) -> bool {
        self.visible_rows() >= self.current_layer().feature_count()
    }

    fn show_h_scrollbar(&self) -> bool {
        !self.all_columns_visible()
    }

    fn show_v_scrollbar(&self) -> bool {
        true
    }

    pub fn render(&mut self, frame: &mut Frame) {
        // TODO: I think we have to render the Feature "Column" separately, not in the table
        // 1. allows distinguishing it visually
        // 2. makes the visible columns calculation more straightforward

        // TODO: If value will not fit the cell, distinguish it, for example with …
        let vert_scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some(DOUBLE_VERTICAL.begin))
                .end_symbol(Some(DOUBLE_VERTICAL.end));

        let [table_area, v_scroll_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(self.table_rect);

        let [table_area, h_scroll_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(table_area);

        // TODO: maybe: only show scrollbars when needed

        let horz_scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::HorizontalBottom)
                .begin_symbol(Some(DOUBLE_HORIZONTAL.begin))
                .end_symbol(Some(DOUBLE_HORIZONTAL.end));

        frame.render_stateful_widget(
            vert_scrollbar,
            v_scroll_area,
            &mut self.v_scroll,
        );

        frame.render_stateful_widget(
            horz_scrollbar,
            h_scroll_area,
            &mut self.h_scroll,
        );

        let table = self.get_table();
        frame.render_stateful_widget(
            table,
            table_area,
            &mut self.table_state.clone(),
        );
    }

    pub fn layer(&self) -> Option<&TatLayer> {
        self.layer.as_ref()
    }
}
