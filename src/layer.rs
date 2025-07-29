use gdal::{vector::{geometry_type_to_name, Layer, LayerAccess}, Dataset};

use cli_log::error;
use crate::types::{TatCrs, TatField, TatGeomField};

/// A struct which holds information about a layer in a GDAL Dataset and can also fetch infromation
/// about features in the layer.
#[derive(Clone)]
pub struct TatLayer {
    name: String,
    crs: Option<TatCrs>,
    geom_fields: Vec<TatGeomField>,
    attribute_fields: Vec<TatField>,
    index: usize,

    /// A cache of feature ids in the underlying GDAL layer. This is needed because the
    /// features are listed and numbered sequentially in the table for clarity and navigation
    /// reasons but the GDAL fids are not always sequential so in order to access the features in
    /// the layer from a sequential index we need to save its corresponding fid.,
    fid_cache: Vec<u64>,
    feature_count: u64,
}

impl TatLayer {
    /// Constructs new object
    pub fn new(
        name: String,
        crs: Option<TatCrs>,
        geom_fields: Vec<TatGeomField>,
        attribute_fields: Vec<TatField>,
        index: usize,
        feature_count: u64,
    ) -> Self {
        Self {
            name,
            feature_count,
            crs,
            attribute_fields,
            geom_fields,
            index,
            fid_cache: vec![], // don't build immediately to be more flexible (maybe?)
        }
    }

    /// Returns the layer's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the total number of features in the layer
    pub fn feature_count(&self) -> u64 {
        self.feature_count
    }

    /// Returns the coordinate reference system of the given layer as a TatCrs
    pub fn crs_from_layer(layer: &Layer) -> Option<TatCrs> {
        if let Some(sref) = layer.spatial_ref() {
            return TatCrs::from_spatial_ref(&sref);
        }

        None
    }

