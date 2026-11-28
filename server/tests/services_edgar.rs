use finance_query::{Capability, FilingSectionForm, Provider, Providers};
use finance_query_server::cache::Cache;
use finance_query_server::services::{edgar, filings};
use mockito::{Matcher, Server};
use serde_json::json;
use std::sync::Arc;

const TENK: &str = "<html><body><p>Item 1. Business</p><p>We make things.</p>\
    <p>Item 1A. Risk Factors</p>\
    <p>Our Supply Chain Is Concentrated</p>\
    <p>A single supplier accounts for most of our inventory.</p>\
    <p>Item 2. Properties</p><p>We lease office space.</p></body></html>";

#[tokio::test]
async fn edgar_services_shape_scoped_sec_responses_and_map_errors() {
    let _ = finance_query::edgar::init("services@example.com");
    let mut server = Server::new_async().await;
    let tickers = server
        .mock("GET", "/files/company_tickers.json")
        .with_body(
            json!({
                "0": {"cik_str": 320193, "ticker": "AAPL", "title": "Apple Inc."},
                "1": {"cik_str": 1, "ticker": "BOOM", "title": "Boom Inc."}
            })
            .to_string(),
        )
        .expect_at_least(1)
        .create_async()
        .await;
    let submissions = server
        .mock("GET", "/submissions/CIK0000320193.json")
        .with_body(
            json!({
                "cik": "320193", "name": "Apple Inc.", "tickers": ["AAPL"],
                "filings": {"recent": {
                    "accessionNumber": ["0000320193-24-000123"],
                    "filingDate": ["2024-11-01"], "reportDate": ["2024-09-28"],
                    "acceptanceDateTime": ["2024-11-01T06:01:36.000Z"],
                    "form": ["10-K"], "size": [1000], "isXBRL": [1], "isInlineXBRL": [1],
                    "primaryDocument": ["aapl-20240928.htm"],
                    "primaryDocDescription": ["10-K"]
                }}
            })
            .to_string(),
        )
        .expect_at_least(1)
        .create_async()
        .await;
    let facts = server
        .mock("GET", "/api/xbrl/companyfacts/CIK0000320193.json")
        .with_body(json!({"cik": 320193, "entityName": "Apple Inc.", "facts": {}}).to_string())
        .create_async()
        .await;
    let search = server
        .mock("GET", "/LATEST/search-index")
        .match_query(Matcher::UrlEncoded("q".into(), "supply chain".into()))
        .with_body(json!({"hits": {"total": {"value": 1}, "hits": []}}).to_string())
        .create_async()
        .await;
    let index = server
        .mock(
            "GET",
            "/Archives/edgar/data/320193/000032019324000123/index.json",
        )
        .with_body(
            json!({"directory": {"item": [
                {"name": "aapl-20240928.htm", "type": "text.gif", "size": "1000"},
                {"name": "R1.htm", "type": "text.gif", "size": "2000"}
            ]}})
            .to_string(),
        )
        .create_async()
        .await;
    let document = server
        .mock(
            "GET",
            "/Archives/edgar/data/320193/000032019324000123/aapl-20240928.htm",
        )
        .with_body(TENK)
        .expect_at_least(1)
        .create_async()
        .await;
    let failing = server
        .mock("GET", "/submissions/CIK0000000001.json")
        .with_status(500)
        .create_async()
        .await;

    let providers = Arc::new(
        Providers::builder()
            .providers([Provider::Edgar])
            .endpoint(Provider::Edgar, server.url())
            .route(Capability::FILINGS, [Provider::Edgar])
            .build()
            .await
            .unwrap(),
    );
    let cache = Cache::new(None).await;

    providers
        .scope(async {
            let value = edgar::get_cik(&cache, "aapl").await.unwrap();
            assert_eq!(value["cik"], 320193);

            let value = edgar::get_submissions(&cache, "AAPL").await.unwrap();
            assert_eq!(value["name"], "Apple Inc.");

            let value = edgar::get_facts(&cache, "AAPL").await.unwrap();
            assert_eq!(value["entityName"], "Apple Inc.");

            let value = edgar::search_edgar(&cache, "supply chain", None, None, None, None, None)
                .await
                .unwrap();
            assert_eq!(value["hits"]["total"]["value"], 1);

            let sections = filings::get_filing_sections(
                &providers,
                "AAPL",
                "0000320193-24-000123",
                FilingSectionForm::TenK,
            )
            .await
            .unwrap();
            let risk = sections
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["section"] == "item_1a")
                .unwrap();
            assert!(
                risk["content"]
                    .as_str()
                    .unwrap()
                    .contains("single supplier")
            );

            let value = filings::get_risk_factors(&cache, &providers, "AAPL")
                .await
                .unwrap();
            assert_eq!(value[0]["title"], "Our Supply Chain Is Concentrated");

            let err = edgar::get_submissions(&cache, "BOOM").await.unwrap_err();
            assert!(!err.to_string().is_empty());
        })
        .await;

    tickers.assert_async().await;
    submissions.assert_async().await;
    facts.assert_async().await;
    search.assert_async().await;
    index.assert_async().await;
    document.assert_async().await;
    failing.assert_async().await;
}
