use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use gdal::{errors::{CplErrType, GdalError}, Dataset, DatasetOptions, GdalOpenFlags};
use tat::Tat;
use unicode_segmentation::UnicodeSegmentation;
use std::{env::temp_dir, fs::{File, OpenOptions}};
use cli_log::*;
use std::io::prelude::*;
use clap::{CommandFactory, Parser};

mod layer;
mod layerlist;
mod navparagraph;
mod numberinput;
mod shared;
mod table;
mod tat;
mod types;

/// Function which is set as GDAL's error handler. Being a terminal app the errors have to be
/// redirected to a file, which can be displayed in the program.
fn error_handler(class: CplErrType, number: i32, message: &str) {
    let class = match class {
        CplErrType::None => "[NONE]",
        CplErrType::Debug => "[DEBUG]",
        CplErrType::Warning => "[WARN]",
        CplErrType::Failure => "[ERROR]",
        CplErrType::Fatal => "[FATAL]",
    };

    let path = format!("{}/tat_gdal.log", temp_dir().display());
    match OpenOptions::new().append(true).open(path.clone()) {
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
fn open_dataset(uri: String) -> Option<&'static Dataset> {
    // deal with vectors only at least for now
    let flags = GdalOpenFlags::GDAL_OF_VECTOR | GdalOpenFlags::GDAL_OF_READONLY;

    let options = DatasetOptions {
        open_flags: flags,
        allowed_drivers: None,
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

#[derive(Parser)]
#[command(arg_required_else_help = true)]
#[command(version, about, long_about = None)]
struct Cli {
    uri: String,

    #[arg(long = "where", value_name = "WHERE", help = "Filter feature based on attributes", long_help = "Filter which features are shown based on their attributes. Given in the format of a SQL WHERE clause e.g. --where=\"field_1 = 12\"")]
    where_sql: Option<String>,

    #[arg(long = "layers", value_name = "LAYERS", help = "Layer(s) to open", long_help = "Specify which layers in the dataset should be opened. Given as a comma-separated list e.g. \"--layers=layer_1,layer_2\"")]
    layers: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let uri = cli.uri;
    let where_clause = cli.where_sql;

    let layers = if let Some(lyrs) = cli.layers {
        let layers = lyrs.split(',');
        let mut _layers = vec![];
        for _lyr in layers {
            _layers.push(_lyr.to_string());
        }

        Some(_layers)
    } else {
        None
    };

    let _ = File::create(format!("{}/tat_gdal.log", temp_dir().display())).unwrap();
    gdal::config::set_error_handler(error_handler);

    if let Some(ds) = open_dataset(uri.to_string()) {
        init_cli_log!();
        let mut terminal = ratatui::init();
        crossterm::execute!(std::io::stdout(), EnableMouseCapture).unwrap();

        let _result = Tat::new(&ds, where_clause, layers).run(&mut terminal);

        // FIXME: if the program terminates this will not happen
        crossterm::execute!(std::io::stdout(), DisableMouseCapture).unwrap();
        ratatui::restore();
    } else {
        Cli::command().print_help().unwrap();
    }
}
