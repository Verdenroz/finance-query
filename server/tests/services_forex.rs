use finance_query::{Capability, Provider, Providers};
use finance_query_server::cache::Cache;
use finance_query_server::services::forex;
use mockito::{Matcher, Server};
use serde_json::json;
use std::sync::Arc;

// One test: the FMP endpoint override is process-global, so cases share a single server.
#[tokio::test]
async fn forex_services_shape_fmp_responses_and_map_errors() {
    let mut server = Server::new_async().await;
    let quote = server
        .mock("GET", "/stable/quote")
        .match_query(Matcher::UrlEncoded("symbol".into(), "EURUSD".into()))
        .with_body(
            json!([{"symbol": "EURUSD", "price": 1.105, "timestamp": 1705363200}]).to_string(),
        )
        .create_async()
        .await;
    let news = server
        .mock("GET", "/stable/news/forex-latest")
        .match_query(Matcher::UrlEncoded("limit".into(), "2".into()))
        .with_body(
            json!([
                {"title": "Euro firms", "url": "https://example.com/a", "site": "example.com",
                 "publishedDate": "2025-01-16 08:00:00"},
                {"title": "Yen slips", "url": "https://example.com/b", "publisher": "Wire",
                 "publishedDate": "2025-01-16 09:00:00"}
            ])
            .to_string(),
        )
        .create_async()
        .await;
    let failing = server
        .mock("GET", "/stable/quote")
        .match_query(Matcher::UrlEncoded("symbol".into(), "GBPJPY".into()))
        .with_status(500)
        .create_async()
        .await;

    let providers = Arc::new(
        Providers::builder()
            .providers([Provider::Fmp])
            .api_key(Provider::Fmp, "forex-service-fixture")
            .endpoint(Provider::Fmp, server.url())
            .route(Capability::FOREX, [Provider::Fmp])
            .build()
            .await
            .unwrap(),
    );
    let cache = Cache::new(None).await;

    let value = forex::get_quote(&cache, &providers, "eur", "usd")
        .await
        .unwrap();
    assert_eq!(value["symbol"], "EURUSD");
    assert_eq!(value["price"], 1.105);
    assert_eq!(
        value["bid"], 1.105,
        "FMP has no bid/ask, so bid mirrors price"
    );
    assert!(value["ask"].is_null());
    quote.assert_async().await;

    let value = forex::get_news(&cache, &providers, 2).await.unwrap();
    let articles = value.as_array().unwrap();
    assert_eq!(articles.len(), 2);
    assert_eq!(articles[0]["title"], "Euro firms");
    assert_eq!(articles[0]["source"], "example.com");
    assert_eq!(
        articles[1]["source"], "Wire",
        "publisher fills in for a missing site"
    );
    news.assert_async().await;

    let err = forex::get_quote(&cache, &providers, "gbp", "jpy")
        .await
        .unwrap_err();
    assert!(!err.to_string().is_empty());
    failing.assert_async().await;
}
