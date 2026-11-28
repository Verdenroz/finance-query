use finance_query::{
    FinanceError, Interval, Provider, Providers, Screener, Sector, TimeRange, finance, format::Raw,
};
use mockito::{Matcher, Mock, Server, ServerGuard};
use serde_json::json;

const SUMMARY: &str = include_str!("../fixtures/yahoo/quote_summary_price.json");
const BATCH: &str = include_str!("../fixtures/yahoo/batch_quote.json");

async fn crumb(server: &mut ServerGuard, hits: usize) -> Mock {
    server
        .mock("GET", "/v1/test/getcrumb")
        .with_body("mock-crumb")
        .expect(hits)
        .create_async()
        .await
}

async fn providers(server: &ServerGuard) -> Providers {
    Providers::builder()
        .endpoint(Provider::Yahoo, server.url())
        .build()
        .await
        .unwrap()
}

fn chart_body() -> String {
    json!({"chart": {"result": [{
        "meta": {"symbol": "AAPL", "currency": "USD"},
        "timestamp": [1705276800, 1705363200],
        "indicators": {"quote": [{
            "open": [1.0, 2.0], "high": [2.0, 3.0], "low": [0.5, 1.5],
            "close": [1.5, 2.5], "volume": [10, 20]
        }]}
    }], "error": null}})
    .to_string()
}

#[tokio::test]
async fn ticker_quote_hits_quote_summary_with_one_crumb() {
    let mut server = Server::new_async().await;
    let crumb = crumb(&mut server, 1).await;
    let summary = server
        .mock("GET", "/v10/finance/quoteSummary/AAPL")
        .match_query(Matcher::Any)
        .with_body(SUMMARY)
        .expect(1)
        .create_async()
        .await;
    let providers = providers(&server).await;
    let ticker = providers.ticker("AAPL").build().await.unwrap();
    let quote = ticker.quote::<Raw>().await.unwrap();
    assert_eq!(quote.symbol, "AAPL");
    crumb.assert_async().await;
    summary.assert_async().await;
}

#[tokio::test]
async fn tickers_quotes_and_charts_hit_the_mock() {
    let mut server = Server::new_async().await;
    let crumb = crumb(&mut server, 1).await;
    let quotes = server
        .mock("GET", "/v7/finance/quote")
        .match_query(Matcher::Any)
        .with_body(BATCH)
        .expect(1)
        .create_async()
        .await;
    let chart = server
        .mock("GET", "/v8/finance/chart/AAPL")
        .match_query(Matcher::Any)
        .with_body(chart_body())
        .expect(1)
        .create_async()
        .await;
    let providers = providers(&server).await;
    let tickers = providers.tickers(["AAPL"]).build().await.unwrap();
    assert!(!tickers.quotes().await.unwrap().quotes.is_empty());
    let charts = tickers
        .charts(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    assert_eq!(charts.charts.len(), 1);
    crumb.assert_async().await;
    quotes.assert_async().await;
    chart.assert_async().await;
}

#[tokio::test]
async fn futures_quote_hits_the_mock() {
    let mut server = Server::new_async().await;
    let _crumb = crumb(&mut server, 1).await;
    let summary = server
        .mock("GET", "/v10/finance/quoteSummary/ES=F")
        .match_query(Matcher::Any)
        .with_body(SUMMARY)
        .expect(1)
        .create_async()
        .await;
    let providers = providers(&server).await;
    providers.futures("ES=F").quote().await.unwrap();
    summary.assert_async().await;
}

#[tokio::test]
async fn a_rejected_crumb_refreshes_once() {
    let mut server = Server::new_async().await;
    let crumb = crumb(&mut server, 2).await;
    let rejected = server
        .mock("GET", "/v10/finance/quoteSummary/AAPL")
        .match_query(Matcher::Any)
        .with_status(401)
        .expect(1)
        .create_async()
        .await;
    let accepted = server
        .mock("GET", "/v10/finance/quoteSummary/AAPL")
        .match_query(Matcher::Any)
        .with_body(SUMMARY)
        .expect(1)
        .create_async()
        .await;
    let providers = providers(&server).await;
    let ticker = providers.ticker("AAPL").build().await.unwrap();
    ticker.quote::<Raw>().await.unwrap();
    crumb.assert_async().await;
    rejected.assert_async().await;
    accepted.assert_async().await;
}

#[tokio::test]
async fn a_redirect_to_another_host_errors_without_being_followed() {
    let mut other = Server::new_async().await;
    let elsewhere = other
        .mock("GET", Matcher::Any)
        .expect(0)
        .create_async()
        .await;
    let mut server = Server::new_async().await;
    let _crumb = crumb(&mut server, 1).await;
    server
        .mock("GET", "/v10/finance/quoteSummary/AAPL")
        .match_query(Matcher::Any)
        .with_status(302)
        .with_header("location", &other.url())
        .create_async()
        .await;
    let providers = providers(&server).await;
    let ticker = providers.ticker("AAPL").build().await.unwrap();
    assert!(ticker.quote::<Raw>().await.is_err());
    elsewhere.assert_async().await;
}

#[tokio::test]
async fn the_builder_validates_a_yahoo_endpoint_without_a_key() {
    for endpoint in [
        "http://example.com",
        "http://127.0.0.1:9/path",
        "http://127.0.0.1:9/?q=1",
    ] {
        let result = Providers::builder()
            .endpoint(Provider::Yahoo, endpoint)
            .build()
            .await;
        assert!(
            matches!(result, Err(FinanceError::InvalidParameter { .. })),
            "{endpoint} should be rejected"
        );
    }
    Providers::builder()
        .endpoint(Provider::Yahoo, "http://127.0.0.1:9")
        .build()
        .await
        .unwrap();
}

#[tokio::test]
async fn scope_points_finance_functions_at_the_mock() {
    let mut server = Server::new_async().await;
    let _crumb = crumb(&mut server, 1).await;
    let trending = server
        .mock("GET", "/v1/finance/trending/US")
        .match_query(Matcher::Any)
        .with_body(
            json!({"finance": {"result": [{"quotes": [{"symbol": "NVDA"}]}]}}).to_string(),
        )
        .expect(1)
        .create_async()
        .await;
    let summary = server
        .mock("GET", "/v6/finance/quote/marketSummary")
        .match_query(Matcher::Any)
        .with_body(json!({"marketSummaryResponse": {"result": []}}).to_string())
        .expect(1)
        .create_async()
        .await;
    let screener = server
        .mock("GET", "/v1/finance/screener/predefined/saved")
        .match_query(Matcher::Any)
        .with_body(
            json!({"finance": {"result": [{"canonicalName": "DAY_GAINERS", "quotes": []}]}})
                .to_string(),
        )
        .expect(1)
        .create_async()
        .await;
    let sector = server
        .mock("GET", "/v1/finance/sectors/technology")
        .match_query(Matcher::Any)
        .with_body(json!({"data": {"name": "Technology", "key": "technology"}}).to_string())
        .expect(1)
        .create_async()
        .await;
    let providers = providers(&server).await;

    let quotes = providers.scope(finance::trending(None)).await.unwrap();
    assert_eq!(quotes[0].symbol, "NVDA");
    providers
        .scope(finance::market_summary(None))
        .await
        .unwrap();
    providers
        .scope(finance::screener(Screener::DayGainers, 5))
        .await
        .unwrap();
    let data = providers
        .scope(finance::sector(Sector::Technology))
        .await
        .unwrap();
    assert_eq!(data.name, "Technology");
    trending.assert_async().await;
    summary.assert_async().await;
    screener.assert_async().await;
    sector.assert_async().await;
}
