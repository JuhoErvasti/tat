use std::fmt::Display;

use tat::fixtures::datasets::*;
use tat::{app::TatApp, layer::TatLayer};
use gdal::Dataset;
use rstest::{fixture, rstest};

pub type LayerRes = Vec<Vec<Option<String>>>;

pub struct LayerResults {
    res: LayerRes,
}

impl Display for LayerResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
    }
}

#[fixture]
fn gpkg_result_point(basic_gpkg: &'static Dataset) -> LayerResults {
    let mut t = TatApp::new(basic_gpkg, None, None);
    t.set_layer_index(0);

    build_test_results_from_layer(&t.table().layer())
}

#[fixture]
fn gpkg_result_line(basic_gpkg: &'static Dataset) -> LayerResults {
    let mut t = TatApp::new(basic_gpkg, None, None);
    t.set_layer_index(1);

    build_test_results_from_layer(&t.table().layer())
}

#[fixture]
fn gpkg_result_polygon(basic_gpkg: &'static Dataset) -> LayerResults {
    let mut t = TatApp::new(basic_gpkg, None, None);
    t.set_layer_index(2);

    build_test_results_from_layer(&t.table().layer())
}

#[fixture]
fn gpkg_result_multipolygon(basic_gpkg: &'static Dataset) -> LayerResults {
    let mut t = TatApp::new(basic_gpkg, None, None);
    t.set_layer_index(3);

    build_test_results_from_layer(&t.table().layer())
}

#[fixture]
fn gpkg_result_nogeom(basic_gpkg: &'static Dataset) -> LayerResults {
    let mut t = TatApp::new(basic_gpkg, None, None);
    t.set_layer_index(4);

    build_test_results_from_layer(&t.table().layer())
}

#[fixture]
fn gpkg_results(
    gpkg_result_point: LayerResults,
    gpkg_result_line: LayerResults,
    gpkg_result_polygon: LayerResults,
    gpkg_result_multipolygon: LayerResults,
    gpkg_result_nogeom: LayerResults,
) -> Vec<LayerResults> {
    vec![
        gpkg_result_point,
        gpkg_result_line,
        gpkg_result_polygon,
        gpkg_result_multipolygon,
        gpkg_result_nogeom,
    ]
}

fn build_test_results_from_layer(layer: &TatLayer) -> LayerResults {
    let mut headers: Vec<Option<String>> = vec![];
    for field in 0..layer.field_count() {
        headers.push(layer.field_name_by_id(field as i32));
    }

    let mut features: Vec<Vec<Option<String>>> = vec![headers];


    for row in 0..layer.feature_count() {
        let mut feature: Vec<Option<String>> = vec![];
        for field in 0..layer.field_count() {
            feature.push(layer.get_value_by_row(row as u64, field as i32));
        }
        features.push(feature);
    }

    LayerResults { res: features }
}

fn compare(results: &LayerResults, expected: &LayerResults) {
    assert_eq!(results.len(), expected.len());

    for row in 0..results.len() {
        let result_row = results.get(row).unwrap();
        let expected_row = expected.get(row).unwrap();

        assert_eq!(result_row.len(), expected_row.len());

        for field in 0..result_row.len() {
            let result = result_row.get(field).unwrap();
            let expected = expected_row.get(field).unwrap();

            assert_eq!(result, expected);
        }
    }
}

fn test_single_layer_dataset(ds: &'static Dataset, expected: &LayerResults) {
    let t = TatApp::new(ds, None, None);
    let result = build_test_results_from_layer(&t.table().layer());
    compare(&result, &expected);
}

fn test_all_layers(ds: &'static Dataset, expected: &Vec<LayerResults>) {
    let mut t = TatApp::new(ds, None, None);
    for index in 0..5 as usize {
        t.set_layer_index(index);
        println!("Testing layer: {}", t.table().layer().name());
        let result = build_test_results_from_layer(&t.table().layer());
        compare(&result, &expected.get(index).unwrap());
    }
}

#[rstest]
fn test_basic_gdb(basic_gdb: &'static Dataset, gpkg_results: Vec<LayerResults>) {
    test_all_layers(basic_gdb, &gpkg_results);
}

