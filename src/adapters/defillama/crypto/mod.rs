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
/// Capital breakdowns DefiLlama reports beside chains in the same map, either
/// bare or suffixed onto a chain name.
const BREAKDOWN_KEYS: [&str; 5] = ["borrowed", "pool2", "staking", "treasury", "vesting"];

fn is_breakdown_key(key: &str) -> bool {
    BREAKDOWN_KEYS.contains(&key)
        || key
            .rsplit_once('-')
            .is_some_and(|(_, suffix)| BREAKDOWN_KEYS.contains(&suffix))
}

pub(super) fn chain_allocations(response: &ProtocolResponse) -> Vec<ChainAllocation> {
    // `chains` is the authoritative allow-list when present. Some protocols
    // now come back with it empty while still reporting per-chain TVL, so fall
    // back to every key that is not a breakdown.
    let named = !response.chains.is_empty();
    let mut out: Vec<ChainAllocation> = response
        .current_chain_tvls
        .iter()
        .filter(|(key, _)| match named {
            true => response.chains.iter().any(|chain| chain == *key),
            false => !is_breakdown_key(key),
        })
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
    let tvl_by_chain = chain_allocations(response);
    // DefiLlama has started returning an empty `chains` for protocols whose
    // `chainTvls` still names them, so fall back to the chains it allocates to.
    let chains = match response.chains.is_empty() {
        true => tvl_by_chain.iter().map(|a| a.chain.clone()).collect(),
        false => response.chains.clone(),
    };
    ProtocolTvl {
        slug: slug.to_string(),
        name: response.name.clone(),
        symbol: response.symbol.clone(),
        url: response.url.clone(),
        chains,
        tvl: history.last().map(|point| point.tvl),
        tvl_by_chain,
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
