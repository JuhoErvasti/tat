use std::{env::temp_dir, fs::OpenOptions};
use std::io::prelude::Write;
use cli_log::{debug, error};

use gdal::errors::CplErrType;

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
                    error!("Could not write to log at \"{path}\": {}", e.to_string());
                }
            }
        },
        Err(e) => {
            error!("Could not open log at \"{path}\" for writing: {}", e.to_string());
        },
    }
}

