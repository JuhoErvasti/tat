use std::{sync::mpsc::{Receiver, Sender}, thread::{self, JoinHandle}};

use cli_log::info;
use ratatui::layout::Rect;
use rstest::{fixture, rstest};
use gdal::Dataset;
use std::sync::mpsc;

use crate::{app::{TatApp, TatEvent}, dataset::{DatasetRequest, DatasetResponse, TatAttributeViewRequest, TatDataset}, layerlist::TatLayerInfo, layerschema::TatLayerSchema, navparagraph::TatNavigableParagraph, table::{TableRects, TatTable}, types::{TatCrs, TatField, TatGeomField}};

const N_TAT_TABLE_INIT_EVENTS: u8 = 2;
const N_TAT_APP_INIT_EVENTS: u8 = 6;

pub struct TatTestUtils {}

impl TatTestUtils {
    pub fn request_attribute_view_update_mocked(layer_index: usize, total_geom_fields: usize, tx: Sender<DatasetRequest>) {
        let r = TatAttributeViewRequest {
            layer_index: layer_index,
            top_row: 1,
            bottom_row: 10,
            first_column: 0,
            last_column: 4,
            total_geom_fields: total_geom_fields,
        };
        tx.send(DatasetRequest::UpdateAttributeView(r)).unwrap();
    }

    pub fn set_layer_index_and_update(index: usize, table: &mut TatTable, rx: &Receiver<TatEvent>) {
        table.set_layer_index(index);

        TatTestUtils::wait_attribute_view_update(rx);
    }

    pub fn wait_attribute_view_update(rx: &Receiver<TatEvent>) {
        match rx.recv().unwrap() {
            TatEvent::Dataset(ds_r) => {
                match ds_r {
                    DatasetResponse::AttributeViewUpdated => {
                    },
                        _ => panic!(),
                }
            },
            _ => panic!(),
        };
    }

    pub fn refresh_table_attribute_view(table: &TatTable, rx: &Receiver<TatEvent>) {
        table.on_visible_attributes_changed();
        TatTestUtils::wait_attribute_view_update(rx);
    }

    pub fn prepare_table_attribute_view_all_features(table: &TatTable, rx: &Receiver<TatEvent>, tx: Sender<DatasetRequest>) {
        let ls = table.layer_schema().unwrap();
        let feature_count = ls.feature_count();
        let field_count = ls.field_count();

        let request = TatAttributeViewRequest {
            layer_index: ls.index(),
            top_row: 1,
            bottom_row: feature_count,
            first_column: 0,
            last_column: field_count,
            total_geom_fields: ls.geom_fields().len(),
        };

        tx.send(DatasetRequest::UpdateAttributeView(request)).unwrap();

        TatTestUtils::wait_attribute_view_update(rx);
    }
}


pub type DatasetChannels = (Sender<TatEvent>, Receiver<DatasetRequest>);

pub struct TatTestStructure {
    pub ds_request_tx: Sender<DatasetRequest>,
    pub ds_join_handle: JoinHandle<()>,
    pub tatevent_rx: Receiver<TatEvent>,
}

impl TatTestStructure {
    pub fn new(ds_request_tx: Sender<DatasetRequest>, ds_join_handle: JoinHandle<()>, tatevent_rx: Receiver<TatEvent>) -> Self {
        Self { ds_request_tx, ds_join_handle, tatevent_rx }
    }

    pub fn terminate(self) {
        self.ds_request_tx.send(DatasetRequest::Terminate).unwrap();
        self.ds_join_handle.join().unwrap();
    }
}

