use gdal::{errors::{CplErrType, GdalError}, Dataset, DatasetOptions, GdalOpenFlags};
use tat::Tat;
use core::panic;
use std::{env, fs::{File, OpenOptions}, process::exit};
use cli_log::*;
use std::io::prelude::*;

mod tat;

fn show_usage() {
    // TODO:
    println!("Attribute Table for GIS data in the terminal.\n");
    println!("Usage: tat [URI]");
}

fn error_handler(class: CplErrType, number: i32, message: &str) {
    let class = match class {
        CplErrType::None => "[NONE]",
        CplErrType::Debug => "[DEBUG]",
        CplErrType::Warning => "[WARN]",
        CplErrType::Failure => "[ERROR]",
        CplErrType::Fatal => "[FATAL]",
    };

    // TODO: no unwrapping
    let mut file = OpenOptions::new()
        .append(true)
        .open("tat_gdal.log")
        .unwrap();

    if let Err(e) = writeln!(file, "{class} [{number}] {message}") {
        // TODO: no panic
        panic!();
    }
}

fn open_dataset(path: String, err: &mut String) -> Option<Dataset> {
    // deal with vectors only at least for now
    let flags = GdalOpenFlags::GDAL_OF_VECTOR | GdalOpenFlags::GDAL_OF_READONLY;

    let options = DatasetOptions {
        open_flags: flags,
        allowed_drivers: None,
        open_options: None,
        sibling_files: None,
    };

    let ds = match Dataset::open_ex(path, options) {
        Err(error) => match error {
            GdalError::FfiNulError(_) => {
                *err = "FFI NULL Error".to_string();
                return None;
            },
            GdalError::FfiIntoStringError(_) => {
                *err = "FFI Into String Error".to_string();
                return None;
            },
            GdalError::StrUtf8Error(_) => {
                *err = "String UTF-8 Error".to_string();
                return None;
            },
            GdalError::CplError { class, number, msg } => {
                *err = format!("CPL Error: {} {} {}", class, number, msg);
                return None;
            },
            GdalError::NullPointer { method_name, msg } => {
                *err = format!("{} {}", method_name, msg);
                return None;
            },
            GdalError::CastToF64Error => {
                *err = "Cast to F64 Error".to_string();
                return None;
            },
            GdalError::OgrError { err, method_name } => {
                match err {
                    _ => {
                        debug!("{}", method_name);
                        todo!();
                    }
                }
            },
            GdalError::UnhandledFieldType { field_type, method_name } => {
                *err = format!("Unhandled Field Type: {} {}", field_type, method_name);
                return None;
            },
            GdalError::InvalidFieldName { field_name, method_name } => {
                *err = format!("Invalid Field Name: {} {}", field_name, method_name);
                return None;
            },
            // GdalError::InvalidFieldIndex { index, method_name } => todo!("{} {}", index, method_name),
            // GdalError::UnlinkedGeometry { method_name } => todo!("{}", method_name),
            // GdalError::InvalidCoordinateRange { from, to, msg } => todo!("{} {} {}", from, to, msg.unwrap()),
            // GdalError::AxisNotFoundError { key, method_name } => todo!("{} {}", key, method_name),
            // GdalError::UnsupportedGdalGeometryType(_) => todo!(),
            // GdalError::UnlinkMemFile { file_name } => todo!("{}", file_name),
            // GdalError::BadArgument(_) => todo!(),
            // GdalError::DateError(str) => todo!("{}", str),
            // GdalError::UnsupportedMdDataType { data_type, method_name } => todo!("{} {}", data_type, method_name),
            // GdalError::IntConversionError(_) => todo!(),
            // GdalError::BufferSizeMismatch(..) => todo!(),
            _ => {
                *err = "Unspecified GDAL error occured".to_string();
                return None;
            },
        },
        Ok(ds) => ds,
    };

    if err.is_empty() {
        println!("{}", err);
    }

    return Some(ds);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        show_usage();
        exit(1);
    }

    let path = &args[1];

    let _ = File::create("tat_gdal.log").unwrap();
    gdal::config::set_error_handler(error_handler);

    init_cli_log!();

    let mut err: String = "".to_string();

    if let Some(ds) = open_dataset(path.to_string(), &mut err) {
        let mut terminal = ratatui::init();
        let _result = Tat::new(ds).run(&mut terminal);
        ratatui::restore();
    } else {
        println!("ERROR: Could not open dataset from path!\n{}\nUsage:", err);
        show_usage();
    }
}
