//! GDELT DOC 2.0 API wire types.
//!
//! `mode=artlist&format=json` answers with a single `articles` array. GDELT
//! reports more fields than modelled here (`url_mobile`, `language`,
//! `sourcecountry`, …) — only the ones [`super::corporate::to_news_at`] maps
//! onto the canonical [`crate::models::corporate::news::News`] are kept.
//! GDELT sometimes omits a field entirely (rather than sending an empty
//! string) for older or thinly-indexed sources, so everything but `url` is
//! optional.

use serde::Deserialize;

/// The envelope returned by `mode=artlist`.
#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct GdeltDocResponse {
    #[serde(default)]
    pub articles: Vec<GdeltArticle>,
}

/// One article as returned by the DOC 2.0 API's `artlist` mode.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GdeltArticle {
    pub url: String,
    #[serde(default)]
    pub title: Option<String>,
    /// `"YYYYMMDDTHHMMSSZ"` — when GDELT first indexed the article.
    #[serde(default)]
    pub seendate: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub socialimage: Option<String>,
}
