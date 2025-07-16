use cli_log::init_cli_log;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use std::{env::temp_dir, fs::File};
use clap::{CommandFactory, Parser};

use tat::app::TatApp;
use tat::utils::{error_handler, open_dataset};

#[derive(Parser)]
#[command(arg_required_else_help = true)]
#[command(version, about, long_about = None)]
struct Cli {
    uri: String,

    #[arg(long = "where", value_name = "WHERE", help = "Filter feature based on attributes", long_help = "Filter which features are shown based on their attributes. Given in the format of a SQL WHERE clause e.g. --where=\"field_1 = 12\"")]
    where_sql: Option<String>,

    #[arg(long = "layers", value_name = "LAYERS", help = "Layer(s) to open", long_help = "Specify which layers in the dataset should be opened. Given as a comma-separated list e.g. \"--layers=layer_1,layer_2\"")]
    layers: Option<String>,

    #[arg(long = "allow-untested-drivers", value_name = "ALLOW_UNTESTED_DRIVERS", help = "Allow attempting to open dataset of any type which has a GDAL-supported vector driver. Use with caution.")]
    all_drivers: bool,
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

    if let Some(ds) = open_dataset(uri.to_string(), cli.all_drivers) {
        init_cli_log!();
        let mut terminal = ratatui::init();
        crossterm::execute!(std::io::stdout(), EnableMouseCapture).unwrap();

        let _result = TatApp::new(&ds, where_clause, layers).run(&mut terminal);

        // FIXME: if the program terminates this will not happen
        crossterm::execute!(std::io::stdout(), DisableMouseCapture).unwrap();
        ratatui::restore();
    } else {
        Cli::command().print_help().unwrap();
    }
}
