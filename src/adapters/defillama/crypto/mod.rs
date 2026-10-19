//! `CRYPTO` capability for DefiLlama — protocol TVL.

use std::collections::HashMap;

use crate::error::{FinanceError, Result};
use crate::models::crypto::defi::{ChainAllocation, ProtocolTvl, TvlPoint};

use super::models::{LiteProtocol, LiteResponse, Period, ProtocolResponse};

/// Normalise a handle id or a protocol name into a DefiLlama protocol slug.
///
/// DefiLlama slugs are lowercase and hyphenated (`"aave"`, `"uniswap"`,
/// `"lido"`), which is the same shape as a CoinGecko coin id, so most ids pass
/// through unchanged. The lite list carries only names, so the same rule
/// derives their slugs.
pub(super) fn slug(id: &str) -> String {
    id.trim()
        .to_lowercase()
        .replace([' ', '_'], "-")
        .replace('\'', "")
}

/// The `chainTvls` key, bare or chain-suffixed, that holds the share of a child's
/// TVL its parent does not count.
const EXCLUDE_PARENT: &str = "excludeParent";

/// Capital breakdowns DefiLlama reports beside chains in the same map, either
/// bare or suffixed onto a chain name.
const BREAKDOWN_KEYS: [&str; 10] = [
    "borrowed",
    "dcAndLsOverlap",
    "doublecounted",
    EXCLUDE_PARENT,
    "liquidstaking",
    "offers",
    "pool2",
    "staking",
    "treasury",
    "vesting",
];

fn is_breakdown_key(key: &str) -> bool {
    BREAKDOWN_KEYS.contains(&key)
        || key
            .rsplit_once('-')
            .is_some_and(|(_, suffix)| BREAKDOWN_KEYS.contains(&suffix))
}

/// Split `chainTvls` into real per-chain allocations.
///
/// DefiLlama mixes genuine chain keys (`"Ethereum"`) with breakdown keys of
/// the *same* capital (`"Ethereum-borrowed"`, `"pool2"`, `"staking"`).
/// Summing everything would double-count, so only keys naming a chain the
/// protocol actually reports are kept.
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

/// Sum of the values present, `None` when none are.
fn sum_present(values: impl Iterator<Item = Option<f64>>) -> Option<f64> {
    values
        .flatten()
        .fold(None, |acc, v| Some(acc.unwrap_or(0.0) + v))
}

/// What a slug names: its own metadata plus the protocols whose TVL it sums.
/// A standalone protocol is a group of one.
struct Resolved<'a> {
    name: Option<&'a str>,
    symbol: Option<&'a str>,
    url: Option<&'a str>,
    chains: &'a [String],
    mcap: Option<f64>,
    members: Vec<&'a LiteProtocol>,
    in_parent: bool,
}

impl Resolved<'_> {
    /// The share of `member` under `key` that a parent does not count.
    fn excluded_share(&self, member: &LiteProtocol, key: &str, period: Period) -> f64 {
        match self.in_parent {
            true => member
                .chain_tvls
                .get(key)
                .and_then(|entry| entry.tvl_at(period))
                .unwrap_or(0.0),
            false => 0.0,
        }
    }

    fn member_tvl(&self, member: &LiteProtocol, period: Period) -> Option<f64> {
        Some(member.tvl_at(period)? - self.excluded_share(member, EXCLUDE_PARENT, period))
    }

    fn tvl(&self) -> Option<f64> {
        sum_present(self.members.iter().map(|m| self.member_tvl(m, Period::Now)))
    }

    /// Change in TVL over one period, across only the members that report both
    /// figures. A member with no prior value would otherwise read as growth.
    fn change(&self, period: Period) -> Option<f64> {
        let (current, before) = self
            .members
            .iter()
            .filter_map(|m| {
                Some((
                    self.member_tvl(m, Period::Now)?,
                    self.member_tvl(m, period)?,
                ))
            })
            .fold(None, |acc, (now, then)| {
                let (current, before) = acc.unwrap_or((0.0, 0.0));
                Some((current + now, before + then))
            })?;
        change_percent(Some(current), Some(before))
    }

    fn chain_tvls(&self) -> HashMap<String, f64> {
        let mut totals: HashMap<String, f64> = HashMap::new();
        for member in &self.members {
            for (key, entry) in &member.chain_tvls {
                if is_breakdown_key(key) {
                    continue;
                }
                let excluded =
                    self.excluded_share(member, &format!("{key}-{EXCLUDE_PARENT}"), Period::Now);
                *totals.entry(key.clone()).or_default() += entry.tvl - excluded;
            }
        }
        totals
    }
}

/// A protocol's own name wins over a parent of the same slug, as it does on
/// DefiLlama's `/protocol/{slug}`.
fn resolve<'a>(list: &'a LiteResponse, id: &str) -> Option<Resolved<'a>> {
    if let Some(protocol) = list.protocols.iter().find(|p| slug(&p.name) == id) {
        return Some(Resolved {
            name: Some(&protocol.name),
            symbol: protocol.symbol.as_deref(),
            url: protocol.url.as_deref(),
            chains: &protocol.chains,
            mcap: protocol.mcap,
            members: vec![protocol],
            in_parent: false,
        });
    }
    let parent = list.parent_protocols.iter().find(|parent| {
        parent
            .id
            .strip_prefix("parent#")
            .is_some_and(|suffix| slug(suffix) == id)
    })?;
    Some(Resolved {
        name: parent.name.as_deref(),
        symbol: parent.symbol.as_deref(),
        url: parent.url.as_deref(),
        chains: &parent.chains,
        mcap: parent.mcap,
        members: list
            .protocols
            .iter()
            .filter(|p| p.parent_protocol.as_deref() == Some(parent.id.as_str()))
            .filter(|p| !p.exclude_tvl_from_parent && !p.deprecated)
            .collect(),
        in_parent: true,
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
    // DefiLlama returns these oldest first, but callers depend on that, so sort
    // anyway.
    points.sort_by_key(|point| point.timestamp);
    points
}

/// Build the public [`ProtocolTvl`] summary for `slug` from the shared list.
pub(super) fn to_protocol_tvl(slug: &str, list: &LiteResponse) -> Result<ProtocolTvl> {
    let resolved = resolve(list, slug).ok_or_else(|| FinanceError::SymbolNotFound {
        symbol: Some(slug.to_string()),
        context: "DefiLlama knows no protocol by this slug".to_string(),
    })?;
    let tvl_by_chain = chain_allocations(resolved.chains, &resolved.chain_tvls());
    let chains = match resolved.chains.is_empty() {
        true => tvl_by_chain.iter().map(|a| a.chain.clone()).collect(),
        false => resolved.chains.to_vec(),
    };
    Ok(ProtocolTvl {
        slug: slug.to_string(),
        name: resolved.name.map(str::to_string),
        symbol: resolved.symbol.map(str::to_string),
        url: resolved.url.map(str::to_string),
        chains,
        tvl: resolved.tvl(),
        tvl_by_chain,
        change_1d_percent: resolved.change(Period::Day),
        change_7d_percent: resolved.change(Period::Week),
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
