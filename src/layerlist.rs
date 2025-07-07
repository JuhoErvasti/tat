use cli_log::debug;
use gdal::{vector::{Layer, LayerAccess}, Dataset};
use ratatui::{layout::{Constraint, Layout, Margin}, style::{palette::tailwind, Style}, symbols::{self, scrollbar::DOUBLE_VERTICAL}, text::Line, widgets::{Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget}};
use ratatui::widgets::HighlightSpacing;
use ratatui::prelude::Stylize;

use crate::{tat::LAYER_LIST_BORDER, types::{TatLayer, TatNavJump}};

pub struct TatLayerList {
    state: ListState,
    scroll: ScrollbarState,
    // TODO: rethink this stuff, at least the gdal_ds being pub
    pub gdal_ds: &'static Dataset,
    layers: Vec<TatLayer>,
}

impl TatLayerList {
    pub fn new(ds: &'static Dataset) -> Self {
        let mut ls = ListState::default();
        ls.select_first();

        let lyrs = TatLayerList::layers_from_ds(&ds);
        let scr = ScrollbarState::new(lyrs.len());

        Self {
            state: ls,
            scroll: scr,
            layers: lyrs,
            gdal_ds: ds,
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

    fn update_scrollbar(&mut self) {
        self.scroll = self.scroll.position(self.state.selected().unwrap());
    }
}

impl Widget for &mut TatLayerList {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::new()
            .title(Line::raw(" Layers ").underlined().bold())
            .borders(Borders::ALL)
            .border_set(LAYER_LIST_BORDER);

        let items: Vec<ListItem> = self
            .gdal_ds
            .layers()
            .map(|layer_item| {
                ListItem::new(Line::raw(layer_item.name()))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_symbol(">")
            .highlight_style(Style::default().fg(tailwind::SLATE.c500))
            .highlight_spacing(HighlightSpacing::WhenSelected);

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some(DOUBLE_VERTICAL.begin))
            .end_symbol(Some(DOUBLE_VERTICAL.end));

        let [_, scrollbar_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        StatefulWidget::render(
            list,
            area,
            buf,
            &mut self.state,
        );

        // TODO: don't show scrollbar if no need for it
        StatefulWidget::render(
            scrollbar,
            scrollbar_area.inner(Margin { horizontal: 0, vertical: 1 }),
            buf,
            &mut self.scroll,
        );
    }
}
