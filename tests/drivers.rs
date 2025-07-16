#[cfg(test)]
use tat::fixtures::datasets::*;
use tat::app::TatApp;
use gdal::Dataset;
use rstest::rstest;
use insta::assert_snapshot;
use ratatui::{Terminal, backend::TestBackend};

fn test_single_layer_dataset(ds: &'static Dataset) {
    let mut t = TatApp::new(ds, None, None);
    t.open_table();

    let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
    terminal.draw(|frame| {t.render(frame)}).unwrap();
    assert_snapshot!(terminal.backend());
}

fn test_all_layers(t: &mut TatApp) {
    {
        t.set_layer_index(0);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());

        t.close_table();
    }

    {
        t.set_layer_index(1);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());

        t.close_table();
    }

    {
        t.set_layer_index(2);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());

        t.close_table();
    }

    {
        t.set_layer_index(3);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());

        t.close_table();
    }

    {
        t.set_layer_index(4);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());

        t.close_table();
    }
}

#[rstest]
fn test_basic_gdb(basic_gdb: &'static Dataset) {
    let mut t = TatApp::new(basic_gdb, None, None);
    test_all_layers(&mut t);
}

#[rstest]
fn test_basic_gml(basic_gml: &'static Dataset) {
    let mut t = TatApp::new(basic_gml, None, None);
    test_all_layers(&mut t);
}

#[rstest]
fn test_basic_jsonfg(basic_jsonfg: &'static Dataset) {
    let mut t = TatApp::new(basic_jsonfg, None, None);
    test_all_layers(&mut t);
}

#[rstest]
fn test_basic_mapml(basic_mapml: &'static Dataset) {
    let mut t = TatApp::new(basic_mapml, None, None);
    test_all_layers(&mut t);
}

#[rstest]
fn test_basic_ods(basic_ods: &'static Dataset) {
    let mut t = TatApp::new(basic_ods, None, None);
    test_all_layers(&mut t);
}

#[rstest]
fn test_basic_sqlite(basic_sqlite: &'static Dataset) {
    let mut t = TatApp::new(basic_sqlite, None, None);
    test_all_layers(&mut t);
}

#[rstest]
fn test_basic_shp(
    basic_shp_point: &'static Dataset,
    basic_shp_line: &'static Dataset,
    basic_shp_polygon: &'static Dataset,
    basic_shp_multipolygon: &'static Dataset,
    basic_shp_nogeom: &'static Dataset,
) {
    test_single_layer_dataset(basic_shp_point);
    test_single_layer_dataset(basic_shp_line);
    test_single_layer_dataset(basic_shp_polygon);
    test_single_layer_dataset(basic_shp_multipolygon);
    test_single_layer_dataset(basic_shp_nogeom);
}

#[rstest]
fn test_basic_csv(
    basic_csv_point: &'static Dataset,
    basic_csv_line: &'static Dataset,
    basic_csv_polygon: &'static Dataset,
    basic_csv_multipolygon: &'static Dataset,
    basic_csv_nogeom: &'static Dataset,
) {
    test_single_layer_dataset(basic_csv_point);
    test_single_layer_dataset(basic_csv_line);
    test_single_layer_dataset(basic_csv_polygon);
    test_single_layer_dataset(basic_csv_multipolygon);
    test_single_layer_dataset(basic_csv_nogeom);
}

#[rstest]
fn test_basic_geojson(
    basic_geojson_point: &'static Dataset,
    basic_geojson_line: &'static Dataset,
    basic_geojson_polygon: &'static Dataset,
    basic_geojson_multipolygon: &'static Dataset,
    basic_geojson_nogeom: &'static Dataset,
) {
    test_single_layer_dataset(basic_geojson_point);
    test_single_layer_dataset(basic_geojson_line);
    test_single_layer_dataset(basic_geojson_polygon);
    test_single_layer_dataset(basic_geojson_multipolygon);
    test_single_layer_dataset(basic_geojson_nogeom);
}

#[rstest]
fn test_basic_geojsonseq(
    basic_geojsonseq_point: &'static Dataset,
    basic_geojsonseq_line: &'static Dataset,
    basic_geojsonseq_polygon: &'static Dataset,
    basic_geojsonseq_multipolygon: &'static Dataset,
    basic_geojsonseq_nogeom: &'static Dataset,
) {
    test_single_layer_dataset(basic_geojsonseq_point);
    test_single_layer_dataset(basic_geojsonseq_line);
    test_single_layer_dataset(basic_geojsonseq_polygon);
    test_single_layer_dataset(basic_geojsonseq_multipolygon);
    test_single_layer_dataset(basic_geojsonseq_nogeom);
}

#[rstest]
fn test_basic_jml(
    basic_jml_point: &'static Dataset,
    basic_jml_line: &'static Dataset,
    basic_jml_polygon: &'static Dataset,
    basic_jml_multipolygon: &'static Dataset,
    basic_jml_nogeom: &'static Dataset,
) {
    test_single_layer_dataset(basic_jml_point);
    test_single_layer_dataset(basic_jml_line);
    test_single_layer_dataset(basic_jml_polygon);
    test_single_layer_dataset(basic_jml_multipolygon);
    test_single_layer_dataset(basic_jml_nogeom);
}

#[rstest]
fn test_basic_mapinfofile(
    basic_mapinfofile_point: &'static Dataset,
    basic_mapinfofile_line: &'static Dataset,
    basic_mapinfofile_polygon: &'static Dataset,
    basic_mapinfofile_multipolygon: &'static Dataset,
    basic_mapinfofile_nogeom: &'static Dataset,
) {
    test_single_layer_dataset(basic_mapinfofile_point);
    test_single_layer_dataset(basic_mapinfofile_line);
    test_single_layer_dataset(basic_mapinfofile_polygon);
    test_single_layer_dataset(basic_mapinfofile_multipolygon);
    test_single_layer_dataset(basic_mapinfofile_nogeom);
}

#[rstest]
fn test_basic_xlsx(
    basic_xlsx_point: &'static Dataset,
    basic_xlsx_line: &'static Dataset,
    basic_xlsx_polygon: &'static Dataset,
    basic_xlsx_multipolygon: &'static Dataset,
    basic_xlsx_nogeom: &'static Dataset,
) {
    test_single_layer_dataset(basic_xlsx_point);
    test_single_layer_dataset(basic_xlsx_line);
    test_single_layer_dataset(basic_xlsx_polygon);
    test_single_layer_dataset(basic_xlsx_multipolygon);
    test_single_layer_dataset(basic_xlsx_nogeom);
}
