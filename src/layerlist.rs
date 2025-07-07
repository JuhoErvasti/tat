use cli_log::debug;
use crossterm::style::Colors;
use gdal::{vector::{field_type_to_name, geometry_type_to_name, Defn, Layer, LayerAccess}, Dataset};
use ratatui::{layout::{Constraint, Layout, Margin}, style::{palette::tailwind, Style}, symbols::{self, scrollbar::DOUBLE_VERTICAL}, text::Line, widgets::{Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget}, Frame};
use ratatui::widgets::HighlightSpacing;
use ratatui::prelude::Stylize;
use std::fmt::Write;

use crate::{navparagraph::TatNavigableParagraph, tat::LAYER_LIST_BORDER, types::{TatLayer, TatNavJump}};

/// Widget for displaying and managing the layers in a dataset
pub struct TatLayerList {
    state: ListState,
    scroll: ScrollbarState,
    // TODO: rethink this stuff, at least the gdal_ds being pub
    pub gdal_ds: &'static Dataset,
    layers: Vec<TatLayer>,
    layer_infos: Vec<TatNavigableParagraph>,
}

impl TatLayerList {
    pub fn new(ds: &'static Dataset) -> Self {
        let mut ls = ListState::default();
        ls.select_first();

        let lyrs = TatLayerList::layers_from_ds(&ds);
        let scr = ScrollbarState::new(lyrs.len());
        let infos = TatLayerList::layer_infos(&lyrs);

        Self {
            state: ls,
            scroll: scr,
            layers: lyrs,
            gdal_ds: ds,
            layer_infos: infos,
        }
    }

    pub fn gdal_layer<'a>(&'a self, layer_index: usize) -> Layer<'a> {
        match self.gdal_ds.layer(layer_index) {
            Ok(lyr) => lyr,
            // TODO: maybe don't panic
            Err(_) => panic!(),
        }
    }

    pub fn layer<'a>(&'a self, layer_index: usize) -> Option <&'a TatLayer> {
        self.layers.get(layer_index)
    }

    pub fn current_layer<'a>(&'a self) -> Option<&'a TatLayer> {
        self.layer(self.state.selected().unwrap())
    }

    // TODO: not sure it's a good idea to return this as mutable
    pub fn current_layer_info(&mut self) -> &mut TatNavigableParagraph {
        self.layer_infos.get_mut(self.state.selected().unwrap()).unwrap()
    }

    pub fn jump(&mut self, conf: TatNavJump) {
        // TODO: figure out the total number of visible layers
        match conf {
            TatNavJump::First => self.state.select_first(),
            TatNavJump::Last => self.state.select_last(),
            TatNavJump::DownOne => self.state.scroll_down_by(1),
            TatNavJump::UpOne => self.state.scroll_up_by(1),
            TatNavJump::DownHalfParagraph => self.state.scroll_down_by(25),
            TatNavJump::UpHalfParagraph => self.state.scroll_up_by(25),
            TatNavJump::DownParagraph => self.state.scroll_down_by(50),
            TatNavJump::UpParagraph => self.state.scroll_up_by(50),
            TatNavJump::Specific(row) => self.state.select(Some(row as usize)),
        }

        self.update_scrollbar();
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

    pub fn layer_infos(layers: &Vec<TatLayer>) -> Vec<TatNavigableParagraph> {
        let mut infos: Vec<TatNavigableParagraph> = vec![];
        for _layer in layers {
            let layer = _layer.gdal_layer();

            let p = TatNavigableParagraph::new(TatLayerList::layer_info_text(&layer));
            infos.push(p);
        }

        infos
    }

    fn layer_info_text(layer: &Layer) -> String {
        let mut text: String = format!("- Name: {}\n", layer.name());

        if let Some(crs) = layer.spatial_ref() {
            // TODO: don't unwrap
            write!(text, "- CRS: {}:{} ({})\n", crs.auth_name().unwrap(), crs.auth_code().unwrap(), crs.name().unwrap()).unwrap();
        }

        write!(text, "- Feature Count: {}\n", layer.feature_count()).unwrap();

        let defn: &Defn = layer.defn();

        let geom_fields_count = defn.geom_fields().count();
        let geom_fields = defn.geom_fields();

        if geom_fields_count > 0 {
            write!(text, "- Geometry fields:\n").unwrap();
            for geom_field in geom_fields {
                let display_str: &str = if geom_field.name().is_empty() {
                    "ANONYMOUS"
                } else {
                    &geom_field.name()
                };

                // TODO: don't unwrap etc.
                write!(
                    text,
                    "    \"{}\" - ({}, {}:{})\n",
                    display_str,
                    geometry_type_to_name(geom_field.field_type()),
                    geom_field.spatial_ref().unwrap().auth_name().unwrap(),
                    geom_field.spatial_ref().unwrap().auth_code().unwrap(),
                ).unwrap();
            }
        }

        let fields_count = defn.fields().count();
        let fields = defn.fields();

        if fields_count > 0 {
            write!(
                text,
                "- Fields:\n"
            ).unwrap();

            for field in fields {
                write!(
                    text,
                    "    \"{}\" - ({})\n",
                    field.name(),
                    field_type_to_name(field.field_type()),
                ).unwrap();
            }
        }

        text
    }

    fn update_scrollbar(&mut self) {
        self.scroll = self.scroll.position(self.state.selected().unwrap());
    }

    pub fn render(&mut self, area: ratatui::prelude::Rect, frame: &mut Frame, selected: bool) {
        // TODO: better paletting system
        let border_color = if selected {
            tailwind::SLATE.c400
        } else {
            tailwind::SLATE.c100
        };

        let block = Block::bordered()
            .title(Line::raw(" Layers ").underlined().bold())
            .border_set(LAYER_LIST_BORDER)
            .border_style(Style::default().fg(border_color)
        );

        let items: Vec<ListItem> = self
            .gdal_ds
            .layers()
            .map(|layer_item| {
                ListItem::new(Line::raw(layer_item.name()))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default()
                .fg(tailwind::SLATE.c950)
                .bg(tailwind::SLATE.c400))
            .highlight_spacing(HighlightSpacing::WhenSelected);

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .style(Style::default())
            .begin_symbol(Some(DOUBLE_VERTICAL.begin))
            .end_symbol(Some(DOUBLE_VERTICAL.end));

        let [_, scrollbar_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        frame.render_stateful_widget(list, area, &mut self.state);
        // TODO: don't show scrollbar if no need for it
        frame.render_stateful_widget(
            scrollbar,
            scrollbar_area.inner(Margin { horizontal: 0, vertical: 1 }),
            &mut self.scroll,
        );
    }
}
