#[cfg(test)]
use tat::fixtures::datasets::*;
use tat::app::TatApp;
use gdal::Dataset;
use rstest::rstest;
use insta::assert_snapshot;
use ratatui::{Terminal, backend::TestBackend};

#[rstest]
fn test_basic_shp(basic_shp_point: &'static Dataset, basic_shp_line: &'static Dataset, basic_shp_polygon: &'static Dataset, basic_shp_multipolygon: &'static Dataset, basic_shp_nogeom: &'static Dataset) {
    {
        let mut t = TatApp::new(basic_shp_point, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_shp_line, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_shp_polygon, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_shp_multipolygon, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_shp_nogeom, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }
}

#[rstest]
fn test_basic_jsonfg(basic_jsonfg: &'static Dataset) {
    let mut t = TatApp::new(basic_jsonfg, None, None);


}

#[rstest]
fn test_basic_mapml(basic_mapml: &'static Dataset) {
    let mut t = TatApp::new(basic_mapml, None, None);


}

#[rstest]
fn test_basic_ods(basic_ods: &'static Dataset) {
    let mut t = TatApp::new(basic_ods, None, None);


}

#[rstest]
fn test_basic_sqlite(basic_sqlite: &'static Dataset) {
    let mut t = TatApp::new(basic_sqlite, None, None);


}

#[rstest]
fn test_basic_csv(basic_csv_point: &'static Dataset, basic_csv_line: &'static Dataset, basic_csv_polygon: &'static Dataset, basic_csv_multipolygon: &'static Dataset, basic_csv_nogeom: &'static Dataset) {
    {
        let mut t = TatApp::new(basic_csv_point, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_csv_line, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_csv_polygon, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_csv_multipolygon, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_csv_nogeom, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }
}

#[rstest]
fn test_basic_geojson(basic_geojson_point: &'static Dataset, basic_geojson_line: &'static Dataset, basic_geojson_polygon: &'static Dataset, basic_geojson_multipolygon: &'static Dataset, basic_geojson_nogeom: &'static Dataset) {
    {
        let mut t = TatApp::new(basic_geojson_point, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_geojson_line, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_geojson_polygon, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_geojson_multipolygon, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_geojson_nogeom, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }
}

#[rstest]
fn test_basic_geojsonseq(basic_geojsonseq_point: &'static Dataset, basic_geojsonseq_line: &'static Dataset, basic_geojsonseq_polygon: &'static Dataset, basic_geojsonseq_multipolygon: &'static Dataset, basic_geojsonseq_nogeom: &'static Dataset) {
    {
        let mut t = TatApp::new(basic_geojsonseq_point, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_geojsonseq_line, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_geojsonseq_polygon, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_geojsonseq_multipolygon, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_geojsonseq_nogeom, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }
}

#[rstest]
fn test_basic_gml(basic_gml: &'static Dataset) {
    let mut t = TatApp::new(basic_gml, None, None);


}

#[rstest]
fn test_basic_jml(basic_jml_point: &'static Dataset, basic_jml_line: &'static Dataset, basic_jml_polygon: &'static Dataset, basic_jml_multipolygon: &'static Dataset, basic_jml_nogeom: &'static Dataset) {
    {
        let mut t = TatApp::new(basic_jml_point, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_jml_line, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_jml_polygon, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_jml_multipolygon, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_jml_nogeom, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }
}

#[rstest]
fn test_basic_mapinfofile(basic_mapinfofile_point: &'static Dataset, basic_mapinfofile_line: &'static Dataset, basic_mapinfofile_polygon: &'static Dataset, basic_mapinfofile_multipolygon: &'static Dataset, basic_mapinfofile_nogeom: &'static Dataset) {
    {
        let mut t = TatApp::new(basic_mapinfofile_point, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_mapinfofile_line, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_mapinfofile_polygon, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_mapinfofile_multipolygon, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_mapinfofile_nogeom, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }
}

#[rstest]
fn test_basic_xlsx(basic_xlsx_point: &'static Dataset, basic_xlsx_line: &'static Dataset, basic_xlsx_polygon: &'static Dataset, basic_xlsx_multipolygon: &'static Dataset, basic_xlsx_nogeom: &'static Dataset) {
    {
        let mut t = TatApp::new(basic_xlsx_point, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_xlsx_line, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_xlsx_polygon, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_xlsx_multipolygon, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }

    {
        let mut t = TatApp::new(basic_xlsx_nogeom, None, None);
        t.open_table();

        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        terminal.draw(|frame| {t.render(frame)}).unwrap();
        assert_snapshot!(terminal.backend());
    }
}
