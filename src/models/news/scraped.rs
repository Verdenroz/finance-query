//! Scraped news article model.

use serde::{Deserialize, Serialize};

/// A news article
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
pub struct News {
    /// Article title
    pub title: String,

    /// Article URL
    pub link: String,

    /// News source/publisher (e.g., "Reuters", "Bloomberg")
    pub source: String,

    /// Thumbnail image URL
    pub img: String,

    /// Relative time when the news was published (e.g., "1 hour ago", "2 days ago")
    pub time: String,
}

impl News {
    /// Create a new News article
    pub(crate) fn new(
        title: String,
        link: String,
        source: String,
        img: String,
        time: String,
    ) -> Self {
        Self {
            title,
            link,
            source,
            img,
            time,
        }
    }
}
