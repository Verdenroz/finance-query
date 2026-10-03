use finance_query::{Capability, Provider, Providers};
use finance_query_server::{AppState, FeedHub, StreamHub, cache::Cache, graphql};
use mockito::{Matcher, Server};
use serde_json::json;
use std::sync::Arc;

// The ticker resolver needs more than the 2 MiB default test-thread stack in debug builds.
#[test]
fn graphql_profile_preserves_identifiers_and_classification() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(profile_round_trip())
        })
        .unwrap()
        .join()
        .unwrap();
}

async fn profile_round_trip() {
    let mut server = Server::new_async().await;
    let body = json!([{
        "symbol": "AAPL",
        "cik": "0000320193",
        "cusip": "037833100",
        "isin": "US0378331005",
        "ipoDate": "1980-12-12",
        "isActivelyTrading": true,
        "isEtf": false,
        "isAdr": false,
        "isFund": false
    }]);
    let fixture = server
        .mock("GET", "/stable/profile")
        .match_query(Matcher::UrlEncoded("symbol".into(), "AAPL".into()))
        .with_body(body.to_string())
        .create_async()
        .await;
    let providers = Providers::builder()
        .providers([Provider::Fmp])
        .api_key(Provider::Fmp, "profile-graphql-fixture")
        .endpoint(Provider::Fmp, server.url())
        .route(Capability::FUNDAMENTALS, [Provider::Fmp])
        .build()
        .await
        .unwrap();
    let schema = graphql::build_schema(AppState {
        cache: Cache::new(None).await,
        stream_hub: StreamHub::new(),
        feed_hub: FeedHub::new(),
        providers: Arc::new(providers),
    });
    let query = r#"{ ticker(symbol: "AAPL") {
        companyProfile { symbol cik cusip isin ipoDate active isEtf isAdr isFund }
    } }"#;
    let response = schema.execute(query).await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    assert_eq!(
        response.data.into_json().unwrap(),
        json!({"ticker":{"companyProfile":{
            "symbol":"AAPL","cik":"0000320193","cusip":"037833100","isin":"US0378331005",
            "ipoDate":"1980-12-12","active":true,"isEtf":false,"isAdr":false,"isFund":false
        }}})
    );
    fixture.assert_async().await;
}
