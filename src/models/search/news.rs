//! Search News Model
//!
//! Represents news articles from search results

use super::thumbnail::NewsThumbnail;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// A collection of search news with DataFrame support.
///
/// This wrapper allows `search_results.news.to_dataframe()` syntax while still
/// acting like a `Vec<SearchNews>` for iteration, indexing, etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SearchNewsList(pub Vec<SearchNews>);

impl Deref for SearchNewsList {
    type Target = Vec<SearchNews>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for SearchNewsList {
    type Item = SearchNews;
    type IntoIter = std::vec::IntoIter<SearchNews>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a SearchNewsList {
    type Item = &'a SearchNews;
    type IntoIter = std::slice::Iter<'a, SearchNews>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(feature = "dataframe")]
impl SearchNewsList {
    /// Converts the news to a polars DataFrame.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        SearchNews::vec_to_dataframe(&self.0)
    }
}

/// A news result from search
///
/// When the `dataframe` feature is enabled, scalar fields can be converted
/// to a DataFrame. Complex fields (thumbnail, related_tickers) are automatically skipped.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct SearchNews {
    /// Unique news article identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Article title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Publisher name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    /// Article URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    /// Publication timestamp (Unix epoch seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_publish_time: Option<i64>,
    /// Article type (STORY, VIDEO, etc.)
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub news_type: Option<String>,
    /// Article thumbnail image (excluded from DataFrame)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<NewsThumbnail>,
    /// Related stock symbols (excluded from DataFrame)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_tickers: Option<Vec<String>>,
}
