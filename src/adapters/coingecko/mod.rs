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

pub(crate) mod chart; // CHART
mod client;
pub(crate) mod discovery; // DISCOVERY
mod models;

use client::CoinGeckoClient;
use std::sync::OnceLock;

use crate::error::Result;
pub use crate::models::crypto::CoinQuote;
pub use discovery::fetch_symbol_search_response;

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

/// Convert one trending-coin wrapper into the canonical [`TrendingCoin`](crate::models::crypto::TrendingCoin).
fn to_trending_coin(w: models::TrendingCoinWrapperDTO) -> crate::models::crypto::TrendingCoin {
    crate::models::crypto::TrendingCoin {
        id: Some(w.item.id),
        symbol: Some(w.item.symbol.to_ascii_uppercase()),
        name: Some(w.item.name),
        market_cap_rank: w.item.market_cap_rank,
        price_btc: w.item.price_btc,
        score: w.item.score,
    }
}

/// Fetch coins trending in the last 24h as canonical [`TrendingCoin`](crate::models::crypto::TrendingCoin)s.
pub async fn fetch_crypto_trending_response() -> Result<Vec<crate::models::crypto::TrendingCoin>> {
    let resp = client()?.trending().await?;
    Ok(resp.coins.into_iter().map(to_trending_coin).collect())
}

/// Convert the raw `/global` payload into canonical [`GlobalCryptoStats`](crate::models::crypto::GlobalCryptoStats).
fn to_global_crypto_stats(data: models::GlobalDataDTO) -> crate::models::crypto::GlobalCryptoStats {
    crate::models::crypto::GlobalCryptoStats {
        active_cryptocurrencies: data.active_cryptocurrencies,
        markets: data.markets,
        total_market_cap_usd: data.total_market_cap.get("usd").copied(),
        total_volume_usd: data.total_volume.get("usd").copied(),
        btc_dominance: data.market_cap_percentage.get("btc").copied(),
        eth_dominance: data.market_cap_percentage.get("eth").copied(),
        market_cap_change_percentage_24h_usd: data.market_cap_change_percentage_24h_usd,
    }
}

/// Fetch aggregate global cryptocurrency market statistics.
pub async fn fetch_crypto_global_response() -> Result<crate::models::crypto::GlobalCryptoStats> {
    let resp = client()?.global().await?;
    Ok(to_global_crypto_stats(resp.data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_trending_wrapper_and_uppercases_symbol() {
        let dto: models::TrendingResponseDTO = serde_json::from_value(serde_json::json!({
            "coins": [{
                "item": {
                    "id": "bitcoin",
                    "name": "Bitcoin",
                    "symbol": "btc",
                    "market_cap_rank": 1,
                    "price_btc": 1.0,
                    "score": 0
                }
            }]
        }))
        .unwrap();

        let coins: Vec<_> = dto.coins.into_iter().map(to_trending_coin).collect();
        assert_eq!(coins.len(), 1);
        assert_eq!(coins[0].id.as_deref(), Some("bitcoin"));
        assert_eq!(coins[0].symbol.as_deref(), Some("BTC"));
        assert_eq!(coins[0].market_cap_rank, Some(1));
        assert_eq!(coins[0].score, Some(0));
    }

    #[test]
    fn maps_global_stats_extracting_usd_and_dominance() {
        let dto: models::GlobalResponseDTO = serde_json::from_value(serde_json::json!({
            "data": {
                "active_cryptocurrencies": 18137,
                "markets": 1510,
                "total_market_cap": { "usd": 3_000_000_000_000.0_f64, "btc": 30_000_000.0_f64 },
                "total_volume": { "usd": 100_000_000_000.0_f64 },
                "market_cap_percentage": { "btc": 45.2, "eth": 18.1 },
                "market_cap_change_percentage_24h_usd": 0.64
            }
        }))
        .unwrap();

        let stats = to_global_crypto_stats(dto.data);
        assert_eq!(stats.active_cryptocurrencies, Some(18137));
        assert_eq!(stats.total_market_cap_usd, Some(3_000_000_000_000.0));
        assert_eq!(stats.total_volume_usd, Some(100_000_000_000.0));
        assert_eq!(stats.btc_dominance, Some(45.2));
        assert_eq!(stats.eth_dominance, Some(18.1));
        assert_eq!(stats.market_cap_change_percentage_24h_usd, Some(0.64));
    }

    #[test]
    fn global_stats_missing_currency_key_yields_none() {
        let dto: models::GlobalResponseDTO = serde_json::from_value(serde_json::json!({
            "data": {
                "active_cryptocurrencies": null,
                "markets": null,
                "total_market_cap": {},
                "total_volume": {},
                "market_cap_percentage": {},
                "market_cap_change_percentage_24h_usd": null
            }
        }))
        .unwrap();

        let stats = to_global_crypto_stats(dto.data);
        assert_eq!(stats.total_market_cap_usd, None);
        assert_eq!(stats.btc_dominance, None);
    }
}
