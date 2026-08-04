//! Compile and runtime tests for docs/library/providers/defillama.md
//!
//! Requires the `defi` feature flag:
//!   cargo test --test doc_defillama --features defi
//!   cargo test --test doc_defillama --features defi -- --ignored
//!
//! Offline behaviour (slug normalisation, breakdown-key exclusion, change
//! computation, peg-keyed supply maps, error mapping) is covered by the mock +
//! unit tests in `src/adapters/defillama/mod.rs`.

#![cfg(feature = "defi")]

use finance_query::defi::{ChainTvl, ProtocolTvl, StablecoinSupply, TvlPoint};
use finance_query::{Capability, Provider, Providers};

/// Verifies the fields documented on the page exist with the stated types.
#[allow(dead_code)]
fn _verify_model_fields(p: ProtocolTvl, c: ChainTvl, s: StablecoinSupply, t: TvlPoint) {
    let _: String = p.slug;
    let _: Option<f64> = p.tvl;
    let _: Vec<String> = p.chains;
    let _: Option<f64> = p.change_1d_percent;
    let _: Option<f64> = p.change_7d_percent;
    let _: Option<f64> = p.market_cap;

    let _: String = c.name;
    let _: Option<f64> = c.tvl;
    let _: Option<i64> = c.chain_id;

    let _: String = s.name;
    let _: Option<String> = s.peg_type;
    let _: Option<f64> = s.circulating;

    let _: i64 = t.timestamp;
    let _: f64 = t.tvl;
}

#[test]
fn defillama_provider_id_round_trips() {
    assert_eq!(Provider::DefiLlama.as_str(), "defillama");
    assert_eq!(
        Provider::from_id_str("defillama"),
        Some(Provider::DefiLlama)
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn protocol_tvl_is_positive_and_chain_split_does_not_double_count() {
    let providers = Providers::builder()
        .route(Capability::CRYPTO, [Provider::DefiLlama])
        .build()
        .await
        .expect("DefiLlama needs no API key");

    let tvl = providers.crypto("aave").tvl().await.unwrap();

    assert_eq!(tvl.slug, "aave");
    assert!(tvl.tvl.is_some_and(|v| v > 0.0));
    assert!(!tvl.chains.is_empty());
    // Every allocation must name a chain the protocol reports — a leaked
    // "-borrowed"/"pool2" breakdown key would fail this.
    assert!(
        tvl.tvl_by_chain
            .iter()
            .all(|a| tvl.chains.contains(&a.chain)),
        "an allocation named something that is not one of the protocol's chains"
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn tvl_history_is_chronological() {
    let providers = Providers::builder()
        .route(Capability::CRYPTO, [Provider::DefiLlama])
        .build()
        .await
        .unwrap();

    let history = providers.crypto("aave").tvl_history().await.unwrap();
    assert!(history.len() > 100);
    assert!(
        history.windows(2).all(|w| w[0].timestamp <= w[1].timestamp),
        "history is not chronological"
    );
}

/// DefiLlama publishes no prices, so a quote must fall through to another
/// routed provider rather than failing the chain.
#[tokio::test]
#[ignore = "requires network access"]
async fn quotes_are_not_supported_by_defillama_alone() {
    let providers = Providers::builder()
        .route(Capability::CRYPTO, [Provider::DefiLlama])
        .build()
        .await
        .unwrap();

    assert!(providers.crypto("aave").quote("usd").await.is_err());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn chains_are_ranked_largest_first() {
    let chains = finance_query::defi::chains().await.unwrap();
    assert!(chains.len() > 50);
    let tvls: Vec<f64> = chains.iter().map(|c| c.tvl.unwrap_or(0.0)).collect();
    assert!(
        tvls.windows(2).all(|w| w[0] >= w[1]),
        "chains are not sorted by TVL descending"
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn stablecoins_are_ranked_and_carry_their_peg_type() {
    let coins = finance_query::defi::stablecoins().await.unwrap();
    assert!(!coins.is_empty());
    assert!(coins[0].circulating.is_some_and(|c| c > 0.0));
    assert!(
        coins.iter().all(|c| c.peg_type.is_some()),
        "peg_type is needed to interpret `circulating`"
    );
}
