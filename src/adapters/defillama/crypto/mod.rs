//! `CRYPTO` capability for DefiLlama — protocol TVL.

use std::collections::HashMap;

use crate::error::{FinanceError, Result};
use crate::models::crypto::defi::{ChainAllocation, ProtocolTvl, TvlPoint};

use super::models::{LiteProtocol, LiteResponse, ProtocolResponse};

/// Normalise a handle id into a DefiLlama protocol slug.
///
/// DefiLlama slugs are lowercase and hyphenated (`"aave"`, `"uniswap"`,
/// `"lido"`), which is the same shape as a CoinGecko coin id, so most ids pass
/// through unchanged.
pub(super) fn slug(id: &str) -> String {
    id.trim().to_lowercase().replace([' ', '_'], "-")
}

/// Split `chainTvls` into real per-chain allocations.
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

pub(super) fn chain_allocations(
    chains: &[String],
    chain_tvls: &HashMap<String, f64>,
) -> Vec<ChainAllocation> {
    // `chains` is the authoritative allow-list when present. Some protocols
    // now come back with it empty while still reporting per-chain TVL, so fall
    // back to every key that is not a breakdown.
    let named = !chains.is_empty();
    let mut out: Vec<ChainAllocation> = chain_tvls
        .iter()
        .filter(|(key, _)| match named {
            true => chains.iter().any(|chain| chain == *key),
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

/// Percentage change from `previous` to `current`, `None` when either is
/// missing or the base is zero.
pub(super) fn change_percent(current: Option<f64>, previous: Option<f64>) -> Option<f64> {
    let (current, previous) = (current?, previous?);
    if previous == 0.0 {
        return None;
    }
    Some((current - previous) / previous * 100.0)
}

/// The slug DefiLlama's own `/protocols` listing gives a lite entry, which
/// carries only a name.
fn derived_slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-").replace('\'', "")
}

/// Sum of the values present, `None` when none are.
fn sum_present(values: impl Iterator<Item = Option<f64>>) -> Option<f64> {
    values
        .flatten()
        .fold(None, |acc, v| Some(acc.unwrap_or(0.0) + v))
}

/// Current and prior-period figures for one slug, whether it names a single
/// protocol or a parent grouping.
struct Resolved {
    name: Option<String>,
    symbol: Option<String>,
    url: Option<String>,
    chains: Vec<String>,
    tvl: Option<f64>,
    tvl_prev_day: Option<f64>,
    tvl_prev_week: Option<f64>,
    chain_tvls: HashMap<String, f64>,
    mcap: Option<f64>,
}

fn resolve(list: &LiteResponse, slug: &str) -> Option<Resolved> {
    if let Some(parent) = list
        .parent_protocols
        .iter()
        .find(|parent| parent.id.strip_prefix("parent#") == Some(slug))
    {
        let children: Vec<&LiteProtocol> = list
            .protocols
            .iter()
            .filter(|p| p.parent_protocol.as_deref() == Some(parent.id.as_str()))
            .filter(|p| !p.exclude_tvl_from_parent && !p.deprecated)
            .collect();
        let mut chain_tvls: HashMap<String, f64> = HashMap::new();
        for (key, entry) in children.iter().flat_map(|p| &p.chain_tvls) {
            *chain_tvls.entry(key.clone()).or_default() += entry.tvl;
        }
        return Some(Resolved {
            name: parent.name.clone(),
            symbol: parent.symbol.clone(),
            url: parent.url.clone(),
            chains: parent.chains.clone(),
            tvl: sum_present(children.iter().map(|p| p.tvl)),
            tvl_prev_day: sum_present(children.iter().map(|p| p.tvl_prev_day)),
            tvl_prev_week: sum_present(children.iter().map(|p| p.tvl_prev_week)),
            chain_tvls,
            mcap: parent.mcap,
        });
    }
    let protocol = list
        .protocols
        .iter()
        .find(|p| derived_slug(&p.name) == slug)?;
    Some(Resolved {
        name: Some(protocol.name.clone()),
        symbol: protocol.symbol.clone(),
        url: protocol.url.clone(),
        chains: protocol.chains.clone(),
        tvl: protocol.tvl,
        tvl_prev_day: protocol.tvl_prev_day,
        tvl_prev_week: protocol.tvl_prev_week,
        chain_tvls: protocol
            .chain_tvls
            .iter()
            .map(|(key, entry)| (key.clone(), entry.tvl))
            .collect(),
        mcap: protocol.mcap,
    })
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

/// Build the public [`ProtocolTvl`] summary for `slug` from the shared list.
pub(super) fn to_protocol_tvl(slug: &str, list: &LiteResponse) -> Result<ProtocolTvl> {
    let resolved = resolve(list, slug).ok_or_else(|| FinanceError::SymbolNotFound {
        symbol: Some(slug.to_string()),
        context: "DefiLlama knows no protocol by this slug".to_string(),
    })?;
    let tvl_by_chain = chain_allocations(&resolved.chains, &resolved.chain_tvls);
    // DefiLlama has started returning an empty `chains` for protocols whose
    // `chainTvls` still names them, so fall back to the chains it allocates to.
    let chains = match resolved.chains.is_empty() {
        true => tvl_by_chain.iter().map(|a| a.chain.clone()).collect(),
        false => resolved.chains,
    };
    Ok(ProtocolTvl {
        slug: slug.to_string(),
        name: resolved.name,
        symbol: resolved.symbol,
        url: resolved.url,
        chains,
        tvl: resolved.tvl,
        tvl_by_chain,
        change_1d_percent: change_percent(resolved.tvl, resolved.tvl_prev_day),
        change_7d_percent: change_percent(resolved.tvl, resolved.tvl_prev_week),
        market_cap: resolved.mcap,
    })
}

/// Fetch a protocol's current TVL summary.
pub(crate) async fn fetch_protocol_tvl_response(protocol: &str) -> Result<ProtocolTvl> {
    let slug = slug(protocol);
    let list = super::client()?.lite_protocols().await?;
    to_protocol_tvl(&slug, &list)
}

/// Fetch a protocol's full TVL history, oldest first.
pub(crate) async fn fetch_protocol_tvl_history_response(protocol: &str) -> Result<Vec<TvlPoint>> {
    let response = super::client()?.protocol(&slug(protocol)).await?;
    Ok(to_history(&response))
}
