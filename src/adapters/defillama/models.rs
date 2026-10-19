//! DefiLlama wire types.

use serde::Deserialize;
use std::collections::HashMap;

/// `GET /protocol/{slug}` — the full TVL history.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ProtocolResponse {
    /// Chronological TVL snapshots.
    #[serde(default)]
    pub tvl: Vec<TvlSnapshot>,
}

/// `GET /lite/protocols2` — every protocol's current TVL and its prior-period
/// values, plus the parent groupings.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LiteResponse {
    #[serde(default)]
    pub protocols: Vec<LiteProtocol>,
    #[serde(default, rename = "parentProtocols")]
    pub parent_protocols: Vec<LiteParent>,
}

/// One protocol of the lite list. It carries no slug; the slug is derived
/// from `name`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LiteProtocol {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub chains: Vec<String>,
    #[serde(default, rename = "chainTvls")]
    pub chain_tvls: HashMap<String, LiteChainTvl>,
    #[serde(default)]
    pub tvl: Option<f64>,
    #[serde(default, rename = "tvlPrevDay")]
    pub tvl_prev_day: Option<f64>,
    #[serde(default, rename = "tvlPrevWeek")]
    pub tvl_prev_week: Option<f64>,
    #[serde(default)]
    pub mcap: Option<f64>,
    /// Id of the grouping this protocol belongs to (`"parent#aave"`).
    #[serde(default, rename = "parentProtocol")]
    pub parent_protocol: Option<String>,
    #[serde(default, rename = "excludeTvlFromParent")]
    pub exclude_tvl_from_parent: bool,
    #[serde(default)]
    pub deprecated: bool,
}

/// A per-chain entry of `chainTvls`; keys follow the same chain and
/// breakdown naming as `currentChainTvls` on the full payload.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LiteChainTvl {
    #[serde(default)]
    pub tvl: f64,
}

/// A grouping of related protocols. It has no TVL of its own.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LiteParent {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub chains: Vec<String>,
    #[serde(default)]
    pub mcap: Option<f64>,
}

/// One TVL snapshot in a protocol's history.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TvlSnapshot {
    /// Unix timestamp in seconds.
    pub date: i64,
    #[serde(rename = "totalLiquidityUSD")]
    pub total_liquidity_usd: f64,
}

/// One entry of `GET /v2/chains`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ChainResponse {
    pub name: String,
    #[serde(default, rename = "tokenSymbol")]
    pub token_symbol: Option<String>,
    #[serde(default)]
    pub gecko_id: Option<String>,
    #[serde(default, rename = "chainId", deserialize_with = "lenient_chain_id")]
    pub chain_id: Option<i64>,
    #[serde(default)]
    pub tvl: Option<f64>,
}

/// DefiLlama serialises `chainId` as a number for most chains and as a string
/// for others, so one string would otherwise fail the whole chains response.
fn lenient_chain_id<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ChainId {
        Int(i64),
        Text(String),
    }

    Ok(match Option::<ChainId>::deserialize(deserializer)? {
        Some(ChainId::Int(id)) => Some(id),
        Some(ChainId::Text(id)) => id.parse().ok(),
        None => None,
    })
}

/// `GET stablecoins.llama.fi/stablecoins`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StablecoinsResponse {
    #[serde(default, rename = "peggedAssets")]
    pub pegged_assets: Vec<PeggedAsset>,
}

/// One stablecoin's circulating supply across time.
///
/// The circulating figures are objects keyed by peg type (`peggedUSD`,
/// `peggedEUR`, …) with a single entry, so they are read as maps rather than
/// as a fixed field name.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PeggedAsset {
    pub name: String,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub gecko_id: Option<String>,
    #[serde(default, rename = "pegType")]
    pub peg_type: Option<String>,
    #[serde(default, rename = "pegMechanism")]
    pub peg_mechanism: Option<String>,
    #[serde(default)]
    pub circulating: HashMap<String, f64>,
    #[serde(default, rename = "circulatingPrevDay")]
    pub circulating_prev_day: HashMap<String, f64>,
    #[serde(default, rename = "circulatingPrevWeek")]
    pub circulating_prev_week: HashMap<String, f64>,
    #[serde(default, rename = "circulatingPrevMonth")]
    pub circulating_prev_month: HashMap<String, f64>,
}
