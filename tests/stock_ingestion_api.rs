#![cfg(all(feature = "polygon", feature = "fmp"))]

use finance_query::{
    Capability, FinanceError, Interval, PageCursor, PriceAdjustment, Provider, Providers,
    StockBarsRequest, StockListingRequest,
};
use mockito::{Matcher, Server};
use serde_json::json;

async fn providers(server: &Server, provider: Provider) -> Providers {
    Providers::builder()
        .providers([provider])
        .api_key(provider, format!("fixture-{}", server.url()))
        .endpoint(provider, server.url())
        .requests_per_minute(provider, 600_000)
        .route(Capability::DISCOVERY, [provider])
        .route(Capability::CHART, [provider])
        .route(Capability::FUNDAMENTALS, [provider])
        .build()
        .await
        .unwrap()
}

#[tokio::test]
async fn stock_ingestion_directory_resumes_after_rebuilding_client() {
    let mut server = Server::new_async().await;
    let next = format!(
        "{}/v3/reference/tickers?cursor=next&apiKey=do-not-persist",
        server.url()
    );
    let first_body = json!({
        "status": "OK",
        "results": [{
            "ticker": "OLD",
            "active": false,
            "cik": "00042",
            "composite_figi": "SECURITY",
            "share_class_figi": "CLASS",
            "delisted_utc": "2021-01-01T00:00:00Z"
        }],
        "next_url": next,
    });
    let first = server
        .mock("GET", "/v3/reference/tickers")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("date".into(), "2020-01-02".into()),
            Matcher::UrlEncoded("active".into(), "false".into()),
            Matcher::UrlEncoded("type".into(), "CS".into()),
            Matcher::Regex("^market=stocks&".into()),
        ]))
        .with_body(first_body.to_string())
        .create_async()
        .await;
    let second = server
        .mock("GET", "/v3/reference/tickers")
        .match_query(Matcher::UrlEncoded("cursor".into(), "next".into()))
        .with_body(r#"{"status":"OK","results":[]}"#)
        .create_async()
        .await;
    let request = StockListingRequest::new("2020-01-02", false)
        .unwrap()
        .with_stock_type("CS");
    let client = providers(&server, Provider::Polygon).await;
    let page = client
        .discovery()
        .stock_listings_page(&request, None)
        .await
        .unwrap();
    assert_eq!(page.items[0].cik.as_deref(), Some("00042"));
    assert_eq!(page.items[0].composite_figi.as_deref(), Some("SECURITY"));
    let saved = serde_json::to_string(&page.next.unwrap()).unwrap();
    assert!(!saved.contains("apiKey"));
    assert!(!saved.contains("do-not-persist"));
    drop(client);
    let cursor: PageCursor = serde_json::from_str(&saved).unwrap();
    let client = providers(&server, Provider::Polygon).await;
    let page = client
        .discovery()
        .stock_listings_page(&request, Some(&cursor))
        .await
        .unwrap();
    assert!(page.items.is_empty());
    assert!(page.next.is_none());
    let changed = StockListingRequest::new("2020-01-03", false)
        .unwrap()
        .with_stock_type("CS");
    assert!(
        client
            .discovery()
            .stock_listings_page(&changed, Some(&cursor))
            .await
            .is_err()
    );
    first.assert_async().await;
    second.assert_async().await;
}

#[tokio::test]
async fn stock_ingestion_listings_keep_rows_whose_current_status_differs() {
    let mut server = Server::new_async().await;
    let body = json!({
        "status": "OK",
        "results": [
            {"ticker": "AAPL", "active": true, "locale": "us", "type": "CS"},
            {
                "ticker": "TWTR",
                "active": false,
                "locale": "us",
                "type": "ADRC",
                "delisted_utc": "2022-11-08T00:00:00Z"
            }
        ]
    });
    let fixture = server
        .mock("GET", "/v3/reference/tickers")
        .match_query(Matcher::UrlEncoded("active".into(), "true".into()))
        .with_body(body.to_string())
        .create_async()
        .await;
    let client = providers(&server, Provider::Polygon).await;
    let request = StockListingRequest::new("2020-01-02", true)
        .unwrap()
        .with_stock_type("CS");
    let page = client
        .discovery()
        .stock_listings_page(&request, None)
        .await
        .unwrap();
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[1].active, Some(false));
    assert_eq!(page.items[1].stock_type.as_deref(), Some("ADRC"));
    assert_eq!(
        page.items[1].delisted_utc.as_deref(),
        Some("2022-11-08T00:00:00Z")
    );
    fixture.assert_async().await;
}

