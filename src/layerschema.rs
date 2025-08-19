#![allow(unused_imports)]
use cli_log::*;

use crate::types::{TatCrs, TatField, TatGeomField};

/// A struct which holds information about a layer in a GDAL Dataset and can also fetch infromation
/// about features in the layer.
#[derive(Debug)]
pub struct TatLayerSchema {
    name: String,
    crs: Option<TatCrs>,
    geom_fields: Vec<TatGeomField>,
    attribute_fields: Vec<TatField>,
    index: usize,
    feature_count: u64,
}

impl TatLayerSchema {
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

    /// Returns the total number of attribute AND geometry fields in the layer. The reason this
    /// returns the sum of attribute and geometry fields is because both are displayed in the
    /// table. For more information, check get_value_by_id()
    pub fn field_count(&self) -> u64 {
        self.attribute_fields.len() as u64 + self.geom_fields().len() as u64
    }

    /// Returns the name of a field based on its index. This includes the geometry fields.
    pub fn field_name_by_id(&self, field_idx: i32) -> Option<&str> {
        let total_geom_fields: i32 = self.geom_fields().len() as i32;

        if field_idx < total_geom_fields {
            if let Some(field) = self.geom_fields.get(field_idx as usize) {
                return Some(field.name());
            } else {
                None
            }
        } else {
            let attribute_field_idx = field_idx - total_geom_fields;
            if let Some(field) = self.attribute_fields.get(attribute_field_idx as usize) {
                return Some(field.name());
            } else {
                None
            }
        }
    }

    /// Returns the layer's attribute fields
    pub fn attribute_fields(&self) -> &[TatField] {
        &self.attribute_fields
    }

    /// Returns the layer's CRS (if any)
    pub fn crs(&self) -> Option<&TatCrs> {
        self.crs.as_ref()
    }

    /// Returns the layer's geometry fields
    pub fn geom_fields(&self) -> &[TatGeomField] {
        &self.geom_fields
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

#[cfg(test)]
mod test {
    #[allow(unused)]
    use super::*;

    #[allow(unused)]
    use crate::fixtures::datasets::basic_gpkg;
    use crate::fixtures::{layer_schema, layer_schema_no_geom, layer_schema_one_geom};

    use rstest::*;

    #[rstest]
    fn test_field_count(
        layer_schema: TatLayerSchema,
        layer_schema_one_geom: TatLayerSchema,
        layer_schema_no_geom: TatLayerSchema,
    ) {
        assert_eq!(layer_schema.field_count(), 5);
        assert_eq!(layer_schema_one_geom.field_count(), 4);
        assert_eq!(layer_schema_no_geom.field_count(), 3);
    }

    #[rstest]
    fn test_field_name_by_id(
        layer_schema: TatLayerSchema,
        layer_schema_one_geom: TatLayerSchema,
        layer_schema_no_geom: TatLayerSchema,
    ) {
        assert_eq!(layer_schema.field_name_by_id(0), Some("geom1"));
        assert_eq!(layer_schema.field_name_by_id(1), Some("geom2"));
        assert_eq!(layer_schema.field_name_by_id(2), Some("Field1"));
        assert_eq!(layer_schema.field_name_by_id(3), Some("Field2"));
        assert_eq!(layer_schema.field_name_by_id(4), Some("Field3"));
        assert_eq!(layer_schema.field_name_by_id(5), None);

        assert_eq!(layer_schema_one_geom.field_name_by_id(0), Some("geom"));
        assert_eq!(layer_schema_one_geom.field_name_by_id(1), Some("Field1"));
        assert_eq!(layer_schema_one_geom.field_name_by_id(2), Some("Field2"));
        assert_eq!(layer_schema_one_geom.field_name_by_id(3), Some("Field3"));
        assert_eq!(layer_schema_one_geom.field_name_by_id(4), None);

        assert_eq!(layer_schema_no_geom.field_name_by_id(0), Some("Field1"));
        assert_eq!(layer_schema_no_geom.field_name_by_id(1), Some("Field2"));
        assert_eq!(layer_schema_no_geom.field_name_by_id(2), Some("Field3"));
        assert_eq!(layer_schema_no_geom.field_name_by_id(3), None);
    }
}
