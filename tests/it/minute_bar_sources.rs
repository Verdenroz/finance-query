#![cfg(all(feature = "polygon", feature = "fmp"))]

use finance_query::{
    Capability, FinanceError, Interval, PriceAdjustment, Provider, Providers, StockBarsRequest,
};
use mockito::{Matcher, Server};

async fn providers(server: &Server, provider: Provider) -> Providers {
    Providers::builder()
        .providers([provider])
        .api_key(provider, "bars-fixture")
        .endpoint(provider, server.url())
        .requests_per_minute(provider, 600_000)
        .route(Capability::DISCOVERY, [provider])
        .route(Capability::CHART, [provider])
        .build()
        .await
        .unwrap()
}

fn window(from: &str, to: &str) -> Matcher {
    Matcher::AllOf(vec![
        Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
        Matcher::UrlEncoded("from".into(), from.into()),
        Matcher::UrlEncoded("to".into(), to.into()),
    ])
}

#[tokio::test]
async fn fmp_minute_pages_walk_weekday_windows_in_new_york_time() {
    let mut server = Server::new_async().await;
    let first = server
        .mock("GET", "/stable/historical-chart/1min")
        .match_query(window("2025-06-02", "2025-06-04"))
        .with_body(
            r#"[{"date":"2025-06-04 16:00:00","open":3,"high":3,"low":3,"close":3,"volume":7},
                {"date":"2025-06-02 09:30:00","open":1,"high":2,"low":0.5,"close":1.5,"volume":100.25}]"#,
        )
        .expect(1)
        .create_async()
        .await;
    let second = server
        .mock("GET", "/stable/historical-chart/1min")
        .match_query(window("2025-06-05", "2025-06-06"))
        .with_body("[]")
        .expect(1)
        .create_async()
        .await;
    let request = StockBarsRequest::new(
        "AAPL",
        "2025-06-02",
        "2025-06-06",
        Interval::OneMinute,
        PriceAdjustment::SplitAdjusted,
    )
    .unwrap();
    let market = providers(&server, Provider::Fmp).await.market();
    let page = market.stock_bars_page(&request, None).await.unwrap();
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].timestamp_ms, 1_748_871_000_000, "09:30 EDT");
    assert_eq!(page.items[0].volume, 100.25);
    assert_eq!(page.items[1].timestamp_ms, 1_749_067_200_000, "16:00 EDT");
    assert_eq!(page.adjusted, Some(true));
    let cursor: finance_query::PageCursor =
        serde_json::from_str(&serde_json::to_string(&page.next.unwrap()).unwrap()).unwrap();
    let last = market
        .stock_bars_page(&request, Some(&cursor))
        .await
        .unwrap();
    assert!(last.items.is_empty() && last.next.is_none());
    first.assert_async().await;
    second.assert_async().await;
}

#[tokio::test]
async fn fmp_daily_pages_are_stamped_at_new_york_midnight() {
    let mut server = Server::new_async().await;
    let daily = server
        .mock("GET", "/stable/historical-price-eod/full")
        .match_query(window("2025-01-02", "2025-01-03"))
        .with_body(r#"[{"date":"2025-01-02","open":1,"high":1,"low":1,"close":1,"volume":5}]"#)
        .expect(1)
        .create_async()
        .await;
    let request = StockBarsRequest::new(
        "AAPL",
        "2025-01-02",
        "2025-01-03",
        Interval::OneDay,
        PriceAdjustment::SplitAdjusted,
    )
    .unwrap();
    let page = providers(&server, Provider::Fmp)
        .await
        .market()
        .stock_bars_page(&request, None)
        .await
        .unwrap();
    assert_eq!(page.items[0].timestamp_ms, 1_735_794_000_000, "00:00 EST");
    assert!(page.next.is_none());
    daily.assert_async().await;
}

#[tokio::test]
async fn fmp_refuses_unadjusted_bars_and_rows_outside_the_window() {
    let mut server = Server::new_async().await;
    let _stray = server
        .mock("GET", "/stable/historical-chart/1min")
        .match_query(Matcher::Any)
        .with_body(
            r#"[{"date":"2025-05-30 09:30:00","open":1,"high":1,"low":1,"close":1,"volume":1}]"#,
        )
        .create_async()
        .await;
    let market = providers(&server, Provider::Fmp).await.market();
    let unadjusted = StockBarsRequest::new(
        "AAPL",
        "2025-06-02",
        "2025-06-04",
        Interval::OneMinute,
        PriceAdjustment::Unadjusted,
    )
    .unwrap();
    assert!(matches!(
        market.stock_bars_page(&unadjusted, None).await,
        Err(FinanceError::InvalidParameter { .. })
    ));
    let adjusted = StockBarsRequest::new(
        "AAPL",
        "2025-06-02",
        "2025-06-04",
        Interval::OneMinute,
        PriceAdjustment::SplitAdjusted,
    )
    .unwrap();
    assert!(matches!(
        market.stock_bars_page(&adjusted, None).await,
        Err(FinanceError::ResponseStructureError { .. })
    ));
}

#[tokio::test]
async fn polygon_ticker_changes_follow_a_figi_through_renames() {
    let mut server = Server::new_async().await;
    let events = server
        .mock("GET", "/vX/reference/tickers/BBG000MM2P62/events")
        .match_query(Matcher::UrlEncoded("types".into(), "ticker_change".into()))
        .with_body(
            r#"{"status":"OK","results":{"name":"Meta Platforms","events":[
                {"ticker_change":{"ticker":"META"},"type":"ticker_change","date":"2022-06-09"},
                {"ticker_change":{"ticker":"FB"},"type":"ticker_change","date":"2012-05-18"}]}}"#,
        )
        .expect(1)
        .create_async()
        .await;
    let changes = providers(&server, Provider::Polygon)
        .await
        .discovery()
        .ticker_changes("BBG000MM2P62")
        .await
        .unwrap();
    let pairs: Vec<_> = changes
        .iter()
        .map(|c| (c.date.as_str(), c.ticker.as_str()))
        .collect();
    assert_eq!(pairs, [("2012-05-18", "FB"), ("2022-06-09", "META")]);
    events.assert_async().await;
}

#[tokio::test]
async fn polygon_plan_window_refusals_are_not_bad_keys() {
    let mut server = Server::new_async().await;
    let _refused = server
        .mock("GET", Matcher::Regex("^/v2/aggs/".into()))
        .match_query(Matcher::Any)
        .with_status(403)
        .with_body(
            r#"{"status":"NOT_AUTHORIZED","message":"Your plan doesn't include this data timeframe. Please upgrade your plan"}"#,
        )
        .create_async()
        .await;
    let request = StockBarsRequest::new(
        "AAPL",
        "2010-06-01",
        "2010-06-01",
        Interval::OneMinute,
        PriceAdjustment::SplitAdjusted,
    )
    .unwrap();
    let error = providers(&server, Provider::Polygon)
        .await
        .market()
        .stock_bars_page(&request, None)
        .await
        .unwrap_err();
    assert!(
        matches!(error, FinanceError::NotEntitled { .. }),
        "{error:?}"
    );
}
