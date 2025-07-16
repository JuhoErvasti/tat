use ratatui::layout::Rect;
use rstest::fixture;
    use gdal::Dataset;

use crate::{table::{TableRects, TatTable}, app::TatApp};

pub mod datasets {
    use rstest::fixture;
    use gdal::Dataset;
    use crate::utils::open_dataset;

    #[fixture]
    pub fn basic_gpkg() -> &'static Dataset {
        let uri = "./testdata/basic.gpkg".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_shp_point() -> &'static Dataset {
        let uri = "./testdata/shp/point.shp".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_shp_line() -> &'static Dataset {
        let uri = "./testdata/shp/line.shp".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_shp_polygon() -> &'static Dataset {
        let uri = "./testdata/shp/polygon.shp".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_shp_multipolygon() -> &'static Dataset {
        let uri = "./testdata/shp/multipolygon.shp".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_shp_nogeom() -> &'static Dataset {
        let uri = "./testdata/shp/nogeom.dbf".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_jsonfg() -> &'static Dataset {
        let uri = "./testdata/basic.jsonfg".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_mapml() -> &'static Dataset {
        let uri = "./testdata/basic.mapml".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_ods() -> &'static Dataset {
        let uri = "./testdata/basic.ods".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_sqlite() -> &'static Dataset {
        let uri = "./testdata/basic.sqlite".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_csv_point() -> &'static Dataset {
        let uri = "./testdata/csv/point.csv".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_csv_line() -> &'static Dataset {
        let uri = "./testdata/csv/line.csv".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_csv_polygon() -> &'static Dataset {
        let uri = "./testdata/csv/polygon.csv".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_csv_multipolygon() -> &'static Dataset {
        let uri = "./testdata/csv/multipolygon.csv".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_csv_nogeom() -> &'static Dataset {
        let uri = "./testdata/csv/nogeom.csv".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_geojson_point() -> &'static Dataset {
        let uri = "./testdata/geojson/point.json".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_geojson_line() -> &'static Dataset {
        let uri = "./testdata/geojson/line.json".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_geojson_polygon() -> &'static Dataset {
        let uri = "./testdata/geojson/polygon.json".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_geojson_multipolygon() -> &'static Dataset {
        let uri = "./testdata/geojson/multipolygon.json".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_geojson_nogeom() -> &'static Dataset {
        let uri = "./testdata/geojson/nogeom.json".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_geojsonseq_point() -> &'static Dataset {
        let uri = "./testdata/geojsonseq/point.json".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_geojsonseq_line() -> &'static Dataset {
        let uri = "./testdata/geojsonseq/line.json".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_geojsonseq_polygon() -> &'static Dataset {
        let uri = "./testdata/geojsonseq/polygon.json".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_geojsonseq_multipolygon() -> &'static Dataset {
        let uri = "./testdata/geojsonseq/multipolygon.json".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_geojsonseq_nogeom() -> &'static Dataset {
        let uri = "./testdata/geojsonseq/nogeom.json".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_gml() -> &'static Dataset {
        let uri = "./testdata/gml/basic.gml".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_jml_point() -> &'static Dataset {
        let uri = "./testdata/jml/point.jml".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_jml_line() -> &'static Dataset {
        let uri = "./testdata/jml/line.jml".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_jml_polygon() -> &'static Dataset {
        let uri = "./testdata/jml/polygon.jml".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_jml_multipolygon() -> &'static Dataset {
        let uri = "./testdata/jml/multipolygon.jml".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_jml_nogeom() -> &'static Dataset {
        let uri = "./testdata/jml/nogeom.jml".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_mapinfofile_point() -> &'static Dataset {
        let uri = "./testdata/mapinfofile/point.tab".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_mapinfofile_line() -> &'static Dataset {
        let uri = "./testdata/mapinfofile/line.tab".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_mapinfofile_polygon() -> &'static Dataset {
        let uri = "./testdata/mapinfofile/polygon.tab".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_mapinfofile_multipolygon() -> &'static Dataset {
        let uri = "./testdata/mapinfofile/multipolygon.tab".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_mapinfofile_nogeom() -> &'static Dataset {
        let uri = "./testdata/mapinfofile/nogeom.tab".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_xlsx_point() -> &'static Dataset {
        let uri = "./testdata/xlsx/point.xlsx".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_xlsx_line() -> &'static Dataset {
        let uri = "./testdata/xlsx/line.xlsx".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_xlsx_polygon() -> &'static Dataset {
        let uri = "./testdata/xlsx/polygon.xlsx".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_xlsx_multipolygon() -> &'static Dataset {
        let uri = "./testdata/xlsx/multipolygon.xlsx".to_string();

        open_dataset(uri, false).unwrap()
    }

    #[fixture]
    pub fn basic_xlsx_nogeom() -> &'static Dataset {
        let uri = "./testdata/xlsx/nogeom.xlsx".to_string();

        open_dataset(uri, false).unwrap()
    }
}

use crate::fixtures::datasets::basic_gpkg;

#[fixture]
pub fn table_rects() -> TableRects {
    let rect = Rect {
        x: 0,
        y: 0,
        width: 250,
        height: 20,
    };

    TatApp::table_rects(rect, false)
}

#[fixture]
pub fn basic_table(basic_gpkg: &'static Dataset, table_rects: TableRects) -> TatTable {
    let mut t = TatTable::new(basic_gpkg, None, None);
    t.set_rects(table_rects);

    t
}

#[fixture]
pub fn basic_tat(basic_gpkg: &'static Dataset) -> TatApp {
    let t = TatApp::new(basic_gpkg, None, None);

    t
}
