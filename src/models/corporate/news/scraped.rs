//! Scraped news article model.

use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use finance_query_derive::PyModel;

/// A news article
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "python", derive(PyModel))]
#[cfg_attr(feature = "python", py_model(dataframe = "columns"))]
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

    /// Which provider supplied this article (None = Yahoo Finance default)
    #[cfg_attr(feature = "python", py_model(skip))]
    pub provider_id: Option<crate::providers::Provider>,

    /// Sentiment score for this article's title (VADER lexicon-based).
    /// Only present when the `sentiment` feature is enabled.
    #[cfg(feature = "sentiment")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[cfg_attr(feature = "python", py_model(skip))]
    pub sentiment: Option<crate::models::sentiment::Sentiment>,
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
            provider_id: None,
            #[cfg(feature = "sentiment")]
            sentiment: None,
        }
    }
}
