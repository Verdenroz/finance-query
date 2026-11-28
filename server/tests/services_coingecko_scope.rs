use finance_query::{Capability, Provider, Providers};
use finance_query_server::cache::Cache;
use finance_query_server::services::crypto;
use mockito::{Matcher, Server};
use serde_json::json;

#[tokio::test]
async fn crypto_services_reach_the_scoped_coingecko_endpoint_and_map_errors() {
    let mut server = Server::new_async().await;
    let bitcoin = json!({
        "id": "bitcoin", "symbol": "btc", "name": "Bitcoin",
        "current_price": 65000.0, "market_cap": 1.3e12
    });
    let coins = server
        .mock("GET", "/api/v3/coins/markets")
        .match_query(Matcher::UrlEncoded("per_page".into(), "2".into()))
        .with_body(
            json!([bitcoin, {"id": "ethereum", "symbol": "eth", "name": "Ethereum"}]).to_string(),
        )
        .create_async()
        .await;
    let coin = server
        .mock("GET", "/api/v3/coins/markets")
        .match_query(Matcher::UrlEncoded("ids".into(), "bitcoin".into()))
        .with_body(json!([bitcoin]).to_string())
        .create_async()
        .await;
    let search = server
        .mock("GET", "/api/v3/search")
        .match_query(Matcher::UrlEncoded("query".into(), "bit".into()))
        .with_body(
            json!({"coins": [
                {"id": "bitcoin", "name": "Bitcoin", "symbol": "btc", "market_cap_rank": 1}
            ]})
            .to_string(),
        )
        .create_async()
        .await;
    let failing = server
        .mock("GET", "/api/v3/search")
        .match_query(Matcher::UrlEncoded("query".into(), "boom".into()))
        .with_status(500)
        .create_async()
        .await;

    let providers = Providers::builder()
        .providers([Provider::CoinGecko])
        .endpoint(Provider::CoinGecko, server.url())
        .route(Capability::CRYPTO, [Provider::CoinGecko])
        .build()
        .await
        .unwrap();
    let cache = Cache::new(None).await;

    providers
        .scope(async {
            let value = crypto::get_coins(&cache, "usd", 2).await.unwrap();
            assert_eq!(value.as_array().unwrap().len(), 2);
            assert_eq!(value[0]["id"], "bitcoin");
            assert_eq!(value[0]["symbol"], "btc");

            let value = crypto::get_coin(&cache, "bitcoin", "usd").await.unwrap();
            assert_eq!(value["name"], "Bitcoin");
            assert_eq!(value["current_price"], 65000.0);

            let value = crypto::search(&cache, "bit", 5).await.unwrap();
            assert_eq!(value[0]["symbol"], "BTC");
            assert_eq!(value[0]["id"], "bitcoin");

            let err = crypto::search(&cache, "boom", 5).await.unwrap_err();
            assert!(!err.to_string().is_empty());
        })
        .await;

    coins.assert_async().await;
    coin.assert_async().await;
    search.assert_async().await;
    failing.assert_async().await;
}
