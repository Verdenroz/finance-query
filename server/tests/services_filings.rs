use finance_query::{Capability, Provider, Providers};
use finance_query_server::cache::Cache;
use finance_query_server::services::filings;
use mockito::{Matcher, Server};
use serde_json::json;
use std::sync::Arc;

// One test: the provider endpoint overrides are process-global, so cases share a single server.
#[tokio::test]
async fn filings_services_shape_provider_responses_and_map_errors() {
    let mut server = Server::new_async().await;
    let trades = server
        .mock("GET", "/stable/senate-trades")
        .match_query(Matcher::UrlEncoded("symbol".into(), "AAPL".into()))
        .with_body(
            json!([{
                "symbol": "AAPL", "transactionDate": "2025-01-10", "disclosureDate": "2025-01-20",
                "firstName": "Jane", "lastName": "Doe", "office": "Jane Doe", "type": "Purchase",
                "amount": "$1,001 - $15,000"
            }])
            .to_string(),
        )
        .create_async()
        .await;
    let ftd = server
        .mock("GET", "/api/v4/fail_to_deliver")
        .match_query(Matcher::UrlEncoded("symbol".into(), "GME".into()))
        .with_body(
            json!([{
                "symbol": "GME", "date": "2025-01-15", "quantity": 1200.0, "price": 24.5,
                "name": "GameStop Corp", "description": "COM"
            }])
            .to_string(),
        )
        .create_async()
        .await;
    let short_volume = server
        .mock("GET", "/stocks/v1/short-volume")
        .match_query(Matcher::UrlEncoded("ticker".into(), "AAPL".into()))
        .with_body(
            json!({"status": "OK", "results": [{
                "ticker": "AAPL", "date": "2025-01-15", "short_volume": 500.0,
                "exempt_volume": 5.0, "total_volume": 1000.0
            }]})
            .to_string(),
        )
        .create_async()
        .await;
    let failing = server
        .mock("GET", "/api/v4/fail_to_deliver")
        .match_query(Matcher::UrlEncoded("symbol".into(), "BAD".into()))
        .with_status(500)
        .create_async()
        .await;

    let providers = Arc::new(
        Providers::builder()
            .providers([Provider::Fmp, Provider::Polygon])
            .api_key(Provider::Fmp, "filings-service-fixture")
            .api_key(Provider::Polygon, "filings-service-fixture")
            .endpoint(Provider::Fmp, server.url())
            .endpoint(Provider::Polygon, server.url())
            .route(Capability::FILINGS, [Provider::Fmp])
            .route(Capability::FUNDAMENTALS, [Provider::Polygon])
            .build()
            .await
            .unwrap(),
    );
    let cache = Cache::new(None).await;

    let value = filings::get_congressional_trades(&cache, &providers, "AAPL")
        .await
        .unwrap();
    let rows = value.as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["symbol"], "AAPL");
    trades.assert_async().await;

    let value = filings::get_fails_to_deliver(&cache, &providers, "GME")
        .await
        .unwrap();
    let rows = value.as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["quantity"], 1200.0);
    assert_eq!(rows[0]["name"], "GameStop Corp");
    ftd.assert_async().await;

    let value = filings::get_short_volume(&cache, &providers, "AAPL")
        .await
        .unwrap();
    let rows = value.as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["short_volume"], 500.0);
    assert_eq!(rows[0]["short_exempt_volume"], 5.0);
    short_volume.assert_async().await;

    let err = filings::get_fails_to_deliver(&cache, &providers, "BAD")
        .await
        .unwrap_err();
    assert!(!err.to_string().is_empty());
    failing.assert_async().await;
}
