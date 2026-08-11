//! Guards the two independent naming layers on every `Gql*` type.
//!
//! REST handlers do not serde-serialize these structs — they execute a
//! GraphQL query and return async-graphql's JSON, so `#[graphql(rename_fields
//! = "camelCase")]` decides the bytes on the wire. Their `Deserialize` impl
//! runs in the opposite direction, parsing `serde_json::to_value` output of the
//! *library* models in `services/`. The two layers must be read separately:
//!
//! * `sdl_*` tests pin the wire names an API spec has to match.
//! * `roundtrip_*` tests pin the parse side. Every `Gql*` type here is
//!   `#[serde(default)]`, so a rename that stops matching the library's keys
//!   does not error — it silently yields zeroed data. These tests assert real
//!   values survive, which is the only way that failure mode shows up.

use finance_query_server::{
    AppState, FeedHub, StreamHub,
    cache::Cache,
    graphql::{
        self,
        types::{
            crypto::{GqlCoinQuote, GqlGlobalCryptoStats},
            indicators::GqlStochasticData,
            keyless::GqlCommitmentsOfTraders,
        },
    },
};
use serde_json::json;

/// Serialize a library model exactly as `services/` does, then parse it into
/// the GraphQL type the resolver returns.
fn relay<L, G>(library_json: serde_json::Value) -> G
where
    L: serde::de::DeserializeOwned + serde::Serialize,
    G: serde::de::DeserializeOwned,
{
    let library: L = serde_json::from_value(library_json)
        .expect("fixture should match the library model's own shape");
    let wire = serde_json::to_value(&library).expect("library model should serialize");
    serde_json::from_value(wire).expect("Gql type should parse the library model's output")
}

#[test]
fn roundtrip_commitments_of_traders_keeps_every_field() {
    let cot: GqlCommitmentsOfTraders =
        relay::<finance_query::cftc::CommitmentsOfTraders, _>(json!({
            "symbol": "GC=F",
            "market_and_exchange_name": "GOLD - COMMODITY EXCHANGE INC.",
            "cftc_contract_market_code": "088691",
            "observations": [{
                "report_date": "2025-01-07",
                "open_interest": 441_234,
                "producer_merchant_long": 100_001,
                "producer_merchant_short": 100_002,
                "swap_dealer_long": 100_003,
                "swap_dealer_short": 100_004,
                "swap_dealer_spread": 100_005,
                "managed_money_long": 100_006,
                "managed_money_short": 100_007,
                "managed_money_spread": 100_008,
                "other_reportable_long": 100_009,
                "other_reportable_short": 100_010,
                "other_reportable_spread": 100_011,
                "total_reportable_long": 100_012,
                "total_reportable_short": 100_013,
                "nonreportable_long": 100_014,
                "nonreportable_short": 100_015,
            }],
        }));

    assert_eq!(cot.symbol, "GC=F");
    assert_eq!(
        cot.market_and_exchange_name,
        "GOLD - COMMODITY EXCHANGE INC."
    );
    assert_eq!(cot.cftc_contract_market_code, "088691");

    let obs = cot
        .observations
        .first()
        .expect("observations must survive the relay, not default to empty");
    assert_eq!(obs.report_date, "2025-01-07");
    assert_eq!(obs.open_interest, Some(441_234));
    assert_eq!(obs.producer_merchant_long, Some(100_001));
    assert_eq!(obs.producer_merchant_short, Some(100_002));
    assert_eq!(obs.swap_dealer_long, Some(100_003));
    assert_eq!(obs.swap_dealer_short, Some(100_004));
    assert_eq!(obs.swap_dealer_spread, Some(100_005));
    assert_eq!(obs.managed_money_long, Some(100_006));
    assert_eq!(obs.managed_money_short, Some(100_007));
    assert_eq!(obs.managed_money_spread, Some(100_008));
    assert_eq!(obs.other_reportable_long, Some(100_009));
    assert_eq!(obs.other_reportable_short, Some(100_010));
    assert_eq!(obs.other_reportable_spread, Some(100_011));
    assert_eq!(obs.total_reportable_long, Some(100_012));
    assert_eq!(obs.total_reportable_short, Some(100_013));
    assert_eq!(obs.nonreportable_long, Some(100_014));
    assert_eq!(obs.nonreportable_short, Some(100_015));
}

