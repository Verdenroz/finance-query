use finance_query::{Capability, Provider, Providers};
use finance_query_server::cache::Cache;
use finance_query_server::services::market;
use mockito::{Matcher, Server};
use serde_json::json;
use std::sync::Arc;

// One test: the FMP endpoint override is process-global, so cases share a single server.
#[tokio::test]
async fn market_services_shape_fmp_responses_and_map_errors() {
    let mut server = Server::new_async().await;
    let performance = server
        .mock("GET", "/stable/sector-performance-snapshot")
        .match_query(Matcher::Any)
        .with_body(
            json!([
                {"date": "2025-01-15", "sector": "Technology", "exchange": "NASDAQ", "averageChange": 1.25},
                {"date": "2025-01-15", "sector": "Energy", "exchange": "NYSE", "averageChange": -0.5},
                {"date": "2025-01-15", "exchange": "NYSE", "averageChange": 9.9}
            ])
            .to_string(),
        )
        .create_async()
        .await;
    let pe = server
        .mock("GET", "/stable/sector-pe-snapshot")
        .match_query(Matcher::Any)
        .with_body(
            json!([{"date": "2025-01-15", "sector": "Technology", "exchange": "NASDAQ", "pe": 31.5}])
                .to_string(),
        )
        .create_async()
        .await;
    let history = server
        .mock("GET", "/stable/historical-sector-performance")
        .match_query(Matcher::UrlEncoded("sector".into(), "Technology".into()))
        .with_body(
            json!([
                {"date": "2025-01-15", "sector": "Technology", "exchange": "NASDAQ", "averageChange": 1.0},
                {"date": "2025-01-14", "sector": "Technology", "exchange": "NASDAQ", "averageChange": 0.5}
            ])
            .to_string(),
        )
        .create_async()
        .await;
    let other_sectors = server
        .mock("GET", "/stable/historical-sector-performance")
        .match_query(Matcher::Any)
        .with_body("[]")
        .expect_at_least(1)
        .create_async()
        .await;
    let changes = server
        .mock("GET", "/stable/historical-sp500-constituent")
        .match_query(Matcher::Any)
        .with_body(
            json!([{
                "date": "2025-01-10", "symbol": "NEW", "addedSecurity": "New Corp",
                "removedTicker": "OLD", "removedSecurity": "Old Corp", "reason": "Market cap"
            }])
            .to_string(),
        )
        .create_async()
        .await;

    let providers = Arc::new(
        Providers::builder()
            .providers([Provider::Fmp])
            .api_key(Provider::Fmp, "market-service-fixture")
            .endpoint(Provider::Fmp, server.url())
            .route(Capability::MARKET, [Provider::Fmp])
            .route(Capability::INDICES, [Provider::Fmp])
            .build()
            .await
            .unwrap(),
    );
    let cache = Cache::new(None).await;

    let value = market::get_sector_performance(&cache, &providers)
        .await
        .unwrap();
    let sectors = value.as_array().unwrap();
    assert_eq!(sectors.len(), 2, "rows without a sector are dropped");
    assert_eq!(sectors[0]["sector"], "Technology");
    assert_eq!(sectors[0]["change_percent"], 1.25);
    performance.assert_async().await;

    let value = market::get_sector_pe(&cache, &providers).await.unwrap();
    assert_eq!(value[0]["sector"], "Technology");
    assert_eq!(value[0]["pe"], 31.5);
    pe.assert_async().await;

    let value = market::get_sector_performance_history(&cache, &providers, 1)
        .await
        .unwrap();
    let days = value.as_array().unwrap();
    assert_eq!(days.len(), 1, "history is trimmed to the requested limit");
    assert_eq!(days[0]["date"], "2025-01-15");
    assert_eq!(days[0]["sectors"][0]["sector"], "Technology");
    history.assert_async().await;
    other_sectors.assert_async().await;

    let value = market::get_index_constituent_changes(&cache, &providers, "^GSPC")
        .await
        .unwrap();
    assert_eq!(value[0]["symbol"], "NEW");
    assert_eq!(value[0]["removed_ticker"], "OLD");
    changes.assert_async().await;
    changes.remove_async().await;

    let failing = server
        .mock("GET", "/stable/historical-sp500-constituent")
        .match_query(Matcher::Any)
        .with_status(500)
        .create_async()
        .await;

    let err = market::get_index_constituent_changes(&cache, &providers, "^GSPC")
        .await
        .unwrap_err();
    assert!(!err.to_string().is_empty());
    failing.assert_async().await;
}