#[rstest]
fn test_basic_gml(basic_gml: &'static Dataset, gpkg_results: Vec<LayerResults>) {
    test_all_layers(basic_gml, &gpkg_results);
}

#[rstest]
fn test_basic_jsonfg(basic_jsonfg: &'static Dataset, gpkg_results: Vec<LayerResults>) {
    test_all_layers(basic_jsonfg, &gpkg_results);
}

#[rstest]
fn test_basic_mapml(basic_mapml: &'static Dataset, gpkg_results: Vec<LayerResults>) {
    test_all_layers(basic_mapml, &gpkg_results);
}

#[rstest]
fn test_basic_ods(basic_ods: &'static Dataset, gpkg_results: Vec<LayerResults>) {
    test_all_layers(basic_ods, &gpkg_results);
}

#[rstest]
fn test_basic_sqlite(basic_sqlite: &'static Dataset, gpkg_results: Vec<LayerResults>) {
    test_all_layers(basic_sqlite, &gpkg_results);
}

#[rstest]
fn test_basic_shp(
    basic_shp_point: &'static Dataset,
    basic_shp_line: &'static Dataset,
    basic_shp_polygon: &'static Dataset,
    basic_shp_multipolygon: &'static Dataset,
    basic_shp_nogeom: &'static Dataset,
    gpkg_result_point: LayerResults,
    gpkg_result_line: LayerResults,
    gpkg_result_polygon: LayerResults,
    gpkg_result_multipolygon: LayerResults,
    gpkg_result_nogeom: LayerResults,
) {
    test_single_layer_dataset(basic_shp_point, &gpkg_result_point);
    test_single_layer_dataset(basic_shp_line, &gpkg_result_line);
    test_single_layer_dataset(basic_shp_polygon, &gpkg_result_polygon);
    test_single_layer_dataset(basic_shp_multipolygon, &gpkg_result_multipolygon);
    test_single_layer_dataset(basic_shp_nogeom, &gpkg_result_nogeom);
}

#[rstest]
fn test_basic_csv(
    basic_csv_point: &'static Dataset,
    basic_csv_line: &'static Dataset,
    basic_csv_polygon: &'static Dataset,
    basic_csv_multipolygon: &'static Dataset,
    basic_csv_nogeom: &'static Dataset,
    gpkg_result_point: LayerResults,
    gpkg_result_line: LayerResults,
    gpkg_result_polygon: LayerResults,
    gpkg_result_multipolygon: LayerResults,
    gpkg_result_nogeom: LayerResults,
) {
    test_single_layer_dataset(basic_csv_point, &gpkg_result_point);
    test_single_layer_dataset(basic_csv_line, &gpkg_result_line);
    test_single_layer_dataset(basic_csv_polygon, &gpkg_result_polygon);
    test_single_layer_dataset(basic_csv_multipolygon, &gpkg_result_multipolygon);
    test_single_layer_dataset(basic_csv_nogeom, &gpkg_result_nogeom);
}

#[rstest]
fn test_basic_geojson(
    basic_geojson_point: &'static Dataset,
    basic_geojson_line: &'static Dataset,
    basic_geojson_polygon: &'static Dataset,
    basic_geojson_multipolygon: &'static Dataset,
    basic_geojson_nogeom: &'static Dataset,
    gpkg_result_point: LayerResults,
    gpkg_result_line: LayerResults,
    gpkg_result_polygon: LayerResults,
    gpkg_result_multipolygon: LayerResults,
    gpkg_result_nogeom: LayerResults,
) {
    test_single_layer_dataset(basic_geojson_point, &gpkg_result_point);
    test_single_layer_dataset(basic_geojson_line, &gpkg_result_line);
    test_single_layer_dataset(basic_geojson_polygon, &gpkg_result_polygon);
    test_single_layer_dataset(basic_geojson_multipolygon, &gpkg_result_multipolygon);
    test_single_layer_dataset(basic_geojson_nogeom, &gpkg_result_nogeom);
}

