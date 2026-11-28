use finance_query::{Capability, Provider, Providers};
use finance_query_server::cache::Cache;
use finance_query_server::services::crypto;
use mockito::Server;
use serde_json::json;
use std::sync::Arc;

// One test: the CoinGecko endpoint override is process-global, so cases share a single server.
#[tokio::test]
async fn crypto_market_services_shape_coingecko_responses_and_map_errors() {
    let mut server = Server::new_async().await;
    let trending = server
        .mock("GET", "/api/v3/search/trending")
        .with_body(
            json!({"coins": [{"item": {
                "id": "bitcoin", "name": "Bitcoin", "symbol": "btc",
                "market_cap_rank": 1, "price_btc": 1.0, "score": 0
            }}]})
            .to_string(),
        )
        .create_async()
        .await;
    let global = server
        .mock("GET", "/api/v3/global")
        .with_body(
            json!({"data": {
                "active_cryptocurrencies": 18137, "markets": 1510,
                "total_market_cap": {"usd": 3.0e12},
                "total_volume": {"usd": 1.0e11},
                "market_cap_percentage": {"btc": 45.2, "eth": 18.1},
                "market_cap_change_percentage_24h_usd": 0.64
            }})
            .to_string(),
        )
        .create_async()
        .await;

    let providers = Arc::new(
        Providers::builder()
            .providers([Provider::CoinGecko])
            .endpoint(Provider::CoinGecko, server.url())
            .route(Capability::CRYPTO, [Provider::CoinGecko])
            .build()
            .await
            .unwrap(),
    );
    let cache = Cache::new(None).await;

    let value = crypto::get_trending(&cache, &providers).await.unwrap();
    assert_eq!(value[0]["id"], "bitcoin");
    assert_eq!(value[0]["symbol"], "BTC");
    assert_eq!(value[0]["market_cap_rank"], 1);
    trending.assert_async().await;

    let value = crypto::get_global(&cache, &providers).await.unwrap();
    assert_eq!(value["active_cryptocurrencies"], 18137);
    assert_eq!(value["total_market_cap_usd"], 3.0e12);
    assert_eq!(value["btc_dominance"], 45.2);
    global.assert_async().await;

    server.reset();
    let failing = server
        .mock("GET", "/api/v3/global")
        .with_status(500)
        .create_async()
        .await;
    let cache = Cache::new(None).await;
    let err = crypto::get_global(&cache, &providers).await.unwrap_err();
    assert!(!err.to_string().is_empty());
    failing.assert_async().await;
}
