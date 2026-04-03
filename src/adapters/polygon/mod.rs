//! Polygon.io API client for financial data.
//!
//! Requires the **`polygon`** feature flag and an API key from
//! <https://polygon.io/>.
//!
//! Call [`init`] once at startup before using any query functions.
//!
//! # Quick Start
//!
//! ```no_run
//! use finance_query::adapters::polygon;
//! use finance_query::adapters::polygon::Timespan;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! polygon::init("YOUR_API_KEY")?;
//!
//! // Stock aggregate bars
//! let bars = polygon::stock_aggregates("AAPL", 1, Timespan::Day, "2024-01-01", "2024-01-31", None).await?;
//!
//! // Snapshot
//! let snap = polygon::stock_snapshot("AAPL").await?;
//!
//! // Last trade
//! let trade = polygon::stock_last_trade("AAPL").await?;
//! # Ok(())
//! # }
//! ```

mod client;
pub mod models;

mod reference;
pub mod stocks;

pub mod crypto;
pub mod forex;
pub mod futures;
pub mod indices;
pub mod options;

mod alternative;
mod benzinga;
mod corporate_events;
mod economy;
mod etf;
pub mod websocket;

use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;
use client::PolygonClientBuilder;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

pub use alternative::*;
pub use benzinga::*;
pub use corporate_events::*;
pub use economy::*;
pub use etf::*;
pub use models::*;
pub use reference::*;

// Re-export items explicitly for modules with overlapping submodule names
// to avoid ambiguous glob re-exports (aggregates, snapshots, etc.).
pub use crypto::{
    CryptoDailyOpenClose, CryptoLastTrade, CryptoLastTradeResponse, CryptoOpenCloseTrade,
    crypto_aggregates, crypto_daily_open_close, crypto_ema, crypto_grouped_daily,
    crypto_last_trade, crypto_macd, crypto_previous_close, crypto_rsi, crypto_sma, crypto_snapshot,
    crypto_snapshots_all, crypto_top_movers, crypto_trades,
};
pub use forex::{
    ConversionLast, CurrencyConversion, ForexLastQuote, ForexQuoteResponse, currency_conversion,
    forex_aggregates, forex_ema, forex_grouped_daily, forex_last_quote, forex_macd,
    forex_previous_close, forex_quotes, forex_rsi, forex_sma, forex_snapshot, forex_snapshots_all,
    forex_top_movers,
};
pub use futures::{
    FuturesContract, FuturesProduct, FuturesSchedule, FuturesSession, FuturesSnapshot,
    FuturesSnapshotResponse, futures_aggregates, futures_contracts, futures_products,
    futures_quotes, futures_schedules, futures_snapshot, futures_trades,
};
pub use indices::{
    IndexSession, IndexSnapshot, IndexSnapshotResponse, index_aggregates, index_daily_open_close,
    index_ema, index_macd, index_previous_close, index_rsi, index_sma, index_snapshot,
};
pub use options::{
    AdditionalUnderlying, OptionsContract, OptionsContractResponse,
    OptionsContractSnapshotResponse, OptionsGreeks, OptionsSnapshot, OptionsSnapshotDetails,
    OptionsSnapshotQuote, OptionsSnapshotTrade, OptionsUnderlyingAsset, options_aggregates,
    options_chain_snapshot, options_contract_details, options_contract_snapshot, options_contracts,
    options_daily_open_close, options_ema, options_last_trade, options_macd,
    options_previous_close, options_quotes, options_rsi, options_sma, options_trades,
};
pub use stocks::*;

/// Polygon.io free-tier rate limit: 5 req/sec.
const PG_RATE_PER_SEC: f64 = 5.0;

struct PolygonSingleton {
    api_key: String,
    timeout: Duration,
    limiter: Arc<RateLimiter>,
}

static PG_SINGLETON: OnceLock<PolygonSingleton> = OnceLock::new();

/// Initialize the global Polygon client with an API key.
///
/// Must be called once before using any query functions. Subsequent calls return an error.
///
/// # Errors
///
/// Returns [`FinanceError::InvalidParameter`] if already initialized.
pub fn init(api_key: impl Into<String>) -> Result<()> {
    init_with_timeout(api_key, Duration::from_secs(30))
}

/// Initialize the Polygon client with a custom timeout.
pub fn init_with_timeout(api_key: impl Into<String>, timeout: Duration) -> Result<()> {
    PG_SINGLETON
        .set(PolygonSingleton {
            api_key: api_key.into(),
            timeout,
            limiter: Arc::new(RateLimiter::new(PG_RATE_PER_SEC)),
        })
        .map_err(|_| FinanceError::InvalidParameter {
            param: "polygon".to_string(),
            reason: "Polygon client already initialized".to_string(),
        })
}

/// Build a fresh client from the singleton state.
pub(crate) fn build_client() -> Result<client::PolygonClient> {
    let s = PG_SINGLETON
        .get()
        .ok_or_else(|| FinanceError::InvalidParameter {
            param: "polygon".to_string(),
            reason: "Polygon not initialized. Call polygon::init(api_key) first.".to_string(),
        })?;
    PolygonClientBuilder::new(&s.api_key)
        .timeout(s.timeout)
        .build_with_limiter(Arc::clone(&s.limiter))
}

/// Build a test client pointing at a mock server URL.
#[cfg(test)]
pub(crate) fn build_test_client(base_url: &str) -> Result<client::PolygonClient> {
    PolygonClientBuilder::new("test-key")
        .timeout(Duration::from_secs(5))
        .base_url(base_url)
        .build_with_limiter(Arc::new(RateLimiter::new(100.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_errors_on_double_init() {
        let _ = init("test-key-1");
        let result = init("test-key-2");
        assert!(matches!(result, Err(FinanceError::InvalidParameter { .. })));
    }
}
