use finance_query::{Interval, Provider, Providers, TimeRange};
use finance_query_server::cache::Cache;
use finance_query_server::services::{chart, quote};
use mockito::{Matcher, Server};
use serde_json::json;

#[tokio::test]
async fn quote_and_chart_services_reach_the_mocked_yahoo() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/v1/test/getcrumb")
        .with_body("mock-crumb")
        .create_async()
        .await;
    let summary = server
        .mock("GET", "/v10/finance/quoteSummary/AAPL")
        .match_query(Matcher::Any)
        .with_body(include_str!(
            "../../tests/fixtures/yahoo/quote_summary_price.json"
        ))
        .create_async()
        .await;
    let batch = server
        .mock("GET", "/v7/finance/quote")
        .match_query(Matcher::Any)
        .with_body(include_str!("../../tests/fixtures/yahoo/batch_quote.json"))
        .create_async()
        .await;
    let history = server
        .mock("GET", "/v8/finance/chart/AAPL")
        .match_query(Matcher::Any)
        .with_body(
            json!({"chart": {"result": [{
                "meta": {"symbol": "AAPL", "currency": "USD"},
                "timestamp": [1705276800, 1705363200],
                "indicators": {"quote": [{
                    "open": [1.0, 2.0], "high": [2.0, 3.0], "low": [0.5, 1.5],
                    "close": [1.5, 2.5], "volume": [10, 20]
                }]}
            }], "error": null}})
            .to_string(),
        )
        .create_async()
        .await;
    let providers = Providers::builder()
        .providers([Provider::Yahoo])
        .endpoint(Provider::Yahoo, server.url())
        .build()
        .await
        .unwrap();
    let cache = Cache::new(None).await;

    let value = providers
        .scope(quote::get_quote(&cache, "AAPL", false, None))
        .await
        .unwrap();
    assert_eq!(value["symbol"], "AAPL");
    summary.assert_async().await;

    let value = providers
        .scope(quote::get_quotes(&cache, vec!["AAPL"], false, None))
        .await
        .unwrap();
    assert!(value["quotes"]["AAPL"].is_object());
    batch.assert_async().await;

    let value = providers
        .scope(chart::get_chart(
            &cache,
            "AAPL",
            Interval::OneDay,
            TimeRange::OneMonth,
            None,
            None,
            false,
            false,
        ))
        .await
        .unwrap();
    assert_eq!(value["candles"].as_array().unwrap().len(), 2);
    history.assert_async().await;
}
