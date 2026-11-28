#![cfg(all(
    feature = "fiscaldata",
    feature = "defi",
    feature = "crypto",
    feature = "fred"
))]

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
async fn coingecko_endpoint_reaches_the_client_without_a_key() {
    let mut server = Server::new_async().await;
    let global = server
        .mock("GET", "/api/v3/global")
        .with_body(r#"{"data":{"active_cryptocurrencies":7,"total_market_cap":{},"total_volume":{},"market_cap_percentage":{}}}"#)
        .expect(1)
        .create_async()
        .await;
    let providers = Providers::builder()
        .providers([Provider::CoinGecko])
        .endpoint(Provider::CoinGecko, server.url())
        .route(Capability::CRYPTO, [Provider::CoinGecko])
        .build()
        .await
        .unwrap();
    let stats = providers.market().crypto_global().await.unwrap();
    assert_eq!(stats.active_cryptocurrencies, Some(7));
    global.assert_async().await;
}

#[tokio::test]
async fn fred_endpoint_reaches_the_client_with_an_explicit_key() {
    let mut server = Server::new_async().await;
    let observations = server
        .mock("GET", "/fred/series/observations")
        .match_query(Matcher::UrlEncoded("api_key".into(), "fixture".into()))
        .with_body(r#"{"observations":[{"date":"2024-01-01","value":"1.5"}]}"#)
        .expect(1)
        .create_async()
        .await;
    let providers = Providers::builder()
        .providers([Provider::Fred])
        .api_key(Provider::Fred, "fixture")
        .endpoint(Provider::Fred, server.url())
        .route(Capability::ECONOMIC, [Provider::Fred])
        .build()
        .await
        .unwrap();
    let series = providers.economic("GDP").as_of("2024-01-15").await.unwrap();
    assert_eq!(series.observations.len(), 1);
    observations.assert_async().await;
}

#[tokio::test]
async fn fred_endpoint_without_a_key_is_rejected() {
    let result = Providers::builder()
        .providers([Provider::Fred])
        .endpoint(Provider::Fred, "https://example.test")
        .build()
        .await;
    assert!(matches!(result, Err(FinanceError::InvalidParameter { .. })));
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
