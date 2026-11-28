#![cfg(all(feature = "fiscaldata", feature = "defi"))]

use finance_query::{Capability, FinanceError, Provider, Providers, TreasuryAuctionQuery};
use mockito::{Matcher, Server};

#[tokio::test]
async fn fiscaldata_endpoint_reaches_the_client_without_a_key() {
    let mut server = Server::new_async().await;
    let auctions = server
        .mock(
            "GET",
            "/services/api/fiscal_service/v1/accounting/od/auctions_query",
        )
        .match_query(Matcher::Any)
        .with_body(r#"{"data":[],"meta":{"total-pages":0}}"#)
        .expect(1)
        .create_async()
        .await;
    let providers = Providers::builder()
        .providers([Provider::FiscalData])
        .endpoint(Provider::FiscalData, server.url())
        .route(Capability::ECONOMIC, [Provider::FiscalData])
        .build()
        .await
        .unwrap();
    let rows = providers
        .economic_catalog()
        .treasury_auctions(&TreasuryAuctionQuery::default())
        .await
        .unwrap();
    assert!(rows.is_empty());
    auctions.assert_async().await;
}

#[tokio::test]
async fn defillama_endpoint_reaches_the_client_without_a_key() {
    let mut server = Server::new_async().await;
    let history = server
        .mock("GET", "/protocol/aave")
        .with_body(r#"{"tvl":[{"date":1705276800,"totalLiquidityUSD":4.0}]}"#)
        .expect(1)
        .create_async()
        .await;
    let providers = Providers::builder()
        .providers([Provider::DefiLlama])
        .endpoint(Provider::DefiLlama, server.url())
        .route(Capability::CRYPTO, [Provider::DefiLlama])
        .build()
        .await
        .unwrap();
    let points = providers.crypto("aave").tvl_history().await.unwrap();
    assert_eq!(points.len(), 1);
    history.assert_async().await;
}

#[tokio::test]
async fn keyed_providers_still_need_a_key_for_an_endpoint() {
    let result = Providers::builder()
        .providers([Provider::Fmp])
        .endpoint(Provider::Fmp, "https://example.test")
        .build()
        .await;
    assert!(matches!(result, Err(FinanceError::InvalidParameter { .. })));
}
