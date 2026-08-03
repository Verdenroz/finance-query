//! `CRYPTO` capability for DefiLlama — protocol TVL.

use crate::error::Result;
use crate::models::crypto::defi::{ChainAllocation, ProtocolTvl, TvlPoint};

use super::models::ProtocolResponse;

/// Seconds in a day, for locating the comparison points of a TVL change.
const DAY: i64 = 86_400;

/// Normalise a handle id into a DefiLlama protocol slug.
///
/// DefiLlama slugs are lowercase and hyphenated (`"aave"`, `"uniswap"`,
/// `"lido"`), which is the same shape as a CoinGecko coin id, so most ids pass
/// through unchanged.
pub(super) fn slug(id: &str) -> String {
    id.trim().to_lowercase().replace([' ', '_'], "-")
}

/// Split `currentChainTvls` into real per-chain allocations.
///
/// DefiLlama mixes genuine chain keys (`"Ethereum"`) with breakdown keys of
/// the *same* capital (`"Ethereum-borrowed"`, `"pool2"`, `"staking"`).
/// Summing everything would double-count, so only keys naming a chain the
/// protocol actually reports are kept.
pub(super) fn chain_allocations(response: &ProtocolResponse) -> Vec<ChainAllocation> {
    let mut out: Vec<ChainAllocation> = response
        .current_chain_tvls
        .iter()
        .filter(|(key, _)| response.chains.iter().any(|chain| chain == *key))
        .map(|(chain, tvl)| ChainAllocation {
            chain: chain.clone(),
            tvl: *tvl,
        })
        .collect();
    // Largest first, and by name on ties so the order is deterministic.
    out.sort_by(|a, b| b.tvl.total_cmp(&a.tvl).then_with(|| a.chain.cmp(&b.chain)));
    out
}

/// Percentage change between the latest TVL and the value `days_ago` before
/// it, using the closest snapshot at or before that instant.
pub(super) fn change_percent(history: &[TvlPoint], days_ago: i64) -> Option<f64> {
    let latest = history.last()?;
    let cutoff = latest.timestamp - days_ago * DAY;
    let past = history
        .iter()
        .rev()
        .find(|point| point.timestamp <= cutoff)?;
    if past.tvl == 0.0 {
        return None;
    }
    Some((latest.tvl - past.tvl) / past.tvl * 100.0)
}

/// Convert the raw TVL history into public points, oldest first.
pub(super) fn to_history(response: &ProtocolResponse) -> Vec<TvlPoint> {
    let mut points: Vec<TvlPoint> = response
        .tvl
        .iter()
        .map(|snapshot| TvlPoint {
            timestamp: snapshot.date,
            tvl: snapshot.total_liquidity_usd,
        })
        .collect();
    // DefiLlama returns these chronologically, but the changes computed above
    // depend on it, so the order is enforced rather than assumed.
    points.sort_by_key(|point| point.timestamp);
    points
}

/// Build the public [`ProtocolTvl`] summary.
pub(super) fn to_protocol_tvl(slug: &str, response: &ProtocolResponse) -> ProtocolTvl {
    let history = to_history(response);
    ProtocolTvl {
        slug: slug.to_string(),
        name: response.name.clone(),
        symbol: response.symbol.clone(),
        url: response.url.clone(),
        chains: response.chains.clone(),
        tvl: history.last().map(|point| point.tvl),
        tvl_by_chain: chain_allocations(response),
        change_1d_percent: change_percent(&history, 1),
        change_7d_percent: change_percent(&history, 7),
        market_cap: response.mcap,
    }
}

/// Fetch a protocol's current TVL summary.
pub(crate) async fn fetch_protocol_tvl_response(protocol: &str) -> Result<ProtocolTvl> {
    let slug = slug(protocol);
    let response = super::client()?.protocol(&slug).await?;
    Ok(to_protocol_tvl(&slug, &response))
}

/// Fetch a protocol's full TVL history, oldest first.
pub(crate) async fn fetch_protocol_tvl_history_response(protocol: &str) -> Result<Vec<TvlPoint>> {
    let response = super::client()?.protocol(&slug(protocol)).await?;
    Ok(to_history(&response))
}
