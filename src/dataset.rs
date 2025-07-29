use std::sync::mpsc::{Receiver, Sender};

use unicode_segmentation::UnicodeSegmentation;

use crate::layer::TatLayer;

pub enum GdalRequest {
    LayerRequest(usize),
    FeatureRequest(usize, u64),
}

pub enum GdalResponse {
    LayerResponse(TatLayer),
    FeatureResponse(Option<Vec<String>>),
}

/// Struct for handling interfacing with GDAL in a separate thread
pub struct TatDataset {
    gdal_ds: &'static gdal::Dataset,
    response_tx: Sender<GdalResponse>,
    request_rx: Receiver<GdalRequest>,
}

impl TatDataset {
    /// Attempts to open a dataset from a string.
    pub fn new(response_tx: Sender<GdalResponse>, request_rx: Receiver<GdalRequest>, uri: String, all_drivers: bool) -> Option<Self> {
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
            }
        )
    }

    pub fn handle_requests(&self) {
        loop {
            // TODO: no unwrap blah blah
            match self.request_rx.recv().unwrap() {
                GdalRequest::LayerRequest(_) => {
                    todo!()
                },
                GdalRequest::FeatureRequest(_, _) => {
                    todo!()
                },
            }
        }
    }
}
