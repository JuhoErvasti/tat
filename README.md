# Terminal Attribute Table (tat)

For inspecting geospatial data in the terminal.

## TODO

Immediately:
- ~~Fix issues with some layers not opening in the table~~
- Fix issue "Error browsing database for PostGIS Raster tables" when attempting to open with PostGIS driver
- ~~Improve performance on large layers (only render what can be seen)~~
  - Improve performance on opening large layers
- Fit columns differently so not all are crammed into the table, instead allow browsing them
- Show FID in table
- Fix issue when attempting navigation on an empty layer
- Fix weird issue with shapefile not being correctly read and stderr output being printed all over the place

Midterm:
- Show scrollbars for the layer list and the table
- Filtering on the layer list
- Improve visuals
- Display geometry column(s) as WKT
- Allow browsing the dataset / layerinfo blocks if the text overflows
- Filtering the dataset itself (something like ogrinfo -where)
- Jumping to specific FID
  - (Maybe) jumping to specific cell?
- (Maybe) allow copying value from table?

Longterm:
- (Maybe) add an inline UI for building PG/WFS etc. description to connect
- (Maybe) look at raster attribute tables?
- (Maybe) some level of mouse support?
- (Maybe) some kind of rendering of geometries?
  - Honestly probably not very useful or necessary but might be interesting to try something out

Probably NOT going to do:
- Editing
