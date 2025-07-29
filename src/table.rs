use std::sync::mpsc::Sender;

use cli_log::{error, info};
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
use unicode_segmentation::UnicodeSegmentation;

use crate::{dataset::GdalRequest, layer::TatFeature, types::{
    TatNavHorizontal, TatNavVertical
}};
use crate::layer::TatLayer;

pub const FEATURE_COLUMN_BORDER_FULL: symbols::border::Set = symbols::border::Set {
    bottom_right: symbols::line::HORIZONTAL_UP,
    ..symbols::border::ROUNDED
};

pub const FEATURE_COLUMN_BORDER_PREVIEW: symbols::border::Set = symbols::border::Set {
    top_right: symbols::line::HORIZONTAL_DOWN,
    ..FEATURE_COLUMN_BORDER_FULL
};

const MIN_COLUMN_LENGTH: i32 = 30;
const THEORETICAL_MAX_COLUMN_UTF8_BYTE_SIZE: i32 = MIN_COLUMN_LENGTH * 4;


pub type TableRects = (Rect, Rect, Rect, Rect);

/// Widget for displaying the attribute table
pub struct TatTable {
    table_state: TableState,

    /// The uppermost visible row.
    top_row: u64,
    first_column: u64,
    v_scroll: ScrollbarState,
    h_scroll: ScrollbarState,
    layer_index: usize,
    layers: Vec<TatLayer>, // TODO: consider making a HashMap
    table_rect: Rect,
    feature_col_rect: Rect,
    v_scroll_area: Rect,
    h_scroll_area: Rect,
    gdal_request_tx: Sender<GdalRequest>,
}

impl TatTable {
    /// Constructs new object
    pub fn new(gdal_request_tx: Sender<GdalRequest>) -> Self {
        let mut ts = TableState::default();
        ts.select_first();
        ts.select_first_column();

        gdal_request_tx.send(GdalRequest::AllLayers).unwrap();

        Self {
            table_state: ts,
            top_row: 1,
            first_column: 0,
            v_scroll: ScrollbarState::default(),
            h_scroll: ScrollbarState::default(),
            layer_index: 0,
            table_rect: Rect::default(),
            feature_col_rect: Rect::default(),
            v_scroll_area: Rect::default(),
            h_scroll_area: Rect::default(),
            layers: vec![],
            gdal_request_tx,
        }
    }

    pub fn add_layer(&mut self, lyr: TatLayer) {
        let i = lyr.index();
        self.layers.push(lyr);

        self.gdal_request_tx.send(
            GdalRequest::FidCache(i),
        ).unwrap();
    }

    pub fn add_feature(&mut self, lyr_index: usize, row: usize, f: TatFeature) {
        self.layers.get_mut(lyr_index).unwrap().add_feature(row, f);
    }

    pub fn add_fid_cache(&mut self, layer_index: usize, cache: Vec<u64>) {
        // TODO: think about this, shouldn't TatDataset have the caches?
        // ALSO  FIXME: i think this'll screw up if there's a layer filter
        self.layers.get_mut(layer_index).unwrap().set_fid_cache(cache);
    }

    /// Sets currently selected layer's index
    pub fn set_layer_index(&mut self, idx: usize) {
        self.layer_index = idx;
    }

    /// Returns currently selected row's index
    pub fn current_row(&self) -> u64 {
        self.top_row + self.relative_highlighted_row()
    }

    /// Returns the name of the currently selected column
    pub fn current_column_name(&self) -> Option<String> {
        Some(
            self.layer()?
            .field_name_by_id(self.current_column() as i32)?
        )
    }

    /// Returns the index of the highlighted column from the current visible column
    pub fn relative_highlighted_column(&self) -> u64 {
        // see above (relative_highlighted_row)
        self.table_state.selected_column().unwrap() as u64
    }

