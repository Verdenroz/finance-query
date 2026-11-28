use finance_query::{Capability, Provider, Providers};
use finance_query_server::cache::Cache;
use finance_query_server::services::fred;
use mockito::{Matcher, Server};
use serde_json::json;
use std::sync::Arc;

// One test: the FRED endpoint override is process-global, so cases share a single server.
#[tokio::test]
async fn fred_as_of_service_shapes_vintage_series_and_maps_errors() {
    let mut server = Server::new_async().await;
    let vintage = server
        .mock("GET", "/fred/series/observations")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("series_id".into(), "GDP".into()),
            Matcher::UrlEncoded("realtime_start".into(), "2024-01-15".into()),
            Matcher::UrlEncoded("realtime_end".into(), "2024-01-15".into()),
        ]))
        .with_body(
            json!({"observations": [
                {"date": "2023-07-01", "value": "27610.1"},
                {"date": "2023-10-01", "value": "."}
            ]})
            .to_string(),
        )
        .create_async()
        .await;

    let providers = Arc::new(
        Providers::builder()
            .providers([Provider::Fred])
            .api_key(Provider::Fred, "fixture")
            .endpoint(Provider::Fred, server.url())
            .route(Capability::ECONOMIC, [Provider::Fred])
            .build()
            .await
            .unwrap(),
    );
    let cache = Cache::new(None).await;

    let value = fred::get_series_as_of(&cache, &providers, "GDP", "2024-01-15")
        .await
        .unwrap();
    assert_eq!(value["series_id"], "GDP");
    let observations = value["observations"].as_array().unwrap();
    assert_eq!(observations[0]["date"], "2023-07-01");
    assert_eq!(observations[0]["value"], 27610.1);
    vintage.assert_async().await;

    server.reset();
    let failing = server
        .mock("GET", "/fred/series/observations")
        .match_query(Matcher::Any)
        .with_status(500)
        .create_async()
        .await;
    let err = fred::get_series_as_of(&cache, &providers, "GDP", "2024-02-15")
        .await
        .unwrap_err();
    assert!(!err.to_string().is_empty());
    failing.assert_async().await;
}