#[tokio::test]
async fn stock_ingestion_bars_preserve_values_and_reject_unsafe_continuations() {
    let mut server = Server::new_async().await;
    let path = "/v2/aggs/ticker/AAPL/range/1/minute/2020-01-02/2020-01-02";
    // Polygon's next_url moves `from` to the next bar's timestamp and keeps `to`.
    let next_path = "/v2/aggs/ticker/AAPL/range/1/minute/1577973720123/2020-01-02";
    let first_body = json!({
        "status": "OK",
        "ticker": "AAPL",
        "adjusted": false,
        "resultsCount": 2,
        "results": [
            {"t": 1577973600123_i64, "o": 1.1, "h": 2.2, "l": 1.0, "c": 2.0, "v": 123.75},
            {
                "t": 1577973660123_i64, "o": 2.0, "h": 2.5, "l": 1.5, "c": 2.25,
                "v": 18.5, "n": 17, "vw": 2.125
            }
        ],
        "next_url": format!("{}{next_path}?cursor=bGltaXQ9MiZzb3J0PWFzYw", server.url()),
    });
    let second_body = json!({
        "status": "OK",
        "ticker": "AAPL",
        "adjusted": false,
        "resultsCount": 1,
        "results": [
            {"t": 1577973720123_i64, "o": 2.25, "h": 2.5, "l": 2.0, "c": 2.4, "v": 7.0}
        ],
    });
    let first = server
        .mock("GET", path)
        .match_query(Matcher::UrlEncoded("adjusted".into(), "false".into()))
        .with_body(first_body.to_string())
        .expect(1)
        .create_async()
        .await;
    let second = server
        .mock("GET", next_path)
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("cursor".into(), "bGltaXQ9MiZzb3J0PWFzYw".into()),
            Matcher::UrlEncoded("adjusted".into(), "false".into()),
        ]))
        .with_body(second_body.to_string())
        .expect(1)
        .create_async()
        .await;
    let client = providers(&server, Provider::Polygon).await;
    let request = StockBarsRequest::new(
        "AAPL",
        "2020-01-02",
        "2020-01-02",
        Interval::OneMinute,
        PriceAdjustment::Unadjusted,
    )
    .unwrap();
    let page = client
        .market()
        .stock_bars_page(&request, None)
        .await
        .unwrap();
    assert_eq!(page.items[0].timestamp_ms, 1577973600123);
    assert_eq!(page.items[0].volume, 123.75);
    assert_eq!(page.items[0].transactions, None);
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[1].transactions, Some(17));
    assert_eq!(page.items[1].vwap, Some(2.125));
    assert_eq!(page.provider_id, Provider::Polygon);
    let cursor = page.next.unwrap();
    let saved = serde_json::to_string(&cursor).unwrap();
    let restored: PageCursor = serde_json::from_str(&saved).unwrap();
    let last = client
        .market()
        .stock_bars_page(&request, Some(&restored))
        .await
        .unwrap();
    assert_eq!(last.items[0].timestamp_ms, 1577973720123);
    assert!(last.next.is_none());

    let aggs = |from: &str, to: &str| format!("/v2/aggs/ticker/AAPL/range/1/minute/{from}/{to}");
    for target in [
        format!("https://other.example{next_path}?cursor=x"),
        format!("{}/v3/reference/tickers?cursor=x", server.url()),
        format!("{}{next_path}?adjusted=true", server.url()),
        format!(
            "{}/v2/aggs/ticker/MSFT/range/1/minute/1577973720123/2020-01-02?cursor=x",
            server.url()
        ),
        format!(
            "{}/v2/aggs/ticker/AAPL/range/1/day/1577973720123/2020-01-02?cursor=x",
            server.url()
        ),
        format!(
            "{}{}?cursor=x",
            server.url(),
            aggs("1577836800000", "2020-01-02")
        ),
        format!(
            "{}{}?cursor=x",
            server.url(),
            aggs("1578268800000", "2020-01-02")
        ),
        format!(
            "{}{}?cursor=x",
            server.url(),
            aggs("2020-01-02", "2020-01-09")
        ),
        format!("{}{}?cursor=x", server.url(), aggs("soon", "2020-01-02")),
    ] {
        let mut value = serde_json::to_value(&cursor).unwrap();
        value["target"] = json!(target);
        let bad = serde_json::from_value(value).unwrap();
        assert!(
            client
                .market()
                .stock_bars_page(&request, Some(&bad))
                .await
                .is_err(),
            "{target}"
        );
    }
    first.assert_async().await;
    second.assert_async().await;
}

