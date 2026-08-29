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
