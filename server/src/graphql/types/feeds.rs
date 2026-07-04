//! GraphQL type for RSS/Atom feed entries.

use async_graphql::SimpleObject;
use finance_query::feeds::FeedEntry;
use serde::Deserialize;

/// A single entry from an RSS/Atom feed.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlFeedEntry {
    pub title: String,
    pub url: String,
    pub published: Option<String>,
    pub summary: Option<String>,
    pub source: String,
}

impl From<FeedEntry> for GqlFeedEntry {
    fn from(e: FeedEntry) -> Self {
        Self {
            title: e.title,
            url: e.url,
            published: e.published,
            summary: e.summary,
            source: e.source,
        }
    }
}
