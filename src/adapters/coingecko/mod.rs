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

use client::CoinGeckoClient;
use std::sync::OnceLock;

use crate::error::Result;
pub use crate::models::crypto::CoinQuote;

/// Process-global CoinGecko client (initialized lazily on first use).
static COINGECKO_CLIENT: OnceLock<CoinGeckoClient> = OnceLock::new();

fn client() -> Result<&'static CoinGeckoClient> {
    if COINGECKO_CLIENT.get().is_none() {
        let _ = COINGECKO_CLIENT.set(CoinGeckoClient::new()?);
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

// ============================================================================
// Canonical model conversion functions
// ============================================================================

/// Fetch canonical CryptoQuote for a CoinGecko coin.
pub async fn fetch_crypto_quote_response(
    id: &str,
    vs_currency: &str,
) -> Result<crate::models::crypto::CryptoQuote> {
    let quote = coin(id, vs_currency).await?;
    Ok(crate::models::crypto::CryptoQuote {
        id: quote.id,
        symbol: quote.symbol,
        name: quote.name,
        price: quote.current_price,
        market_cap: quote.market_cap,
        volume_24h: quote.total_volume,
        change_24h: None,
        change_percent_24h: quote.price_change_percentage_24h,
        high_24h: None,
        low_24h: None,
        circulating_supply: quote.circulating_supply,
    })
}
