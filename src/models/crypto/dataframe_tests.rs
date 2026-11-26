use super::*;
use crate::models::dataframe_test_util::columns;
use serde_json::json;

#[test]
fn crypto_quote_columns() {
    let quotes: Vec<CryptoQuote> = serde_json::from_value(json!([
        {"id": "bitcoin", "symbol": "btc", "name": "Bitcoin", "price": 1.0}
    ]))
    .unwrap();
    let df = CryptoQuote::vec_to_dataframe(&quotes).unwrap();

    assert_eq!(
        columns(&df),
        [
            "id:str",
            "symbol:str",
            "name:str",
            "price:f64",
            "market_cap:f64",
            "volume_24h:f64",
            "change_24h:f64",
            "change_percent_24h:f64",
            "high_24h:f64",
            "low_24h:f64",
            "circulating_supply:f64",
        ]
    );
    assert_eq!(CryptoQuote::vec_to_dataframe(&[]).unwrap().height(), 0);
}

#[cfg(feature = "crypto")]
#[test]
fn coingecko_frames() {
    let coins: Vec<CoinQuote> = serde_json::from_value(json!([
        {"id": "bitcoin", "symbol": "btc", "name": "Bitcoin", "market_cap_rank": 1}
    ]))
    .unwrap();
    assert_eq!(
        columns(&CoinQuote::vec_to_dataframe(&coins).unwrap()),
        [
            "id:str",
            "symbol:str",
            "name:str",
            "current_price:f64",
            "market_cap:f64",
            "price_change_percentage_24h:f64",
            "total_volume:f64",
            "circulating_supply:f64",
            "image:str",
            "market_cap_rank:u32",
        ]
    );

    let trending: Vec<TrendingCoin> = serde_json::from_value(json!([{"score": 0}])).unwrap();
    assert_eq!(
        columns(&TrendingCoin::vec_to_dataframe(&trending).unwrap()),
        [
            "id:str",
            "symbol:str",
            "name:str",
            "market_cap_rank:u32",
            "price_btc:f64",
            "score:u32",
        ]
    );

    let stats: GlobalCryptoStats = serde_json::from_value(json!({"markets": 5})).unwrap();
    assert_eq!(
        columns(&stats.to_dataframe().unwrap()),
        [
            "active_cryptocurrencies:u32",
            "markets:u32",
            "total_market_cap_usd:f64",
            "total_volume_usd:f64",
            "btc_dominance:f64",
            "eth_dominance:f64",
            "market_cap_change_percentage_24h_usd:f64",
        ]
    );
}

#[cfg(feature = "defi")]
#[test]
fn defi_frames() {
    use super::defi::*;

    let protocols: Vec<ProtocolTvl> = serde_json::from_value(json!([{
        "slug": "aave", "chains": ["Ethereum"], "tvl_by_chain": [], "tvl": 1.0
    }]))
    .unwrap();
    let df = ProtocolTvl::vec_to_dataframe(&protocols).unwrap();
    assert_eq!(
        columns(&df),
        [
            "slug:str",
            "name:str",
            "symbol:str",
            "url:str",
            "tvl:f64",
            "change_1d_percent:f64",
            "change_7d_percent:f64",
            "market_cap:f64",
            "chains:list[str]",
        ]
    );

    let points: Vec<TvlPoint> =
        serde_json::from_value(json!([{"timestamp": 1, "tvl": 2.0}])).unwrap();
    assert_eq!(
        columns(&TvlPoint::vec_to_dataframe(&points).unwrap()),
        ["timestamp:i64", "tvl:f64"]
    );

    let allocations: Vec<ChainAllocation> =
        serde_json::from_value(json!([{"chain": "Ethereum", "tvl": 2.0}])).unwrap();
    assert_eq!(
        columns(&ChainAllocation::vec_to_dataframe(&allocations).unwrap()),
        ["chain:str", "tvl:f64"]
    );

    let chains: Vec<ChainTvl> = serde_json::from_value(json!([{"name": "Ethereum"}])).unwrap();
    assert_eq!(
        columns(&ChainTvl::vec_to_dataframe(&chains).unwrap()),
        [
            "name:str",
            "token_symbol:str",
            "gecko_id:str",
            "chain_id:i64",
            "tvl:f64",
        ]
    );

    let stables: Vec<StablecoinSupply> =
        serde_json::from_value(json!([{"name": "Tether"}])).unwrap();
    assert_eq!(
        columns(&StablecoinSupply::vec_to_dataframe(&stables).unwrap()),
        [
            "name:str",
            "symbol:str",
            "gecko_id:str",
            "peg_type:str",
            "peg_mechanism:str",
            "circulating:f64",
            "circulating_prev_day:f64",
            "circulating_prev_week:f64",
            "circulating_prev_month:f64",
        ]
    );
}
