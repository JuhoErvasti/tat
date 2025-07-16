# Terminal Attribute Table (tat)

For inspecting geospatial data in the terminal.

![](img/demo.gif)

## Motivation

Working with geospatial data and using a terminal environment a lot there are some situations
where I find that being able to quickly and _interactively_ inspect attribute data would be useful.
Of course you can use something like `ogrinfo` (or `gdal vector info` nowadays, or even
`ogrinfo <uri> | less` for some interactivity) these are not quite the type of TUI I found myself
wishing for.

I also wanted to try out Rust, so creating this type of terminal attribute table felt like a good
way to learn while also producing a tool that I could see myself actually using and not just leaving
it as an abandoned learning project.

## Install

> [!NOTE]
> The program is in an early state and has not been thoroughly tested.

Currently has been confirmed to work on Linux. Windows (non-WSL) is not (currently) supported.
I do not have the ability to test on macOS.

Currently the only option is to use Cargo to install directly from GitHub.
Cargo needs to be [installed first (alongside Rust)](https://doc.rust-lang.org/cargo/getting-started/installation.html).

```shell
cargo install --git https://github.com/JuhoErvasti/tat
```

Same command can be used to update.

## Usage

```shell
# files
tat example.gpkg
tat example.shp

# ogr dataset
tat PG:service=SERVICE
```

GDAL is used under the hood, so any GDAL-supported vector driver should theoretically work if
you use a correct URI.

> [!NOTE]
> The program is in an early state and has not been tested with all vector drivers thoroughly.

## Tested drivers

Testing status of different GDAL vector drivers is presented in the table.

Status explanation:
* none: no tests exist for data format
* basic: test(s) exist for data format and it can be opened and its features displayed
* extensive: data format and any of its peculiarities are tested for extensively

> [!NOTE]
> These tests indicate only that data can be displayed and browsed correctly and don't consider
> performance. Some drivers might work slowly.

|Driver                                           |Status           |Notes             |
|-------------------------------------------------|-----------------|------------------|
|CSV (Comma Separated Values)                     |planned          |                  |
|DGN (Microstation DGN)                           |planned          |                  |
|DXF (AutoCAD DXF)                                |planned          |                  |
|ESRI FileGDB (OpenFileGDB)                       |planned          |                  |
|FlatGeobuf                                       |planned          |                  |
|GeoJSON                                          |planned          |                  |
|GeoJSONSeq (GeoJSON Sequence)                    |planned          |                  |
|GML (Geography Markup Language)                  |planned          |                  |
|GPKG (GeoPackage)                                |basic            |                  |
|GPX (GPS Exchange Format)                        |planned          |                  |
|JML (OpenJUMP JML)                               |planned          |                  |
|JSON FG (OGC Features and Geometries JSON)       |planned          |                  |
|KML (Keyhole Markup Language)                    |planned          |                  |
|MapML (Map Markup Language)                      |planned          |                  |
|MBTiles                                          |planned          |                  |
|MVT (Mapbox Vector Tiles)                        |planned          |                  |
|netCDF (Network Common Data Form)                |planned          |                  |
|ODS (OpenDocument Spreadsheet)                   |planned          |                  |
|OGR_GMT (GMT ASCII Vectors)                      |planned          |                  |
|OGR_VRT (Virtual Datasource)                     |planned          |                  |
|PCIDSK (PCI Geomatics Database File)             |planned          |                  |
|PDF (Geospatial PDF)                             |planned          |                  |
|PMTiles (ProtoMap Tiles)                         |planned          |                  |
|SHP (ESRI Shapefile)                             |planned          |                  |
|SQLite                                           |planned          |                  |
|TAB (MapInfo File)                               |planned          |                  |
|VDV (VDV 451/VDV 452/INTREST Data Format)        |planned          |                  |
|XLSX (MS Office Open XML spreadsheet)            |planned          |                  |

Unplanned ones for first release:

|Driver                                           |Status              |
|-------------------------------------------------|--------------------|
|AIVector                                         |unplanned for v0.1.0|
|AmigoCloud                                       |unplanned for v0.1.0|
|Arrow                                            |unplanned for v0.1.0|
|AVCBIN (Arc/Info Binary Coverage)                |unplanned for v0.1.0|
|AVCE00 (Arc/Info E00 (ASCII) Coverage)           |unplanned for v0.1.0|
|Carto                                            |unplanned for v0.1.0|
|CSW (Catalog Service for the Web)                |unplanned for v0.1.0|
|DGNv8 (Microstation DGN v8)                      |unplanned for v0.1.0|
|EDIGEO                                           |unplanned for v0.1.0|
|EEDA (Google Earth Engine Data API)              |unplanned for v0.1.0|
|Elasticsearch                                    |unplanned for v0.1.0|
|ESRIJSON                                         |unplanned for v0.1.0|
|GeoRSS                                           |unplanned for v0.1.0|
|GMLAS                                            |unplanned for v0.1.0|
|GMT                                              |unplanned for v0.1.0|
|GPSBabel                                         |unplanned for v0.1.0|
|GTFS                                             |unplanned for v0.1.0|
|HANA (SAP HANA)                                  |unplanned for v0.1.0|
|IDB                                              |unplanned for v0.1.0|
|IDRISI                                           |unplanned for v0.1.0|
|INTERLIS 1                                       |unplanned for v0.1.0|
|LIBKML                                           |unplanned for v0.1.0|
|LVBAG (Dutch Kadaster LV BAG 2.0 Extract)        |unplanned for v0.1.0|
|MEM (Memory)                                     |unplanned for v0.1.0|
|MiraMonVector                                    |unplanned for v0.1.0|
|MongoDBv3                                        |unplanned for v0.1.0|
|MSSQLSpatial                                     |unplanned for v0.1.0|
|MySQL                                            |unplanned for v0.1.0|
|NAS (ALKIS)                                      |unplanned for v0.1.0|
|OAPIF (OGC API - Features)                       |unplanned for v0.1.0|
|OCI (Oracle Spatial)                             |unplanned for v0.1.0|
|ODBC (ODBC RDBMS)                                |unplanned for v0.1.0|
|OSM (OpenStreetMap XML and PBF)                  |unplanned for v0.1.0|
|Parquet ((Geo)Parquet)                           |unplanned for v0.1.0|
|PGeo (ESRI Personal GeoDatabase)                 |unplanned for v0.1.0|
|PLScenes (Planet Labs Scenes/Catalog API)        |unplanned for v0.1.0|
|PMTiles                                          |unplanned for v0.1.0|
|PostgreSQL (PostgreSQL / PostGIS)                |unplanned for v0.1.0|
|S57 (IHO S-57 (ENC))                             |unplanned for v0.1.0|
|Selafin (Selafin)                                |unplanned for v0.1.0|
|SOSI (Norwegian SOSI Standard)                   |unplanned for v0.1.0|
|SXF                                              |unplanned for v0.1.0|
|TopoJSON                                         |unplanned for v0.1.0|
|WAsP (WAsP .map format)                          |unplanned for v0.1.0|
|XLS (MS Excel format)                            |unplanned for v0.1.0|
|XODR (OpenDRIVE Road Description Format)         |unplanned for v0.1.0|

## Roadmap

This is purely a hobby project so development is intermittent and absolutely not guaranteed.
Since this is and likely will remain a single-author repository I'm also using this section
basically as project management because creating and managing issues feels clunky and unnecessary
without any collaborators.

For the first proper version the following are still under development:

- Tests and CI

Further features I've considered for future versions:

<details>
<summary>More details</summary>

  Most important:
  - Optimize performance
    - Some drivers such as CSV and WFS are slow even with a fairly small number of features

  Maybe:
  - Allow setting a limit on the number of features shown
  - More mouse support, such as:
    - Opening layers
    - Selecting cells
    - Copying cell values (Right/Middle click or something?)
  - Preserve table state for each layer instead of resetting it every time when closing layer
  - Some support for looking at raster metadata similar to `gdalinfo` (not displaying raster itself)
  - Allow viewing/copying geometry as WKB in addition to WKT
  - Ability to select a whole feature in the attribute table
    - (Maybe) allow selecting multiple features?
    - (Maybe) copy it/them as GeoJSON/GML(?)
  - Allow exporting dataset as a GeoPackage
    - (Maybe) as any ogr-supported driver?
    - (Maybe) allow selecting which layers are exported?
    - (Maybe) if selecting features are implemented, export only those features?
  - Allow setting a spatial filter on a dataset

  Unlikely:
  - Raster attribute tables
  - Some way of displaying geometries as other whan WKT
    - Probably best bet would be to render the geometry as a temporary image and display it using [viuer](https://github.com/atanunq/viuer)

  Extremely unlikely:
  - Editing of any kind, the main impetus for developing this tool is to just inspect data

</details>
