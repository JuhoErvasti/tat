use gdal::{spatial_ref::SpatialRef, vector::{geometry_type_to_name, Layer, LayerAccess}, Dataset};
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

#[derive(Clone)]
pub struct TatLayer {
    name: String,
    crs: Option<TatCrs>,
    geom_fields: Vec<TatGeomField>,
    fields: Vec<TatField>,
    index: usize,
    feature_index: Vec<u64>,
    ds: &'static Dataset,
}

impl TatLayer {
    pub fn new(dataset: &'static Dataset, i: usize) -> Self {
        let lyr = TatLayer::get_gdal_layer(dataset, i);
        Self {
            ds: dataset,
            name: lyr.name(),
            crs: TatLayer::crs_from_layer(&lyr),
            fields: TatLayer::fields_from_layer(&lyr),
            geom_fields: TatLayer::geom_fields_from_layer(&lyr),
            index: i,
            feature_index: vec![], // don't build immediately to be more flexible (maybe?)
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn feature_count(&self) -> u64 {
        self.gdal_layer().feature_count()
    }

    pub fn gdal_layer(&self) -> Layer {
        TatLayer::get_gdal_layer(&self.ds, self.index)
    }

    pub fn crs_from_layer(layer: &Layer) -> Option<TatCrs> {
        if let Some(sref) = layer.spatial_ref() {
            return TatCrs::from_spatial_ref(&sref);
        }

        None
    }

    pub fn geom_fields_from_layer(layer: &Layer) -> Vec<TatGeomField> {
        let mut fields: Vec<TatGeomField> = vec![];
        for field in layer.defn().geom_fields() {
            let name: &str = if field.name().is_empty() {
                "ANONYMOUS"
            } else {
                &field.name()
            };

            let crs = TatCrs::from_spatial_ref(
                &layer.spatial_ref().unwrap()
            );

            fields.push(
                TatGeomField {
                    name: name.to_string(),
                    geom_type: geometry_type_to_name(field.field_type()),
                    crs,
                }
            );
        }
        fields
    }

    pub fn fields_from_layer(layer: &Layer) -> Vec<TatField> {
        let mut fields: Vec<TatField> = vec![];
        for field in layer.defn().fields() {
            fields.push(
                TatField {
                    name: field.name(),
                    dtype: field.field_type(),
                }
            );
        }

        fields
    }

    pub fn build_feature_index(&mut self) {
        // TODO: there seems to be some weird bug with the indexing, see layer_styles of one of the
        // test GPKGs, might be related to this??
        self.feature_index.clear();

        let mut i: Vec<u64> = vec![];
        for feature in self.gdal_layer().features() {
            i.push(feature.fid().unwrap());
        }

        self.feature_index = i;
    }

    pub fn field_count(&self) -> u64 {
        self.fields.len() as u64
    }

    pub fn get_value(&self, fid: u64, field_name: &str) -> Option<String> {
        if let Some(f) = self.gdal_layer().feature(fid) {
            if let Ok(Some(value)) = f.field_as_string_by_name(field_name) {
                return Some(value)
            } else {
                return None
            }
        } else {
            return None
        }
    }

    fn get_gdal_layer(dataset: &Dataset, layer_index: usize) -> Layer {
        match dataset.layer(layer_index) {
            Ok(lyr) => lyr,
            // TODO: maybe don't panic
            Err(_) => panic!(),
        }
    }

    pub fn fields(&self) -> &[TatField] {
        &self.fields
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn feature_index(&self) -> &[u64] {
        &self.feature_index
    }

    pub fn crs(&self) -> Option<&TatCrs> {
        self.crs.as_ref()
    }

    pub fn geom_fields(&self) -> &[TatGeomField] {
        &self.geom_fields
    }
}

pub enum TatPopUpType {
    // TODO: really not sure this is that necessary?
    Help,
    GdalLog,
    DebugLog,
    FullValue,
}

pub struct TatPopup {
    title: String,
    paragraph: TatNavigableParagraph,
    ptype: TatPopUpType,
}

impl TatPopup {
    pub fn new(title: String, paragraph: TatNavigableParagraph, ptype: TatPopUpType) -> Self {
        Self { title, paragraph, ptype }
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

    pub fn ptype(&self) -> &TatPopUpType {
        &self.ptype
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}
