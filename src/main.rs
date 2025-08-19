#[allow(unused_imports)]
use cli_log::*;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use tat::dataset::{DatasetRequest, DatasetResponse, TatDataset};
use std::sync::mpsc::{self, SendError, TryRecvError};
use std::thread::{self};
use std::time::Duration;
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

fn handle_events(tx: mpsc::Sender<TatEvent>, rx: mpsc::Receiver<bool>) -> Result<(), SendError<TatEvent>> {
    loop {
        let poll_result = crossterm::event::poll(Duration::from_millis(50));
        if let Ok(res) = poll_result {
            if res {
                match crossterm::event::read().unwrap() {
                    crossterm::event::Event::Key(key_event) => {
                        tx.send(TatEvent::Keyboard(key_event))?
                    },
                    crossterm::event::Event::Mouse(mouse_event) => {
                        tx.send(TatEvent::Mouse(mouse_event))?
                    },
                    _ => (),
                }
            }
        }

        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                return Ok(());
            }
            Err(TryRecvError::Empty) => {},
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
    let (event_thread_tx, event_thread_rx) = mpsc::channel();

    // has to be cloned here because it's used later and moved to a closure here
    let cp_tatevent_tx = tatevent_tx.clone();

    let ds_handle = thread::spawn(move || {
        if let Some(mut ds) = TatDataset::new(
            cp_tatevent_tx.clone(),
            dataset_request_rx,
            uri.to_string(),
            cli.all_drivers,
            where_clause,
            layer_filter,
        ) {
            ds.handle_requests();
        } else {
            cp_tatevent_tx.send(
                TatEvent::Dataset(
                    DatasetResponse::InvalidDataset,
                )
            ).unwrap();
        }
    });

    let mut ds_okay = false;
    while !ds_okay {
        match tatevent_rx.recv().unwrap() {
            TatEvent::Dataset(dataset_response) => {
                match dataset_response {
                    DatasetResponse::InvalidDataset => {
                        Cli::command().print_help().unwrap();
                        ds_handle.join().unwrap();
                        return;
                    },
                    DatasetResponse::DatasetCreated => {
                        ds_okay = true;
                    },
                    _ => (),
                }
            }
            _ => (),
        };
    }


    init_cli_log!();
    let mut terminal = ratatui::init();
    crossterm::execute!(std::io::stdout(), EnableMouseCapture).unwrap();

    let event_handle = thread::spawn(move || {
        match handle_events(tatevent_tx, event_thread_rx) {
            Ok(_) => (),
            Err(_) => (),
        }
    });

    let _result = TatApp::new(dataset_request_tx.clone())
        .run(&mut terminal, tatevent_rx);

    match event_thread_tx.send(true) {
        Ok(_) => (),
        Err(err) => error!("Could not send signal to terminate event loop {}", err.to_string()),
    }

    event_handle.join().unwrap();
    ds_handle.join().unwrap();

    // FIXME: if the program panics or is killed this will not happen
    crossterm::execute!(std::io::stdout(), DisableMouseCapture).unwrap();
    ratatui::restore();
}
