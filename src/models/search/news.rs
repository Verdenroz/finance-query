//! Search News Model
//!
//! Represents news articles from search results

use super::thumbnail::NewsThumbnail;
use serde::{Deserialize, Serialize};

/// A news result from search
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchNews {
    /// Unique news article identifier
    pub uuid: Option<String>,
    /// Article title
    pub title: Option<String>,
    /// Publisher name
    pub publisher: Option<String>,
    /// Article URL
    pub link: Option<String>,
    /// Publication timestamp (Unix epoch)
    pub provider_publish_time: Option<i64>,
    /// Article type (STORY, VIDEO, etc.)
    #[serde(rename = "type")]
    pub news_type: Option<String>,
    /// Article thumbnail image
    pub thumbnail: Option<NewsThumbnail>,
    /// Related stock symbols
    pub related_tickers: Option<Vec<String>>,
}
