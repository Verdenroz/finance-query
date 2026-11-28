use finance_query::{Capability, Provider, Providers, Screener, Sector};
use finance_query_server::cache::Cache;
use finance_query_server::services::{commodity_futures, market};
use mockito::{Matcher, Server};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn market_services_reach_the_mocked_yahoo_and_map_errors() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/v1/test/getcrumb")
        .with_body("mock-crumb")
        .create_async()
        .await;
    let trending = server
        .mock("GET", "/v1/finance/trending/US")
        .match_query(Matcher::Any)
        .with_body(
            json!({"finance": {"result": [{"quotes": [{"symbol": "AAPL"}, {"symbol": "SPY"}]}]}})
                .to_string(),
        )
        .create_async()
        .await;
    let batch = server
        .mock("GET", "/v7/finance/quote")
        .match_query(Matcher::Any)
        .with_body(include_str!("../../tests/fixtures/yahoo/batch_quote.json"))
        .expect_at_least(2)
        .create_async()
        .await;
    let summary = server
        .mock("GET", "/v6/finance/quote/marketSummary")
        .match_query(Matcher::Any)
        .with_body(json!({"marketSummaryResponse": {"result": []}}).to_string())
        .create_async()
        .await;
    let screener = server
        .mock("GET", "/v1/finance/screener/predefined/saved")
        .match_query(Matcher::Any)
        .with_body(
            json!({"finance": {"result": [{"canonicalName": "DAY_GAINERS", "quotes": []}]}})
                .to_string(),
        )
        .create_async()
        .await;
    let sector = server
        .mock("GET", "/v1/finance/sectors/technology")
        .match_query(Matcher::Any)
        .with_body(json!({"data": {"name": "Technology", "key": "technology"}}).to_string())
        .create_async()
        .await;
    let industry = server
        .mock("GET", "/v1/finance/industries/semiconductors")
        .match_query(Matcher::Any)
        .with_body(json!({"data": {"name": "Semiconductors", "key": "semiconductors"}}).to_string())
        .create_async()
        .await;
    let futures = server
        .mock("GET", "/v10/finance/quoteSummary/ES=F")
        .match_query(Matcher::Any)
        .with_body(include_str!(
            "../../tests/fixtures/yahoo/quote_summary_price.json"
        ))
        .create_async()
        .await;
    let failing = server
        .mock("GET", "/v1/finance/sectors/energy")
        .match_query(Matcher::Any)
        .with_status(500)
        .create_async()
        .await;

    let providers = Arc::new(
        Providers::builder()
            .providers([Provider::Yahoo])
            .endpoint(Provider::Yahoo, server.url())
            .route(Capability::FUTURES, [Provider::Yahoo])
            .build()
            .await
            .unwrap(),
    );
    let cache = Cache::new(None).await;

    let value = providers
        .scope(market::get_trending(&cache, None))
        .await
        .unwrap();
    assert_eq!(value[0]["symbol"], "AAPL");
    trending.assert_async().await;

    providers
        .scope(market::get_indices(&cache, None))
        .await
        .unwrap();
    batch.assert_async().await;

    providers
        .scope(market::get_market_summary(&cache, None, None))
        .await
        .unwrap();
    summary.assert_async().await;

    providers
        .scope(market::get_screener(
            &cache,
            Screener::DayGainers,
            "day_gainers",
            5,
        ))
        .await
        .unwrap();
    screener.assert_async().await;

    let value = providers
        .scope(market::get_sector(
            &cache,
            Sector::Technology,
            "technology",
            None,
        ))
        .await
        .unwrap();
    assert_eq!(value["name"], "Technology");
    sector.assert_async().await;

    let value = providers
        .scope(market::get_industry(&cache, "semiconductors", None))
        .await
        .unwrap();
    assert_eq!(value["name"], "Semiconductors");
    industry.assert_async().await;

    providers
        .scope(commodity_futures::get_futures_quote(
            &cache, &providers, "ES=F",
        ))
        .await
        .unwrap();
    futures.assert_async().await;

    let err = providers
        .scope(market::get_sector(&cache, Sector::Energy, "energy", None))
        .await
        .unwrap_err();
    assert!(!err.to_string().is_empty());
    failing.assert_async().await;
}
