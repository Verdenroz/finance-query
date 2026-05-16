//! External financial data source adapters (internal — use `Ticker` or re-export modules).
//!
//! Adapters are `pub(crate)` — users interact with data through [`Ticker`](crate::Ticker),
//! [`Tickers`](crate::Tickers), and the public re-export modules ([`edgar`](crate::edgar),
//! [`fred`](crate::fred), [`crypto`](crate::crypto)). The provider system automatically
//! dispatches to the right adapter based on configured providers.
//!
//! Each adapter follows the standard structure (mod, client, models, optional endpoints)
//! with capability-mapped subdirectories.
//!
//! # Available adapters
//!
//! | Feature | Provider | Free tier | Endpoints | Coverage |
//! |---------|----------|-----------|-----------|----------|
//! | `alphavantage` | [Alpha Vantage](https://www.alphavantage.co/) | 25 req/day | ~100 | Stocks, forex, crypto, commodities, economic indicators, 50+ technical indicators |
//! | `polygon` | [Polygon.io](https://polygon.io/) | 5 req/sec | ~100 | Stocks, options, forex, crypto, indices, futures, economy, analyst data, WebSocket streaming |
//! | `fmp` | [Financial Modeling Prep](https://financialmodelingprep.com/) | 250 req/day | ~100 | Fundamentals, DCF/ratings, insider trading, institutional holdings, screener, 60+ exchanges |
//! | `crypto` | [CoinGecko](https://www.coingecko.com/) | 30 req/min | 2 | Top coins by market cap, single coin quotes (keyless) |
//! | `fred` | [FRED](https://fred.stlouisfed.org/) | 120 req/min | 2+ | 800k+ macro time series, US Treasury yield curve |
//! | *(always)* | [SEC EDGAR](https://www.sec.gov/edgar) | 10 req/sec | 5+ | Filing history, XBRL financials, full-text search (keyless, requires contact email) |
//!
//! # Quick comparison
//!
//! - **Alpha Vantage**: Best free option for technical indicators (50+) and economic data. Lowest rate limits.
//! - **Polygon.io**: Tick-level trades/quotes, SEC filings, real-time WebSocket streams. Best for market microstructure.
//! - **CoinGecko**: Best for cryptocurrency data. Keyless (public API). Rate-limited to 30 req/min.
//! - **FRED**: 800k+ US macro-economic time series (GDP, CPI, employment, interest rates). Free API key required.
//! - **SEC EDGAR**: Company filings, XBRL financials, full-text search. Always available, no feature flag needed.
//!
//! All adapters share:
//! - Singleton pattern with `init(api_key)` / `init_with_timeout(api_key, timeout)`
//! - Built-in rate limiting via shared token-bucket limiter
//! - Error mapping to [`crate::FinanceError`] variants
//! - Full mockito test coverage (no API key needed to run tests)

pub(crate) mod common;

/// Alpha Vantage financial data API (requires `alphavantage` feature).
#[cfg(feature = "alphavantage")]
pub(crate) mod alphavantage;

/// Polygon.io financial data API (requires `polygon` feature).
#[cfg(feature = "polygon")]
pub(crate) mod polygon;

/// Financial Modeling Prep (FMP) financial data API (requires `fmp` feature).
#[cfg(feature = "fmp")]
pub(crate) mod fmp;

/// CoinGecko cryptocurrency data API (keyless, requires `crypto` feature).
#[cfg(feature = "crypto")]
pub(crate) mod coingecko;

/// FRED economic data API (requires `fred` feature).
#[cfg(feature = "fred")]
pub(crate) mod fred;

/// SEC EDGAR API client (always available, requires init with contact email).
pub(crate) mod edgar;

/// Yahoo Finance API endpoints (always available — the canonical data source).
pub(crate) mod yahoo;
