use tat::{fixtures::{datasets::*, init_table, TatTestStructure, TatTestUtils}, table::TatTable};
use rstest::rstest;
use insta::assert_snapshot;

fn test_single_layer_dataset(driver_name: &str, structs: (TatTestStructure, TatTable)) {
    let (test, mut t) = structs;
    TatTestUtils::set_layer_index_and_update(0, &mut t, &test.tatevent_rx);
    TatTestUtils::prepare_table_attribute_view_all_features(&t, &test.tatevent_rx, test.ds_request_tx.clone());

    let snapshot_name = format!("{}_{}", driver_name, t.layer_schema().unwrap().name());
    assert_snapshot!(snapshot_name, t);
}

fn test_all_layers(driver_name: &str, structs: (TatTestStructure, TatTable)) {
    let (test, mut t) = structs;
    for index in 0..t.layer_schemas().len() as usize {
        TatTestUtils::set_layer_index_and_update(index, &mut t, &test.tatevent_rx);
        TatTestUtils::prepare_table_attribute_view_all_features(&t, &test.tatevent_rx, test.ds_request_tx.clone());

        assert_eq!(t.layer_schema().unwrap().index(), index);
        let snapshot_name = format!("{}_{}", driver_name, t.layer_schema().unwrap().name());
        assert_snapshot!(snapshot_name, t);
    }

    test.terminate();
}

#[rstest]
fn test_basic_gdb(basic_gdb: TatTestStructure) {
    test_all_layers("openfilegdb", init_table(basic_gdb));
}

#[rstest]
fn test_basic_gml(basic_gml: TatTestStructure) {
    test_all_layers("gml", init_table(basic_gml));
}

#[rstest]
fn test_basic_jsonfg(basic_jsonfg: TatTestStructure) {
    test_all_layers("jsonfg", init_table(basic_jsonfg));
}

#[rstest]
fn test_basic_mapml(basic_mapml: TatTestStructure) {
    test_all_layers("mapml", init_table(basic_mapml));
}

#[rstest]
fn test_basic_ods(basic_ods: TatTestStructure) {
    test_all_layers("ods", init_table(basic_ods));
}

// the reason these are individual functions like this is because it works
// better with the insta snapshots
#[rstest]
fn test_basic_shp_point(basic_shp_point: TatTestStructure) {
    test_single_layer_dataset("esri_shapefile", init_table(basic_shp_point));
}

#[rstest]
fn test_basic_shp_line(basic_shp_line: TatTestStructure) {
    test_single_layer_dataset("esri_shapefile", init_table(basic_shp_line));
}

#[rstest]
fn test_basic_shp_polygon(basic_shp_polygon: TatTestStructure) {
    test_single_layer_dataset("esri_shapefile", init_table(basic_shp_polygon));
}

#[rstest]
fn test_basic_shp_multipolygon(basic_shp_multipolygon: TatTestStructure) {
    test_single_layer_dataset("esri_shapefile", init_table(basic_shp_multipolygon));
}

#[rstest]
fn test_basic_shp_nogeom(basic_shp_nogeom: TatTestStructure) {
    test_single_layer_dataset("esri_shapefile", init_table(basic_shp_nogeom));
}

#[rstest]
fn test_basic_csv_point(basic_csv_point: TatTestStructure) {
    test_single_layer_dataset("csv", init_table(basic_csv_point));
}

#[rstest]
fn test_basic_csv_line(basic_csv_line: TatTestStructure) {
    test_single_layer_dataset("csv", init_table(basic_csv_line));
}

#[rstest]
fn test_basic_csv_polygon(basic_csv_polygon: TatTestStructure) {
    test_single_layer_dataset("csv", init_table(basic_csv_polygon));
}

#[rstest]
fn test_basic_csv_multipolygon(basic_csv_multipolygon: TatTestStructure) {
    test_single_layer_dataset("csv", init_table(basic_csv_multipolygon));
}

#[rstest]
fn test_basic_csv_nogeom(basic_csv_nogeom: TatTestStructure) {
    test_single_layer_dataset("csv", init_table(basic_csv_nogeom));
}

#[rstest]
fn test_basic_geojson_point(basic_geojson_point: TatTestStructure) {
    test_single_layer_dataset("geojson", init_table(basic_geojson_point));
}

#[rstest]
fn test_basic_geojson_line(basic_geojson_line: TatTestStructure) {
    test_single_layer_dataset("geojson", init_table(basic_geojson_line));
}

#[rstest]
fn test_basic_geojson_polygon(basic_geojson_polygon: TatTestStructure) {
    test_single_layer_dataset("geojson", init_table(basic_geojson_polygon));
}

