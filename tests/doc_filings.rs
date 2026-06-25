//! Compile and runtime tests for docs/library/filings.md
//!
//! Run: cargo test --test doc_filings
//! Network tests: cargo test --test doc_filings -- --ignored

#[tokio::test]
#[ignore = "requires network access"]
async fn test_filings_get() {
    use finance_query::Providers;
    let providers = Providers::builder().build().await.unwrap();
    let filings = providers.filings("AAPL");
    let result = filings.get().await.unwrap();
    let _ = result; // ProviderFilings; presence of data is provider-dependent
}
