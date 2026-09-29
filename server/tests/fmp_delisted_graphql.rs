use finance_query::{Capability, Provider, Providers};
use finance_query_server::{AppState, FeedHub, StreamHub, cache::Cache, graphql};
use mockito::{Matcher, Server};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn graphql_delisted_listings_preserve_provider_dates() {
    let mut server = Server::new_async().await;
    let first = server
        .mock("GET", "/stable/delisted-companies")
        .match_query(Matcher::UrlEncoded("page".into(), "0".into()))
        .with_body(r#"[{"symbol":"OLD","ipoDate":"1980-12-12","delistedDate":"2003-01-02"}]"#)
        .create_async()
        .await;
    let end = server
        .mock("GET", "/stable/delisted-companies")
        .match_query(Matcher::UrlEncoded("page".into(), "1".into()))
        .with_body("[]")
        .create_async()
        .await;
    let providers = Providers::builder()
        .providers([Provider::Fmp])
        .api_key(Provider::Fmp, "graphql-fixture")
        .endpoint(Provider::Fmp, server.url())
        .requests_per_minute(Provider::Fmp, 600_000)
        .route(Capability::DISCOVERY, [Provider::Fmp])
        .build()
        .await
        .unwrap();
    let schema = graphql::build_schema(AppState {
        cache: Cache::new(None).await,
        stream_hub: StreamHub::new(),
        feed_hub: FeedHub::new(),
        providers: Arc::new(providers),
    });
    let response = schema
        .execute(
            "{ listingStatus(active: false) { edges { node { symbol ipoDate delistedDate } } } }",
        )
        .await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    assert_eq!(
        response.data.into_json().unwrap(),
        json!({
            "listingStatus": { "edges": [{ "node": {
                "symbol": "OLD", "ipoDate": "1980-12-12", "delistedDate": "2003-01-02"
            } }] }
        })
    );
    first.assert_async().await;
    end.assert_async().await;
}
