use cli_log::{info, init_cli_log};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use tat::dataset::{DatasetRequest, DatasetResponse, TatDataset};
use std::sync::mpsc::{self, SendError};
use std::thread;
use std::{env::temp_dir, fs::File};
use clap::{CommandFactory, Parser};

use tat::app::{TatApp, TatEvent};
use tat::utils::error_handler;

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

fn handle_events(tx: mpsc::Sender<TatEvent>) -> Result<(), SendError<TatEvent>> {
    loop {
        // TODO: don't unwrap
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => {
                tx.send(TatEvent::Keyboard(key_event))?
            },
            crossterm::event::Event::Mouse(mouse_event) => {
                tx.send(TatEvent::Mouse(mouse_event))?
            },
            _ => {},
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let uri = cli.uri;
    let where_clause = cli.where_sql;

    let layer_filter = if let Some(lyrs) = cli.layers {
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

    let (dataset_request_tx, dataset_request_rx) = mpsc::channel::<DatasetRequest>();
    let (tatevent_tx, tatevent_rx) = mpsc::channel::<TatEvent>();

    // has to be cloned here because it's used later and moved to a closure here
    let cp_tatevent_tx = tatevent_tx.clone();

    let ds_handle = thread::spawn(move || {
        if let Some(mut ds) = TatDataset::new(
            cp_tatevent_tx,
            dataset_request_rx,
            uri.to_string(),
            cli.all_drivers,
            where_clause,
            layer_filter,
        ) {
            ds.handle_requests();
        } else {
            return;
        }
    });

    // TODO: really not sure this works at all
    // maybe we need to send a GdalResponse to confirm the dataset was opened or something
    if ds_handle.is_finished() {
        Cli::command().print_help().unwrap();
        return;
    }

    init_cli_log!();
    let mut terminal = ratatui::init();

    // TODO: maybe we do rendering 30fps or something

    let event_handle = thread::spawn(move || {
        handle_events(tatevent_tx).unwrap();
    });

    let _result = TatApp::new(dataset_request_tx.clone())
        .run(&mut terminal, tatevent_rx);

    dataset_request_tx.send(DatasetRequest::Terminate).unwrap();
    ds_handle.join().unwrap();

    // ds_handle.join().unwrap();
    // event_handle.join().unwrap();

    // FIXME: if the program panics or is killed this will not happen
    // crossterm::execute!(std::io::stdout(), DisableMouseCapture).unwrap();
    ratatui::restore();
}
