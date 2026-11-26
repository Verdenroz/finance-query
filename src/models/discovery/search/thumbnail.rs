//! Search Thumbnail Models
//!
//! Image thumbnail data for search news results

use serde::{Deserialize, Serialize};

/// Thumbnail image with multiple resolutions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
pub struct NewsThumbnail {
    /// Available image resolutions
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub resolutions: Option<Vec<ThumbnailResolution>>,
}

/// Individual thumbnail resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
pub struct ThumbnailResolution {
    /// Image URL
    pub url: Option<String>,
    /// Image width in pixels
    pub width: Option<i32>,
    /// Image height in pixels
    pub height: Option<i32>,
    /// Resolution tag (e.g., "original", "140x140")
    pub tag: Option<String>,
}
