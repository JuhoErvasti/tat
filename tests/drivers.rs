use std::fmt::Display;

// use tat::fixtures::datasets::*;
// use tat::{app::TatApp, layerschema::TatLayerSchema};
// use gdal::Dataset;
// use rstest::rstest;
// use insta::assert_snapshot;
//
// pub type LayerRes = Vec<Vec<Option<String>>>;
//
// pub struct LayerResults {
//     res: LayerRes,
// }
//
// impl Display for LayerResults {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         for row in self.res.iter() {
//             for _field in 0..row.len() {
//                 let field = row.get(_field).unwrap();
//                 write!(f, "{}", if field.is_some() { field.clone().unwrap() } else { "NULL".to_string() })?;
//
//                 if _field < row.len() - 1 {
//                     write!(f, ",")?;
//                 }
//             }
//             writeln!(f)?;
//         }
//
//         Ok(())
//     }
// }
//
// fn build_test_results_from_layer(layer: &TatLayerSchema) -> LayerResults {
//     let mut headers: Vec<Option<String>> = vec![];
//     for field in 0..layer.field_count() {
//         headers.push(layer.field_name_by_id(field as i32));
//     }
//
//     let mut features: Vec<Vec<Option<String>>> = vec![headers];
//
//
//     for row in 0..layer.feature_count() {
//         let mut feature: Vec<Option<String>> = vec![];
//         for field in 0..layer.field_count() {
//             feature.push(layer.get_value_by_row(row as u64, field as i32));
//         }
//         features.push(feature);
//     }
//
//     LayerResults { res: features }
// }
//
// fn test_single_layer_dataset(ds: &'static Dataset) {
//     let t = TatApp::new(ds, None, None);
//     let result = build_test_results_from_layer(&t.table().layer_schema());
//
//     let ds_name = ds.driver().short_name().to_lowercase().replace(" ", "_");
//     let snapshot_name = format!("{}_{}", ds_name, t.table().layer_schema().name());
//     assert_snapshot!(snapshot_name, result);
// }
//
// fn test_all_layers(ds: &'static Dataset) {
//     let mut t = TatApp::new(ds, None, None);
//     for index in 0..t.table().layer_schemas().len() as usize {
//         t.set_layer_index(index);
//         let result = build_test_results_from_layer(&t.table().layer_schema());
//         let ds_name = ds.driver().short_name().to_lowercase().replace(" ", "_");
//         let snapshot_name = format!("{}_{}", ds_name, t.table().layer_schema().name());
//         assert_snapshot!(snapshot_name, result);
//     }
// }
//
// #[rstest]
// fn test_basic_gdb(basic_gdb: &'static Dataset) {
//     test_all_layers(basic_gdb);
// }
//
// #[rstest]
// fn test_basic_gml(basic_gml: &'static Dataset) {
//     test_all_layers(basic_gml);
// }
//
// #[rstest]
// fn test_basic_jsonfg(basic_jsonfg: &'static Dataset) {
//     test_all_layers(basic_jsonfg);
// }
//
// #[rstest]
// fn test_basic_mapml(basic_mapml: &'static Dataset) {
//     test_all_layers(basic_mapml);
// }
//
// #[rstest]
// fn test_basic_ods(basic_ods: &'static Dataset) {
//     test_all_layers(basic_ods);
// }
//
// // the reason these are individual functions like this is because it works
// // better with the insta snapshots
// #[rstest]
// fn test_basic_shp_point(basic_shp_point: &'static Dataset) {
//     test_single_layer_dataset(basic_shp_point);
// }
//
// #[rstest]
// fn test_basic_shp_line(basic_shp_line: &'static Dataset) {
//     test_single_layer_dataset(basic_shp_line);
// }
//
// #[rstest]
// fn test_basic_shp_polygon(basic_shp_polygon: &'static Dataset) {
//     test_single_layer_dataset(basic_shp_polygon);
// }
//
// #[rstest]
// fn test_basic_shp_multipolygon(basic_shp_multipolygon: &'static Dataset) {
//     test_single_layer_dataset(basic_shp_multipolygon);
// }
//
// #[rstest]
// fn test_basic_shp_nogeom(basic_shp_nogeom: &'static Dataset) {
//     test_single_layer_dataset(basic_shp_nogeom);
// }
//
// #[rstest]
// fn test_basic_csv_point(basic_csv_point: &'static Dataset) {
//     test_single_layer_dataset(basic_csv_point);
// }
//
// #[rstest]
// fn test_basic_csv_line(basic_csv_line: &'static Dataset) {
//     test_single_layer_dataset(basic_csv_line);
// }
//
// #[rstest]
// fn test_basic_csv_polygon(basic_csv_polygon: &'static Dataset) {
//     test_single_layer_dataset(basic_csv_polygon);
// }
//
// #[rstest]
// fn test_basic_csv_multipolygon(basic_csv_multipolygon: &'static Dataset) {
//     test_single_layer_dataset(basic_csv_multipolygon);
// }
//
// #[rstest]
// fn test_basic_csv_nogeom(basic_csv_nogeom: &'static Dataset) {
//     test_single_layer_dataset(basic_csv_nogeom);
// }
//
// #[rstest]
// fn test_basic_geojson_point(basic_geojson_point: &'static Dataset) {
//     test_single_layer_dataset(basic_geojson_point);
// }
//
// #[rstest]
// fn test_basic_geojson_line(basic_geojson_line: &'static Dataset) {
//     test_single_layer_dataset(basic_geojson_line);
// }
//
// #[rstest]
// fn test_basic_geojson_polygon(basic_geojson_polygon: &'static Dataset) {
//     test_single_layer_dataset(basic_geojson_polygon);
// }
//
// #[rstest]
// fn test_basic_geojson_multipolygon(basic_geojson_multipolygon: &'static Dataset) {
//     test_single_layer_dataset(basic_geojson_multipolygon);
// }
//
// #[rstest]
// fn test_basic_geojson_nogeom(basic_geojson_nogeom: &'static Dataset) {
//     test_single_layer_dataset(basic_geojson_nogeom);
// }
//
// #[rstest]
// fn test_basic_geojsonseq_point(basic_geojsonseq_point: &'static Dataset) {
//     test_single_layer_dataset(basic_geojsonseq_point);
// }
//
// #[rstest]
// fn test_basic_geojsonseq_line(basic_geojsonseq_line: &'static Dataset) {
//     test_single_layer_dataset(basic_geojsonseq_line);
// }
//
// #[rstest]
// fn test_basic_geojsonseq_polygon(basic_geojsonseq_polygon: &'static Dataset) {
//     test_single_layer_dataset(basic_geojsonseq_polygon);
// }
//
// #[rstest]
// fn test_basic_geojsonseq_multipolygon(basic_geojsonseq_multipolygon: &'static Dataset) {
//     test_single_layer_dataset(basic_geojsonseq_multipolygon);
// }
//
// #[rstest]
// fn test_basic_geojsonseq_nogeom(basic_geojsonseq_nogeom: &'static Dataset) {
//     test_single_layer_dataset(basic_geojsonseq_nogeom);
// }
//
// #[rstest]
// fn test_basic_jml_point(basic_jml_point: &'static Dataset) {
//     test_single_layer_dataset(basic_jml_point);
// }
//
// #[rstest]
// fn test_basic_jml_line(basic_jml_line: &'static Dataset) {
//     test_single_layer_dataset(basic_jml_line);
// }
//
// #[rstest]
// fn test_basic_jml_polygon(basic_jml_polygon: &'static Dataset) {
//     test_single_layer_dataset(basic_jml_polygon);
// }
//
// #[rstest]
// fn test_basic_jml_multipolygon(basic_jml_multipolygon: &'static Dataset) {
//     test_single_layer_dataset(basic_jml_multipolygon);
// }
//
// #[rstest]
// fn test_basic_jml_nogeom(basic_jml_nogeom: &'static Dataset) {
//     test_single_layer_dataset(basic_jml_nogeom);
// }
//
// #[rstest]
// fn test_basic_mapinfofile_point(basic_mapinfofile_point: &'static Dataset) {
//     test_single_layer_dataset(basic_mapinfofile_point);
// }
//
// #[rstest]
// fn test_basic_mapinfofile_line(basic_mapinfofile_line: &'static Dataset) {
//     test_single_layer_dataset(basic_mapinfofile_line);
// }
//
// #[rstest]
// fn test_basic_mapinfofile_polygon(basic_mapinfofile_polygon: &'static Dataset) {
//     test_single_layer_dataset(basic_mapinfofile_polygon);
// }
//
// #[rstest]
// fn test_basic_mapinfofile_multipolygon(basic_mapinfofile_multipolygon: &'static Dataset) {
//     test_single_layer_dataset(basic_mapinfofile_multipolygon);
// }
//
// #[rstest]
// fn test_basic_mapinfofile_nogeom(basic_mapinfofile_nogeom: &'static Dataset) {
//     test_single_layer_dataset(basic_mapinfofile_nogeom);
// }
//
// #[rstest]
// fn test_basic_xlsx_point(basic_xlsx_point: &'static Dataset) {
//     test_single_layer_dataset(basic_xlsx_point);
// }
//
// #[rstest]
// fn test_basic_xlsx_line(basic_xlsx_line: &'static Dataset) {
//     test_single_layer_dataset(basic_xlsx_line);
// }
//
// #[rstest]
// fn test_basic_xlsx_polygon(basic_xlsx_polygon: &'static Dataset) {
//     test_single_layer_dataset(basic_xlsx_polygon);
// }
//
// #[rstest]
// fn test_basic_xlsx_multipolygon(basic_xlsx_multipolygon: &'static Dataset) {
//     test_single_layer_dataset(basic_xlsx_multipolygon);
// }
//
// #[rstest]
// fn test_basic_xlsx_nogeom(basic_xlsx_nogeom: &'static Dataset) {
//     test_single_layer_dataset(basic_xlsx_nogeom);
// }
