#![cfg(feature = "fmp")]

use finance_query::{Capability, FinanceError, Provider, Providers};
use mockito::{Matcher, Server};

async fn providers(server: &Server) -> Providers {
    Providers::builder()
        .providers([Provider::Fmp])
        .api_key(Provider::Fmp, format!("fixture-{}", server.url()))
        .endpoint(Provider::Fmp, server.url())
        .requests_per_minute(Provider::Fmp, 600_000)
        .route(Capability::DISCOVERY, [Provider::Fmp])
        .route(Capability::FUNDAMENTALS, [Provider::Fmp])
        .build()
        .await
        .unwrap()
}

#[tokio::test]
async fn directories_use_stock_routes_and_preserve_unknown_classification() {
    let mut server = Server::new_async().await;
    let all = server
        .mock("GET", "/stable/stock-list")
        .match_query(Matcher::Any)
        .with_body(
            r#"[{"symbol":"AAPL","companyName":"Apple"},{"symbol":"OLD","companyName":null}]"#,
        )
        .expect(1)
        .create_async()
        .await;
    let active = server
        .mock("GET", "/stable/actively-trading-list")
        .match_query(Matcher::Any)
        .with_body(r#"[{"symbol":"AAPL","name":"Apple"},{"symbol":"SPY","name":"An ETF"}]"#)
        .expect(1)
        .create_async()
        .await;
    let client = providers(&server).await;
    let rows = client.discovery().stock_list().await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name.as_deref(), Some("Apple"));
    assert_eq!(rows[0].active, None);
    assert_eq!(rows[0].exchange, None);
    assert_eq!(rows[0].asset_type, None);
    let rows = client.discovery().listing_status(true).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row.active == Some(true)));
    assert_eq!(rows[1].asset_type, None);
    all.assert_async().await;
    active.assert_async().await;
}

#[tokio::test]
async fn bulk_csv_handles_bom_quotes_newlines_optional_cells_and_exact_identifiers() {
    let mut server = Server::new_async().await;
    let csv = concat!(
        "\u{feff}symbol,companyName,description,cik,cusip,isin,ipoDate,",
        "isActivelyTrading,isEtf,isAdr,isFund,marketCap,exchange\r\n",
        "AAPL,\"Apple, Inc.\",\"Line one\nLine two\",0000320193,037833100,US0378331005,",
        "1980-12-12,true,false,false,false,100.5,NASDAQ\r\n",
        "OLD,,,,,,,,,,,,\r\n",
    );
    let fixture = server
        .mock("GET", "/stable/profile-bulk")
        .match_query(Matcher::UrlEncoded("part".into(), "3".into()))
        .with_body(csv)
        .expect(1)
        .create_async()
        .await;
    let rows = providers(&server)
        .await
        .discovery()
        .company_profiles_bulk(3)
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name.as_deref(), Some("Apple, Inc."));
    assert_eq!(rows[0].description.as_deref(), Some("Line one\nLine two"));
    assert_eq!(rows[0].cik.as_deref(), Some("0000320193"));
    assert_eq!(rows[0].cusip.as_deref(), Some("037833100"));
    assert_eq!(rows[0].isin.as_deref(), Some("US0378331005"));
    assert_eq!(rows[0].ipo_date.as_deref(), Some("1980-12-12"));
    assert_eq!(rows[0].active, Some(true));
    assert_eq!(rows[0].is_etf, Some(false));
    assert_eq!(rows[0].market_capitalization, Some(100.5));
    assert_eq!(rows[0].provider_id, Some(Provider::Fmp));
    assert_eq!(rows[1].cik, None);
    assert_eq!(rows[1].ipo_date, None);
    assert_eq!(rows[1].active, None);
    fixture.assert_async().await;
}

#[tokio::test]
async fn bulk_json_and_individual_profiles_preserve_the_same_fields() {
    let mut server = Server::new_async().await;
    let body = serde_json::json!([{
        "symbol": "AAPL",
        "companyName": "Apple",
        "cik": "0000320193",
        "cusip": "037833100",
        "isin": "US0378331005",
        "ipoDate": "1980-12-12",
        "isActivelyTrading": true,
        "isEtf": false,
        "isAdr": false,
        "isFund": false
    }])
    .to_string();
    let bulk = server
        .mock("GET", "/stable/profile-bulk")
        .match_query(Matcher::Any)
        .with_body(&body)
        .create_async()
        .await;
    let single = server
        .mock("GET", "/stable/profile")
        .match_query(Matcher::UrlEncoded("symbol".into(), "AAPL".into()))
        .with_body(body)
        .create_async()
        .await;
    let client = providers(&server).await;
    let rows = client.discovery().company_profiles_bulk(0).await.unwrap();
    let row = client
        .ticker("AAPL")
        .build()
        .await
        .unwrap()
        .company_profile()
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&row).unwrap(),
        serde_json::to_value(&rows[0]).unwrap()
    );
    bulk.assert_async().await;
    single.assert_async().await;
}

