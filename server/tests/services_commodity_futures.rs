use finance_query::{Capability, Provider, Providers};
use finance_query_server::cache::Cache;
use finance_query_server::services::commodity_futures;
use mockito::{Matcher, Server};
use serde_json::json;
use std::sync::Arc;

// One test: the FMP endpoint override is process-global, so cases share a single server.
#[tokio::test]
async fn commodity_and_index_services_shape_fmp_responses_and_map_errors() {
    let mut server = Server::new_async().await;
    let quote = server
        .mock("GET", "/stable/quote")
        .match_query(Matcher::UrlEncoded("symbol".into(), "GCUSD".into()))
        .with_body(
            json!([{"symbol": "GCUSD", "name": "Gold", "price": 2650.5, "change": 12.0,
                    "changesPercentage": 0.45}])
            .to_string(),
        )
        .create_async()
        .await;
    let failing = server
        .mock("GET", "/stable/quote")
        .match_query(Matcher::UrlEncoded("symbol".into(), "BADUSD".into()))
        .with_status(500)
        .create_async()
        .await;
    let constituents = server
        .mock("GET", "/stable/sp500-constituent")
        .match_query(Matcher::Any)
        .with_body(
            json!([
                {"symbol": "AAPL", "name": "Apple Inc.", "sector": "Technology"},
                {"symbol": "XOM", "name": "Exxon Mobil", "sector": "Energy"}
            ])
            .to_string(),
        )
        .create_async()
        .await;

    let providers = Arc::new(
        Providers::builder()
            .providers([Provider::Fmp])
            .api_key(Provider::Fmp, "commodity-service-fixture")
            .endpoint(Provider::Fmp, server.url())
            .route(Capability::COMMODITIES, [Provider::Fmp])
            .route(Capability::INDICES, [Provider::Fmp])
            .build()
            .await
            .unwrap(),
    );
    let cache = Cache::new(None).await;

    let value = commodity_futures::get_commodity_quote(&cache, &providers, "GCUSD")
        .await
        .unwrap();
    assert_eq!(value["symbol"], "GCUSD");
    assert_eq!(value["name"], "Gold");
    assert_eq!(value["price"], 2650.5);
    assert_eq!(value["change_percent"], 0.45);
    quote.assert_async().await;

    let value = commodity_futures::get_index_constituents(&cache, &providers, "^GSPC")
        .await
        .unwrap();
    let rows = value.as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["symbol"], "AAPL");
    assert_eq!(rows[1]["sector"], "Energy");
    constituents.assert_async().await;

    let err = commodity_futures::get_index_constituents(&cache, &providers, "^FTSE")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("major indices only"));

    let err = commodity_futures::get_commodity_quote(&cache, &providers, "BADUSD")
        .await
        .unwrap_err();
    assert!(!err.to_string().is_empty());
    failing.assert_async().await;
}
