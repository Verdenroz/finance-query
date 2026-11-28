use finance_query::{Capability, Provider, Providers};
use finance_query_server::cache::Cache;
use finance_query_server::services::crypto;
use mockito::{Matcher, Server};
use serde_json::json;
use std::sync::Arc;

// One test: the FMP endpoint override is process-global, so cases share a single server.
#[tokio::test]
async fn crypto_news_service_shapes_fmp_response_and_maps_errors() {
    let mut server = Server::new_async().await;
    let news = server
        .mock("GET", "/stable/news/crypto-latest")
        .match_query(Matcher::UrlEncoded("limit".into(), "2".into()))
        .with_body(
            json!([
                {"title": "Bitcoin steadies", "url": "https://example.com/a", "site": "example.com",
                 "publishedDate": "2025-01-16 08:00:00"},
                {"title": "Ether climbs", "url": "https://example.com/b", "publisher": "Wire",
                 "publishedDate": "2025-01-16 09:00:00"}
            ])
            .to_string(),
        )
        .create_async()
        .await;
    let failing = server
        .mock("GET", "/stable/news/crypto-latest")
        .match_query(Matcher::UrlEncoded("limit".into(), "7".into()))
        .with_status(500)
        .create_async()
        .await;

    let providers = Arc::new(
        Providers::builder()
            .providers([Provider::Fmp])
            .api_key(Provider::Fmp, "crypto-service-fixture")
            .endpoint(Provider::Fmp, server.url())
            .route(Capability::CRYPTO, [Provider::Fmp])
            .build()
            .await
            .unwrap(),
    );
    let cache = Cache::new(None).await;

    let value = crypto::get_news(&cache, &providers, 2).await.unwrap();
    let articles = value.as_array().unwrap();
    assert_eq!(articles.len(), 2);
    assert_eq!(articles[0]["title"], "Bitcoin steadies");
    assert_eq!(articles[0]["source"], "example.com");
    assert_eq!(
        articles[1]["source"], "Wire",
        "publisher fills in for a missing site"
    );
    news.assert_async().await;

    let err = crypto::get_news(&cache, &providers, 7).await.unwrap_err();
    assert!(!err.to_string().is_empty());
    failing.assert_async().await;
}

// One test: the DefiLlama endpoint override is process-global, so cases share a single server.
#[tokio::test]
async fn crypto_tvl_services_shape_defillama_responses_and_map_errors() {
    let mut server = Server::new_async().await;
    let lite = server
        .mock("GET", "/lite/protocols2")
        .match_query(Matcher::Any)
        .with_body(
            json!({
                "protocols": [{
                    "name": "Aave V3", "symbol": "AAVE", "url": "https://aave.com",
                    "chains": ["Ethereum", "Base"],
                    "chainTvls": {
                        "Ethereum": {"tvl": 9000.0, "tvlPrevDay": 8000.0, "tvlPrevWeek": 9000.0},
                        "Base": {"tvl": 1000.0}
                    },
                    "tvl": 10000.0, "tvlPrevDay": 9000.0, "tvlPrevWeek": 8000.0,
                    "mcap": 2500000.0
                }],
                "parentProtocols": []
            })
            .to_string(),
        )
        .create_async()
        .await;
    let history = server
        .mock("GET", "/protocol/aave-v3")
        .with_body(
            json!({"tvl": [
                {"date": 1705363200, "totalLiquidityUSD": 5.0},
                {"date": 1705276800, "totalLiquidityUSD": 4.0}
            ]})
            .to_string(),
        )
        .create_async()
        .await;
    let failing = server
        .mock("GET", "/protocol/broken")
        .with_status(500)
        .create_async()
        .await;

    let providers = Arc::new(
        Providers::builder()
            .providers([Provider::DefiLlama])
            .endpoint(Provider::DefiLlama, server.url())
            .route(Capability::CRYPTO, [Provider::DefiLlama])
            .build()
            .await
            .unwrap(),
    );
    let cache = Cache::new(None).await;

    let value = crypto::get_protocol_tvl(&cache, &providers, "aave-v3")
        .await
        .unwrap();
    assert_eq!(value["name"], "Aave V3");
    assert_eq!(value["tvl"], 10000.0);
    lite.assert_async().await;

    let value = crypto::get_protocol_tvl_history(&cache, &providers, "aave-v3")
        .await
        .unwrap();
    let points = value.as_array().unwrap();
    assert_eq!(points.len(), 2);
    assert_eq!(points[0]["timestamp"], 1705276800);
    assert_eq!(points[1]["tvl"], 5.0);
    history.assert_async().await;

    let err = crypto::get_protocol_tvl_history(&cache, &providers, "broken")
        .await
        .unwrap_err();
    assert!(!err.to_string().is_empty());
    failing.assert_async().await;

    server.reset();
    let failing_lite = server
        .mock("GET", "/lite/protocols2")
        .match_query(Matcher::Any)
        .with_status(500)
        .create_async()
        .await;
    let err = crypto::get_protocol_tvl(&cache, &providers, "other")
        .await
        .unwrap_err();
    assert!(!err.to_string().is_empty());
    failing_lite.assert_async().await;
}
