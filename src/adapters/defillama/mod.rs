//! DefiLlama — keyless DeFi TVL, chain rankings, and stablecoin supplies.
//!
//! Requires the **`defi`** feature flag.
//!
//! The library covered CeFi crypto (exchange quotes, aggregated market caps)
//! but nothing on-chain. DefiLlama is the FOSS standard here: keyless, no
//! registration, stable endpoints.
//!
//! Two surfaces, split by what the data is *about*:
//! - Protocol-shaped data routes through `Capability::CRYPTO` and appears on
//!   [`CryptoCoin::tvl`](crate::CryptoCoin::tvl) and
//!   [`tvl_history`](crate::CryptoCoin::tvl_history).
//! - Market-wide views have no symbol to hang off, so they are crate-level
//!   functions in [`crate::defi`].

pub(crate) mod client;
pub(crate) mod crypto;
pub(crate) mod models;

use std::time::Duration;

use crate::adapters::singleton::keyless_limiter;
use crate::error::Result;
use crate::models::crypto::defi::{ChainTvl, StablecoinSupply};
use client::DefiLlamaClient;

/// Self-imposed pacing. DefiLlama publishes no quota for the free API but
/// throttles abusive clients.
const DEFILLAMA_RATE_PER_SEC: f64 = 5.0;

/// The stablecoins and protocol endpoints return large payloads; give them
/// room beyond the usual timeout.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(45);

keyless_limiter!(rate = DEFILLAMA_RATE_PER_SEC);

/// Build a client against the live API, reusing the shared token bucket.
fn client() -> Result<DefiLlamaClient> {
    DefiLlamaClient::new(
        DEFAULT_TIMEOUT,
        shared_limiter(),
        client::LLAMA_BASE,
        client::STABLECOINS_BASE,
    )
}

pub(crate) use crypto::{fetch_protocol_tvl_history_response, fetch_protocol_tvl_response};

/// Read the single value out of one of DefiLlama's peg-type-keyed maps
/// (`{"peggedUSD": 183195719854.9}`).
fn peg_value(map: &std::collections::HashMap<String, f64>) -> Option<f64> {
    map.values().copied().next()
}

