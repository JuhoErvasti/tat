use gdal::{
    vector::{
        field_type_to_name, LayerAccess
    },
    Dataset,
};
use ratatui::{layout::{Constraint, Layout, Margin}, style::Style, symbols::{self, scrollbar::DOUBLE_VERTICAL}, text::Line, widgets::{Block, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState}, Frame};
use ratatui::widgets::HighlightSpacing;
use ratatui::prelude::Stylize;
use std::fmt::Write;

use crate::{
    navparagraph::TatNavigableParagraph,
    types::TatNavVertical,
    layer::TatLayer,
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

impl TatLayerList {
    /// Constructs new widget
    pub fn new(ds: &'static Dataset, layers: Option<Vec<String>>) -> Self {
        let mut ls = ListState::default();
        ls.select_first();

        let infos = TatLayerList::layer_infos(&ds, layers);
        let scr = ScrollbarState::new(infos.len());

        Self {
            state: ls,
            scroll: scr,
            layer_infos: infos,
            available_rows: 0,
        }
    }

    /// Returns the displayable layer information as a navigable paragraph based on the currently
    /// selected layer
    pub fn current_layer_info_paragraph(&mut self) -> &mut TatNavigableParagraph {
        let (_, para) = self.layer_infos.get_mut(self.state.selected().unwrap()).unwrap();
        para
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
    pub fn layer_index(&self) -> usize {
        let i = self.state.selected().unwrap();

        if i >= self.layer_infos.len() {
            return self.layer_infos.len() - 1;
        }

        i
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

    /// Constructs all the displayable layer information objects from the dataset
    fn layer_infos(ds: &'static Dataset, layers: Option<Vec<String>>) -> Vec<TatLayerInfo> {
        let mut infos: Vec<TatLayerInfo> = vec![];
        for (i, layer) in ds.layers().enumerate() {
            if let Some(lyrs) = layers.clone() {
                if !lyrs.contains(&layer.name()) {
                    continue
                }
            }

            let p = TatNavigableParagraph::new(TatLayerList::layer_info_text(ds, i));
            infos.push((layer.name().to_string(), p));
        }

        infos
    }

    /// Constructs the layer information object for one layer
    fn layer_info_text(ds: &'static Dataset, layer_index: usize) -> String {
        // TODO: not sure I like the fact that these are constructed twice
        let layer = TatLayer::new(&ds, layer_index, None);

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

        if layer.attribute_fields().len() > 0 {
            write!(
                text,
                "- Fields ({}):\n",
                layer.attribute_fields().len(),
            ).unwrap();

            for field in layer.attribute_fields() {
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

    /// Updates the state of the scrollbar. Should be called anytime navigation of the list
    /// happens.
    fn update_scrollbar(&mut self) {
        self.scroll = self.scroll.position(self.state.selected().unwrap());
    }
}

#[cfg(test)]
mod test {
    #[allow(unused)]
    use super::*;

    #[allow(unused)]
    use crate::fixtures::*;

    use rstest::*;

    #[rstest]
    fn test_layer_infos(basic_gpkg: &'static Dataset) {
        let ll = TatLayerList::new(basic_gpkg, None);
        assert_eq!(ll.layer_infos.len(), 5);

        {
            let (name, info) = ll.layer_infos.get(0).unwrap();
            assert_eq!(name, "point");

            let expected = "- Name: point
- CRS: EPSG:3857 (WGS 84 / Pseudo-Mercator)
- Feature Count: 4
- Geometry fields:
    \"geom\" - (Point, EPSG:3857)
- Fields (1):
    \"field\" - (Integer)
";

            assert_eq!(info.text(), expected);
        }

        {
            let (name, info) = ll.layer_infos.get(1).unwrap();
            assert_eq!(name, "line");

            let expected = "- Name: line
- CRS: EPSG:4326 (WGS 84)
- Feature Count: 3
- Geometry fields:
    \"geom\" - (Multi Line String, EPSG:4326)
- Fields (1):
    \"field\" - (Integer)
";

            assert_eq!(info.text(), expected);
        }

        {
            let (name, info) = ll.layer_infos.get(2).unwrap();
            assert_eq!(name, "polygon");

            let expected = "- Name: polygon
- CRS: EPSG:3067 (ETRS89 / TM35FIN(E,N))
- Feature Count: 2
- Geometry fields:
    \"geom\" - (Polygon, EPSG:3067)
- Fields (1):
    \"field\" - (Integer)
";
            assert_eq!(info.text(), expected);
        }

        {
            let (name, info) = ll.layer_infos.get(3).unwrap();
            assert_eq!(name, "multipolygon");

            let expected = "- Name: multipolygon
- CRS: EPSG:3067 (ETRS89 / TM35FIN(E,N))
- Feature Count: 3
- Geometry fields:
    \"geom\" - (Multi Polygon, EPSG:3067)
";
            assert_eq!(info.text(), expected);
        }

        {
            let (name, info) = ll.layer_infos.get(4).unwrap();
            assert_eq!(name, "nogeom");

            let expected = "- Name: nogeom
- Feature Count: 1
- Fields (9):
    \"text_field\" - (String)
    \"i32_field\" - (Integer)
    \"i64_field\" - (Integer64)
    \"decimal_field\" - (Real)
    \"date_field\" - (Date)
    \"datetime_field\" - (DateTime)
    \"bool_field\" - (Integer)
    \"blob_field\" - (Binary)
    \"json_field\" - (String)
";
            assert_eq!(info.text(), expected);
        }
    }

    #[rstest]
    fn test_nav(basic_gpkg: &'static Dataset) {
        let mut ll = TatLayerList::new(basic_gpkg, None);
        ll.available_rows = 2;

        assert_eq!(ll.layer_index(), 0);

        ll.nav(TatNavVertical::Last);
        assert_eq!(ll.layer_index(), 4);
        ll.nav(TatNavVertical::DownOne);
        assert_eq!(ll.layer_index(), 4);

        ll.nav(TatNavVertical::First);
        assert_eq!(ll.layer_index(), 0);
        ll.nav(TatNavVertical::UpOne);
        assert_eq!(ll.layer_index(), 0);
        ll.nav(TatNavVertical::DownOne);
        assert_eq!(ll.layer_index(), 1);
        ll.nav(TatNavVertical::DownHalfParagraph);
        assert_eq!(ll.layer_index(), 2);
        ll.nav(TatNavVertical::DownParagraph);
        assert_eq!(ll.layer_index(), 4);
        ll.nav(TatNavVertical::UpHalfParagraph);
        assert_eq!(ll.layer_index(), 3);
        ll.nav(TatNavVertical::UpParagraph);
        assert_eq!(ll.layer_index(), 1);
        ll.nav(TatNavVertical::UpParagraph);
        assert_eq!(ll.layer_index(), 0);

        ll.nav(TatNavVertical::Specific(2));
        assert_eq!(ll.layer_index(), 2);
    }

    #[rstest]
    fn test_with_layer_filter(basic_gpkg: &'static Dataset) {
        let filter = Some(vec![
            "nogeom".to_string(),
            "multipolygon".to_string(),
            ]);
        let ll = TatLayerList::new(basic_gpkg, filter);
        assert_eq!(ll.layer_infos.len(), 2);

        {
            let (name, _) = ll.layer_infos.get(0).unwrap();
            assert_eq!(name, "multipolygon");
        }

        {
            let (name, _) = ll.layer_infos.get(1).unwrap();
            assert_eq!(name, "nogeom");
        }
    }
}
