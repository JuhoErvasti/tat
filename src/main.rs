use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use gdal::Dataset;
use ratatui::{prelude::CrosstermBackend, Terminal};
use tat::Tat;
use std::{env, io::{BufWriter, Result}, panic, process::exit};
use cli_log::*;

mod tat;

fn show_usage() {
    // TODO:
    println!("Attribute Table for GIS data in the terminal.\n");
    println!("Usage: tat [URI]");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        show_usage();
        exit(1);
    }

    let file_path = &args[1];

    // TODO: expand on these options and the opening of the dataset otherwise.
    // For example currently you can't open a PostGIS dataset because by default
    // gdal tries to read both vector and raster datasets and you get some
    // discovery error with the rasters.
    // let flags = GdalOpenFlags::GDAL_OF_VECTOR | GdalOpenFlags::GDAL_OF_READONLY;
    //
    // let options = DatasetOptions {
    //     open_flags: flags,
    //     allowed_drivers: None,
    //     open_options: None,
    //     sibling_files: None,
    // };
    //
    // let ds = match Dataset::open_ex(file_path, options) {
    let ds = match Dataset::open(file_path) {
        // TODO : we get a GdalError here, it would
        // probably be better to handle the different
        // error cases and maybe give a hint to as to went
        // wrong
        Err(_) => panic!(),
        Ok(ds) => ds,
    };

    init_cli_log!();

    let mut terminal = ratatui::init();
    let result = Tat::new(ds).run(&mut terminal);

    ratatui::restore();

    result
}
