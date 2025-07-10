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

# ogr URI
tat PG:service=SERVICE
```

GDAL is used under the hood, so any GDAL-supported vector driver should theoretically work if
you use a correct URI.

> [!NOTE]
> The program is in an early state and has not been tested with all vector drivers thoroughly.
> Some drivers (such as WFS) are known to work quite slowly.

## Roadmap

This is purely a hobby project so development is intermittent and absolutely not guaranteed.
Since this is and likely will remain a single-author repository I'm also using this section
basically as project management because creating and managing issues feels clunky and unnecessary
without any collaborators.

Here are the features which I'd like to add for the first proper version (more or less in order of importance):

- Tests and CI
- Display preview table in Main Menu
- Jumping to specific FID
- Display geometry column(s) as WKT
  - (Maybe) allow copying geometry as WKB in addition to WKT?
- Optimize performance
- Some kind of feedback when a value has been copied to the clipboard (or copying failed for that matter)
- Mouse support
  - At least:
    - scrolling table
  - Maybe:
    - scrolling text/layer list contents
  - Nice-to-have:
    - Opening layers
    - Selecting cells
    - Copying cell values (Right/Middle click or something?)

<details>
<summary>More details</summary>
<details>
<summary>Completed</summary>

- Fix issues with some layers not opening in the table
- Improve performance on large layers (only render what can be seen)
  - Improve performance on opening large layers
- Fit columns differently so not all are crammed into the table, instead allow browsing them
- Show FID in table
  - Fix issue with the bottom-most rows not showing
- Fix issue when attempting navigation on an empty layer
- Fix issue "Error browsing database for PostGIS Raster tables" when attempting to open with PostGIS driver
- Fix weird issue with shapefile not being correctly read and (probably?) stderr output from gdal being printed all over the place
  - The worst of it is fixed by setting an error handler for gdal, which currently does nothing special. This is obviously not the best solution,
  maybe we collect the errors and add a pop-up widget to show a log of them or something like that?
- Show scrollbars for the layer list and the table
  - Also a scrollbar for the columns. Or some other visual indicator when not every column is shown
- Allow copying value from cell
- Allow inspecting long attributes better, maybe in a pop-up
- Allow browsing the dataset / layerinfo blocks if the text overflows
- Distinguish the "Feature" column more clearly
- Visual polish
</details>

Wontfix:
- ~~(Maybe) jumping to specific cell?~~
  - I figure there's really no clean solution for this that would be actually convenient


Following are features which I've thought of but aren't very high in priority.

Maybe (nice-to-haves):
- Some support for looking at raster metadata (not displaying raster itself, similar to `gdalinfo`)?
- Ability to select a whole feature in the attribute table
  - (Maybe) allow selecting multiple features?
  - (Maybe) copy it/them as GeoJSON/GML(?)
- Allow exporting dataset as a GeoPackage
  - (Maybe) as any ogr-supported driver
  - (Maybe) allow selecting which layers are exported
  - (Maybe) if selecting features are implemented, export only those features

Unlikely:
- Raster attribute tables
- Some way of displaying geometries as other whan WKT/WKB
  - Probably best bet would be to render the geometry as a temporary image and display it using the kitty terminal graphics protocol
  - However, this would be a significant undertaking and the actual utility of it is fairly minimal
  - But it would be pretty cool

Extremely unlikely:
- Editing of any kind, the main impetus for developing this tool is to just inspect data
</details>