#[rstest]
fn test_basic_geojsonseq(
    basic_geojsonseq_point: &'static Dataset,
    basic_geojsonseq_line: &'static Dataset,
    basic_geojsonseq_polygon: &'static Dataset,
    basic_geojsonseq_multipolygon: &'static Dataset,
    basic_geojsonseq_nogeom: &'static Dataset,
    gpkg_result_point: LayerResults,
    gpkg_result_line: LayerResults,
    gpkg_result_polygon: LayerResults,
    gpkg_result_multipolygon: LayerResults,
    gpkg_result_nogeom: LayerResults,
) {
    test_single_layer_dataset(basic_geojsonseq_point, &gpkg_result_point);
    test_single_layer_dataset(basic_geojsonseq_line, &gpkg_result_line);
    test_single_layer_dataset(basic_geojsonseq_polygon, &gpkg_result_polygon);
    test_single_layer_dataset(basic_geojsonseq_multipolygon, &gpkg_result_multipolygon);
    test_single_layer_dataset(basic_geojsonseq_nogeom, &gpkg_result_nogeom);
}

#[rstest]
fn test_basic_jml(
    basic_jml_point: &'static Dataset,
    basic_jml_line: &'static Dataset,
    basic_jml_polygon: &'static Dataset,
    basic_jml_multipolygon: &'static Dataset,
    basic_jml_nogeom: &'static Dataset,
    gpkg_result_point: LayerResults,
    gpkg_result_line: LayerResults,
    gpkg_result_polygon: LayerResults,
    gpkg_result_multipolygon: LayerResults,
    gpkg_result_nogeom: LayerResults,
) {
    test_single_layer_dataset(basic_jml_point, &gpkg_result_point);
    test_single_layer_dataset(basic_jml_line, &gpkg_result_line);
    test_single_layer_dataset(basic_jml_polygon, &gpkg_result_polygon);
    test_single_layer_dataset(basic_jml_multipolygon, &gpkg_result_multipolygon);
    test_single_layer_dataset(basic_jml_nogeom, &gpkg_result_nogeom);
}

#[rstest]
fn test_basic_mapinfofile(
    basic_mapinfofile_point: &'static Dataset,
    basic_mapinfofile_line: &'static Dataset,
    basic_mapinfofile_polygon: &'static Dataset,
    basic_mapinfofile_multipolygon: &'static Dataset,
    basic_mapinfofile_nogeom: &'static Dataset,
    gpkg_result_point: LayerResults,
    gpkg_result_line: LayerResults,
    gpkg_result_polygon: LayerResults,
    gpkg_result_multipolygon: LayerResults,
    gpkg_result_nogeom: LayerResults,
) {
    test_single_layer_dataset(basic_mapinfofile_point, &gpkg_result_point);
    test_single_layer_dataset(basic_mapinfofile_line, &gpkg_result_line);
    test_single_layer_dataset(basic_mapinfofile_polygon, &gpkg_result_polygon);
    test_single_layer_dataset(basic_mapinfofile_multipolygon, &gpkg_result_multipolygon);
    test_single_layer_dataset(basic_mapinfofile_nogeom, &gpkg_result_nogeom);
}

#[rstest]
fn test_basic_xlsx(
    basic_xlsx_point: &'static Dataset,
    basic_xlsx_line: &'static Dataset,
    basic_xlsx_polygon: &'static Dataset,
    basic_xlsx_multipolygon: &'static Dataset,
    basic_xlsx_nogeom: &'static Dataset,
    gpkg_result_point: LayerResults,
    gpkg_result_line: LayerResults,
    gpkg_result_polygon: LayerResults,
    gpkg_result_multipolygon: LayerResults,
    gpkg_result_nogeom: LayerResults,
) {
    test_single_layer_dataset(basic_xlsx_point, &gpkg_result_point);
    test_single_layer_dataset(basic_xlsx_line, &gpkg_result_line);
    test_single_layer_dataset(basic_xlsx_polygon, &gpkg_result_polygon);
    test_single_layer_dataset(basic_xlsx_multipolygon, &gpkg_result_multipolygon);
    test_single_layer_dataset(basic_xlsx_nogeom, &gpkg_result_nogeom);
}
