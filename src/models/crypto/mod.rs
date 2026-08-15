//! Cryptocurrency data models.
//!
//! Canonical public types for cryptocurrency quotes from multiple providers.

/// DeFi protocol and chain models — DefiLlama.
#[cfg(feature = "defi")]
pub mod defi;

use serde::{Deserialize, Serialize};

/// A provider-agnostic cryptocurrency quote.
///
/// Obtain via [`Providers::crypto`](crate::Providers::crypto) then
/// [`.quote()`](crate::domains::CryptoCoin::quote). Supported providers:
/// Alpha Vantage, CoinGecko, FMP, Polygon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CryptoQuote {
    /// Coin identifier (e.g., `"bitcoin"` for CoinGecko, `"BTC"` for others)
    pub id: String,
    /// Ticker symbol in uppercase (e.g., `"BTC"`, `"ETH"`)
    pub symbol: String,
    /// Full coin name (e.g., `"Bitcoin"`)
    pub name: String,
    /// Current price in the requested currency
    pub price: Option<f64>,
    /// Market capitalisation
    pub market_cap: Option<f64>,
    /// 24-hour trading volume
    pub volume_24h: Option<f64>,
    /// 24-hour absolute price change
    pub change_24h: Option<f64>,
    /// 24-hour price change percentage
    pub change_percent_24h: Option<f64>,
    /// 24-hour high
    pub high_24h: Option<f64>,
    /// 24-hour low
    pub low_24h: Option<f64>,
    /// Circulating supply
    pub circulating_supply: Option<f64>,
}

/// A cryptocurrency quote from CoinGecko.
///
/// Obtain via [`crypto::coins`](crate::crypto::coins) or [`crypto::coin`](crate::crypto::coin).
#[cfg(feature = "crypto")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CoinQuote {
    /// CoinGecko coin ID (e.g., `"bitcoin"`, `"ethereum"`)
    pub id: String,
    /// Ticker symbol in uppercase (e.g., `"BTC"`, `"ETH"`)
    pub symbol: String,
    /// Full coin name (e.g., `"Bitcoin"`)
    pub name: String,
    /// Current price in the requested currency
    pub current_price: Option<f64>,
    /// Market capitalisation
    pub market_cap: Option<f64>,
    /// 24-hour price change percentage
    pub price_change_percentage_24h: Option<f64>,
    /// 24-hour trading volume
    pub total_volume: Option<f64>,
    /// Circulating supply
    pub circulating_supply: Option<f64>,
    /// URL to the coin's logo image
    pub image: Option<String>,
    /// Market cap rank (1 = highest market cap)
    pub market_cap_rank: Option<u32>,
}

/// A coin trending in the last 24h, from CoinGecko's `/search/trending`.
///
/// Obtain via [`Market::crypto_trending`](crate::domains::Market::crypto_trending).
#[cfg(feature = "crypto")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TrendingCoin {
    /// CoinGecko coin ID (e.g., `"bitcoin"`)
    pub id: Option<String>,
    /// Ticker symbol in uppercase (e.g., `"BTC"`)
    pub symbol: Option<String>,
    /// Full coin name (e.g., `"Bitcoin"`)
    pub name: Option<String>,
    /// Market cap rank (1 = highest market cap)
    pub market_cap_rank: Option<u32>,
    /// Price denominated in BTC
    pub price_btc: Option<f64>,
    /// Trending rank (0 = most trending)
    pub score: Option<u32>,
}

/// Aggregate global cryptocurrency market statistics, from CoinGecko's `/global`.
///
/// Obtain via [`Market::crypto_global`](crate::domains::Market::crypto_global).
#[cfg(feature = "crypto")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GlobalCryptoStats {
    /// Number of active cryptocurrencies tracked
    pub active_cryptocurrencies: Option<u32>,
    /// Number of markets tracked
    pub markets: Option<u32>,
    /// Total market capitalisation in USD across all tracked coins
    pub total_market_cap_usd: Option<f64>,
    /// Total 24-hour trading volume in USD across all tracked coins
    pub total_volume_usd: Option<f64>,
    /// Bitcoin's percentage of total market cap
    pub btc_dominance: Option<f64>,
    /// Ethereum's percentage of total market cap
    pub eth_dominance: Option<f64>,
    /// 24-hour percentage change in total market cap (USD)
    pub market_cap_change_percentage_24h_usd: Option<f64>,
}
