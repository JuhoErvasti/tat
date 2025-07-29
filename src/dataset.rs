use std::fmt::Write;
use std::sync::mpsc::{Receiver, Sender};

use cli_log::{error, info};
use gdal::vector::field_type_to_name;
use gdal::{vector::{geometry_type_to_name, Layer, LayerAccess}, Metadata};
use unicode_segmentation::UnicodeSegmentation;

use crate::app::TatEvent;
use crate::navparagraph::TatNavigableParagraph;
use crate::{layer::TatLayer, layerlist::TatLayerInfo, types::{TatCrs, TatField, TatGeomField}};

pub enum GdalRequest {
    AllLayers,
    AllLayerInfos,
    Feature(usize, u64),
    FidCache(usize),
    DatasetInfo,
}

#[derive(Debug)]
pub enum GdalResponse {
    Layer(TatLayer),
    LayerInfo(TatLayerInfo),
    Feature(Option<Vec<String>>),
    FidCache((usize, Vec<u64>)),
    DatasetInfo(String),
}

/// Struct for handling interfacing with GDAL in a separate thread
pub struct TatDataset {
    gdal_ds: &'static gdal::Dataset,
    response_tx: Sender<TatEvent>,
    request_rx: Receiver<GdalRequest>,
    layer_filter: Option<Vec<String>>,
    where_clause: Option<String>,
}

impl TatDataset {
    /// Attempts to open a dataset from a string.
    pub fn new(
        response_tx: Sender<TatEvent>,
        request_rx: Receiver<GdalRequest>,
        uri: String,
        all_drivers: bool,
        where_clause: Option<String>,
        layer_filter: Option<Vec<String>>,
    ) -> Option<Self> {
        // deal with vectors only at least for now
        let flags = gdal::GdalOpenFlags::GDAL_OF_VECTOR | gdal::GdalOpenFlags::GDAL_OF_READONLY;

        let allowed_drivers = vec![
            "CSV".to_string(),
            "OpenFileGDB".to_string(),
            "GeoJSON".to_string(),
            "GeoJSONSeq".to_string(),
            "GML".to_string(),
            "GPKG".to_string(),
            "JML".to_string(),
            "JSONFG".to_string(),
            "MapML".to_string(),
            "ODS".to_string(),
            "ESRI Shapefile".to_string(),
            "MapInfo File".to_string(),
            "XLSX".to_string(),
        ];
        let v: Vec<&str> = allowed_drivers.iter().map(|x| x.as_ref()).collect();

        let options = gdal::DatasetOptions {
            open_flags: flags,
            allowed_drivers: if all_drivers { None } else {Some(&v)},
            open_options: None,
            sibling_files: None,
        };

        let ds = match gdal::Dataset::open_ex(uri, options) {
            Ok(ds) => ds,
            Err(error) => {
                match error {
                    gdal::errors::GdalError::NullPointer { method_name: _, msg } => {
                        if msg.is_empty() {
                            println!("ERROR! Could not open dataset.");
                            println!();
                            return None;
                        }

                        let mut display_string = msg.clone();
                        let max_length = 100;

                        let squish_contents: bool = if msg.len() > 100 as usize {
                            true
                        } else if msg.chars().count() > max_length as usize {
                            true
                        } else {
                            false
                        };

                        if squish_contents {
                            let graph = msg.graphemes(true);
                            let squished: String = graph.into_iter().take(max_length as usize).collect();
                            display_string = format!("{}…", squished);
                        } 
                        println!("ERROR! Could not open dataset. GDAL message:\n{}", display_string);
                        println!();

                        return None;
                    }
                    _ => {
                        println!("ERROR! Could not open dataset:\n{}", error.to_string());
                        println!();

                        return None;
                    }
                }
            },
        };

        Some(
            Self {
                gdal_ds: Box::leak(Box::new(ds)),
                response_tx,
                request_rx,
                where_clause,
                layer_filter,
            }
        )
    }

    fn layer_fid_cache(&self, layer_index: usize) -> Vec<u64> {
        let mut cache: Vec<u64> = vec![];
        for feature in self.gdal_ds.layer(layer_index).unwrap().features() {
            let fid = feature.fid().unwrap();
            cache.push(fid);
        }

        cache
    }

    pub fn handle_requests(&self) {
        loop {
            match self.request_rx.recv() {
                Ok(request) => {
                    match request {
                        GdalRequest::AllLayers => {
                            for (i, ref mut layer) in self.gdal_ds.layers().enumerate() {
                                if let Some(lyr_filter) = self.layer_filter.as_ref() {
                                    if lyr_filter.contains(&layer.name()) {
                                        continue;
                                    }
                                }

                                self.response_tx.send(
                                    TatEvent::Gdal(
                                        GdalResponse::Layer(
                                            self.layer_from_gdal_layer(i, layer)
                                        )
                                    )
                                ).unwrap();
                            }
                        },
                        GdalRequest::AllLayerInfos => {
                            for (i, layer) in self.gdal_ds.layers().enumerate() {
                                if let Some(lyr_filter) = self.layer_filter.as_ref() {
                                    if lyr_filter.contains(&layer.name()) {
                                        continue;
                                    }
                                }

                                self.response_tx.send(
                                    TatEvent::Gdal(
                                        GdalResponse::LayerInfo(
                                            (
                                                layer.name(),
                                                TatNavigableParagraph::new(
                                                    self.layer_info_text(i),
                                                ),
                                            ),
                                        )
                                    )
                                ).unwrap();
                            }
                        },
                        GdalRequest::Feature(_, _) => {
                            info!("FEATURE REQUESTED!");
                        },
                        GdalRequest::FidCache(index) => {
                            self.response_tx.send(
                                TatEvent::Gdal(
                                    GdalResponse::FidCache(
                                        (index, self.layer_fid_cache(index)),
                                    )
                                )
                            ).unwrap();
                        },
                        GdalRequest::DatasetInfo => {
                            self.response_tx.send(
                                TatEvent::Gdal(
                                    GdalResponse::DatasetInfo(
                                        self.dataset_info_text()
                                    )
                                )
                            ).unwrap();
                        },
                    }
                },
                Err(err) => {
                    error!("{}", err.to_string());
                },
            }
        }
    }

    fn dataset_info_text(&self) -> String {
        format!(
            "- URI: \"{}\"\n- Driver: {} ({})",
            self.gdal_ds.description().unwrap_or("ERROR: COULD NOT READ DATASET DESCRIPTION!".to_string()),
            self.gdal_ds.driver().long_name(),
            self.gdal_ds.driver().short_name(),
        )
    }

    pub fn layer_from_gdal_layer(&self, layer_index: usize, layer: &mut Layer) -> TatLayer {
        if let Some(wc) = self.where_clause.as_ref() {
            layer.set_attribute_filter(wc.as_str()).unwrap();
        }

        TatLayer::new(
            layer.name(),
            TatDataset::crs_from_layer(&layer),
            TatDataset::geom_fields_from_layer(&layer),
            TatDataset::attribute_fields_from_layer(&layer),
            layer_index,
            layer.feature_count(),
        )
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
    pub fn attribute_fields_from_layer(layer: &Layer) -> Vec<TatField> {
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

    /// Constructs the layer information object for one layer
    fn layer_info_text(&self, layer_index: usize) -> String {
        let mut gdal_layer = self.gdal_ds.layer(layer_index).unwrap();
        // TODO: not sure I like the fact that these are constructed twice
        let layer = self.layer_from_gdal_layer(layer_index, &mut gdal_layer);

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
}
