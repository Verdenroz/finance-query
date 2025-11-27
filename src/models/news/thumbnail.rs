use serde::{Deserialize, Serialize};

/// News article thumbnail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsThumbnail {
    /// Available resolutions
    pub resolutions: Option<Vec<ThumbnailResolution>>,
}

/// Thumbnail image resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailResolution {
    /// Image URL
    pub url: String,

    /// Image width in pixels
    pub width: Option<i32>,

    /// Image height in pixels
    pub height: Option<i32>,

    /// Resolution tag
    pub tag: Option<String>,
}
