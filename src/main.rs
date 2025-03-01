use gdal::Dataset;
use tat::Tat;
use std::{env, io::Result, panic, process::exit};

mod tat;

fn show_usage() {
    println!("usage");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        show_usage();
        exit(1);
    }

    let file_path = &args[1];

    let ds = match Dataset::open(file_path) {
        // TODO : we get a GdalError here, it would
        // probably be better to handle the different
        // error cases and maybe give a hint to as to went
        // wrong
        Err(_) => panic!(),
        Ok(ds) => ds,
    };

    let terminal = ratatui::init();
    let result = Tat::new(&ds).run(terminal);

    ratatui::restore();

    result
}