    /// Handles horizontal navigation (in columns)
    pub fn nav_h(&mut self, conf: TatNavHorizontal) {
        if self.layer().is_none() {
            return;
        }

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
                self.set_first_column(self.layer().unwrap().field_count() as i64 - self.visible_columns() as i64);
                self.table_state.select_column(Some(self.visible_columns() as usize - 1));
                self.update_h_scrollbar();
            },
            TatNavHorizontal::RightOne => {
                let relative_col = self.relative_highlighted_column();
                let real_col = self.current_column();

                if relative_col == self.visible_columns() - 1 {
                    let cols =  self.layer().unwrap().field_count();

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

    /// Handles vertical navigation (in rows)
    pub fn nav_v(&mut self, conf: TatNavVertical) {
        if self.layer().is_none() {
            return;
        }

        let visible_rows = self.visible_rows() as i64;
        if visible_rows <= 0 {
            return;
        }
        let mut nav_by = |amount: i64| {
            let row = self.relative_highlighted_row();

            if amount > 0 {
                if row + amount as u64 >= visible_rows as u64 {
                    self.set_top_row(self.top_row as i64 + amount as i64);
                } else {
                    self.table_state.scroll_down_by(amount as u16);
                }
            } else {
                let abs_amount = amount * -1;
                if (row as i16 - abs_amount as i16) < 0 {
                    self.set_top_row(self.top_row as i64 - abs_amount as i64);
                } else {
                    self.table_state.scroll_up_by(abs_amount as u16);
                }
            }
        };

        match conf {
            TatNavVertical::First => {
                self.set_top_row(1);
                self.table_state.select_first();
            },
            TatNavVertical::Last => {
                let jump_to_relative = if self.all_rows_visible() {
                    if self.layer().unwrap().feature_count() > 0 { self.layer().unwrap().feature_count() as i64 - 1 } else { 0 }
                } else {
                    visible_rows - 1
                };

                self.set_top_row(self.max_top_row());
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
            TatNavVertical::Specific(row) => {
                if row >= self.layer().unwrap().feature_count() as i64 {
                    self.nav_v(TatNavVertical::Last);
                    return;
                }
                if self.row_visible(row as i64) {
                    self.table_state.select(Some(self.feature_relative_row(row).unwrap() as usize));
                } else {
                    self.set_top_row(row as i64 - self.relative_highlighted_row() as i64);
                    self.table_state.select(Some(self.feature_relative_row(row).unwrap() as usize));
                }
            },
        }

        self.update_v_scrollbar();
    }

    /// Returns the currently selected cell's value as a string (if any)
    pub fn selected_value(&self) -> Option<&str> {
        self.layer()?.get_value_by_row(self.current_row() as usize, self.current_column() as usize)
    }

    /// Resets the table's state
    pub fn reset(&mut self) {
        self.top_row = 1;
        self.first_column = 0;
        self.table_state.select_first_column();
        self.table_state.select_first();
        self.table_rect = Rect::default();
    }

    /// Sets the areas which the table renders itself in
    pub fn set_rects(&mut self, (table_rect, feature_col_rect, v_scroll_area, h_scroll_area): TableRects) {
        let old_row = self.current_row();
        let first_update = self.table_rect.is_empty();

        let rect_changed = if self.table_rect != table_rect {
            true
        } else { false };

        if rect_changed {
            if self.layer().is_none() {
                return;
            }

            self.table_rect = table_rect;
            self.feature_col_rect = feature_col_rect;
            self.v_scroll_area = v_scroll_area;
            self.h_scroll_area = h_scroll_area;

            self.update_v_scrollbar();
            self.update_h_scrollbar();

            if self.bottom_row() + self.top_row >= self.layer().unwrap().feature_count() {
                self.set_top_row(self.max_top_row());
            }

            if !first_update {
                self.nav_v(TatNavVertical::Specific(old_row as i64));
            }
        }
    }

    /// Renders the table in preview format
    pub fn render_preview(&mut self, frame: &mut Frame) {
        if self.feature_col_rect.is_empty() || self.table_rect.is_empty() {
            return;
        }

        self.render_feature_column(frame, true);

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

    /// Renders the table fully
    pub fn render(&mut self, frame: &mut Frame) {
        self.render_feature_column(frame, false);

        let vert_scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some(DOUBLE_VERTICAL.begin))
                .end_symbol(Some(DOUBLE_VERTICAL.end));

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
            height: self.table_rect.height - if self.table_rect.height >= 3 { 3 } else { self.table_rect.height },
        };

        let table = self.get_table();
        frame.render_stateful_widget(
            table,
            table_widget_rect,
            &mut self.table_state.clone(),
        );

        // HACK: this is really hacky, probably table should only have one rect to begin with and then the table_rect
        // and fid_col_rect are calculated from that in here, not in Tat
        let union = self.table_rect.union(self.feature_col_rect);
        let block = Block::new()
            .title(
                Line::raw(
                    format!("{}", if self.layer().is_some() { self.layer().unwrap().name() } else {"NO LAYER!!!"})
                ).centered().bold().underlined(),
            )
            .title_bottom(
                Line::raw(
                    crate::shared::SHOW_HELP
                ).centered(),
            )
            .borders(Borders::BOTTOM)
            .border_style(crate::shared::palette::DEFAULT.default_style());

        frame.render_widget(block, union);
    }

    /// Returns currently selected layer
    pub fn layer(&self) -> Option<&TatLayer> {
        Some(self.layers.get(self.layer_index)?)
    }

    /// Returns the currently selected column index which can be used in TatLayer
    fn current_column(&self) -> u64 {
        self.first_column + self.relative_highlighted_column()
    }

    /// Returns the index of the highlighted row from the current visible rows
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

    /// Updates the state of the vertical scrollbar
    fn update_v_scrollbar(&mut self) {
        if self.layer().is_none() {
            return;
        }

        self.v_scroll = ScrollbarState::new(self.layer().unwrap().feature_count() as usize - self.visible_rows() as usize + 1);
        self.v_scroll = self.v_scroll.position(self.top_row as usize);
    }

    /// Updates the state of the horizontal scrollbar
    fn update_h_scrollbar(&mut self) {
        if self.layer().is_none() {
            return;
        }

        self.h_scroll = ScrollbarState::new(self.layer().unwrap().field_count() as usize - self.visible_columns() as usize + 1);
        self.h_scroll = self.h_scroll.position(self.first_column as usize);
    }

    /// Returns the relative row of a feature in the currently visible rows
    fn feature_relative_row(&self, row: i64) -> Result<u64, &str> {
        if !self.row_visible(row) {
            return Err("Feature is not visible!");
        }

        Ok((row - self.top_row as i64) as u64)
    }

    /// Returns whether the given row is currently visible
    fn row_visible(&self, row: i64) -> bool {
        let top = self.top_row as i64;
        let bottom = self.bottom_row() as i64;

        return row >= top && row <= bottom;
    }

    /// Sets the column which is displayed first
    fn set_first_column(&mut self, col: i64) {
        if self.layer().is_none() {
            return;
        }

        let max_first_column: i64 = self.layer().unwrap().field_count() as i64 - self.visible_columns() as i64;

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

    /// Sets the column which is displayed first
    fn set_top_row(&mut self, row: i64) {
        if row == self.top_row as i64 {
            return;
        }

        if self.max_top_row() <= 1 {
            self.top_row = 1;
            return;
        }

        if row >= self.max_top_row() {
            self.top_row = self.max_top_row() as u64;
            return;
        }

        if row <= 1 {
            self.top_row = 1;
            return;
        }

        self.top_row = row as u64;
    }

    /// Returns the currently visible bottom row
    fn bottom_row(&self) -> u64 {
        self.top_row + self.visible_rows() as u64 - 1
    }

    /// Returns the row which if the top row all other rows will be visible as well
    fn max_top_row(&self) -> i64 {
        if self.layer().is_none() {
            return 1;
        }

        self.layer().unwrap().feature_count() as i64 - self.visible_rows() as i64 + 1
    }

    /// Returns table based on current state
    fn get_table(&self) -> Table {
        if self.layer().is_none() {
            return Table::default();
        }

        let layer = self.layer().unwrap();

        let mut header_items: Vec<String> = vec![];

        for i in self.first_column..self.first_column + self.visible_columns() {
            if let Some(field_name) = layer.field_name_by_id(i as i32) {
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

        for i in self.top_row..self.bottom_row() + 1 {
            let mut row_items: Vec<&str> = vec![];

            for j in self.first_column..self.first_column + self.visible_columns() {
                if let Some(str) = layer.get_value_by_row(j as usize, j as usize) {
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
                        // TODO: HANDLE THIS
                        row_items.push(str);
                    } else {
                        row_items.push(str);
                    }
                } else {
                    let fid = match layer.fid_cache().get(i as usize - 1) {
                        Some(fid) => fid,
                        None => break,
                    };

                    self.gdal_request_tx.send(
                        GdalRequest::Feature(
                            layer.index(),
                            i as usize,
                            *fid,
                        )
                    ).unwrap();

                    row_items.push(crate::shared::MISSING_VALUE);
                }

            }

            rows.push(Row::new(row_items));
        }

        let header = Row::new(header_items);

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

    /// Returns the number of rows currently visible
    fn visible_rows(&self) -> u64 {
        if self.layer().is_none() {
            return 0;
        }

        let value = if self.table_rect.height >= 4 {
            (self.table_rect.height - 4) as u64
        } else {
            0
        };

        if value > self.layer().unwrap().feature_count() {
            return self.layer().unwrap().feature_count();
        }

        value
    }

    /// Returns the number of columns currently visible
    fn visible_columns(&self) -> u64 {
        if self.layer().is_none() {
            return 0;
        }

        if self.layer().unwrap().field_count() * (MIN_COLUMN_LENGTH as u64) < self.table_rect.width as u64 {
            self.layer().unwrap().field_count() as u64
        } else {
            (self.table_rect.width / MIN_COLUMN_LENGTH as u16) as u64
        }
    }

    /// Returns whether all rows are currently visible
    fn all_rows_visible(&self) -> bool {
        if self.layer().is_none() {
            return true;
        }

        self.visible_rows() >= self.layer().unwrap().feature_count()
    }

    /// Returns the "feature" column, i.e. the indexes of the rows in which the features are
    /// rendered
    fn render_feature_column(&mut self, frame: &mut Frame, preview: bool) {
        if self.feature_col_rect.height <= 2 {
            return;
        }

        let borders = if preview { Borders::RIGHT | Borders::BOTTOM } else { Borders::BOTTOM | Borders::RIGHT };
        let border_symbols = if preview { FEATURE_COLUMN_BORDER_PREVIEW } else { FEATURE_COLUMN_BORDER_FULL };

        let block = Block::new()
            .border_set(border_symbols)
            .borders(borders)
            .fg(crate::shared::palette::DEFAULT.default_fg);

        let fid_header = Line::raw(
            "Feature"
        ).bold().underlined().fg(crate::shared::palette::DEFAULT.default_fg);

        let header_area = if preview {
            Rect {
                x: self.feature_col_rect.x,
                y: self.feature_col_rect.y + 1,
                height: 1,
                width: 11,
            }
        } else { 
            Rect {
                x: self.feature_col_rect.x,
                y: self.feature_col_rect.y + 2,
                height: 1,
                width: 11,
            }
        };

        let block_rect = if preview {
            Rect {
                x: self.feature_col_rect.x,
                y: self.feature_col_rect.y,
                height: self.feature_col_rect.height + 1,
                width: self.feature_col_rect.width,
            }
        } else {
            Rect {
                x: self.feature_col_rect.x,
                y: self.feature_col_rect.y + 2,
                height: self.feature_col_rect.height - 2,
                width: self.feature_col_rect.width,
            }
        };

        frame.render_widget(block, block_rect);
        frame.render_widget(fid_header, header_area);

        for (i, fid) in (self.top_row..=self.bottom_row()).enumerate() {
            let line = Line::raw(
                format!(
                    "{}",
                    fid,
                ),
            ).bold().fg(crate::shared::palette::DEFAULT.default_fg);
            let rect = Rect {
                x: self.feature_col_rect.x,
                y: self.feature_col_rect.y + i as u16 + if preview { 2 } else { 3 },
                height: 1,
                width: 11,
            };

            frame.render_widget(line, rect);
        }
    }

    pub fn layers(&self) -> &[TatLayer] {
        &self.layers
    }
}


#[cfg(test)]
mod test {
    #[allow(unused)]
    use super::*;

    use crate::fixtures::basic_table;
    use crate::fixtures::datasets::basic_gpkg;

    use rstest::*;

    #[rstest]
    fn test_new(basic_gpkg: &'static Dataset) {
        // covers:
        // layers_from_ds()
        {
            let t = TatTable::new(basic_gpkg, None, None);
            assert_eq!(t.layer_index, 0);
            assert_eq!(t.top_row, 1);

            assert_eq!(t.layers.len(), 5);

            assert_eq!(t.layers.get(0).unwrap().name(), "point".to_string());
            assert_eq!(t.layers.get(1).unwrap().name(), "line".to_string());
            assert_eq!(t.layers.get(2).unwrap().name(), "polygon".to_string());
            assert_eq!(t.layers.get(3).unwrap().name(), "multipolygon".to_string());
            assert_eq!(t.layers.get(4).unwrap().name(), "nogeom".to_string());
        }

        {
            let filter = Some(vec![
                "nogeom".to_string(),
            ]);
            let t = TatTable::new(basic_gpkg, None, filter);
            assert_eq!(t.layers.len(), 1);

            assert_eq!(t.layers.get(0).unwrap().name(), "nogeom".to_string());
        }
    }

    #[rstest]
    fn test_dataset_info_text(basic_gpkg: &'static Dataset) {
        let t = TatTable::new(basic_gpkg, None, None);
        let expected = "- URI: \"./testdata/basic.gpkg\"
- Driver: GeoPackage (GPKG)";

        assert_eq!(t.dataset_info_text(), expected);
    }

    #[rstest]
    fn test_layer(basic_gpkg: &'static Dataset) {
        let mut t = TatTable::new(basic_gpkg, None, None);
        assert_eq!(t.layer_index, 0);

        assert_eq!(t.layer().name(), "point");

        t.set_layer_index(2);

        assert_eq!(t.layer().name(), "polygon");
    }

    #[rstest]
    fn test_nav_v(basic_table: TatTable) {
        // covers:
        // current_row()
        // relative_highlighted_row()
        // visible_rows()
        // set_top_row()
        // bottom_row()
        // all_rows_visible()
        let mut t = basic_table;
        t.set_layer_index(4); // nogeom, has most features

        assert_eq!(t.current_row(), 1);
        assert_eq!(t.max_top_row(), 46);
        assert_eq!(t.relative_highlighted_row(), 0);
        assert_eq!(t.visible_rows(), 15);
        assert_eq!(t.top_row, 1);
        assert_eq!(t.bottom_row(), 15);
        assert!(!t.all_rows_visible());

        t.nav_v(TatNavVertical::Last);
        assert_eq!(t.current_row(), 60);
        assert_eq!(t.relative_highlighted_row(), 14);
        assert_eq!(t.top_row, 46);
        assert_eq!(t.bottom_row(), 60);

        t.nav_v(TatNavVertical::UpOne);
        assert_eq!(t.current_row(), 59);
        assert_eq!(t.relative_highlighted_row(), 13);
        assert_eq!(t.top_row, 46);
        assert_eq!(t.bottom_row(), 60);

        t.nav_v(TatNavVertical::Specific(46));
        assert_eq!(t.current_row(), 46);
        assert_eq!(t.relative_highlighted_row(), 0);
        assert_eq!(t.top_row, 46);
        assert_eq!(t.bottom_row(), 60);

        t.nav_v(TatNavVertical::UpOne);
        assert_eq!(t.current_row(), 45);
        assert_eq!(t.relative_highlighted_row(), 0);
        assert_eq!(t.top_row, 45);
        assert_eq!(t.bottom_row(), 59);

        t.nav_v(TatNavVertical::Last);

        t.nav_v(TatNavVertical::UpHalfParagraph);
        assert_eq!(t.current_row(), 53);
        assert_eq!(t.relative_highlighted_row(), 7);
        assert_eq!(t.top_row, 46);
        assert_eq!(t.bottom_row(), 60);

        t.nav_v(TatNavVertical::UpHalfParagraph);
        assert_eq!(t.current_row(), 46);
        assert_eq!(t.relative_highlighted_row(), 0);
        assert_eq!(t.top_row, 46);
        assert_eq!(t.bottom_row(), 60);

        t.nav_v(TatNavVertical::DownOne);
        assert_eq!(t.current_row(), 47);
        assert_eq!(t.relative_highlighted_row(), 1);
        assert_eq!(t.top_row, 46);
        assert_eq!(t.bottom_row(), 60);

        t.nav_v(TatNavVertical::UpHalfParagraph);
        assert_eq!(t.current_row(), 40);
        assert_eq!(t.relative_highlighted_row(), 1);
        assert_eq!(t.top_row, 39);
        assert_eq!(t.bottom_row(), 53);

        t.nav_v(TatNavVertical::UpHalfParagraph);
        t.nav_v(TatNavVertical::UpHalfParagraph);
        t.nav_v(TatNavVertical::UpHalfParagraph);
        t.nav_v(TatNavVertical::UpHalfParagraph);
        t.nav_v(TatNavVertical::UpHalfParagraph);
        t.nav_v(TatNavVertical::UpHalfParagraph);
        assert_eq!(t.current_row(), 2);
        assert_eq!(t.relative_highlighted_row(), 1);
        assert_eq!(t.top_row, 1);
        assert_eq!(t.bottom_row(), 15);

        t.nav_v(TatNavVertical::First);
        assert_eq!(t.current_row(), 1);
        assert_eq!(t.relative_highlighted_row(), 0);
        assert_eq!(t.top_row, 1);
        assert_eq!(t.bottom_row(), 15);

        t.nav_v(TatNavVertical::Last);

        t.nav_v(TatNavVertical::UpParagraph);
        assert_eq!(t.current_row(), 45);
        assert_eq!(t.relative_highlighted_row(), 14);
        assert_eq!(t.top_row, 31);
        assert_eq!(t.bottom_row(), 45);

        t.nav_v(TatNavVertical::First);
        assert_eq!(t.current_row(), 1);
        assert_eq!(t.relative_highlighted_row(), 0);
        assert_eq!(t.top_row, 1);
        assert_eq!(t.bottom_row(), 15);

        t.nav_v(TatNavVertical::DownOne);
        assert_eq!(t.current_row(), 2);
        assert_eq!(t.relative_highlighted_row(), 1);
        assert_eq!(t.top_row, 1);
        assert_eq!(t.bottom_row(), 15);

        t.nav_v(TatNavVertical::Specific(15));
        assert_eq!(t.current_row(), 15);
        assert_eq!(t.relative_highlighted_row(), 14);
        assert_eq!(t.top_row, 1);
        assert_eq!(t.bottom_row(), 15);

        t.nav_v(TatNavVertical::DownOne);
        assert_eq!(t.current_row(), 16);
        assert_eq!(t.relative_highlighted_row(), 14);
        assert_eq!(t.top_row, 2);
        assert_eq!(t.bottom_row(), 16);

        t.nav_v(TatNavVertical::DownHalfParagraph);
        assert_eq!(t.current_row(), 23);
        assert_eq!(t.relative_highlighted_row(), 14);
        assert_eq!(t.top_row, 9);
        assert_eq!(t.bottom_row(), 23);

        t.nav_v(TatNavVertical::DownParagraph);
        assert_eq!(t.current_row(), 38);
        assert_eq!(t.relative_highlighted_row(), 14);
        assert_eq!(t.top_row, 24);
        assert_eq!(t.bottom_row(), 38);

        t.nav_v(TatNavVertical::DownParagraph);
        t.nav_v(TatNavVertical::DownParagraph);
        t.nav_v(TatNavVertical::DownParagraph);
        assert_eq!(t.current_row(), 60);
        assert_eq!(t.relative_highlighted_row(), 14);
        assert_eq!(t.top_row, 46);
        assert_eq!(t.bottom_row(), 60);

        t.nav_v(TatNavVertical::MouseScrollUp);
        assert_eq!(t.current_row(), 55);
        assert_eq!(t.relative_highlighted_row(), 9);
        assert_eq!(t.top_row, 46);
        assert_eq!(t.bottom_row(), 60);

        t.nav_v(TatNavVertical::MouseScrollUp);
        t.nav_v(TatNavVertical::MouseScrollUp);
        assert_eq!(t.current_row(), 45);
        assert_eq!(t.relative_highlighted_row(), 4);
        assert_eq!(t.top_row, 41);
        assert_eq!(t.bottom_row(), 55);

        t.nav_v(TatNavVertical::MouseScrollDown);
        assert_eq!(t.current_row(), 50);
        assert_eq!(t.relative_highlighted_row(), 9);
        assert_eq!(t.top_row, 41);
        assert_eq!(t.bottom_row(), 55);

        t.nav_v(TatNavVertical::MouseScrollDown);
        t.nav_v(TatNavVertical::MouseScrollDown);
        t.nav_v(TatNavVertical::MouseScrollDown);
        assert_eq!(t.current_row(), 60);
        assert_eq!(t.relative_highlighted_row(), 14);
        assert_eq!(t.top_row, 46);
        assert_eq!(t.bottom_row(), 60);

        t.nav_v(TatNavVertical::Specific(5));
        assert_eq!(t.current_row(), 5);
        assert_eq!(t.relative_highlighted_row(), 4);
        assert_eq!(t.top_row, 1);
        assert_eq!(t.bottom_row(), 15);

        t.nav_v(TatNavVertical::Specific(55));
        assert_eq!(t.current_row(), 55);
        assert_eq!(t.relative_highlighted_row(), 9);
        assert_eq!(t.top_row, 46);
        assert_eq!(t.bottom_row(), 60);

        t.nav_v(TatNavVertical::Specific(25));
        assert_eq!(t.current_row(), 25);
        assert_eq!(t.relative_highlighted_row(), 9);
        assert_eq!(t.top_row, 16);
        assert_eq!(t.bottom_row(), 30);
    }

    #[rstest]
    fn test_nav_h(basic_table: TatTable) {
        let mut t = basic_table;
        t.set_layer_index(4); // nogeom, has most features and columns

        assert_eq!(t.current_column(), 0);
        assert_eq!(t.relative_highlighted_column(), 0);
        assert_eq!(t.visible_columns(), 7);
        assert_eq!(t.first_column, 0);

        t.nav_h(TatNavHorizontal::RightOne);
        assert_eq!(t.current_column(), 1);
        assert_eq!(t.relative_highlighted_column(), 1);
        assert_eq!(t.first_column, 0);

        t.nav_h(TatNavHorizontal::RightOne);
        t.nav_h(TatNavHorizontal::RightOne);
        t.nav_h(TatNavHorizontal::RightOne);
        t.nav_h(TatNavHorizontal::RightOne);
        t.nav_h(TatNavHorizontal::RightOne);
        assert_eq!(t.current_column(), 6);
        assert_eq!(t.relative_highlighted_column(), 6);
        assert_eq!(t.first_column, 0);

        t.nav_h(TatNavHorizontal::RightOne);
        assert_eq!(t.current_column(), 7);
        assert_eq!(t.relative_highlighted_column(), 6);
        assert_eq!(t.first_column, 1);

        t.nav_h(TatNavHorizontal::RightOne);
        assert_eq!(t.current_column(), 8);
        assert_eq!(t.relative_highlighted_column(), 6);
        assert_eq!(t.first_column, 2);

        t.nav_h(TatNavHorizontal::RightOne);
        assert_eq!(t.current_column(), 8);
        assert_eq!(t.relative_highlighted_column(), 6);
        assert_eq!(t.first_column, 2);

        t.nav_h(TatNavHorizontal::LeftOne);
        assert_eq!(t.current_column(), 7);
        assert_eq!(t.relative_highlighted_column(), 5);
        assert_eq!(t.first_column, 2);

        t.nav_h(TatNavHorizontal::LeftOne);
        t.nav_h(TatNavHorizontal::LeftOne);
        t.nav_h(TatNavHorizontal::LeftOne);
        t.nav_h(TatNavHorizontal::LeftOne);
        t.nav_h(TatNavHorizontal::LeftOne);
        assert_eq!(t.current_column(), 2);
        assert_eq!(t.relative_highlighted_column(), 0);
        assert_eq!(t.first_column, 2);

        t.nav_h(TatNavHorizontal::LeftOne);
        assert_eq!(t.current_column(), 1);
        assert_eq!(t.relative_highlighted_column(), 0);
        assert_eq!(t.first_column, 1);

        t.nav_h(TatNavHorizontal::LeftOne);
        assert_eq!(t.current_column(), 0);
        assert_eq!(t.relative_highlighted_column(), 0);
        assert_eq!(t.first_column, 0);

        t.nav_h(TatNavHorizontal::LeftOne);
        assert_eq!(t.current_column(), 0);
        assert_eq!(t.relative_highlighted_column(), 0);
        assert_eq!(t.first_column, 0);

        t.nav_h(TatNavHorizontal::End);
        assert_eq!(t.current_column(), 8);
        assert_eq!(t.relative_highlighted_column(), 6);
        assert_eq!(t.first_column, 2);

        t.nav_h(TatNavHorizontal::Home);
        assert_eq!(t.current_column(), 0);
        assert_eq!(t.relative_highlighted_column(), 0);
        assert_eq!(t.first_column, 0);
    }

    #[rstest]
    fn test_selected_value(basic_table: TatTable) {
        let mut t = basic_table;
        t.set_layer_index(4);

        assert_eq!(t.selected_value(), Some("text".to_string()));

        t.nav_h(TatNavHorizontal::RightOne);
        assert_eq!(t.selected_value(), Some("10".to_string()));

        t.nav_h(TatNavHorizontal::RightOne);
        assert_eq!(t.selected_value(), Some("100".to_string()));

        t.nav_h(TatNavHorizontal::RightOne);
        assert_eq!(t.selected_value(), Some("1.541".to_string()));

        t.nav_h(TatNavHorizontal::RightOne);
        assert_eq!(t.selected_value(), Some("1970/07/10".to_string()));

        t.nav_h(TatNavHorizontal::RightOne);
        assert_eq!(t.selected_value(), Some("2025/07/19 20:45:45+00".to_string()));

        t.nav_h(TatNavHorizontal::RightOne);
        assert_eq!(t.selected_value(), Some("1".to_string()));

        t.nav_h(TatNavHorizontal::RightOne);
        assert_eq!(t.selected_value(), Some("626C6F620A".to_string()));

        t.nav_h(TatNavHorizontal::RightOne);
        assert_eq!(t.selected_value(), Some("{\"another_key\":\"another_value\",\"key\":\"value\"}".to_string()));

        t.nav_h(TatNavHorizontal::Home);
        t.nav_v(TatNavVertical::Specific(5));
        assert_eq!(t.selected_value(), Some("participate".to_string()));

        t.set_layer_index(0); // point, has null values and geom field

        t.nav_h(TatNavHorizontal::Home);
        t.nav_v(TatNavVertical::First);
        assert_eq!(t.selected_value(), Some("POINT (0 0)".to_string()));

        t.nav_h(TatNavHorizontal::End);
        assert_eq!(t.selected_value(), None);
    }

    #[rstest]
    fn test_where_clause(basic_gpkg: &'static Dataset) {
        let mut t = TatTable::new(basic_gpkg, Some("text_field = 'participate'".to_string()), None);
        t.set_layer_index(4);

        let expected = Some("participate".to_string());

        assert_eq!(t.layer().feature_count(), 4);
        assert_eq!(t.layer().get_value_by_row(0, 0), expected);
        assert_eq!(t.layer().get_value_by_row(1, 0), expected);
        assert_eq!(t.layer().get_value_by_row(2, 0), expected);
        assert_eq!(t.layer().get_value_by_row(3, 0), expected);
    }

    #[rstest]
    fn test_where_clause_and_layer_filter(basic_gpkg: &'static Dataset) {
        let filter = Some(vec!["nogeom".to_string()]);
        let t = TatTable::new(basic_gpkg, Some("text_field = 'verify' AND i32_field = -28".to_string()), filter);

        assert_eq!(t.layer().feature_count(), 1);
        assert_eq!(t.layer().get_value_by_row(0, 0), Some("verify".to_string()));
        assert_eq!(t.layer().get_value_by_row(0, 1), Some("-28".to_string()));
    }
}