#[tokio::test]
async fn stock_ingestion_details_and_types_use_real_public_routes() {
    let mut server = Server::new_async().await;
    let details_body = json!({
        "status": "OK",
        "results": {
            "ticker": "BRK.B",
            "cik": "0001",
            "composite_figi": "F1",
            "share_class_figi": "F2",
            "active": false,
            "list_date": "1996-05-09",
            "delisted_utc": "2025-01-01T00:00:00Z"
        }
    });
    let details = server
        .mock("GET", "/v3/reference/tickers/BRK.B")
        .match_query(Matcher::UrlEncoded("date".into(), "2020-01-02".into()))
        .with_body(details_body.to_string())
        .create_async()
        .await;
    let types = server
        .mock("GET", "/v3/reference/tickers/types")
        .match_query(Matcher::UrlEncoded("asset_class".into(), "stocks".into()))
        .with_body(
            r#"{"status":"OK","results":[{"code":"NEW","description":"Future stock type"}]}"#,
        )
        .create_async()
        .await;
    let client = providers(&server, Provider::Polygon).await;
    let row = client
        .discovery()
        .details_at("BRK.B", "2020-01-02")
        .await
        .unwrap();
    assert_eq!(row.share_class_figi.as_deref(), Some("F2"));
    assert_eq!(row.active, Some(false));
    assert_eq!(
        client.discovery().stock_types("us").await.unwrap()[0].code,
        "NEW"
    );
    details.assert_async().await;
    types.assert_async().await;
}

