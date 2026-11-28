use finance_query::{Capability, FinanceError, Provider, Providers, edgar};
use mockito::{Server, ServerGuard};
use serde_json::json;

async fn providers(server: &ServerGuard) -> Providers {
    let _ = edgar::init("it@example.com");
    Providers::builder()
        .providers([Provider::Edgar])
        .endpoint(Provider::Edgar, server.url())
        .route(Capability::FILINGS, [Provider::Edgar])
        .build()
        .await
        .unwrap()
}

#[tokio::test]
async fn edgar_endpoint_reaches_the_client_without_a_key() {
    let mut server = Server::new_async().await;
    let tickers = server
        .mock("GET", "/files/company_tickers.json")
        .with_body(r#"{"0":{"cik_str":320193,"ticker":"AAPL","title":"Apple Inc."}}"#)
        .create_async()
        .await;
    let submissions = server
        .mock("GET", "/submissions/CIK0000320193.json")
        .with_body(
            json!({
                "cik": "320193", "name": "Apple Inc.",
                "filings": {"recent": {
                    "accessionNumber": ["0000320193-24-000123"],
                    "filingDate": ["2024-11-01"], "form": ["10-K"],
                    "primaryDocument": ["aapl-20240928.htm"]
                }}
            })
            .to_string(),
        )
        .create_async()
        .await;
    let providers = providers(&server).await;

    let filings = providers.filings("AAPL").get().await.unwrap();

    assert_eq!(filings.filings.len(), 1);
    assert_eq!(
        filings.filings[0].company_name.as_deref(),
        Some("Apple Inc.")
    );
    assert_eq!(
        filings.filings[0].filing_url.as_deref(),
        Some("https://www.sec.gov/Archives/edgar/data/320193/000032019324000123/aapl-20240928.htm")
    );
    tickers.assert_async().await;
    submissions.assert_async().await;
}

#[tokio::test]
async fn edgar_endpoint_fails_closed_on_a_host_it_cannot_rewrite() {
    let server = Server::new_async().await;
    let providers = providers(&server).await;

    let result = providers
        .scope(edgar::archive("https://xbrl.sec.gov/dei/2026/dei-2026.xsd"))
        .await;

    assert!(matches!(result, Err(FinanceError::InvalidParameter { .. })));
}

#[tokio::test]
async fn edgar_cik_map_does_not_leak_between_origins() {
    let mut first = Server::new_async().await;
    first
        .mock("GET", "/files/company_tickers.json")
        .with_body(r#"{"0":{"cik_str":42,"ticker":"MOCK","title":"Mock Inc."}}"#)
        .create_async()
        .await;
    let mut second = Server::new_async().await;
    second
        .mock("GET", "/files/company_tickers.json")
        .with_body(r#"{"0":{"cik_str":7,"ticker":"OTHER","title":"Other Inc."}}"#)
        .create_async()
        .await;
    let first = providers(&first).await;
    let second = providers(&second).await;

    assert_eq!(first.scope(edgar::resolve_cik("MOCK")).await.unwrap(), 42);
    let leaked = second.scope(edgar::resolve_cik("MOCK")).await;

    assert!(matches!(leaked, Err(FinanceError::SymbolNotFound { .. })));
}
