//! CoinGecko data models.
//!
//! Re-exports canonical type from `crate::models::crypto`.

use std::collections::HashMap;

use serde::Deserialize;

pub use crate::models::crypto::CoinQuote;

// ============================================================================
// Search (`/search`)
// ============================================================================

/// Raw `/search` response.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResponseDTO {
    #[serde(default)]
    pub coins: Vec<SearchCoinDTO>,
}

/// One coin match from `/search`.
///
/// CoinGecko's response also carries `symbol`/`market_cap_rank`/thumbnail
/// URLs, but the canonical mapping only needs `id` (the identifier
/// [`Providers::crypto`](crate::Providers::crypto) accepts) and `name`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchCoinDTO {
    pub id: String,
    pub name: String,
}

// ============================================================================
// Trending (`/search/trending`)
// ============================================================================

/// Raw `/search/trending` response.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TrendingResponseDTO {
    #[serde(default)]
    pub coins: Vec<TrendingCoinWrapperDTO>,
}

/// A trending entry wraps its coin data under `"item"`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TrendingCoinWrapperDTO {
    pub item: TrendingCoinItemDTO,
}

/// One coin's data within a trending entry.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TrendingCoinItemDTO {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub market_cap_rank: Option<u32>,
    pub price_btc: Option<f64>,
    pub score: Option<u32>,
}

// ============================================================================
// Global (`/global`)
// ============================================================================

/// Raw `/global` response.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GlobalResponseDTO {
    pub data: GlobalDataDTO,
}

/// The `"data"` payload of `/global`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct GlobalDataDTO {
    pub active_cryptocurrencies: Option<u32>,
    pub markets: Option<u32>,
    #[serde(default)]
    pub total_market_cap: HashMap<String, f64>,
    #[serde(default)]
    pub total_volume: HashMap<String, f64>,
    #[serde(default)]
    pub market_cap_percentage: HashMap<String, f64>,
    pub market_cap_change_percentage_24h_usd: Option<f64>,
}