#[tokio::test]
async fn stock_ingestion_fmp_profile_preserves_identity_and_rejects_mismatches() {
    let mut server = Server::new_async().await;
    let good_body = json!([{
        "symbol": "BRK-B",
        "cik": "00042",
        "ipoDate": "1996-05-09",
        "companyName": "Berkshire",
        "marketCap": 123.5
    }]);
    let good = server
        .mock("GET", "/stable/profile")
        .match_query(Matcher::UrlEncoded("symbol".into(), "BRK-B".into()))
        .with_body(good_body.to_string())
        .create_async()
        .await;
    let bad = server
        .mock("GET", "/stable/profile")
        .match_query(Matcher::UrlEncoded("symbol".into(), "WRONG".into()))
        .with_body(r#"[{"symbol":"OTHER","cik":"42"}]"#)
        .create_async()
        .await;
    let client = providers(&server, Provider::Fmp).await;
    let row = client
        .ticker("BRK-B")
        .build()
        .await
        .unwrap()
        .company_profile()
        .await
        .unwrap();
    assert_eq!(row.cik.as_deref(), Some("00042"));
    assert_eq!(row.ipo_date.as_deref(), Some("1996-05-09"));
    assert_eq!(row.market_capitalization, Some(123.5));
    assert!(
        client
            .ticker("WRONG")
            .build()
            .await
            .unwrap()
            .company_profile()
            .await
            .is_err()
    );
    good.assert_async().await;
    bad.assert_async().await;
}

#[tokio::test]
async fn stock_ingestion_fmp_profile_accepts_blank_fields_and_symbol_case() {
    let mut server = Server::new_async().await;
    let body = json!([{
        "symbol": "AAPL",
        "cik": "",
        "ipoDate": "",
        "isin": "",
        "companyName": "Apple Inc."
    }]);
    let fixture = server
        .mock("GET", "/stable/profile")
        .match_query(Matcher::UrlEncoded("symbol".into(), "aapl".into()))
        .with_body(body.to_string())
        .create_async()
        .await;
    let client = providers(&server, Provider::Fmp).await;
    let row = client
        .ticker("aapl")
        .build()
        .await
        .unwrap()
        .company_profile()
        .await
        .unwrap();
    assert_eq!(row.symbol.as_deref(), Some("AAPL"));
    assert_eq!(row.cik, None);
    assert_eq!(row.ipo_date, None);
    assert_eq!(row.isin, None);
    assert_eq!(row.name.as_deref(), Some("Apple Inc."));
    fixture.assert_async().await;
}

#[tokio::test]
async fn stock_ingestion_empty_and_retry_responses_remain_distinct() {
    for body in [
        r#"{"status":"OK"}"#,
        r#"{"status":"OK","resultsCount":1,"results":[]}"#,
        r#"{"status":"ERROR","error":"not entitled"}"#,
    ] {
        let mut server = Server::new_async().await;
        let fixture = server
            .mock("GET", Matcher::Any)
            .with_body(body)
            .create_async()
            .await;
        let client = providers(&server, Provider::Polygon).await;
        let request = StockBarsRequest::new(
            "AAPL",
            "2020-01-02",
            "2020-01-02",
            Interval::OneDay,
            PriceAdjustment::Unadjusted,
        )
        .unwrap();
        assert!(
            client
                .market()
                .stock_bars_page(&request, None)
                .await
                .is_err()
        );
        fixture.assert_async().await;
    }
    let mut server = Server::new_async().await;
    let fixture = server
        .mock("GET", Matcher::Any)
        .with_status(429)
        .with_header("retry-after", "7")
        .create_async()
        .await;
    let client = providers(&server, Provider::Polygon).await;
    assert!(matches!(
        client.discovery().stock_types("us").await,
        Err(FinanceError::RateLimited {
            retry_after: Some(7)
        })
    ));
    fixture.assert_async().await;
}

#[tokio::test]
async fn stock_ingestion_final_empty_page_and_optional_profile_fields() {
    let mut server = Server::new_async().await;
    let empty = server
        .mock(
            "GET",
            "/v2/aggs/ticker/NEW/range/1/day/2020-01-01/2020-01-02",
        )
        .match_query(Matcher::Any)
        .with_body(r#"{"status":"OK","ticker":"NEW","resultsCount":0}"#)
        .create_async()
        .await;
    let client = providers(&server, Provider::Polygon).await;
    let request = StockBarsRequest::new(
        "NEW",
        "2020-01-01",
        "2020-01-02",
        Interval::OneDay,
        PriceAdjustment::Unadjusted,
    )
    .unwrap();
    let page = client
        .market()
        .stock_bars_page(&request, None)
        .await
        .unwrap();
    assert!(page.items.is_empty() && page.next.is_none());
    assert_eq!(page.results_count, Some(0));
    empty.assert_async().await;

    let profile = server
        .mock("GET", "/stable/profile")
        .match_query(Matcher::Any)
        .with_body(r#"[{"symbol":"NEW"}]"#)
        .create_async()
        .await;
    let client = providers(&server, Provider::Fmp).await;
    let row = client
        .ticker("NEW")
        .build()
        .await
        .unwrap()
        .company_profile()
        .await
        .unwrap();
    assert!(row.cik.is_none() && row.ipo_date.is_none());
    profile.assert_async().await;
}

#[tokio::test]
async fn stock_ingestion_failures_never_become_empty_history() {
    for status in [401, 403, 500] {
        let mut server = Server::new_async().await;
        let fixture = server
            .mock("GET", Matcher::Any)
            .with_status(status)
            .create_async()
            .await;
        let client = providers(&server, Provider::Polygon).await;
        let error = client
            .discovery()
            .details_at("AAPL", "2020-01-01")
            .await
            .unwrap_err();
        assert!(!format!("{error:?}").contains("fixture-"));
        if status == 500 {
            assert!(matches!(
                error,
                FinanceError::ServerError { status: 500, .. }
            ));
        } else {
            assert!(matches!(error, FinanceError::AuthenticationFailed { .. }));
        }
        fixture.assert_async().await;
    }
    for body in [
        "[]",
        r#"[{"symbol":"NEW"},{"symbol":"NEW"}]"#,
        r#"{"Error Message":"API key fixture-private is invalid"}"#,
    ] {
        let mut server = Server::new_async().await;
        let fixture = server
            .mock("GET", "/stable/profile")
            .match_query(Matcher::Any)
            .with_body(body)
            .create_async()
            .await;
        let client = providers(&server, Provider::Fmp).await;
        assert!(
            client
                .ticker("NEW")
                .build()
                .await
                .unwrap()
                .company_profile()
                .await
                .is_err()
        );
        fixture.assert_async().await;
    }
}

#[tokio::test]
async fn stock_ingestion_profile_body_is_bounded() {
    let mut server = Server::new_async().await;
    let body = format!(
        "[{{\"symbol\":\"NEW\",\"description\":\"{}\"}}]",
        "x".repeat(1024 * 1024)
    );
    let fixture = server
        .mock("GET", "/stable/profile")
        .match_query(Matcher::Any)
        .with_body(body)
        .create_async()
        .await;
    let client = providers(&server, Provider::Fmp).await;
    let error = client
        .ticker("NEW")
        .build()
        .await
        .unwrap()
        .company_profile()
        .await
        .unwrap_err();
    assert!(error.to_string().contains("byte limit"));
    fixture.assert_async().await;
}

#[tokio::test]
async fn stock_ingestion_malformed_profile_cannot_echo_the_api_key() {
    let mut server = Server::new_async().await;
    let key = format!("fixture-{}", server.url());
    let fixture = server
        .mock("GET", "/stable/profile")
        .match_query(Matcher::Any)
        .with_body(json!([{"symbol":"NEW","marketCap":key}]).to_string())
        .create_async()
        .await;
    let client = providers(&server, Provider::Fmp).await;
    let error = client
        .ticker("NEW")
        .build()
        .await
        .unwrap()
        .company_profile()
        .await
        .unwrap_err();
    assert!(!format!("{error:?}").contains(&key));
    assert!(!error.to_string().contains(&key));
    fixture.assert_async().await;
}

#[test]
fn stock_ingestion_compatibility_with_older_serialized_models() {
    let profile: finance_query::CompanyProfile =
        serde_json::from_value(json!({"symbol":"AAPL"})).unwrap();
    let value = serde_json::to_value(profile).unwrap();
    assert!(value.get("cik").is_none());
    assert!(value.get("ipo_date").is_none());
    let details: finance_query::SymbolDetails =
        serde_json::from_value(json!({"symbol":"AAPL"})).unwrap();
    assert!(
        serde_json::to_value(details)
            .unwrap()
            .get("composite_figi")
            .is_none()
    );
    assert!(StockListingRequest::new("2020-02-30", true).is_err());
    assert!(
        StockBarsRequest::new(
            "AAPL",
            "2020-01-02",
            "2020-01-01",
            Interval::OneDay,
            PriceAdjustment::Unadjusted
        )
        .is_err()
    );
}

#[test]
fn stock_ingestion_connection_cache_survives_runtime_replacement() {
    let mut server = Server::new();
    let fixture = server
        .mock("GET", "/v3/reference/tickers/types")
        .match_query(Matcher::Any)
        .with_body(r#"{"status":"OK","results":[{"code":"CS"}]}"#)
        .expect(3)
        .create();
    let first = tokio::runtime::Runtime::new().unwrap();
    let client = first.block_on(async {
        let client = providers(&server, Provider::Polygon).await;
        assert_eq!(
            client.discovery().stock_types("us").await.unwrap()[0].code,
            "CS"
        );
        assert_eq!(
            client.discovery().stock_types("us").await.unwrap()[0].code,
            "CS"
        );
        client
    });
    drop(first);
    let second = tokio::runtime::Runtime::new().unwrap();
    second.block_on(async {
        assert_eq!(
            client.discovery().stock_types("us").await.unwrap()[0].code,
            "CS"
        );
    });
    fixture.assert();
}

#[tokio::test]
async fn stock_ingestion_continuation_never_falls_back_even_with_parallel_routes() {
    use finance_query::ProviderCore;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    struct Fallback(Arc<AtomicUsize>);
    impl finance_query::ProviderCore for Fallback {
        fn id(&self) -> Provider {
            Provider::custom("stock-page-fallback")
        }
    }
    #[finance_query::async_trait]
    impl finance_query::ChartProvider for Fallback {
        async fn fetch_chart(
            &self,
            _: &str,
            _: Interval,
            _: finance_query::TimeRange,
        ) -> finance_query::Result<finance_query::Chart> {
            Err(self.not_supported(finance_query::Operation::Chart))
        }
        async fn fetch_stock_bars_page(
            &self,
            _: &StockBarsRequest,
            _: Option<&PageCursor>,
        ) -> finance_query::Result<finance_query::ProviderPage<finance_query::StockBar>> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Err(self.not_supported(finance_query::Operation::StockBarsPage))
        }
    }
    #[finance_query::async_trait]
    impl finance_query::ProviderAdapter for Fallback {
        fn as_chart(&self) -> Option<&dyn finance_query::ChartProvider> {
            Some(self)
        }
    }
    let mut server = Server::new_async().await;
    let path = "/v2/aggs/ticker/NEW/range/1/day/2020-01-01/2020-01-02";
    let first_body = json!({
        "status": "OK",
        "resultsCount": 0,
        "next_url": format!("{}{path}?cursor=blocked", server.url()),
    });
    let first = server
        .mock("GET", path)
        .match_query(Matcher::Regex("^adjusted=".into()))
        .with_body(first_body.to_string())
        .create_async()
        .await;
    let blocked = server
        .mock("GET", path)
        .match_query(Matcher::UrlEncoded("cursor".into(), "blocked".into()))
        .with_status(403)
        .create_async()
        .await;
    let request = StockBarsRequest::new(
        "NEW",
        "2020-01-01",
        "2020-01-02",
        Interval::OneDay,
        PriceAdjustment::Unadjusted,
    )
    .unwrap();
    let cursor = providers(&server, Provider::Polygon)
        .await
        .market()
        .stock_bars_page(&request, None)
        .await
        .unwrap()
        .next
        .unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let client = Providers::builder()
        .providers([Provider::Polygon])
        .api_key(Provider::Polygon, "fixture-continuation")
        .endpoint(Provider::Polygon, server.url())
        .with_adapter(Arc::new(Fallback(Arc::clone(&calls))))
        .route_with(
            Capability::CHART,
            [Provider::Polygon, Provider::custom("stock-page-fallback")],
            finance_query::Fetch::Parallel,
        )
        .build()
        .await
        .unwrap();
    assert!(matches!(
        client
            .market()
            .stock_bars_page(&request, Some(&cursor))
            .await,
        Err(FinanceError::AuthenticationFailed { .. })
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    first.assert_async().await;
    blocked.assert_async().await;
}

#[tokio::test]
async fn stock_ingestion_builder_rejects_options_a_provider_would_ignore() {
    struct Plain;
    impl finance_query::ProviderCore for Plain {
        fn id(&self) -> Provider {
            Provider::custom("plain-endpoint")
        }
    }
    #[finance_query::async_trait]
    impl finance_query::ProviderAdapter for Plain {}

    let unconfigured = Providers::builder()
        .providers([Provider::Fmp])
        .api_key(Provider::Fmp, "fixture")
        .api_key(Provider::Polygon, "fixture")
        .endpoint(Provider::Polygon, "https://example.test")
        .build()
        .await;
    let keyless = Providers::builder()
        .providers([Provider::Fmp])
        .requests_per_minute(Provider::Fmp, 10)
        .build()
        .await;
    let ignored = Providers::builder()
        .providers([Provider::Fmp])
        .with_adapter(std::sync::Arc::new(Plain))
        .api_key(Provider::custom("plain-endpoint"), "fixture")
        .endpoint(Provider::custom("plain-endpoint"), "https://example.test")
        .build()
        .await;
    for result in [unconfigured, keyless, ignored] {
        assert!(matches!(result, Err(FinanceError::InvalidParameter { .. })));
    }
}
