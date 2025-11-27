use super::thumbnail::NewsThumbnail;
use serde::{Deserialize, Serialize};

/// A news article from Yahoo Finance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsArticle {
    /// Article UUID
    pub uuid: String,

    /// Article title
    pub title: String,

    /// Publisher name
    pub publisher: Option<String>,

    /// Article link/URL
    pub link: String,

    /// Publish time (Unix timestamp)
    pub provider_publish_time: Option<i64>,

    /// Article type (e.g., "STORY", "VIDEO")
    #[serde(rename = "type")]
    pub article_type: Option<String>,

    /// Thumbnail image data
    pub thumbnail: Option<NewsThumbnail>,

    /// Related ticker symbols
    pub related_tickers: Option<Vec<String>>,
}
