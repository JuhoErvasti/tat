use ratatui::{layout::{Constraint, Layout, Margin}, style::Style, symbols::{self, scrollbar::DOUBLE_VERTICAL}, text::Line, widgets::{Block, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState}, Frame};
use ratatui::widgets::HighlightSpacing;
use ratatui::prelude::Stylize;
use std::sync::mpsc::Sender;

use crate::{
    dataset::DatasetRequest, navparagraph::TatNavigableParagraph, types::TatNavVertical
};

const BORDER_LAYER_LIST: symbols::border::Set = symbols::border::Set {
    top_left: symbols::line::ROUNDED.vertical,
    top_right: symbols::line::ROUNDED.horizontal,
    bottom_right: symbols::line::ROUNDED.horizontal,
    vertical_right: " ",
    ..symbols::border::ROUNDED
};

pub type TatLayerInfo = (String, TatNavigableParagraph);


/// A widget which displays the layers in the opened dataset and holds displayable information
/// about them
pub struct TatLayerList {
    state: ListState,
    scroll: ScrollbarState,
    layer_infos: Vec<TatLayerInfo>,
    available_rows: usize,
}

#[cfg(test)]
impl Default for TatLayerList {
    fn default() -> Self {
        let mut ls = ListState::default();
        ls.select_first();
        Self {
            state: ls,
            scroll: ScrollbarState::new(0),
            layer_infos: vec![],
            available_rows: 0,
        }
    }
}

impl TatLayerList {
    /// Constructs new widget
    pub fn new(request_rx: Sender<DatasetRequest>) -> Self {
        let mut ls = ListState::default();
        ls.select_first();

        request_rx.send(DatasetRequest::LayerInfos).unwrap();

        let scr = ScrollbarState::new(0);
        Self {
            state: ls,
            scroll: scr,
            layer_infos: vec![],
            available_rows: 0,
        }
    }

    pub fn set_infos(&mut self, infos: Vec<TatLayerInfo>) {
        self.layer_infos = infos;
    }

    /// Returns the displayable layer information as a navigable paragraph based on the currently
    /// selected layer
    pub fn current_layer_info_paragraph(&mut self) -> Option<&mut TatNavigableParagraph> {
        let (_, para) = self.layer_infos.get_mut(self.state.selected()?)?;
        Some(para)
    }

    /// Handles navigation of the list
    pub fn nav(&mut self, conf: TatNavVertical) {
        match conf {
            TatNavVertical::First => self.state.select_first(),
            TatNavVertical::Last => self.state.select(Some(self.layer_infos.len() - 1)),
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

    /// Returns the index of the currently selected layer
    pub fn layer_index(&self) -> Option<usize> {
        let i = self.state.selected()?;

        if self.layer_infos.len() == 0 {
            return None;
        }

        if i >= self.layer_infos.len() {
            return Some(self.layer_infos.len() - 1);
        }

        Some(i)
    }

    /// Renders the current state of the widget
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

        let mut items: Vec<ListItem> = vec![];

        for (name, _) in self.layer_infos.iter() {
            items.push(
                ListItem::new(
                    Line::raw(name),
                ),
            );
        }


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

        if self.layer_infos.len() > self.available_rows as usize {
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

    /// Updates the state of the scrollbar. Should be called anytime navigation of the list
    /// happens.
    fn update_scrollbar(&mut self) {
        self.scroll = self.scroll.position(self.state.selected().unwrap());
    }

    /// Sets the available rows for displaying layers
    #[cfg(test)]
    pub fn set_available_rows(&mut self, available_rows: usize) {
        self.available_rows = available_rows;
    }

    pub fn available_rows(&self) -> usize {
        self.available_rows
    }
}

#[cfg(test)]
mod test {
    #[allow(unused)]
    use super::*;

    #[allow(unused)]
    use crate::fixtures::datasets::basic_gpkg;
    use crate::fixtures::layer_infos;

    use rstest::*;

    #[rstest]
    fn test_set_infos(layer_infos: Vec<TatLayerInfo>) {
        // TODO: not sure this actually warrants a test
        let mut ll = TatLayerList::default();
        ll.set_infos(layer_infos);

        assert_eq!(ll.layer_infos.get(0).unwrap().0, "Layer1");
        assert_eq!(ll.layer_infos.get(1).unwrap().0, "Layer2");
        assert_eq!(ll.layer_infos.get(2).unwrap().0, "Layer3");

        assert_eq!(ll.layer_infos.get(0).unwrap().1.text(), "Layer 1 info");
        assert_eq!(ll.layer_infos.get(1).unwrap().1.text(), "Layer 2 info");
        assert_eq!(ll.layer_infos.get(2).unwrap().1.text(), "Layer 3 info");
    }

    #[rstest]
    fn test_nav(layer_infos: Vec<TatLayerInfo>) {
        let mut ll = TatLayerList::default();
        ll.set_infos(layer_infos);
        ll.available_rows = 2;

        assert_eq!(ll.layer_index(), Some(0));

        ll.nav(TatNavVertical::Last);
        assert_eq!(ll.layer_index(), Some(4));
        ll.nav(TatNavVertical::DownOne);
        assert_eq!(ll.layer_index(), Some(4));

        ll.nav(TatNavVertical::First);
        assert_eq!(ll.layer_index(), Some(0));
        ll.nav(TatNavVertical::UpOne);
        assert_eq!(ll.layer_index(), Some(0));
        ll.nav(TatNavVertical::DownOne);
        assert_eq!(ll.layer_index(), Some(1));
        ll.nav(TatNavVertical::DownHalfParagraph);
        assert_eq!(ll.layer_index(), Some(2));
        ll.nav(TatNavVertical::DownParagraph);
        assert_eq!(ll.layer_index(), Some(4));
        ll.nav(TatNavVertical::UpHalfParagraph);
        assert_eq!(ll.layer_index(), Some(3));
        ll.nav(TatNavVertical::UpParagraph);
        assert_eq!(ll.layer_index(), Some(1));
        ll.nav(TatNavVertical::UpParagraph);
        assert_eq!(ll.layer_index(), Some(0));

        ll.nav(TatNavVertical::Specific(2));
        assert_eq!(ll.layer_index(), Some(2));
    }
}
