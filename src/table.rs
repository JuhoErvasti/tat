use ratatui::{layout::{Constraint, Layout, Margin}, style::{palette::tailwind, Style, Stylize}, symbols::{self, scrollbar::{DOUBLE_HORIZONTAL, DOUBLE_VERTICAL}}, text::Line, widgets::{Block, Borders, Padding, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Widget}};
use gdal::vector::LayerAccess;

use crate::types::{TatLayer, TatNavJump};

pub enum TatTableMode {
    Scrolling,
    GdalLog,
    JumpTo,
}

/// Widget for displaying the attribute table
pub struct TatTable {
    table_state: TableState,
    top_fid: u64,
    visible_rows: u64,
    first_column: u64,
    visible_columns: u64,
    v_scroll: ScrollbarState,
    h_scroll: ScrollbarState,
    layer: Option<TatLayer>,

}

impl TatTable {
    pub fn new() -> Self {
        let mut ts = TableState::default();
        ts.select_first();
        ts.select_first_column();

        Self {
            table_state: ts,
            top_fid: 1,
            visible_rows: 0,
            first_column: 0,
            visible_columns: 0,
            v_scroll: ScrollbarState::default(),
            h_scroll: ScrollbarState::default(),
            layer: Option::None,
        }
    }

    pub fn set_layer(&mut self, layer: TatLayer) {
        self.layer = Some(layer);
    }

    fn current_row(&self) -> u64 {
        // the idea is to avoid unwrapping/whatever everywhere
        // TODO: not sure how idiomatic this is in Rust, maybe reconsider
        // the approach, feels like this is just trying skirt around
        // the whole point of using Options and such, but idk
        self.table_state.selected().unwrap() as u64
    }

    fn current_column(&self) -> u64 {
        // see above (current_row)
        self.table_state.selected_column().unwrap() as u64
    }

    fn update_v_scrollbar(&mut self) {
        self.v_scroll = self.v_scroll.position((self.top_fid + self.current_row() - 1) as usize);
    }

    fn update_h_scrollbar(&mut self) {
        self.h_scroll = self.h_scroll.position((self.first_column + self.current_column()) as usize);
    }

