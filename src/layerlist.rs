use cli_log::debug;
use gdal::{vector::{field_type_to_name, Layer, LayerAccess}, Dataset, Metadata};
use ratatui::{layout::{Constraint, Layout, Margin}, style::Style, symbols::scrollbar::DOUBLE_VERTICAL, text::Line, widgets::{Block, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState}, Frame};
use ratatui::widgets::HighlightSpacing;
use ratatui::prelude::Stylize;
use std::fmt::Write;

use crate::{navparagraph::TatNavigableParagraph, tat::BORDER_LAYER_LIST, types::{TatLayer, TatNavVertical}};

/// Widget for displaying and managing the layers in a dataset
pub struct TatLayerList {
    state: ListState,
    scroll: ScrollbarState,
    gdal_ds: &'static Dataset,
    layers: Vec<TatLayer>,
    layer_infos: Vec<TatNavigableParagraph>,
    available_rows: usize,
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
            available_rows: 0,
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

    pub fn current_layer_info(&mut self) -> &mut TatNavigableParagraph {
        self.layer_infos.get_mut(self.state.selected().unwrap()).unwrap()
    }

    pub fn nav(&mut self, conf: TatNavVertical) {
        match conf {
            TatNavVertical::First => self.state.select_first(),
            TatNavVertical::Last => self.state.select(Some(self.layers.len() - 1)),
            TatNavVertical::DownOne => self.state.scroll_down_by(1),
            TatNavVertical::UpOne => self.state.scroll_up_by(1),
            TatNavVertical::DownHalfParagraph => self.state.scroll_down_by(self.available_rows as u16 / 2),
            TatNavVertical::UpHalfParagraph => self.state.scroll_up_by(self.available_rows as u16 / 2),
            TatNavVertical::DownParagraph => self.state.scroll_down_by(self.available_rows as u16),
            TatNavVertical::UpParagraph => self.state.scroll_up_by(self.available_rows as u16),
            TatNavVertical::MouseScrollDown => self.state.scroll_down_by(self.available_rows as u16 / 3),
            TatNavVertical::MouseScrollUp => self.state.scroll_up_by(self.available_rows as u16 / 3),
            TatNavVertical::Specific(row) => self.state.select(Some(row as usize)),
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

    pub fn dataset_info_text(&self) -> String {
        format!(
            "- URI: \"{}\"\n- Driver: {} ({})",
            self.gdal_ds().description().unwrap(),
            self.gdal_ds().driver().long_name(),
            self.gdal_ds().driver().short_name(),
        )
    }

    pub fn layer_infos(layers: &Vec<TatLayer>) -> Vec<TatNavigableParagraph> {
        let mut infos: Vec<TatNavigableParagraph> = vec![];
        for layer in layers {
            let p = TatNavigableParagraph::new(TatLayerList::layer_info_text(&layer));
            infos.push(p);
        }

        infos
    }

    fn layer_info_text(layer: &TatLayer) -> String {
        let mut text: String = format!("- Name: {}\n", layer.name());

        if let Some(crs) = layer.crs() {
            write!(
                text,
                "- CRS: {}:{} ({})\n",
                crs.auth_name(),
                crs.auth_code(),
                crs.name(),
            ).unwrap();
        }

        write!(
            text,
            "- Feature Count: {}\n",
            layer.feature_count(),
        ).unwrap();

        if layer.geom_fields().len() > 0 {
            write!(text, "- Geometry fields:\n").unwrap();

            for field in layer.geom_fields() {
                write!(
                    text,
                    "    \"{}\" - ({}",
                    field.name(),
                    field.geom_type(),
                ).unwrap();

                if let Some(crs) = field.crs() {
                    write!(
                        text,
                        ", {}:{}",
                        crs.auth_name(),
                        crs.auth_code(),
                    ).unwrap();
                }

                write!(text, ")\n").unwrap();
            }
        }

        if layer.fields().len() > 0 {
            write!(
                text,
                "- Fields ({}):\n",
                layer.fields().len(),
            ).unwrap();

            for field in layer.fields() {
                write!(
                    text,
                    "    \"{}\" - ({})\n",
                    field.name(),
                    field_type_to_name(field.dtype()),
                ).unwrap();
            }
        }

        text
    }

    fn update_scrollbar(&mut self) {
        self.scroll = self.scroll.position(self.state.selected().unwrap());
    }

    pub fn render(&mut self, area: ratatui::prelude::Rect, frame: &mut Frame, highlight: bool) {
        let border_color = if highlight {
            crate::shared::palette::DEFAULT.highlighted_fg
        } else {
            crate::shared::palette::DEFAULT.default_fg
        };

        let block = Block::bordered()
            .title(Line::raw(crate::shared::TITLE_LAYER_LIST).underlined().bold())
            .border_set(BORDER_LAYER_LIST)
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
            .highlight_style(crate::shared::palette::DEFAULT.selected_style())
            .highlight_spacing(HighlightSpacing::WhenSelected);


        frame.render_stateful_widget(list, area, &mut self.state);

        if area.height >= 2 {
            self.available_rows = area.height as usize - 2; // account for borders
        } else {
            self.available_rows = 0;
        }

        if self.layers.len() > self.available_rows as usize {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some(DOUBLE_VERTICAL.begin))
                .end_symbol(Some(DOUBLE_VERTICAL.end));

            let [_, mut scrollbar_area] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .areas(area);

            scrollbar_area = scrollbar_area.inner(Margin { horizontal: 0, vertical: 1 });
            if !scrollbar_area.is_empty() {
                frame.render_stateful_widget(
                    scrollbar,
                    scrollbar_area,
                    &mut self.scroll,
                );
            }
        }
    }

    fn gdal_ds(&self) -> &Dataset {
        self.gdal_ds
    }
}
