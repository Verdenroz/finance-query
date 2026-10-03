#![cfg(feature = "fmp")]

use finance_query::{Capability, Provider, Providers, SymbolMatch};
use mockito::{Matcher, Mock, Server};
use serde_json::json;

async fn discovery(server: &Server) -> finance_query::Discovery {
    Providers::builder()
        .providers([Provider::Fmp])
        .api_key(Provider::Fmp, format!("fixture-{}", server.url()))
        .endpoint(Provider::Fmp, server.url())
        .requests_per_minute(Provider::Fmp, 600_000)
        .route(Capability::DISCOVERY, [Provider::Fmp])
        .build()
        .await
        .unwrap()
        .discovery()
}

fn page(server: &mut Server, number: u32) -> Mock {
    server
        .mock("GET", "/stable/delisted-companies")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("page".into(), number.to_string()),
            Matcher::UrlEncoded("limit".into(), "100".into()),
        ]))
}

#[tokio::test]
async fn fmp_delisted_listing_status_reads_one_page_and_preserves_dates_and_reused_symbols() {
    let mut server = Server::new_async().await;
    let mut rows: Vec<_> = (0..98)
        .map(|n| {
            json!({
                "symbol": format!("T{n}"), "companyName": format!("Company {n}"),
                "exchange": "NYSE", "ipoDate": "1980-12-12", "delistedDate": "2004-01-02"
            })
        })
        .collect();
    rows.push(json!({
        "symbol": "T0",
        "companyName": "Different lifetime",
        "exchange": "NASDAQ",
        "ipoDate": "2009-02-03",
        "delistedDate": "2025-06-07"
    }));
    rows.push(json!({"symbol": "UNKNOWN_DATES", "ipoDate": null, "delistedDate": null}));
    let first = page(&mut server, 0)
        .with_body(json!(rows).to_string())
        .expect(1)
        .create_async()
        .await;
    let second = page(&mut server, 1)
        .with_body(r#"[{"symbol":"NEXT"}]"#)
        .expect(0)
        .create_async()
        .await;
    let client = discovery(&server).await;
    let result = client.listing_status(false).await.unwrap();
    assert_eq!(result.len(), 100);
    assert_eq!(result[0].symbol, "T0");
    assert_eq!(result[0].ipo_date.as_deref(), Some("1980-12-12"));
    assert_eq!(result[0].delisted_date.as_deref(), Some("2004-01-02"));
    assert_eq!(result[98].symbol, "T0");
    assert_eq!(result[98].name.as_deref(), Some("Different lifetime"));
    assert_eq!(result[98].exchange.as_deref(), Some("NASDAQ"));
    assert_eq!(result[98].ipo_date.as_deref(), Some("2009-02-03"));
    assert_eq!(result[98].delisted_date.as_deref(), Some("2025-06-07"));
    assert_eq!(result[98].active, Some(false));
    assert_eq!(result[99].ipo_date, None);
    assert_eq!(result[99].delisted_date, None);
    let encoded = serde_json::to_value(&result[98]).unwrap();
    assert_eq!(encoded["ipo_date"], "2009-02-03");
    assert_eq!(encoded["delisted_date"], "2025-06-07");
    assert_eq!(client.listing_status(false).await.unwrap().len(), 100);
    first.assert_async().await;
    second.assert_async().await;
}

#[tokio::test]
async fn fmp_delisted_failure_is_not_cached() {
    for status in [401, 429, 500] {
        let mut server = Server::new_async().await;
        let failure = page(&mut server, 0)
            .with_status(status)
            .with_body("{}")
            .expect(1)
            .create_async()
            .await;
        let client = discovery(&server).await;
        assert!(
            client.listing_status(false).await.is_err(),
            "status {status}"
        );
        failure.assert_async().await;
        failure.remove_async().await;
        let recovered = page(&mut server, 0)
            .with_body(r#"[{"symbol":"OLD","ipoDate":"1980-01-01","delistedDate":"2003-01-01"}]"#)
            .expect(1)
            .create_async()
            .await;
        let result = client.listing_status(false).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].ipo_date.as_deref(), Some("1980-01-01"));
        recovered.assert_async().await;
    }
}

#[tokio::test]
async fn fmp_delisted_empty_first_page_is_a_complete_empty_result() {
    let mut server = Server::new_async().await;
    let empty = page(&mut server, 0).with_body("[]").create_async().await;
    assert!(
        discovery(&server)
            .await
            .listing_status(false)
            .await
            .unwrap()
            .is_empty()
    );
    empty.assert_async().await;
}

#[test]
fn old_symbol_json_remains_readable_without_dates() {
    let row: SymbolMatch = serde_json::from_value(json!({"symbol":"OLD"})).unwrap();
    assert!(row.ipo_date.is_none());
    assert!(row.delisted_date.is_none());
    let encoded = serde_json::to_value(row).unwrap();
    assert!(encoded.get("ipo_date").is_none());
    assert!(encoded.get("delisted_date").is_none());
}
