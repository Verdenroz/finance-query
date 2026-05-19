//! Data models for finance-query responses.
//!
//! This module contains all the data structures returned by the library's API
//! methods. Types are organized by capability. Yahoo-backed capabilities
//! compile unconditionally; provider-specific capabilities are gated behind
//! the corresponding feature flag.

// ── Capability directories ──────────────────────────────────────────────────

// Yahoo-backed (always available)
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
/// Commodities market data (gold, silver, oil, etc.) — FMP / Alpha Vantage.
#[cfg(any(feature = "fmp", feature = "alphavantage"))]
pub mod commodities;
/// Cryptocurrency market data — CoinGecko, Alpha Vantage, FMP, Polygon.
#[cfg(any(
    feature = "crypto",
    feature = "alphavantage",
    feature = "fmp",
    feature = "polygon"
))]
pub mod crypto;
/// Macro-economic data — FRED, Alpha Vantage, Polygon.
#[cfg(any(feature = "fred", feature = "alphavantage", feature = "polygon"))]
pub mod economic;
/// Forex (foreign exchange) data models — Polygon / FMP / Alpha Vantage.
#[cfg(any(feature = "polygon", feature = "fmp", feature = "alphavantage"))]
pub mod forex;
/// Futures market data models — Polygon.
#[cfg(feature = "polygon")]
pub mod futures;
/// Stock market index data models — Polygon / FMP.
#[cfg(any(feature = "polygon", feature = "fmp"))]
pub mod indices;
/// Technical analysis indicator models (SMA, EMA, RSI, MACD, etc.).
#[cfg(any(feature = "polygon", feature = "fmp", feature = "alphavantage"))]
pub mod technicals;
