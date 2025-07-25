use std::{env::temp_dir, fs::OpenOptions};
use std::io::prelude::Write;
use cli_log::debug;

use gdal::{errors::{CplErrType, GdalError}, Dataset, DatasetOptions, GdalOpenFlags};
use unicode_segmentation::UnicodeSegmentation;

/// Function which is set as GDAL's error handler. Being a terminal app the errors have to be
/// redirected to a file, which can be displayed in the program.
pub fn error_handler(class: CplErrType, number: i32, message: &str) {
    let class = match class {
        CplErrType::None => "[NONE]",
        CplErrType::Debug => "[DEBUG]",
        CplErrType::Warning => "[WARN]",
        CplErrType::Failure => "[ERROR]",
        CplErrType::Fatal => "[FATAL]",
    };

    let path = format!("{}/tat_gdal.log", temp_dir().display());
    match OpenOptions::new().append(true).open(path.as_str()) {
        Ok(mut file) => {
            match writeln!(file, "{class} [{number}] {message}") {
                Ok(()) => return,
                Err(e) => {
                    debug!("Could not write to log at \"{path}\": {}", e.to_string());
                }
            }
        },
        Err(e) => {
            debug!("Could not open log at \"{path}\" for writing: {}", e.to_string());
        },
    }
}

/// Attempts to open a GDAL dataset from a string. This dataset is required from the beginning to
/// the end of the program so it is returned as a static variable.
pub fn open_dataset(uri: String, all_drivers: bool) -> Option<&'static Dataset> {
    // deal with vectors only at least for now
    let flags = GdalOpenFlags::GDAL_OF_VECTOR | GdalOpenFlags::GDAL_OF_READONLY;

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

    let options = DatasetOptions {
        open_flags: flags,
        allowed_drivers: if all_drivers { None } else {Some(&v)},
        open_options: None,
        sibling_files: None,
    };

    let ds = match Dataset::open_ex(uri, options) {
        Ok(ds) => ds,
        Err(error) => {
            match error {
                GdalError::NullPointer { method_name: _, msg } => {
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

    Some(Box::leak(Box::new(ds)))
}