pub fn init_test_dataset(uri: String) -> TatTestStructure {
    let (dataset_request_tx, dataset_request_rx) = mpsc::channel::<DatasetRequest>();
    let (tatevent_tx, tatevent_rx) = mpsc::channel::<TatEvent>();

    let ds_handle = thread::spawn(move || {
        if let Some(mut ds) = TatDataset::new(
            tatevent_tx,
            dataset_request_rx,
            uri,
            false,
            None,
            None,
        ) {
            ds.handle_requests();
        } else {
            return;
        }
    });


    TatTestStructure::new(dataset_request_tx, ds_handle, tatevent_rx)
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
pub fn app_rect() -> Rect {
    Rect {
        x: 0,
        y: 0,
        width: 250,
        height: 20,
    }
}


#[fixture]
pub fn basic_table(basic_gpkg: TatTestStructure, table_rects: TableRects) -> (TatTestStructure, TatTable) {
    let (tts, mut table) = init_table(basic_gpkg);
    table.set_rects(table_rects);

    (tts, table)
}

pub fn init_table(tts: TatTestStructure) -> (TatTestStructure, TatTable) {
    tts.ds_request_tx.send(DatasetRequest::BuildLayers).unwrap();
    match tts.tatevent_rx.recv().unwrap() {
        TatEvent::Dataset(dataset_response) => {
            match dataset_response {
                DatasetResponse::LayersBuilt => {
                },
                _ => panic!(),
            }
        },
        _ => panic!(),
    }

    let mut t = TatTable::new(tts.ds_request_tx.clone());
    for _ in 0..N_TAT_TABLE_INIT_EVENTS {
        match tts.tatevent_rx.recv().unwrap() {
            TatEvent::Dataset(ds_r) => {
                match ds_r {
                    DatasetResponse::LayerSchemas(tat_layer_schemas) => {
                        t.set_layer_schemas(tat_layer_schemas);
                    },
                    DatasetResponse::AttributeView(view) => {
                        t.set_attribute_view(view);
                    },
                    _ => panic!(),
                }
            },
            _ => panic!(),
        };
    }


    (tts, t)
}

pub fn init_app(tts: TatTestStructure) -> (TatTestStructure, TatApp) {
    let mut t = TatApp::new(tts.ds_request_tx.clone());

    for _ in 0..N_TAT_APP_INIT_EVENTS {
        match tts.tatevent_rx.recv().unwrap() {
            TatEvent::Dataset(response) => {
                t.handle_dataset(response);
            }
            _ => panic!(),
        }
    }

    (tts, t)
}

#[fixture]
pub fn basic_app(basic_gpkg: TatTestStructure) -> (TatTestStructure, TatApp) {
    init_app(basic_gpkg)
}

#[fixture]
pub fn layer_infos() -> Vec<TatLayerInfo> {
    vec![
        (
            "Layer1".to_string(),
            TatNavigableParagraph::new(
                "Layer 1 info".to_string(),
            ),
        ),
        (
            "Layer2".to_string(),
            TatNavigableParagraph::new(
                "Layer 2 info".to_string(),
            ),
        ),
        (
            "Layer3".to_string(),
            TatNavigableParagraph::new(
                "Layer 3 info".to_string(),
            ),
        ),
        (
            "Layer4".to_string(),
            TatNavigableParagraph::new(
                "Layer 4 info".to_string(),
            ),
        ),
        (
            "Layer5".to_string(),
            TatNavigableParagraph::new(
                "Layer 5 info".to_string(),
            ),
        ),
    ]
}

#[fixture]
pub fn crs_3857() -> TatCrs {
    TatCrs::new(
        "EPSG".to_string(),
        3857,
        "EPSG:3857".to_string(),
    )
}

#[fixture]
pub fn crs_4326() -> TatCrs {
    TatCrs::new(
        "EPSG".to_string(),
        4326,
        "EPSG:4326".to_string(),
    )
}

#[fixture]
pub fn geom_fields(crs_3857: TatCrs, crs_4326: TatCrs) -> Vec<TatGeomField> {
    vec![
        TatGeomField::new("geom1".to_string(), "Polygon".to_string(), Some(crs_3857)),
        TatGeomField::new("geom2".to_string(), "Point".to_string(), Some(crs_4326)),
    ]
}

#[fixture]
pub fn geom_field(crs_3857: TatCrs) -> Vec<TatGeomField> {
    vec![
        TatGeomField::new("geom".to_string(), "Polygon".to_string(), Some(crs_3857)),
    ]
}

#[fixture]
pub fn no_geom_field() -> Vec<TatGeomField> {
    vec![]
}

#[fixture]
pub fn attribute_fields() -> Vec<TatField> {
    vec![
        TatField::new("Field1".to_string(), 0),
        TatField::new("Field2".to_string(), 1),
        TatField::new("Field3".to_string(), 2),
    ]
}

#[fixture]
pub fn layer_schema(
    crs_4326: TatCrs,
    geom_fields: Vec<TatGeomField>,
    attribute_fields: Vec<TatField>,
) -> TatLayerSchema {
    TatLayerSchema::new(
        "Layer1".to_string(),
        Some(crs_4326),
        geom_fields,
        attribute_fields,
        0,
        100,
    )
}

#[fixture]
pub fn layer_schema_one_geom(
    crs_4326: TatCrs,
    geom_field: Vec<TatGeomField>,
    attribute_fields: Vec<TatField>,
) -> TatLayerSchema {
    TatLayerSchema::new(
        "Layer1".to_string(),
        Some(crs_4326),
        geom_field,
        attribute_fields,
        0,
        100,
    )
}

#[fixture]
pub fn layer_schema_no_geom(
    crs_4326: TatCrs,
    no_geom_field: Vec<TatGeomField>,
    attribute_fields: Vec<TatField>,
) -> TatLayerSchema {
    TatLayerSchema::new(
        "Layer1".to_string(),
        Some(crs_4326),
        no_geom_field,
        attribute_fields,
        0,
        100,
    )
}

pub mod datasets {
    use rstest::fixture;
    use super::{TatTestStructure, init_test_dataset};

    #[fixture]
    pub fn basic_gpkg() -> TatTestStructure {
        init_test_dataset("./testdata/basic.gpkg".to_string())
    }

    #[fixture]
    pub fn basic_gdb() -> TatTestStructure {
        init_test_dataset("./testdata/basic.gdb".to_string())
    }

    #[fixture]
    pub fn basic_shp_point() -> TatTestStructure {
        init_test_dataset("./testdata/shp/point.shp".to_string())
    }

    #[fixture]
    pub fn basic_shp_line() -> TatTestStructure {
        init_test_dataset("./testdata/shp/line.shp".to_string())
    }

    #[fixture]
    pub fn basic_shp_polygon() -> TatTestStructure {
        init_test_dataset("./testdata/shp/polygon.shp".to_string())
    }

    #[fixture]
    pub fn basic_shp_multipolygon() -> TatTestStructure {
        init_test_dataset("./testdata/shp/multipolygon.shp".to_string())
    }

    #[fixture]
    pub fn basic_shp_nogeom() -> TatTestStructure {
        init_test_dataset("./testdata/shp/nogeom.dbf".to_string())
    }

    #[fixture]
    pub fn basic_jsonfg() -> TatTestStructure {
        init_test_dataset("./testdata/basic.jsonfg".to_string())
    }

    #[fixture]
    pub fn basic_mapml() -> TatTestStructure {
        init_test_dataset("./testdata/basic.mapml".to_string())
    }

    #[fixture]
    pub fn basic_ods() -> TatTestStructure {
        init_test_dataset("./testdata/basic.ods".to_string())
    }

    #[fixture]
    pub fn basic_csv_point() -> TatTestStructure {
        init_test_dataset("./testdata/csv/point.csv".to_string())
    }

    #[fixture]
    pub fn basic_csv_line() -> TatTestStructure {
        init_test_dataset("./testdata/csv/line.csv".to_string())
    }

    #[fixture]
    pub fn basic_csv_polygon() -> TatTestStructure {
        init_test_dataset("./testdata/csv/polygon.csv".to_string())
    }

    #[fixture]
    pub fn basic_csv_multipolygon() -> TatTestStructure {
        init_test_dataset("./testdata/csv/multipolygon.csv".to_string())
    }

    #[fixture]
    pub fn basic_csv_nogeom() -> TatTestStructure {
        init_test_dataset("./testdata/csv/nogeom.csv".to_string())
    }

    #[fixture]
    pub fn basic_geojson_point() -> TatTestStructure {
        init_test_dataset("./testdata/geojson/point.json".to_string())
    }

    #[fixture]
    pub fn basic_geojson_line() -> TatTestStructure {
        init_test_dataset("./testdata/geojson/line.json".to_string())
    }

    #[fixture]
    pub fn basic_geojson_polygon() -> TatTestStructure {
        init_test_dataset("./testdata/geojson/polygon.json".to_string())
    }

    #[fixture]
    pub fn basic_geojson_multipolygon() -> TatTestStructure {
        init_test_dataset("./testdata/geojson/multipolygon.json".to_string())
    }

    #[fixture]
    pub fn basic_geojson_nogeom() -> TatTestStructure {
        init_test_dataset("./testdata/geojson/nogeom.json".to_string())
    }

    #[fixture]
    pub fn basic_geojsonseq_point() -> TatTestStructure {
        init_test_dataset("./testdata/geojsonseq/point.json".to_string())
    }

    #[fixture]
    pub fn basic_geojsonseq_line() -> TatTestStructure {
        init_test_dataset("./testdata/geojsonseq/line.json".to_string())
    }

    #[fixture]
    pub fn basic_geojsonseq_polygon() -> TatTestStructure {
        init_test_dataset("./testdata/geojsonseq/polygon.json".to_string())
    }

    #[fixture]
    pub fn basic_geojsonseq_multipolygon() -> TatTestStructure {
        init_test_dataset("./testdata/geojsonseq/multipolygon.json".to_string())
    }

    #[fixture]
    pub fn basic_geojsonseq_nogeom() -> TatTestStructure {
        init_test_dataset("./testdata/geojsonseq/nogeom.json".to_string())
    }

    #[fixture]
    pub fn basic_gml() -> TatTestStructure {
        init_test_dataset("./testdata/gml/basic.gml".to_string())
    }

    #[fixture]
    pub fn basic_jml_point() -> TatTestStructure {
        init_test_dataset("./testdata/jml/point.jml".to_string())
    }

    #[fixture]
    pub fn basic_jml_line() -> TatTestStructure {
        init_test_dataset("./testdata/jml/line.jml".to_string())
    }

    #[fixture]
    pub fn basic_jml_polygon() -> TatTestStructure {
        init_test_dataset("./testdata/jml/polygon.jml".to_string())
    }

    #[fixture]
    pub fn basic_jml_multipolygon() -> TatTestStructure {
        init_test_dataset("./testdata/jml/multipolygon.jml".to_string())
    }

    #[fixture]
    pub fn basic_jml_nogeom() -> TatTestStructure {
        init_test_dataset("./testdata/jml/nogeom.jml".to_string())
    }

    #[fixture]
    pub fn basic_mapinfofile_point() -> TatTestStructure {
        init_test_dataset("./testdata/mapinfofile/point.tab".to_string())
    }

    #[fixture]
    pub fn basic_mapinfofile_line() -> TatTestStructure {
        init_test_dataset("./testdata/mapinfofile/line.tab".to_string())
    }

    #[fixture]
    pub fn basic_mapinfofile_polygon() -> TatTestStructure {
        init_test_dataset("./testdata/mapinfofile/polygon.tab".to_string())
    }

    #[fixture]
    pub fn basic_mapinfofile_multipolygon() -> TatTestStructure {
        init_test_dataset("./testdata/mapinfofile/multipolygon.tab".to_string())
    }

    #[fixture]
    pub fn basic_mapinfofile_nogeom() -> TatTestStructure {
        init_test_dataset("./testdata/mapinfofile/nogeom.tab".to_string())
    }

    #[fixture]
    pub fn basic_xlsx_point() -> TatTestStructure {
        init_test_dataset("./testdata/xlsx/point.xlsx".to_string())
    }

    #[fixture]
    pub fn basic_xlsx_line() -> TatTestStructure {
        init_test_dataset("./testdata/xlsx/line.xlsx".to_string())
    }

    #[fixture]
    pub fn basic_xlsx_polygon() -> TatTestStructure {
        init_test_dataset("./testdata/xlsx/polygon.xlsx".to_string())
    }

    #[fixture]
    pub fn basic_xlsx_multipolygon() -> TatTestStructure {
        init_test_dataset("./testdata/xlsx/multipolygon.xlsx".to_string())
    }

    #[fixture]
    pub fn basic_xlsx_nogeom() -> TatTestStructure {
        init_test_dataset("./testdata/xlsx/nogeom.xlsx".to_string())
    }
}
