use gdal::{vector::Layer, Dataset, vector::LayerAccess};

pub enum TatNavJump {
    First,
    Last,
    DownOne,
    UpOne,
    DownHalfParagraph, // half "paragraph"
    UpHalfParagraph,
    DownParagraph,
    UpParagraph,
    Specific(u64),
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
}

// TODO: consider the pub members
#[derive(Clone)]
pub struct TatLayer {
    pub name: String,
    crs: TatCrs,
    fields: Vec<TatField>,
    index: usize,
    pub feature_index: Vec<u64>,
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
            index: i,
            feature_index: vec![], // don't build immediately to be more flexible (maybe?)
        }
    }

    pub fn feature_count(&self) -> u64 {
        self.gdal_layer().feature_count()
    }

    pub fn gdal_layer(&self) -> Layer {
        TatLayer::get_gdal_layer(&self.ds, self.index)
    }

    pub fn get_gdal_layer(dataset: &Dataset, layer_index: usize) -> Layer {
        match dataset.layer(layer_index) {
            Ok(lyr) => lyr,
            // TODO: maybe don't panic
            Err(_) => panic!(),
        }
    }

    pub fn crs_from_layer(layer: &Layer) -> TatCrs {
        let mut aname = "UNKNOWN".to_string();
        let mut acode = 0;
        let mut name = "UNKNOWN".to_string();

        if let Some(crs) = layer.spatial_ref() {
            match crs.auth_name() {
                Ok(a_name) => aname = a_name,
                _ => (),
            }
            match crs.auth_code() {
                Ok(a_code) => acode = a_code,
                _ => (),
            }
            match crs.name() {
                Ok(c_name) => name = c_name,
                _ => (),
            }
        }

        TatCrs::new(aname, acode, name)
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
}

