use gdal::{vector::{geometry_type_to_name, Layer, LayerAccess}, Dataset};

use crate::types::{TatCrs, TatField, TatGeomField};
use cli_log::debug;

#[derive(Clone)]
pub struct TatLayer {
    name: String,
    crs: Option<TatCrs>,
    geom_fields: Vec<TatGeomField>,
    fields: Vec<TatField>,
    index: usize,
    fid_cache: Vec<u64>,
    feature_count: u64,
    ds: &'static Dataset,
}

impl TatLayer {
    pub fn new(dataset: &'static Dataset, i: usize) -> Self {
        let lyr = TatLayer::get_gdal_layer(dataset, i);
        Self {
            ds: dataset,
            name: lyr.name(),
            feature_count: lyr.feature_count(),
            crs: TatLayer::crs_from_layer(&lyr),
            fields: TatLayer::fields_from_layer(&lyr),
            geom_fields: TatLayer::geom_fields_from_layer(&lyr),
            index: i,
            fid_cache: vec![], // don't build immediately to be more flexible (maybe?)
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn feature_count(&self) -> u64 {
        self.feature_count
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
                &field.spatial_ref().unwrap()
            );

            fields.push(
                TatGeomField::new(
                    name.to_string(),
                    geometry_type_to_name(field.field_type()),
                    crs,
                )
            );
        }
        fields
    }

    pub fn fields_from_layer(layer: &Layer) -> Vec<TatField> {
        let mut fields: Vec<TatField> = vec![];
        for field in layer.defn().fields() {
            fields.push(
                TatField::new(
                    field.name(),
                    field.field_type(),
                )
            );
        }

        fields
    }

    pub fn build_fid_cache(&mut self) {
        self.fid_cache.clear();

        let mut cache: Vec<u64> = vec![];
        for feature in self.gdal_layer().features() {
            let fid = feature.fid().unwrap();
            cache.push(fid);
        }

        self.fid_cache = cache;
    }

    pub fn field_count(&self) -> u64 {
        self.fields.len() as u64 + self.geom_fields().len() as u64
    }

    pub fn field_name_by_id(&self, field_idx: i32) -> Option<String> {
        let total_geom_fields: i32 = self.geom_fields().len() as i32;

        if field_idx < total_geom_fields {
            if let Some(field) = self.geom_fields.get(field_idx as usize) {
                return Some(field.name().to_string());
            } else {
                panic!();
            }
        } else {
            let attribute_field_idx = field_idx - total_geom_fields;
            if let Some(field) = self.fields.get(attribute_field_idx as usize) {
                return Some(field.name().to_string());
            } else {
                panic!(); // FIXME: this happens
            }
        }
    }

    pub fn get_value_by_id(&self, fid: u64, field_idx: i32) -> Option<String> {
        // TODO: handle the potential GdalError better
        // also this function is kind of a mess otherwise too,
        // refactor
        if let Some(f) = self.gdal_layer().feature(fid) {
            let total_geom_fields: i32 = self.geom_fields().len() as i32;

            if total_geom_fields == 0 {
                if let Ok(Some(value)) = f.field_as_string(field_idx as usize) {
                    return Some(value);
                } else {
                    return None;
                }
            }

            if field_idx < total_geom_fields {
                if let Ok(geom) = f.geometry_by_index(field_idx as usize) {
                    if let Ok(wkt) = geom.wkt() {
                        return Some(wkt);
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                let attribute_field_idx = field_idx - total_geom_fields;
                if let Ok(Some(value)) = f.field_as_string(attribute_field_idx as usize) {
                    return Some(value);
                } else {
                    return None;
                }
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

    pub fn fid_cache(&self) -> &[u64] {
        &self.fid_cache
    }

    pub fn crs(&self) -> Option<&TatCrs> {
        self.crs.as_ref()
    }

    pub fn geom_fields(&self) -> &[TatGeomField] {
        &self.geom_fields
    }
}

