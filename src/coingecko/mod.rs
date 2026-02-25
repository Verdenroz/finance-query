//! CoinGecko cryptocurrency data.
//!
//! Requires the **`crypto`** feature flag.
//!
//! Uses the CoinGecko public API (no key required, 30 req/min free tier).
//! Rate limiting is handled automatically via a process-global client.
//!
//! # Quick Start
//!
//! ```no_run
//! use finance_query::crypto;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Top 10 coins by market cap in USD
//! let top = crypto::coins("usd", 10).await?;
//! for coin in &top {
//!     println!("{}: ${:.2}", coin.symbol, coin.current_price.unwrap_or(0.0));
//! }
//!
//! // Single coin by CoinGecko ID
//! let btc = crypto::coin("bitcoin", "usd").await?;
//! println!("BTC: ${:.2}", btc.current_price.unwrap_or(0.0));
//! # Ok(())
//! # }
//! ```

mod client;
mod models;
mod rate_limiter;

use client::CoinGeckoClient;
use std::sync::OnceLock;

use crate::error::Result;
pub use models::CoinQuote;

/// Process-global CoinGecko client (initialized lazily on first use).
static COINGECKO_CLIENT: OnceLock<CoinGeckoClient> = OnceLock::new();

fn client() -> Result<&'static CoinGeckoClient> {
    // Lazy init: no user-facing `init()` required since no key is needed.
    if COINGECKO_CLIENT.get().is_none() {
        let c = CoinGeckoClient::new()?;
        // Ignore error if another thread already set it
        let _ = COINGECKO_CLIENT.set(c);
    }
    Ok(COINGECKO_CLIENT.get().expect("just set above"))
}

/// Fetch the top `count` cryptocurrencies by market cap.
///
/// # Arguments
///
/// * `vs_currency` - Quote currency (e.g., `"usd"`, `"eur"`, `"btc"`)
/// * `count` - Number of coins to return (max 250)
///
/// # Errors
///
/// Returns an error on network failure or if the CoinGecko API rate limit is exceeded.
pub async fn coins(vs_currency: &str, count: usize) -> Result<Vec<CoinQuote>> {
    client()?.coins(vs_currency, count).await
}

/// Fetch a single coin by its CoinGecko ID (e.g., `"bitcoin"`, `"ethereum"`).
///
/// Use <https://api.coingecko.com/api/v3/coins/list> to discover CoinGecko IDs.
///
/// # Arguments
///
/// * `id` - CoinGecko coin ID
/// * `vs_currency` - Quote currency (e.g., `"usd"`)
pub async fn coin(id: &str, vs_currency: &str) -> Result<CoinQuote> {
    client()?.coin(id, vs_currency).await
}
