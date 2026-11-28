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
