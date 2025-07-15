use gdal::Dataset;
use ratatui::layout::Rect;
use rstest::fixture;

use crate::{table::{TableRects, TatTable}, tat::Tat, utils::open_dataset};

#[fixture]
pub fn basic_gpkg() -> &'static Dataset {
    let uri = "./testdata/basic.gpkg".to_string();

    open_dataset(uri).unwrap()
}

#[fixture]
pub fn table_rects() -> TableRects {
    let rect = Rect {
        x: 0,
        y: 0,
        width: 250,
        height: 20,
    };

    Tat::table_rects(rect, false)
}

#[fixture]
pub fn basic_table(basic_gpkg: &'static Dataset, table_rects: TableRects) -> TatTable {
    let mut t = TatTable::new(basic_gpkg, None, None);
    t.set_rects(table_rects);

    t
}

#[fixture]
pub fn basic_tat(basic_gpkg: &'static Dataset) -> Tat {
    let t = Tat::new(basic_gpkg, None, None);

    t
}
