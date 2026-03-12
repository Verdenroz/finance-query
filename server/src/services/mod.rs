//! Service layer: shared business logic for REST handlers and GraphQL resolvers.
//!
//! Each service function encapsulates cache key construction, TTL selection,
//! and library calls. Returns `serde_json::Value` (matching cache storage format).

pub mod analysis;
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
pub mod search;
pub mod transcripts;

use finance_query::{Interval, TimeRange};

/// Shared error type for service functions.
pub type ServiceError = Box<dyn std::error::Error + Send + Sync>;

/// Shared result type for service functions.
pub type ServiceResult = Result<serde_json::Value, ServiceError>;

/// Parse an interval string into the library's `Interval` enum.
pub fn parse_interval(s: &str) -> Interval {
    match s {
        "1m" => Interval::OneMinute,
        "5m" => Interval::FiveMinutes,
        "15m" => Interval::FifteenMinutes,
        "30m" => Interval::ThirtyMinutes,
        "1h" => Interval::OneHour,
        "1d" => Interval::OneDay,
        "1wk" => Interval::OneWeek,
        "1mo" => Interval::OneMonth,
        "3mo" => Interval::ThreeMonths,
        _ => Interval::OneDay,
    }
}

/// Parse a range string into the library's `TimeRange` enum.
pub fn parse_range(s: &str) -> TimeRange {
    match s {
        "1d" => TimeRange::OneDay,
        "5d" => TimeRange::FiveDays,
        "1mo" => TimeRange::OneMonth,
        "3mo" => TimeRange::ThreeMonths,
        "6mo" => TimeRange::SixMonths,
        "1y" => TimeRange::OneYear,
        "2y" => TimeRange::TwoYears,
        "5y" => TimeRange::FiveYears,
        "10y" => TimeRange::TenYears,
        "ytd" => TimeRange::YearToDate,
        "max" => TimeRange::Max,
        _ => TimeRange::OneMonth,
    }
}
