use std::fmt::{Display, Write};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use cli_log::{error, info};
use gdal::vector::field_type_to_name;
use gdal::Dataset;
use gdal::{vector::{geometry_type_to_name, Layer, LayerAccess}, Metadata};
use unicode_segmentation::UnicodeSegmentation;

use crate::app::TatEvent;
use crate::layer::TatFeature;
use crate::navparagraph::TatNavigableParagraph;
use crate::{layer::TatLayer, layerlist::TatLayerInfo, types::{TatCrs, TatField, TatGeomField}};

#[derive(Debug)]
pub enum DatasetRequest {
    AllLayers,
    AllLayerInfos,

    /// layer_index, row, fid
    Feature(usize, usize, u64),
    FidCache(usize),
    DatasetInfo,
}

impl Display for DatasetRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatasetRequest::AllLayers => write!(f, "GdalRequest::AllLayers"),
            DatasetRequest::AllLayerInfos => write!(f, "GdalRequest::AllLayerInfos"),
            DatasetRequest::Feature(i, j, k) => write!(f, "GdalRequest::Feature({i}, {j}, {k})"),
            DatasetRequest::FidCache(i) => write!(f, "GdalRequest::FidCache({i})"),
            DatasetRequest::DatasetInfo => write!(f, "GdalRequest::DatasetInfo"),
        }
    }
}

#[derive(Debug)]
pub enum DatasetResponse {
    Layer(TatLayer),
    LayerInfo(TatLayerInfo),
    Feature(usize, usize, TatFeature),
    FidCache(usize, Vec<u64>),
    DatasetInfo(String),
}

impl Display for DatasetResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatasetResponse::Layer(tat_layer) => write!(f, "GdalResponse::Layer({})", tat_layer.name()),
            DatasetResponse::LayerInfo(info) => write!(f, "GdalResponse::LayerInfo({})", info.0),
            DatasetResponse::Feature(_, _, feat) => write!(f, "GdalResponse::Feature({:?})", feat),
            DatasetResponse::FidCache(_, _) => write!(f, "GdalResponse::FidCache()"),
            DatasetResponse::DatasetInfo(info) => write!(f, "GdalResponse::DatasetInfo({})", info),
        }
    }
}

/// Struct for handling interfacing with GDAL in a separate thread
pub struct TatDataset {
    gdal_ds: Arc<Mutex<Dataset>>,
    response_tx: Sender<TatEvent>,
    request_rx: Receiver<DatasetRequest>,
    layer_filter: Option<Vec<String>>,
    where_clause: Option<String>,
    feature_requests: Vec<(usize, usize)>,
}

impl TatDataset {
    /// Attempts to open a dataset from a string.
    pub fn new(
        response_tx: Sender<TatEvent>,
        request_rx: Receiver<DatasetRequest>,
        uri: String,
        all_drivers: bool,
        where_clause: Option<String>,
        layer_filter: Option<Vec<String>>,
    ) -> Option<Self> {
        // deal with vectors only at least for now
        let flags = gdal::GdalOpenFlags::GDAL_OF_VECTOR | gdal::GdalOpenFlags::GDAL_OF_READONLY;

        let allowed_drivers = vec![
            "CSV",
            "OpenFileGDB",
            "GeoJSON",
            "GeoJSONSeq",
            "GML",
            "GPKG",
            "JML",
            "JSONFG",
            "MapML",
            "ODS",
            "ESRI Shapefile",
            "MapInfo File",
            "XLSX",
        ];

        let options = gdal::DatasetOptions {
            open_flags: flags,
            allowed_drivers: if all_drivers { None } else {Some(&allowed_drivers)},
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
                gdal_ds: Arc::new(Mutex::new(ds)),
                response_tx,
                request_rx,
                where_clause,
                layer_filter,
                feature_requests: vec![],
            }
        )
    }

    fn layer_fid_cache(&self, lyr: &mut Layer) -> Vec<u64> {
        let mut cache: Vec<u64> = vec![];
        for feature in lyr.features() {
            let fid = feature.fid().unwrap();
            cache.push(fid);
        }

        cache
    }

    fn send_response(&self, r: DatasetResponse) {
        self.response_tx.send(
            TatEvent::Dataset(
                r
            )
        ).unwrap();
    }

    pub fn handle_requests(&mut  self) {
        loop {
            match self.request_rx.recv() {
                Ok(request) => {
                    info!("HANDLING {request}");
                    let gdal_ds = self.gdal_ds.lock().unwrap();
                    match request {
                        DatasetRequest::AllLayers => {
                            for (i, ref mut layer) in gdal_ds.layers().enumerate() {
                                if let Some(lyr_filter) = self.layer_filter.as_ref() {
                                    if lyr_filter.contains(&layer.name()) {
                                        continue;
                                    }
                                }

                                self.send_response(
                                    DatasetResponse::Layer(
                                        self.layer_from_gdal_layer(i, layer)
                                    )
                                );
                            }
                        },
                        DatasetRequest::AllLayerInfos => {
                            for mut layer in gdal_ds.layers() {
                                if let Some(lyr_filter) = self.layer_filter.as_ref() {
                                    if lyr_filter.contains(&layer.name()) {
                                        continue;
                                    }
                                }

                                self.send_response(
                                    DatasetResponse::LayerInfo(
                                        (
                                            layer.name(),
                                            TatNavigableParagraph::new(
                                                self.layer_info_text(&mut layer),
                                            ),
                                        ),
                                    )
                                )
                            }
                        },
                        DatasetRequest::Feature(layer_index, row, fid) => {
                            let info = (layer_index, row);

                            if self.feature_requests.contains(&info) {
                                continue
                            } else {
                                self.feature_requests.push(info);
                            }


                            let lyr = gdal_ds.layer(layer_index).unwrap();
                            self.send_response(
                                DatasetResponse::Feature(
                                    layer_index,
                                    row,
                                    self.feature_from_gdal_feature(fid, &lyr),
                                )
                            );
                        },
                        DatasetRequest::FidCache(index) => {
                            let mut lyr = gdal_ds.layer(index).unwrap();
                            self.send_response(
                                DatasetResponse::FidCache(
                                    index,
                                    self.layer_fid_cache(&mut lyr),
                                )
                            )
                        },
                        DatasetRequest::DatasetInfo => {
                            self.send_response(
                                DatasetResponse::DatasetInfo(
                                    format!(
                                        "- URI: \"{}\"\n- Driver: {} ({})",
                                        gdal_ds.description().unwrap_or("ERROR: COULD NOT READ DATASET DESCRIPTION!".to_string()),
                                        gdal_ds.driver().long_name(),
                                        gdal_ds.driver().short_name(),
                                    )
                                )
                            );
                        },
                    }
                },
                Err(err) => {
                    error!("{}", err.to_string());
                },
            }
        }
    }

    fn feature_from_gdal_feature(&self, fid: u64, lyr: &Layer) -> TatFeature {
        let mut f: Vec<String> = vec![];
        let gdal_f = lyr.feature(fid).unwrap();

        for (i, _) in lyr.defn().geom_fields().enumerate() {
            f.push(gdal_f.geometry_by_index(i).unwrap().wkt().unwrap());
        }

        for (i, _) in gdal_f.fields().enumerate() {
            f.push(gdal_f.field_as_string(i as usize).unwrap().unwrap());
        }

        f
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
    fn layer_info_text(&self, lyr: &mut Layer) -> String {
        // TODO: not sure I like the fact that these are constructed twice
        let layer = self.layer_from_gdal_layer(0, lyr);

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
