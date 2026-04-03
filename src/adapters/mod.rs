//! External financial data source adapters.
//!
//! Each adapter is behind a feature flag and follows the same pattern:
//!
//! 1. **Enable the feature** in `Cargo.toml`:
//!    ```toml
//!    [dependencies]
//!    finance-query = { version = "2.4", features = ["polygon"] }
//!    ```
//!
//! 2. **Initialize once** at startup with your API key:
//!    ```no_run
//!    # #[cfg(feature = "polygon")]
//!    finance_query::adapters::polygon::init("YOUR_KEY").unwrap();
//!    ```
//!
//! 3. **Call any endpoint** — all functions are async:
//!    ```no_run
//!    # #[cfg(feature = "polygon")]
//!    # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!    use finance_query::adapters::polygon::{self, Timespan};
//!    let bars = polygon::stock_aggregates("AAPL", 1, Timespan::Day, "2024-01-01", "2024-12-31", None).await?;
//!    # Ok(())
//!    # }
//!    ```
//!
//! # Available adapters
//!
//! | Feature | Provider | Free tier | Endpoints | Coverage |
//! |---------|----------|-----------|-----------|----------|
//! | `alphavantage` | [Alpha Vantage](https://www.alphavantage.co/) | 25 req/day | ~100 | Stocks, forex, crypto, commodities, economic indicators, 50+ technical indicators |
//! | `polygon` | [Polygon.io](https://polygon.io/) | 5 req/sec | ~100 | Stocks, options, forex, crypto, indices, futures, economy, analyst data, WebSocket streaming |
//! | `fmp` | [Financial Modeling Prep](https://financialmodelingprep.com/) | 250 req/day | ~100 | Fundamentals, DCF/ratings, insider trading, institutional holdings, screener, 60+ exchanges |
//!
//! # Quick comparison
//!
//! - **Alpha Vantage**: Best free option for technical indicators (50+) and economic data. Lowest rate limits.
//! - **Polygon.io**: Tick-level trades/quotes, SEC filings, real-time WebSocket streams. Best for market microstructure.
//! - **FMP**: Deepest fundamentals coverage (as-reported financials, DCF, ratings, analyst estimates, earnings transcripts). Best for fundamental analysis.
//!
//! All adapters share:
//! - Singleton pattern with `init(api_key)` / `init_with_timeout(api_key, timeout)`
//! - Built-in rate limiting via shared token-bucket limiter
//! - Error mapping to [`crate::FinanceError`] variants
//! - Full mockito test coverage (no API key needed to run tests)

/// Alpha Vantage financial data API (requires `alphavantage` feature).
#[cfg(feature = "alphavantage")]
pub mod alphavantage;

/// Polygon.io financial data API (requires `polygon` feature).
#[cfg(feature = "polygon")]
pub mod polygon;

/// Financial Modeling Prep (FMP) financial data API (requires `fmp` feature).
#[cfg(feature = "fmp")]
pub mod fmp;
