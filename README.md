# Terminal Attribute Table (tat)

For inspecting geospatial data in the terminal.

## TODO

Immediately:
- ~~Fix issues with some layers not opening in the table~~
- ~~Improve performance on large layers (only render what can be seen)~~
  - ~~Improve performance on opening large layers~~
- ~~Fit columns differently so not all are crammed into the table, instead allow browsing them~~
- ~~Show FID in table~~
  - ~~Fix issue with the bottom-most rows not showing~~
- ~~Fix issue when attempting navigation on an empty layer~~
- Fix issue "Error browsing database for PostGIS Raster tables" when attempting to open with PostGIS driver
- Fix weird issue with shapefile not being correctly read and (probably?) stderr output from gdal being printed all over the place
  - Note: mostly a guess but I think some data drivers start indexing at 0 causing the errors, you also get weird behaviour with the gml test file

Midterm:
- Show scrollbars for the layer list and the table
  - Also a scrollbar for the columns? Or some other visual indicator when not every column is shown
  - (Maybe) somewhat relatedly; distinguish the "fid" column more clearly
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