/// Fetch aggregate TVL for every chain, largest first.
pub(crate) async fn chains() -> Result<Vec<ChainTvl>> {
    let mut chains: Vec<ChainTvl> = client()?
        .chains()
        .await?
        .into_iter()
        .map(|chain| ChainTvl {
            name: chain.name,
            token_symbol: chain.token_symbol,
            gecko_id: chain.gecko_id,
            chain_id: chain.chain_id,
            tvl: chain.tvl,
        })
        .collect();
    // DefiLlama returns these unordered; rank them so the list is useful
    // as-is. `None` sorts last.
    chains.sort_by(|a, b| {
        b.tvl
            .unwrap_or(f64::MIN)
            .total_cmp(&a.tvl.unwrap_or(f64::MIN))
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(chains)
}

/// Fetch circulating supply for every tracked stablecoin, largest first.
pub(crate) async fn stablecoins() -> Result<Vec<StablecoinSupply>> {
    let mut coins: Vec<StablecoinSupply> = client()?
        .stablecoins()
        .await?
        .pegged_assets
        .into_iter()
        .map(|asset| StablecoinSupply {
            name: asset.name,
            symbol: asset.symbol,
            gecko_id: asset.gecko_id,
            peg_type: asset.peg_type,
            peg_mechanism: asset.peg_mechanism,
            circulating: peg_value(&asset.circulating),
            circulating_prev_day: peg_value(&asset.circulating_prev_day),
            circulating_prev_week: peg_value(&asset.circulating_prev_week),
            circulating_prev_month: peg_value(&asset.circulating_prev_month),
        })
        .collect();
    coins.sort_by(|a, b| {
        b.circulating
            .unwrap_or(f64::MIN)
            .total_cmp(&a.circulating.unwrap_or(f64::MIN))
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(coins)
}

#[cfg(test)]
mod tests {
    use super::crypto::{chain_allocations, change_percent, slug, to_history, to_protocol_tvl};
    use super::models::LiteResponse;
    use super::*;
    use crate::error::FinanceError;
    use crate::rate_limiter::RateLimiter;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn test_client(base_url: &str) -> DefiLlamaClient {
        DefiLlamaClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
            base_url,
        )
        .unwrap()
    }

    /// Shape from api.llama.fi/protocol/{slug}, trimmed to a few snapshots.
    fn protocol_payload() -> String {
        serde_json::json!({
            "id": "111",
            "name": "AAVE V3",
            "symbol": "AAVE",
            "url": "https://aave.com",
            "chains": ["Ethereum", "Arbitrum"],
            "currentChainTvls": {
                "Ethereum": 9_000_000_000.0_f64,
                "Arbitrum": 1_000_000_000.0_f64,
                // Breakdown keys of the same capital — must not be counted.
                "Ethereum-borrowed": 5_000_000_000.0_f64,
                "pool2": 12_345.0_f64
            },
            "tvl": [
                { "date": 1785110400_i64, "totalLiquidityUSD": 8_000_000_000.0_f64 },
                { "date": 1785628800_i64, "totalLiquidityUSD": 9_500_000_000.0_f64 },
                { "date": 1785715200_i64, "totalLiquidityUSD": 10_000_000_000.0_f64 }
            ],
            "mcap": 3_500_000_000.0_f64
        })
        .to_string()
    }

    /// Shape from api.llama.fi/lite/protocols2, trimmed to a parent with two
    /// children (one excluded from the parent's TVL) and a standalone protocol.
    fn lite_payload() -> String {
        serde_json::json!({
            "chains": ["Ethereum"],
            "protocolCategories": ["Lending"],
            "parentProtocols": [{
                "id": "parent#aave",
                "name": "Aave",
                "url": "https://aave.com",
                "symbol": "AAVE",
                "chains": ["Ethereum", "Arbitrum"],
                "mcap": 3_500_000_000.0_f64,
                "gecko_id": "aave"
            }],
            "protocols": [
                {
                    "name": "Aave V3",
                    "symbol": "AAVE",
                    "chains": ["Ethereum", "Arbitrum"],
                    "chainTvls": {
                        "Ethereum": { "tvl": 9.0e9, "tvlPrevDay": 8.0e9 },
                        "Arbitrum": { "tvl": 1.0e9 },
                        "Ethereum-borrowed": { "tvl": 5.0e9 }
                    },
                    "tvl": 10.0e9,
                    "tvlPrevDay": 9.0e9,
                    "tvlPrevWeek": 8.0e9,
                    "parentProtocol": "parent#aave"
                },
                {
                    "name": "Aave V2",
                    "chains": ["Ethereum"],
                    "chainTvls": { "Ethereum": { "tvl": 1.0e9 } },
                    "tvl": 1.0e9,
                    "tvlPrevDay": 1.0e9,
                    "tvlPrevWeek": 1.0e9,
                    "parentProtocol": "parent#aave",
                    "excludeTvlFromParent": true
                },
                {
                    "name": "Curve DEX",
                    "symbol": "CRV",
                    "url": "https://curve.fi",
                    "chains": ["Ethereum"],
                    "chainTvls": { "Ethereum": { "tvl": 2.0e9 } },
                    "tvl": 2.0e9,
                    "tvlPrevDay": 2.5e9,
                    "tvlPrevWeek": 2.0e9,
                    "mcap": 1.0e9,
                    "deprecated": false
                }
            ]
        })
        .to_string()
    }

    #[test]
    fn lite_list_deserialises_parents_children_and_standalones() {
        let list: LiteResponse = serde_json::from_str(&lite_payload()).unwrap();

        assert_eq!(list.parent_protocols.len(), 1);
        assert_eq!(list.parent_protocols[0].id, "parent#aave");
        assert_eq!(list.parent_protocols[0].mcap, Some(3_500_000_000.0));

        let v3 = &list.protocols[0];
        assert_eq!(v3.parent_protocol.as_deref(), Some("parent#aave"));
        assert!(!v3.exclude_tvl_from_parent);
        assert_eq!(v3.chain_tvls["Ethereum"].tvl, 9.0e9);
        assert_eq!(v3.tvl_prev_week, Some(8.0e9));
        assert!(list.protocols[1].exclude_tvl_from_parent);

        let curve = &list.protocols[2];
        assert_eq!(curve.parent_protocol, None);
        assert_eq!(curve.mcap, Some(1.0e9));
    }

    #[tokio::test]
    async fn lite_protocols_reads_the_shared_list() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/lite/protocols2")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(lite_payload())
            .create_async()
            .await;

        let list = test_client(&server.url()).lite_protocols().await.unwrap();
        assert_eq!(list.protocols.len(), 3);
    }

    #[test]
    fn ids_normalise_to_defillama_slugs() {
        assert_eq!(slug("aave"), "aave");
        assert_eq!(slug("AAVE"), "aave");
        assert_eq!(slug("Curve DEX"), "curve-dex");
        assert_eq!(slug(" lido_finance "), "lido-finance");
    }

    #[test]
    fn breakdown_keys_are_excluded_from_chain_allocations() {
        let chains = vec!["Ethereum".to_string(), "Arbitrum".to_string()];
        let tvls = HashMap::from([
            ("Ethereum".to_string(), 9_000_000_000.0),
            ("Arbitrum".to_string(), 1_000_000_000.0),
            ("Ethereum-borrowed".to_string(), 5_000_000_000.0),
            ("pool2".to_string(), 12_345.0),
        ]);
        let allocations = chain_allocations(&chains, &tvls);

        // Only the two keys that name a chain the protocol reports.
        assert_eq!(allocations.len(), 2);
        // Largest first.
        assert_eq!(allocations[0].chain, "Ethereum");
        assert_eq!(allocations[0].tvl, 9_000_000_000.0);
        assert_eq!(allocations[1].chain, "Arbitrum");
    }

    #[test]
    fn an_empty_chains_list_falls_back_to_the_non_breakdown_keys() {
        let payload = serde_json::json!({
            "protocols": [{
                "name": "Aave",
                "chains": [],
                "chainTvls": {
                    "Ethereum": { "tvl": 9.0e9 },
                    "Ethereum-borrowed": { "tvl": 5.0e9 },
                    "Ethereum-staking": { "tvl": 1.0e6 },
                    "OP Mainnet": { "tvl": 2.0e9 },
                    "borrowed": { "tvl": 7.0e9 },
                    "pool2": { "tvl": 12_345.0_f64 },
                    "staking": { "tvl": 999.0_f64 }
                }
            }]
        })
        .to_string();
        let list: LiteResponse = serde_json::from_str(&payload).unwrap();
        let summary = to_protocol_tvl("aave", &list).unwrap();

        assert_eq!(summary.chains, vec!["Ethereum", "OP Mainnet"]);
        assert_eq!(summary.tvl_by_chain.len(), 2);
    }

    #[test]
    fn a_parent_sums_its_included_children() {
        let list: LiteResponse = serde_json::from_str(&lite_payload()).unwrap();
        let summary = to_protocol_tvl("aave", &list).unwrap();

        assert_eq!(summary.slug, "aave");
        assert_eq!(summary.name.as_deref(), Some("Aave"));
        assert_eq!(summary.url.as_deref(), Some("https://aave.com"));
        assert_eq!(summary.chains, vec!["Ethereum", "Arbitrum"]);
        assert_eq!(summary.market_cap, Some(3_500_000_000.0));
        // V2 is excluded from the parent, so only V3 counts.
        assert_eq!(summary.tvl, Some(10.0e9));
        assert_eq!(summary.tvl_by_chain.len(), 2);
        assert_eq!(summary.tvl_by_chain[0].chain, "Ethereum");
        assert_eq!(summary.tvl_by_chain[0].tvl, 9.0e9);
        assert_eq!(summary.tvl_by_chain[1].tvl, 1.0e9);
    }

    #[test]
    fn a_parent_chain_total_adds_across_children() {
        let mut list: LiteResponse = serde_json::from_str(&lite_payload()).unwrap();
        list.protocols[1].exclude_tvl_from_parent = false;
        let summary = to_protocol_tvl("aave", &list).unwrap();

        assert_eq!(summary.tvl, Some(11.0e9));
        assert_eq!(summary.tvl_by_chain[0].tvl, 10.0e9);
    }

    #[test]
    fn a_standalone_protocol_wins_over_a_parent_of_the_same_slug() {
        let mut list: LiteResponse = serde_json::from_str(&lite_payload()).unwrap();
        list.parent_protocols[0].id = "parent#curve-dex".to_string();
        list.protocols[0].parent_protocol = Some("parent#curve-dex".to_string());
        let summary = to_protocol_tvl("curve-dex", &list).unwrap();

        assert_eq!(summary.name.as_deref(), Some("Curve DEX"));
        assert_eq!(summary.tvl, Some(2.0e9));
    }

    #[test]
    fn a_parent_change_ignores_children_missing_the_prior_value() {
        let mut list: LiteResponse = serde_json::from_str(&lite_payload()).unwrap();
        list.protocols[1].exclude_tvl_from_parent = false;
        list.protocols[1].tvl_prev_day = None;
        let summary = to_protocol_tvl("aave", &list).unwrap();

        assert_eq!(summary.tvl, Some(11.0e9));
        // V3 alone: 9 -> 10.
        assert!((summary.change_1d_percent.unwrap() - (10.0 - 9.0) / 9.0 * 100.0).abs() < 1e-9);
        // Both children report a prior week: 9 -> 11.
        assert!((summary.change_7d_percent.unwrap() - (11.0 - 9.0) / 9.0 * 100.0).abs() < 1e-9);
    }

    #[test]
    fn a_parent_with_no_matched_child_has_no_change() {
        let mut list: LiteResponse = serde_json::from_str(&lite_payload()).unwrap();
        list.protocols[0].tvl_prev_day = None;
        let summary = to_protocol_tvl("aave", &list).unwrap();

        assert_eq!(summary.change_1d_percent, None);
    }

    #[test]
    fn a_child_resolves_by_its_derived_slug() {
        let list: LiteResponse = serde_json::from_str(&lite_payload()).unwrap();
        let summary = to_protocol_tvl("aave-v3", &list).unwrap();

        assert_eq!(summary.name.as_deref(), Some("Aave V3"));
        assert_eq!(summary.tvl, Some(10.0e9));
    }

    #[test]
    fn a_standalone_protocol_resolves_by_its_derived_slug() {
        let list: LiteResponse = serde_json::from_str(&lite_payload()).unwrap();
        let summary = to_protocol_tvl("curve-dex", &list).unwrap();

        assert_eq!(summary.name.as_deref(), Some("Curve DEX"));
        assert_eq!(summary.chains, vec!["Ethereum"]);
        assert_eq!(summary.tvl, Some(2.0e9));
        assert_eq!(summary.market_cap, Some(1.0e9));
    }

    #[test]
    fn an_unknown_slug_maps_to_symbol_not_found() {
        let list: LiteResponse = serde_json::from_str(&lite_payload()).unwrap();
        let err = to_protocol_tvl("not-a-protocol", &list).unwrap_err();

        assert!(
            matches!(err, FinanceError::SymbolNotFound { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn changes_compare_against_the_prior_day_and_week() {
        let list: LiteResponse = serde_json::from_str(&lite_payload()).unwrap();

        let aave = to_protocol_tvl("aave", &list).unwrap();
        assert!((aave.change_1d_percent.unwrap() - (10.0 - 9.0) / 9.0 * 100.0).abs() < 1e-9);
        assert!((aave.change_7d_percent.unwrap() - 25.0).abs() < 1e-9);

        let curve = to_protocol_tvl("curve-dex", &list).unwrap();
        assert!((curve.change_1d_percent.unwrap() - -20.0).abs() < 1e-9);
        assert_eq!(curve.change_7d_percent, Some(0.0));
    }

    #[test]
    fn a_missing_or_zero_base_has_no_change() {
        assert_eq!(change_percent(Some(10.0), None), None);
        assert_eq!(change_percent(Some(10.0), Some(0.0)), None);
        assert_eq!(change_percent(None, Some(10.0)), None);
    }

    #[tokio::test]
    async fn protocol_maps_to_a_history_oldest_first() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/protocol/aave")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(protocol_payload())
            .create_async()
            .await;

        let response = test_client(&server.url()).protocol("aave").await.unwrap();
        let history = to_history(&response);
        assert_eq!(history.len(), 3);
        assert!(history.windows(2).all(|w| w[0].timestamp < w[1].timestamp));
    }

    #[tokio::test]
    async fn unknown_protocol_maps_to_symbol_not_found() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/protocol/not-a-protocol")
            .with_status(400)
            .with_body("Protocol not found")
            .create_async()
            .await;

        let err = test_client(&server.url())
            .protocol("not-a-protocol")
            .await
            .unwrap_err();
        assert!(
            matches!(err, FinanceError::SymbolNotFound { .. }),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn chains_come_back_ranked_by_tvl() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/v2/chains")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!([
                    { "name": "Harmony", "tokenSymbol": "ONE", "gecko_id": "harmony",
                      "chainId": 1666600000_i64, "tvl": 204577.23_f64 },
                    { "name": "Ethereum", "tokenSymbol": "ETH", "gecko_id": "ethereum",
                      "chainId": 1_i64, "tvl": 60_000_000_000.0_f64 },
                    { "name": "Corn", "tokenSymbol": "CORN", "gecko_id": "corn-3",
                      "chainId": 21000000_i64, "tvl": 0.0_f64 }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let raw = test_client(&server.url()).chains().await.unwrap();
        let mut ranked: Vec<_> = raw
            .into_iter()
            .map(|c| (c.name, c.tvl.unwrap_or(0.0)))
            .collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
        assert_eq!(ranked[0].0, "Ethereum");
    }

    #[tokio::test]
    async fn stablecoin_supplies_read_the_peg_keyed_maps() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/stablecoins")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "peggedAssets": [{
                        "id": "1", "name": "Tether", "symbol": "USDT",
                        "gecko_id": "tether", "pegType": "peggedUSD",
                        "pegMechanism": "fiat-backed",
                        // The value key varies with pegType, so it is read as
                        // a map rather than a fixed field.
                        "circulating": { "peggedUSD": 183_195_719_854.93_f64 },
                        "circulatingPrevDay": { "peggedUSD": 183_301_799_410.99_f64 },
                        "circulatingPrevWeek": { "peggedUSD": 184_242_294_121.55_f64 },
                        "circulatingPrevMonth": { "peggedUSD": 184_050_168_657.80_f64 }
                    }]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let response = test_client(&server.url()).stablecoins().await.unwrap();
        let asset = &response.pegged_assets[0];
        assert_eq!(asset.symbol.as_deref(), Some("USDT"));
        assert_eq!(peg_value(&asset.circulating), Some(183_195_719_854.93));
        assert_eq!(
            peg_value(&asset.circulating_prev_day),
            Some(183_301_799_410.99)
        );
    }
}
