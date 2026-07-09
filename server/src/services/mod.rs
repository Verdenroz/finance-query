//! Service layer: shared business logic for REST handlers and GraphQL resolvers.
//!
//! Each service function encapsulates cache key construction, TTL selection,
//! and library calls. Returns `serde_json::Value` (matching cache storage format).

pub mod analysis;
pub mod calendar;
pub mod chart;
pub mod crypto;
pub mod edgar;
pub mod events;
pub mod feeds;
pub mod financials;
pub mod fred;
pub mod holders;
pub mod indicators;
pub mod market;
pub mod metadata;
pub mod news;
pub mod options;
pub mod quote;
pub mod risk;
pub mod screener;
pub mod search;
pub mod transcripts;

use finance_query::{Interval, TimeRange};

/// Shared error type for service functions.
pub type ServiceError = Box<dyn std::error::Error + Send + Sync>;

/// Shared result type for service functions.
pub type ServiceResult = Result<serde_json::Value, ServiceError>;

/// Translate a typed library response in place when a target language is set.
#[cfg(feature = "translation")]
pub async fn translate<T: finance_query::translation::Translatable>(
    value: &mut T,
    lang: Option<&str>,
) -> Result<(), ServiceError> {
    if let Some(lang) = lang {
        finance_query::translation::translate(value, lang)
            .await
            .map_err(|e| Box::new(e) as ServiceError)?;
    }
    Ok(())
}

/// No-op when the `translation` feature is disabled.
#[cfg(not(feature = "translation"))]
pub async fn translate<T>(_value: &mut T, _lang: Option<&str>) -> Result<(), ServiceError> {
    Ok(())
}

/// Cache-key segment for a resolved language (`en` when untranslated).
pub fn lang_key(lang: Option<&str>) -> &str {
    lang.unwrap_or("en")
}

/// Parse an interval string into the library's `Interval` enum, falling back to
/// `OneDay` for anything invalid (upstream GraphQL enums already validate this).
pub fn parse_interval(s: &str) -> Interval {
    s.parse().unwrap_or(Interval::OneDay)
}

/// Parse a range string into the library's `TimeRange` enum, falling back to
/// `OneMonth` for anything invalid (upstream GraphQL enums already validate this).
pub fn parse_range(s: &str) -> TimeRange {
    s.parse().unwrap_or(TimeRange::OneMonth)
}