    /// Returns all geometry field found in the given layer
    pub fn geom_fields_from_layer(layer: &Layer) -> Vec<TatGeomField> {
        let mut fields: Vec<TatGeomField> = vec![];
        for field in layer.defn().geom_fields() {
            let name: &str = if field.name().is_empty() {
                "geometry"
            } else {
                &field.name()
            };

            let crs = match &field.spatial_ref() {
                Ok(sref) => TatCrs::from_spatial_ref(sref),
                Err(_) => None,
            };

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

    /// Return all the attribute fields in the given layer
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

    /// Returns the total number of attribute AND geometry fields in the layer. The reason this
    /// returns the sum of attribute and geometry fields is because both are displayed in the
    /// table. For more information, check get_value_by_id()
    pub fn field_count(&self) -> u64 {
        self.attribute_fields.len() as u64 + self.geom_fields().len() as u64
    }

    /// Returns the name of a field based on its index. This includes the geometry fields.
    pub fn field_name_by_id(&self, field_idx: i32) -> Option<String> {
        let total_geom_fields: i32 = self.geom_fields().len() as i32;

        if field_idx < total_geom_fields {
            if let Some(field) = self.geom_fields.get(field_idx as usize) {
                return Some(field.name().to_string());
            } else {
                None
            }
        } else {
            let attribute_field_idx = field_idx - total_geom_fields;
            if let Some(field) = self.attribute_fields.get(attribute_field_idx as usize) {
                return Some(field.name().to_string());
            } else {
                None
            }
        }
    }

    /// Returns a value of a specific row (if any)
    #[allow(unused)]
    pub fn get_value_by_row(&self, row: u64, field_idx: i32) -> Option<String> {
        self.get_value_by_fid(*self.fid_cache.get(row as usize).unwrap(), field_idx)
    }

    /// Returns a value from a specific feature and field. This also includes geometry fields if
    /// any exist. The given field index matches how the fields are displayed in the table i.e.
    /// geometry columns are always displayed first, attribute fields second.
    pub fn get_value_by_fid(&self, fid: u64, field_idx: i32) -> Option<String> {
        return None;
        // TODO: the geom + attribute field abstraction could probably refactored to be a lot
        // clearer

        // if let Some(f) = self.gdal_layer().feature(fid) {
        //     let total_geom_fields: i32 = self.geom_fields().len() as i32;
        //
        //     if total_geom_fields == 0 {
        //         return f.field_as_string(field_idx as usize)
        //             .unwrap_or(None);
        //     }
        //
        //     if field_idx < total_geom_fields {
        //         let res = f.geometry_by_index(field_idx as usize);
        //         if res.is_err() {
        //             return None;
        //         }
        //
        //         let wkt_res = res.unwrap().wkt();
        //         if wkt_res.is_err() {
        //             return None;
        //         }
        //
        //         return Some(wkt_res.unwrap());
        //     } else {
        //         let attribute_field_idx = field_idx - total_geom_fields;
        //         return f.field_as_string(attribute_field_idx as usize)
        //             .unwrap_or(None);
        //     }
        // }
        //
        // None
    }

    /// Returns the layer's attribute fields
    pub fn attribute_fields(&self) -> &[TatField] {
        &self.attribute_fields
    }

    /// Returns the fid cache
    pub fn fid_cache(&self) -> &[u64] {
        &self.fid_cache
    }

    /// Returns the layer's CRS (if any)
    pub fn crs(&self) -> Option<&TatCrs> {
        self.crs.as_ref()
    }

    /// Returns the layer's geometry fields
    pub fn geom_fields(&self) -> &[TatGeomField] {
        &self.geom_fields
    }

    /// Returns the GDAL layer from the given dataset based on its index
    fn get_gdal_layer(dataset: &Dataset, layer_index: usize, where_clause: Option<String>) -> Layer {
        match dataset.layer(layer_index) {
            Ok(mut lyr) => {
                if where_clause.is_some() {
                    match lyr.set_attribute_filter(&where_clause.unwrap()) {
                            Ok(()) => (),
                            Err(e) => error!("ERROR! Could not set attribute filter: {}", e.to_string()),
                        }
                }

                lyr
            },
            // TODO: maybe don't panic
            Err(_) => panic!(),
        }
    }

    pub fn set_fid_cache(&mut self, fid_cache: Vec<u64>) {
        self.fid_cache = fid_cache;
    }
}

#[cfg(test)]
mod test {
    #[allow(unused)]
    use super::*;

    #[allow(unused)]
    use crate::fixtures::datasets::basic_gpkg;

    use rstest::*;

    #[rstest]
    fn test_new(basic_gpkg: &'static Dataset) {
        // covers:
        // name()
        // feature_count()
        // crs_from_layer()
        // geom_fields_from_layer()
        // fields_from_layer()
        // also I think this sufficiently covers
        // types::TatCrs,TatField,TatGeomField
        let l = TatLayer::new(basic_gpkg, 0, None);

        assert_eq!(l.name(), "point");
        assert_eq!(l.feature_count(), 4);
        assert!(l.crs().is_some());
        assert_eq!(l.crs().unwrap().auth_name(), "EPSG");
        assert_eq!(l.crs().unwrap().auth_code(), 3857);

        assert_eq!(l.attribute_fields.len(), 1);
        assert_eq!(l.attribute_fields.get(0).unwrap().name(), "field");
        assert_eq!(l.attribute_fields.get(0).unwrap().dtype(), 0);

        assert_eq!(l.geom_fields.len(), 1);
        assert_eq!(l.geom_fields.get(0).unwrap().name(), "geom");
        assert_eq!(l.geom_fields.get(0).unwrap().geom_type(), "Point");
        assert_eq!(l.geom_fields.get(0).unwrap().crs().unwrap().auth_name(), "EPSG");
        assert_eq!(l.geom_fields.get(0).unwrap().crs().unwrap().auth_code(), 3857);

        assert_eq!(l.index, 0);
    }

    #[rstest]
    fn test_build_fid_cache(basic_gpkg: &'static Dataset) {
        // use "line" layer which has a deleted feature making the fids non-sequential
        let mut l = TatLayer::new(basic_gpkg, 1, None);
        l.build_fid_cache();

        assert_eq!(l.feature_count, 3);
        assert_eq!(l.fid_cache.len(), 3);

        assert_eq!(*l.fid_cache.get(0).unwrap(), 1);
        assert_eq!(*l.fid_cache.get(1).unwrap(), 2);
        assert_eq!(*l.fid_cache.get(2).unwrap(), 4);
    }

    #[rstest]
    fn test_field_count(basic_gpkg: &'static Dataset) {
        {
            // multi polygon layer, has only a geom field
            let l = TatLayer::new(basic_gpkg, 3, None);
            assert_eq!(l.field_count(), 1);
        }

        {
            // no geoms
            let l = TatLayer::new(basic_gpkg, 4, None);
            assert_eq!(l.field_count(), 9);
        }

        {
            // polygon, has one of each
            let l = TatLayer::new(basic_gpkg, 2, None);
            assert_eq!(l.field_count(), 2);
        }
    }

    #[rstest]
    fn test_field_name_by_id(basic_gpkg: &'static Dataset) {
        // covers gdal_layer() -> get_gdal_layer()
        {
            // multi polygon layer, has only a geom field
            let l = TatLayer::new(basic_gpkg, 3, None);
            assert_eq!(l.field_name_by_id(0), Some("geom".to_string()));
            assert_eq!(l.field_name_by_id(1), None);
        }

        {
            // no geoms
            let l = TatLayer::new(basic_gpkg, 4, None);
            assert_eq!(l.field_name_by_id(0), Some("text_field".to_string()));
            assert_eq!(l.field_name_by_id(1), Some("i32_field".to_string()));
            assert_eq!(l.field_name_by_id(2), Some("i64_field".to_string()));
            assert_eq!(l.field_name_by_id(3), Some("decimal_field".to_string()));
            assert_eq!(l.field_name_by_id(4), Some("date_field".to_string()));
            assert_eq!(l.field_name_by_id(5), Some("datetime_field".to_string()));
            assert_eq!(l.field_name_by_id(6), Some("bool_field".to_string()));
            assert_eq!(l.field_name_by_id(7), Some("blob_field".to_string()));
            assert_eq!(l.field_name_by_id(8), Some("json_field".to_string()));
        }

        {
            // polygon, has one of each
            let l = TatLayer::new(basic_gpkg, 2, None);
            assert_eq!(l.field_name_by_id(0), Some("geom".to_string()));
            assert_eq!(l.field_name_by_id(1), Some("field".to_string()));
        }
    }

    #[rstest]
    fn test_get_value_by_fid(basic_gpkg: &'static Dataset) {
        // covers gdal_layer() -> get_gdal_layer()
        {
            let l = TatLayer::new(basic_gpkg, 2, None);
            assert_eq!(l.get_value_by_fid(0, 0), None);
            assert_eq!(l.get_value_by_fid(1, 2), None);
            assert_eq!(l.get_value_by_fid(3, 0), None);

            // feature 1
            assert_eq!(l.get_value_by_fid(1, 0), Some("POLYGON ((-9 3,-9 1,-7 1,-7 3,-9 3))".to_string()));
            assert_eq!(l.get_value_by_fid(1, 1), Some("456".to_string()));

            // feature 2
            assert_eq!(l.get_value_by_fid(2, 0), Some("POLYGON ((-5 6,-5 3,-2 3,-2 6,-5 6))".to_string()));
            assert_eq!(l.get_value_by_fid(2, 1), Some("213".to_string()));
        }
    }
}
