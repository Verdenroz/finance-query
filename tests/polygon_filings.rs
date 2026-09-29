#![cfg(feature = "polygon")]

use finance_query::{Capability, FilingSectionForm, FinanceError, Provider, Providers};
use mockito::{Matcher, Server};
use serde_json::json;

async fn providers(server: &Server) -> Providers {
    Providers::builder()
        .providers([Provider::Polygon])
        .api_key(Provider::Polygon, "filings-fixture")
        .endpoint(Provider::Polygon, server.url())
        .requests_per_minute(Provider::Polygon, 600_000)
        .route(Capability::FILINGS, [Provider::Polygon])
        .build()
        .await
        .unwrap()
}

fn filer_year(cik: &str, from: &str, to: &str) -> Vec<Matcher> {
    vec![
        Matcher::UrlEncoded("cik".into(), cik.into()),
        Matcher::UrlEncoded("filing_date.gte".into(), from.into()),
        Matcher::UrlEncoded("filing_date.lte".into(), to.into()),
    ]
}

#[tokio::test]
async fn filings_come_from_the_ticker_filtered_index() {
    let mut server = Server::new_async().await;
    let body = json!({
        "status": "OK",
        "results": [{
            "cik": "0000320193",
            "issuer_name": "Apple Inc.",
            "form_type": "10-K",
            "filing_date": "2025-10-31",
            "filing_url": "https://www.sec.gov/Archives/edgar/data/320193/0000320193-25-000079.txt",
            "accession_number": "0000320193-25-000079",
            "ticker": "AAPL"
        }]
    });
    let index = server
        .mock("GET", "/stocks/filings/vX/index")
        .match_query(Matcher::UrlEncoded("ticker".into(), "AAPL".into()))
        .with_body(body.to_string())
        .create_async()
        .await;
    let filings = providers(&server)
        .await
        .filings("AAPL")
        .get()
        .await
        .unwrap();
    let filing = &filings.filings[0];
    assert_eq!(
        filing.accession_number.as_deref(),
        Some("0000320193-25-000079")
    );
    assert_eq!(filing.filing_type.as_deref(), Some("10-K"));
    assert_eq!(filing.company_name.as_deref(), Some("Apple Inc."));
    assert_eq!(filing.cik.as_deref(), Some("0000320193"));
    index.assert_async().await;
}

#[tokio::test]
async fn ten_k_sections_scan_the_filers_year_for_the_accession() {
    let mut server = Server::new_async().await;
    let path = "/stocks/filings/10-K/vX/sections";
    let section = |accession: &str, name: &str| {
        json!({
            "cik": "0000320193",
            "ticker": "AAPL",
            "section": name,
            "filing_date": "2025-10-31",
            "filing_url": format!("https://www.sec.gov/Archives/edgar/data/320193/{accession}.txt"),
            "text": format!("{name} text")
        })
    };
    let first_body = json!({
        "status": "OK",
        "results": [section("0000320193-25-000011", "risk_factors")],
        "next_url": format!("{}{path}?cursor=two", server.url()),
    });
    let second_body = json!({
        "status": "OK",
        "results": [
            section("0000320193-25-000079", "risk_factors"),
            section("0000320193-25-000079", "business")
        ],
    });
    let first = server
        .mock("GET", path)
        .match_query(Matcher::AllOf(filer_year(
            "0000320193",
            "2025-01-01",
            "2026-01-15",
        )))
        .with_body(first_body.to_string())
        .expect(1)
        .create_async()
        .await;
    let second = server
        .mock("GET", path)
        .match_query(Matcher::UrlEncoded("cursor".into(), "two".into()))
        .with_body(second_body.to_string())
        .expect(1)
        .create_async()
        .await;
    let sections = providers(&server)
        .await
        .filings("AAPL")
        .sections("0000320193-25-000079", FilingSectionForm::TenK)
        .await
        .unwrap();
    let names: Vec<_> = sections
        .iter()
        .filter_map(|s| s.section.as_deref())
        .collect();
    assert_eq!(names, ["risk_factors", "business"]);
    assert_eq!(sections[1].content.as_deref(), Some("business text"));
    first.assert_async().await;
    second.assert_async().await;
}

#[tokio::test]
async fn eight_k_text_matches_the_accession_or_reports_it_missing() {
    let mut server = Server::new_async().await;
    let body = json!({
        "status": "OK",
        "results": [
            {"accession_number": "0000320193-26-000017", "form_type": "8-K", "items_text": "Item 5.02"},
            {"accession_number": "0000320193-26-000018", "form_type": "8-K", "items_text": "Item 2.02"}
        ]
    });
    server
        .mock("GET", "/stocks/filings/8-K/vX/text")
        .match_query(Matcher::AllOf(filer_year(
            "0000320193",
            "2026-01-01",
            "2027-01-15",
        )))
        .with_body(body.to_string())
        .create_async()
        .await;
    let filings = providers(&server).await.filings("AAPL");
    let sections = filings
        .sections("0000320193-26-000018", FilingSectionForm::EightK)
        .await
        .unwrap();
    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].section.as_deref(), Some("items"));
    assert_eq!(sections[0].content.as_deref(), Some("Item 2.02"));

    let missing = filings
        .sections("0000320193-26-000099", FilingSectionForm::EightK)
        .await;
    assert!(matches!(missing, Err(FinanceError::SymbolNotFound { .. })));
    let malformed = filings
        .sections("320193-26-18", FilingSectionForm::EightK)
        .await;
    assert!(matches!(
        malformed,
        Err(FinanceError::InvalidParameter { .. })
    ));
}

#[tokio::test]
async fn risk_factors_map_the_taxonomy_and_plan_refusals_keep_their_message() {
    let mut server = Server::new_async().await;
    let body = json!({
        "status": "OK",
        "results": [{
            "cik": "0000320193",
            "ticker": "AAPL",
            "primary_category": "financial_and_market",
            "secondary_category": "capital_structure_and_performance",
            "tertiary_category": "dividend_policy_and_capital_allocation",
            "filing_date": "2024-11-01",
            "supporting_text": "The Company believes the price of its stock..."
        }]
    });
    server
        .mock("GET", "/stocks/filings/vX/risk-factors")
        .match_query(Matcher::UrlEncoded("ticker".into(), "AAPL".into()))
        .with_body(body.to_string())
        .create_async()
        .await;
    let refusal = json!({
        "status": "NOT_AUTHORIZED",
        "message": "You are not entitled to this data. Please upgrade your plan"
    });
    server
        .mock("GET", "/stocks/filings/vX/risk-factors")
        .match_query(Matcher::UrlEncoded("ticker".into(), "MSFT".into()))
        .with_status(403)
        .with_body(refusal.to_string())
        .create_async()
        .await;
    let client = providers(&server).await;

    let risks = client.filings("AAPL").risk_factors().await.unwrap();
    assert_eq!(risks[0].category.as_deref(), Some("financial_and_market"));
    assert_eq!(
        risks[0].title.as_deref(),
        Some("dividend_policy_and_capital_allocation")
    );
    assert_eq!(risks[0].filing_date.as_deref(), Some("2024-11-01"));

    match client.filings("MSFT").risk_factors().await {
        Err(FinanceError::AuthenticationFailed { context }) => {
            assert!(context.contains("not entitled"), "{context}");
        }
        other => panic!("unexpected {other:?}"),
    }
}