#[test]
fn roundtrip_coin_quote_keeps_every_field() {
    let quote: GqlCoinQuote = relay::<finance_query::crypto::CoinQuote, _>(json!({
        "id": "bitcoin",
        "symbol": "BTC",
        "name": "Bitcoin",
        "current_price": 96_500.25,
        "market_cap": 1_910_000_000_000.0_f64,
        "price_change_percentage_24h": -1.75,
        "total_volume": 42_000_000_000.0_f64,
        "circulating_supply": 19_800_000.0_f64,
        "image": "https://example.invalid/btc.png",
        "market_cap_rank": 1,
    }));

    assert_eq!(quote.id, "bitcoin");
    assert_eq!(quote.symbol, "BTC");
    assert_eq!(quote.name, "Bitcoin");
    assert_eq!(quote.current_price, Some(96_500.25));
    assert_eq!(quote.market_cap, Some(1_910_000_000_000.0));
    assert_eq!(quote.price_change_percentage_24h, Some(-1.75));
    assert_eq!(quote.total_volume, Some(42_000_000_000.0));
    assert_eq!(quote.circulating_supply, Some(19_800_000.0));
    assert_eq!(
        quote.image.as_deref(),
        Some("https://example.invalid/btc.png")
    );
    assert_eq!(quote.market_cap_rank, Some(1));
}

#[test]
fn roundtrip_global_crypto_stats_keeps_every_field() {
    let stats: GqlGlobalCryptoStats = relay::<finance_query::crypto::GlobalCryptoStats, _>(json!({
        "active_cryptocurrencies": 17_432,
        "markets": 1_180,
        "total_market_cap_usd": 3_400_000_000_000.0_f64,
        "total_volume_usd": 155_000_000_000.0_f64,
        "btc_dominance": 56.25,
        "eth_dominance": 12.5,
        "market_cap_change_percentage_24h_usd": -0.85,
    }));

    assert_eq!(stats.active_cryptocurrencies, Some(17_432));
    assert_eq!(stats.markets, Some(1_180));
    assert_eq!(stats.total_market_cap_usd, Some(3_400_000_000_000.0));
    assert_eq!(stats.total_volume_usd, Some(155_000_000_000.0));
    assert_eq!(stats.btc_dominance, Some(56.25));
    assert_eq!(stats.eth_dominance, Some(12.5));
    assert_eq!(stats.market_cap_change_percentage_24h_usd, Some(-0.85));
}

#[test]
fn roundtrip_stochastic_keeps_percent_k_and_percent_d() {
    // `StochasticData` renames its fields to the literal `"%K"`/`"%D"`, which
    // are not legal GraphQL names — the two layers cannot share one spelling.
    let stoch: GqlStochasticData = relay::<finance_query::StochasticData, _>(json!({
        "%K": 82.5,
        "%D": 78.25,
    }));

    assert_eq!(stoch.k, Some(82.5));
    assert_eq!(stoch.d, Some(78.25));
}

#[tokio::test]
async fn sdl_field_names_are_the_graphql_spelling_not_the_serde_one() {
    let state = AppState {
        cache: Cache::new(None).await,
        stream_hub: StreamHub::new(),
        feed_hub: FeedHub::new(),
    };
    let sdl = graphql::build_schema(state).sdl();

    // Every name here is what a REST response actually contains, and differs
    // from what the type's serde attributes alone would suggest.
    for expected in [
        // GqlCotObservation / GqlCommitmentsOfTraders: no serde rename at all,
        // yet the wire is camelCase.
        "reportDate: String!",
        "openInterest: Int",
        "producerMerchantLong: Int",
        "marketAndExchangeName: String!",
        "cftcContractMarketCode: String!",
        // async-graphql capitalises the digit-led segment; serde's camelCase
        // would leave it as `...24h` / `...24hUsd`.
        "priceChangePercentage24H: Float",
        "marketCapChangePercentage24HUsd: Float",
        // `%K`/`%D` on the serde side, plain `k`/`d` on the wire.
        "\tk: Float",
        "\td: Float",
        // `gmt_offset`, camelCased by async-graphql.
        "gmtOffset: Int",
    ] {
        assert!(
            sdl.contains(expected),
            "live schema is missing `{expected}` — an API spec generated from \
             serde attributes alone would disagree with the wire here"
        );
    }
}
