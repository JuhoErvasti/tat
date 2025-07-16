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

## Installation

Currently has been confirmed to work on Linux.

### Dependencies

GDAL has to be [installed](https://gdal.org/en/stable/download.html).

### Cargo

Currently the only option is to use Cargo to install directly from GitHub.
Cargo needs to be [installed first (alongside Rust)](https://doc.rust-lang.org/cargo/getting-started/installation.html).

```shell
cargo install --git https://github.com/JuhoErvasti/tat
```

> [!NOTE]
> Same command can be used to update.

## Usage

```
Terminal UI for inspecting geospatial data

Usage: tat [OPTIONS] <URI>

Arguments:
  <URI>


Options:
      --where <WHERE>
          Filter which features are shown based on their attributes. Given in the format of a SQL WHERE clause e.g. --where="field_1 = 12"

      --layers <LAYERS>
          Specify which layers in the dataset should be opened. Given as a comma-separated list e.g. "--layers=layer_1,layer_2"

      --allow-untested-drivers
          Allow attempting to open dataset of any type which has a GDAL-supported vector driver. Use with caution.

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Examples

```shell
# open file
tat example.gpkg
```

```shell
# open only some layers
tat example.gpkg --layers=layer_1,layer_2
```

```shell
# open specific layer
tat example.gpkg --layers=layer_1
```

```shell
# open with an attribute query
tat example.gpkg --where="field = 'value'"
```

## Supported data formats

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
|CSV (Comma Separated Values)                     |basic            |                  |
|ESRI FileGDB (OpenFileGDB)                       |basic            |                  |
|GeoJSON                                          |basic            |                  |
|GeoJSONSeq (GeoJSON Sequence)                    |basic            |                  |
|GML (Geography Markup Language)                  |basic            |                  |
|GPKG (GeoPackage)                                |basic            |                  |
|JML (OpenJUMP JML)                               |basic            |                  |
|JSON FG (OGC Features and Geometries JSON)       |basic            |                  |
|MapML (Map Markup Language)                      |basic            |                  |
|ODS (OpenDocument Spreadsheet)                   |basic            |                  |
|SHP (ESRI Shapefile)                             |basic            |                  |
|TAB (MapInfo File)                               |basic            |                  |
|XLSX (MS Office Open XML spreadsheet)            |basic            |                  |

Currently untested drivers:

<details>
<summary>Details</summary>

|Driver                                           |Status                   |
|-------------------------------------------------|-------------------------|
|AIVector                                         |unplanned for v0.1.0     |
|AmigoCloud                                       |unplanned for v0.1.0     |
|Arrow                                            |unplanned for v0.1.0     |
|AVCBIN (Arc/Info Binary Coverage)                |unplanned for v0.1.0     |
|AVCE00 (Arc/Info E00 (ASCII) Coverage)           |unplanned for v0.1.0     |
|Carto                                            |unplanned for v0.1.0     |
|CSW (Catalog Service for the Web)                |unplanned for v0.1.0     |
|DGN (Microstation DGN)                           |unplanned for v0.1.0     |
|DGNv8 (Microstation DGN v8)                      |unplanned for v0.1.0     |
|DXF (AutoCAD DXF)                                |unplanned for v0.1.0     |
|EDIGEO                                           |unplanned for v0.1.0     |
|EEDA (Google Earth Engine Data API)              |unplanned for v0.1.0     |
|Elasticsearch                                    |unplanned for v0.1.0     |
|ESRIJSON                                         |unplanned for v0.1.0     |
|FlatGeobuf                                       |unplanned for v0.1.0     |
|GeoRSS                                           |unplanned for v0.1.0     |
|GMLAS                                            |unplanned for v0.1.0     |
|GMT                                              |unplanned for v0.1.0     |
|GPSBabel                                         |unplanned for v0.1.0     |
|GPX (GPS Exchange Format)                        |unplanned for v0.1.0     |
|GTFS                                             |unplanned for v0.1.0     |
|HANA (SAP HANA)                                  |unplanned for v0.1.0     |
|IDB                                              |unplanned for v0.1.0     |
|IDRISI                                           |unplanned for v0.1.0     |
|INTERLIS 1                                       |unplanned for v0.1.0     |
|KML (Keyhole Markup Language)                    |unplanned for v0.1.0     |
|LIBKML                                           |unplanned for v0.1.0     |
|LVBAG (Dutch Kadaster LV BAG 2.0 Extract)        |unplanned for v0.1.0     |
|MBTiles                                          |unplanned for v0.1.0     |
|MEM (Memory)                                     |unplanned for v0.1.0     |
|MiraMonVector                                    |unplanned for v0.1.0     |
|MongoDBv3                                        |unplanned for v0.1.0     |
|MSSQLSpatial                                     |unplanned for v0.1.0     |
|MVT (Mapbox Vector Tiles)                        |unplanned for v0.1.0     |
|MySQL                                            |unplanned for v0.1.0     |
|NAS (ALKIS)                                      |unplanned for v0.1.0     |
|netCDF (Network Common Data Form)                |unplanned for v0.1.0     |
|OAPIF (OGC API - Features)                       |unplanned for v0.1.0     |
|OCI (Oracle Spatial)                             |unplanned for v0.1.0     |
|ODBC (ODBC RDBMS)                                |unplanned for v0.1.0     |
|OGR_GMT (GMT ASCII Vectors)                      |unplanned for v0.1.0     |
|OSM (OpenStreetMap XML and PBF)                  |unplanned for v0.1.0     |
|Parquet ((Geo)Parquet)                           |unplanned for v0.1.0     |
|PCIDSK (PCI Geomatics Database File)             |unplanned for v0.1.0     |
|PDF (Geospatial PDF)                             |unplanned for v0.1.0     |
|PGeo (ESRI Personal GeoDatabase)                 |unplanned for v0.1.0     |
|PLScenes (Planet Labs Scenes/Catalog API)        |unplanned for v0.1.0     |
|PMTiles (ProtoMap Tiles)                         |unplanned for v0.1.0     |
|PMTiles                                          |unplanned for v0.1.0     |
|PostgreSQL (PostgreSQL / PostGIS)                |unplanned for v0.1.0     |
|S57 (IHO S-57 (ENC))                             |unplanned for v0.1.0     |
|Selafin (Selafin)                                |unplanned for v0.1.0     |
|SOSI (Norwegian SOSI Standard)                   |unplanned for v0.1.0     |
|SXF                                              |unplanned for v0.1.0     |
|TopoJSON                                         |unplanned for v0.1.0     |
|VDV (VDV 451/VDV 452/INTREST Data Format)        |unplanned for v0.1.0     |
|WAsP (WAsP .map format)                          |unplanned for v0.1.0     |
|XLS (MS Excel format)                            |unplanned for v0.1.0     |
|XODR (OpenDRIVE Road Description Format)         |unplanned for v0.1.0     |

</details>

## Roadmap

This is purely a hobby project so development is intermittent and absolutely not guaranteed.
Since this is and likely will remain a single-author repository I'm also using this section
basically as project management because creating and managing issues feels clunky and unnecessary
without any collaborators.

For the first proper version the following are still under development:

- CI

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
