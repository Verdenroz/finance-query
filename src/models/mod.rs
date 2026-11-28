//! Data models for finance-query responses.
//!
//! This module contains all the data structures returned by the library's API
//! methods. Types are organized by capability. Yahoo-backed capabilities
//! compile unconditionally; provider-specific capabilities are gated behind
//! the corresponding feature flag.

// ── Format type parameter ──────────────────────────────────────────────────
pub mod format;

// ── Capability directories ──────────────────────────────────────────────────

// Yahoo-backed (always available)
/// Financial event calendar models (earnings, dividends, options expirations).
pub mod calendar;
/// Chart/historical data models, including spark sparklines.
pub mod chart;
/// Corporate data: profiles, officers, ownership, news, transcripts, recommendations.
pub mod corporate;
/// Discovery: search, lookup, screeners, trending.
pub mod discovery;
/// SEC EDGAR filing data models.
pub mod filings;
/// Fundamental financial statement models (income, balance sheet, cash flow).
pub mod fundamentals;
/// Market-level data: summary, sectors, industries, hours, currencies, exchanges.
pub mod market;
/// Options contract models.
pub mod options;
/// Quote models for detailed stock information.
pub mod quote;
/// Market sentiment models (Fear & Greed Index).
pub mod sentiment;

// Provider-specific (gated on the provider feature that supplies them)
/// Commodities market data (gold, silver, oil, etc.) — Yahoo / FMP / Alpha Vantage.
pub mod commodities;
/// Cryptocurrency market data — CoinGecko, Alpha Vantage, FMP, Polygon.
pub mod crypto;
/// Macro-economic data — FRED, Alpha Vantage, Polygon.
pub mod economic;
/// Forex (foreign exchange) data models — Polygon / FMP / Alpha Vantage.
pub mod forex;
/// Futures market data models — Yahoo / Polygon / CFTC.
pub mod futures;
/// Stock market index data models — Yahoo / Polygon / FMP.
pub mod indices;

#[cfg(feature = "dataframe")]
mod dataframe;

#[cfg(all(test, feature = "dataframe"))]
pub(crate) mod dataframe_test_util {
    use polars::prelude::DataFrame;

    /// The first row of a `Datetime` column, in epoch microseconds.
    pub(crate) fn micros(df: &DataFrame, name: &str) -> Option<i64> {
        df.column(name).unwrap().datetime().unwrap().phys.get(0)
    }

    /// The first row of a `Date` column, in days since the epoch.
    pub(crate) fn days(df: &DataFrame, name: &str) -> Option<i32> {
        df.column(name).unwrap().date().unwrap().phys.get(0)
    }

    /// Lists `name:dtype` per column after checking the frame's time naming.
    pub(crate) fn columns(df: &DataFrame) -> Vec<String> {
        columns_exempt(df, &[])
    }

    /// Like [`columns`], skipping the time naming check for the named columns.
    pub(crate) fn columns_exempt(df: &DataFrame, exempt: &[&str]) -> Vec<String> {
        crate::__dataframe::assert_time_naming("frame", df, exempt);
        df.schema()
            .iter()
            .map(|(name, dtype)| format!("{name}:{dtype}"))
            .collect()
    }
}
