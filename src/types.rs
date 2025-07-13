use cli_log::debug;
use gdal::spatial_ref::SpatialRef;
use ratatui::widgets::{Paragraph, ScrollbarState};

use crate::navparagraph::TatNavigableParagraph;

pub enum TatNavVertical {
    First,
    Last,
    DownOne,
    UpOne,
    DownHalfParagraph,
    UpHalfParagraph,
    DownParagraph,
    UpParagraph,
    Specific(i64),
    MouseScrollUp,
    MouseScrollDown,
}

pub enum TatNavHorizontal {
    Home,
    End,
    RightOne,
    LeftOne,
}

#[derive(Clone)]
pub struct TatCrs {
    auth_name: String,
    auth_code: i32,
    name: String
}

impl TatCrs {
    pub fn new(a_name: String, a_code: i32, crs_name: String) -> Self {
        Self {
            auth_name: a_name,
            auth_code: a_code,
            name: crs_name,
        }
    }

    pub fn from_spatial_ref(sref: &SpatialRef) -> Option<Self> {
        let aname = match sref.auth_name() {
            Ok(a_name) => a_name,
            _ => return None,
        };

        let acode = match sref.auth_code() {
            Ok(a_code) => a_code,
            _ => return None,
        };

        let name = match sref.name() {
            Ok(c_name) => c_name,
            _ => return None,
        };

        Some(
            TatCrs::new(
                aname,
                acode,
                name,
            )
        )
    }


    pub fn auth_name(&self) -> &str {
        &self.auth_name
    }

    pub fn auth_code(&self) -> i32 {
        self.auth_code
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
pub struct TatField {
    name: String,
    dtype: u32,
}

impl TatField {
    pub fn new(name: String, dtype: u32) -> Self {
        Self {
            name,
            dtype,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dtype(&self) -> u32 {
        self.dtype
    }
}

#[derive(Clone)]
pub struct TatGeomField {
    name: String,
    geom_type: String,
    crs: Option<TatCrs>,
}

impl TatGeomField {
    pub fn new(name: String, geom_type: String, crs: Option<TatCrs>) -> Self {
        Self {
            name,
            geom_type,
            crs,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn geom_type(&self) -> &str {
        &self.geom_type
    }

    pub fn crs(&self) -> Option<&TatCrs> {
        self.crs.as_ref()
    }
}

pub struct TatPopup {
    title: String,
    paragraph: TatNavigableParagraph,
}

impl TatPopup {
    pub fn new(title: String, paragraph: TatNavigableParagraph) -> Self {
        Self { title, paragraph }
    }

    pub fn set_available_rows(&mut self, value: usize) {
        self.paragraph.set_available_rows(value);
    }

    pub fn set_available_cols(&mut self, value: usize) {
        self.paragraph.set_available_cols(value);
    }

    pub fn paragraph(&self) -> Paragraph {
        self.paragraph.paragraph()
    }

    pub fn max_line_len(&self) -> usize {
        self.paragraph.max_line_len()
    }

    pub fn scroll_state_v(&self) -> ScrollbarState {
        self.paragraph.scroll_state_v()
    }

    pub fn scroll_state_h(&self) -> ScrollbarState {
        self.paragraph.scroll_state_h()
    }

    pub fn total_lines(&self) -> usize {
        self.paragraph.total_lines()
    }

    pub fn nav_h(&mut self, conf: TatNavHorizontal) {
        self.paragraph.nav_h(conf)
    }

    pub fn nav_v(&mut self, conf: TatNavVertical) {
        self.paragraph.nav_v(conf);
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}
