//! Cryptocurrency data models.
//!
//! Canonical public types for cryptocurrency quotes from multiple providers.

use serde::{Deserialize, Serialize};

/// A provider-agnostic cryptocurrency quote.
///
/// Obtain via [`Ticker::crypto_quote`](crate::Ticker::crypto_quote). Supported providers:
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