#[rstest]
fn test_basic_geojson_multipolygon(basic_geojson_multipolygon: TatTestStructure) {
    test_single_layer_dataset("geojson", init_table(basic_geojson_multipolygon));
}

#[rstest]
fn test_basic_geojson_nogeom(basic_geojson_nogeom: TatTestStructure) {
    test_single_layer_dataset("geojson", init_table(basic_geojson_nogeom));
}

#[rstest]
fn test_basic_geojsonseq_point(basic_geojsonseq_point: TatTestStructure) {
    test_single_layer_dataset("geojsonseq", init_table(basic_geojsonseq_point));
}

#[rstest]
fn test_basic_geojsonseq_line(basic_geojsonseq_line: TatTestStructure) {
    test_single_layer_dataset("geojsonseq", init_table(basic_geojsonseq_line));
}

#[rstest]
fn test_basic_geojsonseq_polygon(basic_geojsonseq_polygon: TatTestStructure) {
    test_single_layer_dataset("geojsonseq", init_table(basic_geojsonseq_polygon));
}

#[rstest]
fn test_basic_geojsonseq_multipolygon(basic_geojsonseq_multipolygon: TatTestStructure) {
    test_single_layer_dataset("geojsonseq", init_table(basic_geojsonseq_multipolygon));
}

#[rstest]
fn test_basic_geojsonseq_nogeom(basic_geojsonseq_nogeom: TatTestStructure) {
    test_single_layer_dataset("geojsonseq", init_table(basic_geojsonseq_nogeom));
}

#[rstest]
fn test_basic_jml_point(basic_jml_point: TatTestStructure) {
    test_single_layer_dataset("jml", init_table(basic_jml_point));
}

#[rstest]
fn test_basic_jml_line(basic_jml_line: TatTestStructure) {
    test_single_layer_dataset("jml", init_table(basic_jml_line));
}

#[rstest]
fn test_basic_jml_polygon(basic_jml_polygon: TatTestStructure) {
    test_single_layer_dataset("jml", init_table(basic_jml_polygon));
}

#[rstest]
fn test_basic_jml_multipolygon(basic_jml_multipolygon: TatTestStructure) {
    test_single_layer_dataset("jml", init_table(basic_jml_multipolygon));
}

#[rstest]
fn test_basic_jml_nogeom(basic_jml_nogeom: TatTestStructure) {
    test_single_layer_dataset("jml", init_table(basic_jml_nogeom));
}

#[rstest]
fn test_basic_mapinfofile_point(basic_mapinfofile_point: TatTestStructure) {
    test_single_layer_dataset("mapinfo_file", init_table(basic_mapinfofile_point));
}

#[rstest]
fn test_basic_mapinfofile_line(basic_mapinfofile_line: TatTestStructure) {
    test_single_layer_dataset("mapinfo_file", init_table(basic_mapinfofile_line));
}

#[rstest]
fn test_basic_mapinfofile_polygon(basic_mapinfofile_polygon: TatTestStructure) {
    test_single_layer_dataset("mapinfo_file", init_table(basic_mapinfofile_polygon));
}

#[rstest]
fn test_basic_mapinfofile_multipolygon(basic_mapinfofile_multipolygon: TatTestStructure) {
    test_single_layer_dataset("mapinfo_file", init_table(basic_mapinfofile_multipolygon));
}

#[rstest]
fn test_basic_mapinfofile_nogeom(basic_mapinfofile_nogeom: TatTestStructure) {
    test_single_layer_dataset("mapinfo_file", init_table(basic_mapinfofile_nogeom));
}

#[rstest]
fn test_basic_xlsx_point(basic_xlsx_point: TatTestStructure) {
    test_single_layer_dataset("xlsx", init_table(basic_xlsx_point));
}

#[rstest]
fn test_basic_xlsx_line(basic_xlsx_line: TatTestStructure) {
    test_single_layer_dataset("xlsx", init_table(basic_xlsx_line));
}

#[rstest]
fn test_basic_xlsx_polygon(basic_xlsx_polygon: TatTestStructure) {
    test_single_layer_dataset("xlsx", init_table(basic_xlsx_polygon));
}

#[rstest]
fn test_basic_xlsx_multipolygon(basic_xlsx_multipolygon: TatTestStructure) {
    test_single_layer_dataset("xlsx", init_table(basic_xlsx_multipolygon));
}

#[rstest]
fn test_basic_xlsx_nogeom(basic_xlsx_nogeom: TatTestStructure) {
    test_single_layer_dataset("xlsx", init_table(basic_xlsx_nogeom));
}
