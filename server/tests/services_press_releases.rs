use finance_query::{Capability, Provider, Providers};
use finance_query_server::cache::Cache;
use finance_query_server::services::news;
use mockito::{Matcher, Server};
use serde_json::json;
use std::sync::Arc;

// One test: the FMP endpoint override is process-global, so cases share a single server.
#[tokio::test]
async fn press_release_service_shapes_fmp_response_and_maps_errors() {
    let mut server = Server::new_async().await;
    let releases = server
        .mock("GET", "/stable/news/press-releases")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("symbols".into(), "AAPL".into()),
            Matcher::UrlEncoded("limit".into(), "2".into()),
        ]))
        .with_body(
            json!([
                {"symbol": "AAPL", "publishedDate": "2025-01-16 08:00:00",
                 "title": "Apple reports results", "text": "Cupertino, California"},
                {"publishedDate": "2025-01-10 08:00:00", "title": "Apple announces event"}
            ])
            .to_string(),
        )
        .create_async()
        .await;
    let failing = server
        .mock("GET", "/stable/news/press-releases")
        .match_query(Matcher::UrlEncoded("symbols".into(), "BAD".into()))
        .with_status(500)
        .create_async()
        .await;

    let providers = Arc::new(
        Providers::builder()
            .providers([Provider::Fmp])
            .api_key(Provider::Fmp, "press-release-service-fixture")
            .endpoint(Provider::Fmp, server.url())
            .route(Capability::CORPORATE, [Provider::Fmp])
            .build()
            .await
            .unwrap(),
    );
    let cache = Cache::new(None).await;

    let value = news::get_press_releases(&cache, &providers, "AAPL", 2)
        .await
        .unwrap();
    let rows = value.as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["title"], "Apple reports results");
    assert_eq!(rows[0]["text"], "Cupertino, California");
    assert_eq!(
        rows[1]["symbol"], "AAPL",
        "the requested symbol fills in for a missing one"
    );
    releases.assert_async().await;

    let err = news::get_press_releases(&cache, &providers, "BAD", 2)
        .await
        .unwrap_err();
    assert!(!err.to_string().is_empty());
    failing.assert_async().await;
}
