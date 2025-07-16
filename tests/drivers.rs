#[cfg(test)]
use tat::fixtures::basic_shp_point;
use gdal::Dataset;
use rstest::rstest;

#[rstest]
fn test_some_driver(basic_shp_point: &'static Dataset) {
    ()
}
