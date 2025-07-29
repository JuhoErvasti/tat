use gdal::spatial_ref::SpatialRef;

/// Enum describing different kinds of vertical navigation
pub enum TatNavVertical {
    First,
    Last,
    DownOne,
    UpOne,
    DownHalfParagraph,
    UpHalfParagraph,
    DownParagraph,
    UpParagraph,
    Specific(i64),
    MouseScrollUp,
    MouseScrollDown,
}

/// Enum describing different kinds of horizontal navigation
pub enum TatNavHorizontal {
    Home,
    End,
    RightOne,
    LeftOne,
}

/// A struct which holds information about a coordinate reference system for displaying purposes
#[derive(Clone, Debug)]
pub struct TatCrs {
    auth_name: String,
    auth_code: i32,
    name: String
}

impl TatCrs {
    /// Constructs a new object
    pub fn new(a_name: String, a_code: i32, crs_name: String) -> Self {
        Self {
            auth_name: a_name,
            auth_code: a_code,
            name: crs_name,
        }
    }

    /// Constructs a new object from a GDAL SpatialRef object
    pub fn from_spatial_ref(sref: &SpatialRef) -> Option<Self> {
        let aname = match sref.auth_name() {
            Some(a_name) => a_name,
            _ => return None,
        };

        let acode = match sref.auth_code() {
            Ok(a_code) => a_code,
            _ => return None,
        };

        let name = match sref.name() {
            Some(c_name) => c_name,
            _ => return None,
        };

        Some(
            TatCrs::new(
                aname,
                acode,
                name,
            )
        )
    }

    /// Returns the CRS authority name (e.g. EPSG etc.)
    pub fn auth_name(&self) -> &str {
        &self.auth_name
    }

    /// Returns the CRS numerical code (e.g. 4326)
    pub fn auth_code(&self) -> i32 {
        self.auth_code
    }

    /// Return the name of the CRS
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A struct describing a field in a GDAL layer for displaying purposes
#[derive(Clone, Debug)]
pub struct TatField {
    name: String,
    dtype: u32,
}

impl TatField {
    /// Constructs a new object
    pub fn new(name: String, dtype: u32) -> Self {
        Self {
            name,
            dtype,
        }
    }

    /// Returns the name of the field
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the data type of the field as a u32, which can be turned into a string using
    /// gdal::field_type_to_name()
    pub fn dtype(&self) -> u32 {
        self.dtype
    }
}

/// A struct describing a geometry field in a GDAL layer for displaying purposes
#[derive(Clone, Debug)]
pub struct TatGeomField {
    name: String,
    geom_type: String,
    crs: Option<TatCrs>,
}

impl TatGeomField {
    /// Constructs a new object
    pub fn new(name: String, geom_type: String, crs: Option<TatCrs>) -> Self {
        Self {
            name,
            geom_type,
            crs,
        }
    }

    /// Returns the field's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the field's geometry type as a string slice
    pub fn geom_type(&self) -> &str {
        &self.geom_type
    }

    /// Returns the field's CRS (if any)
    pub fn crs(&self) -> Option<&TatCrs> {
        self.crs.as_ref()
    }
}