#[tokio::test]
async fn only_well_formed_empty_parts_are_empty_results() {
    for body in ["symbol,companyName\n", "[]"] {
        let mut server = Server::new_async().await;
        let fixture = server
            .mock("GET", "/stable/profile-bulk")
            .match_query(Matcher::Any)
            .with_body(body)
            .expect(1)
            .create_async()
            .await;
        assert!(
            providers(&server)
                .await
                .discovery()
                .company_profiles_bulk(9)
                .await
                .unwrap()
                .is_empty()
        );
        fixture.assert_async().await;
    }
    for body in [
        "",
        "  ",
        "<html>Failure</html>",
        "name\nApple\n",
        "symbol,ipoDate\nAAPL,2020-02-30\n",
        "symbol,isEtf\nAAPL,unknown\n",
        "symbol,name\nAAPL,Apple,extra\n",
        "symbol\n\" \"\n",
        "[{}]",
    ] {
        let mut server = Server::new_async().await;
        let fixture = server
            .mock("GET", "/stable/profile-bulk")
            .match_query(Matcher::Any)
            .with_body(body)
            .expect(1)
            .create_async()
            .await;
        assert!(
            providers(&server)
                .await
                .discovery()
                .company_profiles_bulk(0)
                .await
                .is_err(),
            "accepted malformed fixture {body:?}"
        );
        fixture.assert_async().await;
    }
}

#[tokio::test]
async fn failures_do_not_follow_redirects_retry_or_become_empty_parts() {
    for status in [301, 401, 403, 404, 429, 500] {
        let mut server = Server::new_async().await;
        let target = server
            .mock("GET", "/redirect-target")
            .match_query(Matcher::Any)
            .expect(0)
            .create_async()
            .await;
        let fixture = server
            .mock("GET", "/stable/profile-bulk")
            .match_query(Matcher::Any)
            .with_status(status)
            .with_header("Location", &format!("{}/redirect-target", server.url()))
            .with_header("Retry-After", "7")
            .with_body("[]")
            .expect(1)
            .create_async()
            .await;
        let error = providers(&server)
            .await
            .discovery()
            .company_profiles_bulk(0)
            .await
            .unwrap_err();
        if status == 429 {
            assert!(matches!(
                error,
                FinanceError::RateLimited {
                    retry_after: Some(7)
                }
            ));
        }
        fixture.assert_async().await;
        target.assert_async().await;
    }
}

#[tokio::test]
async fn error_envelopes_redact_credentials_and_failed_calls_are_not_cached() {
    let mut server = Server::new_async().await;
    let secret = format!("fixture-{}", server.url());
    let bad = server
        .mock("GET", "/stable/profile-bulk")
        .match_query(Matcher::Any)
        .with_body(
            serde_json::json!({"Error Message":format!("Invalid API KEY {secret}")}).to_string(),
        )
        .expect(1)
        .create_async()
        .await;
    let client = providers(&server).await;
    let error = client
        .discovery()
        .company_profiles_bulk(0)
        .await
        .unwrap_err();
    assert!(!error.to_string().contains(&secret));
    bad.assert_async().await;
    bad.remove_async().await;
    let good = server
        .mock("GET", "/stable/profile-bulk")
        .match_query(Matcher::Any)
        .with_body("[]")
        .expect(1)
        .create_async()
        .await;
    assert!(
        client
            .discovery()
            .company_profiles_bulk(0)
            .await
            .unwrap()
            .is_empty()
    );
    good.assert_async().await;
}

#[tokio::test]
async fn directory_rejects_missing_or_blank_symbols() {
    for body in [r#"[{"companyName":"Unknown"}]"#, r#"[{"symbol":" "}]"#] {
        let mut server = Server::new_async().await;
        let fixture = server
            .mock("GET", "/stable/stock-list")
            .match_query(Matcher::Any)
            .with_body(body)
            .create_async()
            .await;
        assert!(
            providers(&server)
                .await
                .discovery()
                .stock_list()
                .await
                .is_err()
        );
        fixture.assert_async().await;
    }
}