    pub fn jump_row(&mut self, conf: TatNavJump) {
        let visible_rows = self.visible_rows as i64;
        let mut jump_by = |amount: i64| {
            let row = self.current_row();

            if amount > 0 {
                if row + amount as u64 > self.visible_rows {
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
            TatNavJump::First => {
                self.set_top_fid(1);
                self.table_state.select_first();
            },
            TatNavJump::Last => {
                self.set_top_fid(self.max_top_fid());
                self.table_state.select(Some(visible_rows as usize));
            },
            TatNavJump::DownOne => {
                jump_by(1);
            },
            TatNavJump::UpOne => {
                jump_by(-1);
            },
            TatNavJump::DownHalfParagraph => {
                jump_by(visible_rows / 2 );
            },
            TatNavJump::UpHalfParagraph => {
                jump_by(-(visible_rows / 2));
            },
            TatNavJump::DownParagraph => {
                jump_by(visible_rows);
            },
            TatNavJump::UpParagraph => {
                jump_by(-(visible_rows));
            },
            TatNavJump::Specific(row) => {
                panic!("Not implemented! Cannot jump to row {}", row);
            },
        }

        self.update_v_scrollbar();
    }

    pub fn jump_first_column(&mut self) {
        self.first_column = 0;
        self.table_state.select_first_column();
        self.update_h_scrollbar();
    }

    pub fn jump_last_column(&mut self) {
        self.set_first_column(self.layer.as_ref().unwrap().field_count() as i64 - self.visible_columns as i64);
        self.table_state.select_column(Some(self.visible_columns as usize));
        self.update_h_scrollbar();
    }

    fn set_first_column(&mut self, col: i64) {
        let max_first_column: i64 = self.layer.as_ref().unwrap().field_count() as i64 - self.visible_columns as i64;

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

    pub fn reset(&mut self) {
        self.top_fid = 1;
        self.first_column = 0;
        self.visible_columns = 0;
        self.table_state.select_first_column();
        self.table_state.select_first();
    }

    fn max_top_fid(&self) -> i64 {
        self.layer.as_ref().unwrap().gdal_layer().feature_count() as i64 - self.visible_rows as i64 + 1
    }

    pub fn nav_left(&mut self) {
        let col = self.current_column();
        if col == 0 {
            if self.first_column == 0 {
                let cols =  self.layer.as_ref().unwrap().field_count();
                self.set_first_column(cols as i64 - self.visible_columns as i64);
                self.table_state.select_column(Some(self.first_column as usize + self.visible_columns as usize));
            } else {
                self.set_first_column(self.first_column as i64 - 1);
            }
            self.update_h_scrollbar();
            return;
        }
        self.table_state.select_previous_column();
        self.update_h_scrollbar();
    }

    pub fn nav_right(&mut self) {
        let col = self.current_column();
        if col == self.visible_columns {
            let cols =  self.layer.as_ref().unwrap().field_count();
            if self.first_column + col == cols {
                self.set_first_column(0);
                self.table_state.select_column(Some(0));
            } else {
                self.set_first_column(self.first_column as i64 + 1);
            }
            self.update_h_scrollbar();
            return;
        }
        self.table_state.select_next_column();
        self.update_h_scrollbar();
    }

    fn current_layer(&self) -> TatLayer{
        if let Some(lyr) = &self.layer {
            lyr.clone()
        } else {
            panic!();
        }
    }
}

impl Widget for &mut TatTable {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let [table_area, footer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        // This weird order of operations is to prevent self.selected_layer()
        // borrowing self.visible_columns
        // To be honest I don't really get it but I'm just going with this
        // for now. Honestly at the end of the day I might want to have
        // some kind of layer struct to handle some of this stuff more cleanly
        if self.current_layer().field_count() * 30 < table_area.width as u64 {
            self.visible_columns = self.current_layer().field_count() as u64;
        } else {
            self.visible_columns = (table_area.width / 30) as u64;
        }

        let all_columns_visible = self.visible_columns == self.current_layer().field_count() as u64;

        self.visible_rows = (table_area.height - 6) as u64;

        if all_columns_visible {
            self.visible_rows +=2;
        }

        let layer = self.current_layer();
        let gdal_layer = layer.gdal_layer();

        // self.render_footer(footer_area, frame);

        let block = Block::new()
            .title(Line::raw(format!(" {} (debug - visible_rows: {}, visible_columns: {} bottom_fid: {}, top_fid: {}, table_area_width: {})", layer.name, self.visible_rows, self.visible_columns, self.bottom_fid(), self.top_fid, table_area.width)).centered().bold().underlined())
            // .title(Line::raw(format!(" {} ", layer.name())))
            .borders(Borders::ALL)
            .padding(Padding::top(1))
            .border_set(symbols::border::ROUNDED);

        let mut header_items: Vec<String> = vec![
            String::from("Feature")
        ];

        let mut field_idx = 0;
        for field in layer.gdal_layer().defn().fields() {
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

        widths.push(Constraint::Fill(1));

        for _ in 0..self.visible_columns {
            widths.push(Constraint::Fill(3));
        }

        for i in self.top_fid..self.bottom_fid() + 1 {
            let fid = match layer.feature_index.get(i as usize - 1) {
                Some(fid) => fid,
                None => break,
            };


            let feature = match gdal_layer.feature(*fid) {
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
                .begin_symbol(Some(DOUBLE_VERTICAL.begin))
                .end_symbol(Some(DOUBLE_VERTICAL.end));

        let horz_scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::HorizontalBottom)
                .begin_symbol(Some(DOUBLE_HORIZONTAL.begin))
                .end_symbol(Some(DOUBLE_HORIZONTAL.end));

        if !all_columns_visible {
            StatefulWidget::render(
                vert_scrollbar,
                table_area.inner(Margin {horizontal: 0, vertical: 1}),
                buf,
                &mut self.v_scroll,
            );

            StatefulWidget::render(
                table,
                table_area.inner(Margin {horizontal: 1, vertical: 1}),
                buf,
                &mut self.table_state,
            );

            StatefulWidget::render(
                horz_scrollbar,
                table_area.inner(Margin {horizontal: 1, vertical: 0}),
                buf,
                &mut self.h_scroll,
            );
        } else {
            StatefulWidget::render(
                vert_scrollbar,
                table_area,
                buf,
                &mut self.v_scroll,
            );

            StatefulWidget::render(
                table,
                table_area.inner(Margin {horizontal: 1, vertical: 0}),
                buf,
                &mut self.table_state,
            );
        }
    }
}

