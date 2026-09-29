use finance_query::{Capability, Provider, Providers};
use finance_query_server::{AppState, FeedHub, StreamHub, cache::Cache, graphql};
use mockito::{Matcher, Server};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn graphql_profile_preserves_identifiers_and_classification() {
    let mut server = Server::new_async().await;
    let fixture = server.mock("GET", "/stable/profile")
        .match_query(Matcher::UrlEncoded("symbol".into(), "AAPL".into()))
        .with_body(r#"[{"symbol":"AAPL","cik":"0000320193","cusip":"037833100","isin":"US0378331005","ipoDate":"1980-12-12","isActivelyTrading":true,"isEtf":false,"isAdr":false,"isFund":false}]"#)
        .create_async().await;
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
    let response = schema.execute("{ ticker(symbol: \"AAPL\") { companyProfile { symbol cik cusip isin ipoDate active isEtf isAdr isFund } } }").await;
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
