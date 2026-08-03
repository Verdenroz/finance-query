//! DeFi / on-chain models.
//!
//! Populated by the DefiLlama adapter. These describe *protocols and chains*
//! rather than tradable instruments, so they live beside — not inside —
//! [`CryptoQuote`](super::CryptoQuote).

use serde::{Deserialize, Serialize};

/// Total value locked in a DeFi protocol, with the metadata needed to
/// identify it.
///
/// Obtain via [`CryptoCoin::tvl`](crate::CryptoCoin::tvl).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProtocolTvl {
    /// DefiLlama protocol slug, as queried (e.g. `"aave"`).
    pub slug: String,
    /// Display name (e.g. `"AAVE"`).
    pub name: Option<String>,
    /// Governance/token symbol, when the protocol has one.
    pub symbol: Option<String>,
    /// Project homepage.
    pub url: Option<String>,
    /// Chains the protocol is deployed on.
    pub chains: Vec<String>,
    /// Latest total value locked, in USD.
    pub tvl: Option<f64>,
    /// TVL broken down by chain, in USD.
    pub tvl_by_chain: Vec<ChainAllocation>,
    /// Change in TVL over the last day, as a percentage.
    pub change_1d_percent: Option<f64>,
    /// Change in TVL over the last seven days, as a percentage.
    pub change_7d_percent: Option<f64>,
    /// Market capitalisation of the protocol's token, in USD.
    pub market_cap: Option<f64>,
}

/// One chain's share of a protocol's TVL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChainAllocation {
    /// Chain name (e.g. `"Ethereum"`).
    pub chain: String,
    /// Value locked on that chain, in USD.
    pub tvl: f64,
}

/// One point of a protocol's TVL history.
///
/// Obtain via [`CryptoCoin::tvl_history`](crate::CryptoCoin::tvl_history).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TvlPoint {
    /// Unix timestamp (seconds) of the snapshot.
    pub timestamp: i64,
    /// Total value locked at that moment, in USD.
    pub tvl: f64,
}

/// Aggregate value locked on one blockchain.
///
/// Obtain via [`defi::chains`](crate::defi::chains).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChainTvl {
    /// Chain name (e.g. `"Ethereum"`).
    pub name: String,
    /// Native token symbol, when the chain has one.
    pub token_symbol: Option<String>,
    /// CoinGecko id of the native token, for cross-referencing a price.
    pub gecko_id: Option<String>,
    /// EVM chain id, where one exists.
    pub chain_id: Option<i64>,
    /// Total value locked across every protocol on the chain, in USD.
    pub tvl: Option<f64>,
}

/// Circulating supply of one stablecoin.
///
/// Obtain via [`defi::stablecoins`](crate::defi::stablecoins).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StablecoinSupply {
    /// Stablecoin name (e.g. `"Tether"`).
    pub name: String,
    /// Ticker symbol (e.g. `"USDT"`).
    pub symbol: Option<String>,
    /// CoinGecko id, for cross-referencing a price.
    pub gecko_id: Option<String>,
    /// What the coin is pegged to, e.g. `"peggedUSD"`.
    pub peg_type: Option<String>,
    /// How the peg is maintained, e.g. `"fiat-backed"`, `"crypto-backed"`.
    pub peg_mechanism: Option<String>,
    /// Current circulating supply, denominated in the pegged asset.
    pub circulating: Option<f64>,
    /// Circulating supply one day ago, for change calculations.
    pub circulating_prev_day: Option<f64>,
    /// Circulating supply one week ago.
    pub circulating_prev_week: Option<f64>,
    /// Circulating supply one month ago.
    pub circulating_prev_month: Option<f64>,
}
