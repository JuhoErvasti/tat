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
- ~~Fix issue "Error browsing database for PostGIS Raster tables" when attempting to open with PostGIS driver~~
- ~~Fix weird issue with shapefile not being correctly read and (probably?) stderr output from gdal being printed all over the place~~
  - ~~The worst of it is fixed by setting an error handler for gdal, which currently does nothing special. This is obviously not the best solution,
  maybe we collect the errors and add a pop-up widget to show a log of them or something like that?~~
  - ~~Note: mostly a guess but I think some data drivers start indexing at 0 causing the errors, you also get weird behaviour with the gml test file~~

Midterm:
- ~~Show scrollbars for the layer list and the table~~
  - ~~Also a scrollbar for the columns? Or some other visual indicator when not every column is shown~~
  - (Maybe) somewhat relatedly; distinguish the "fid" column more clearly
- Filtering on the layer list
- Improve visuals
- Display geometry column(s) as WKT
- Allow browsing the dataset / layerinfo blocks if the text overflows
- Filtering the dataset itself (something like ogrinfo -where)
- Optimize performance, I'd like the program to feel very fast and responsive and ideally opening and browsing even huge layers should be almost instant. Not sure
  it's entirely possible especially if talking about data drivers with a remote connection and huge datasets, but I can try at least. At least the program should
  ideally remain responsive even if rendering the data to the screen is not instant.
- Jumping to specific FID
  - (Maybe) jumping to specific cell?
- (Maybe) allow copying value from table?
  - Also (maybe) allow inspecting long values better, maybe in a pop-up?

Longterm:
- (Maybe) add an inline UI for building PG/WFS etc. description to connect
- (Maybe) look at raster attribute tables?
- (Maybe) some level of mouse support?
- (Maybe) some kind of rendering of geometries?
  - Honestly probably not very useful or necessary but might be interesting to try something out

Probably NOT going to do:
- Editing
