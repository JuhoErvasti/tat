use gdal::Dataset;
use rstest::fixture;

use crate::utils::open_dataset;

#[fixture]
pub fn basic_gpkg() -> &'static Dataset {
    let uri = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/basic.gpkg").to_string();

    open_dataset(uri).unwrap()
}
