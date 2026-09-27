//! Compile the public calls documented for durable stock consumers.
#![cfg(all(feature = "polygon", feature = "fmp"))]

#[allow(dead_code)]
async fn stock_ingestion_documented_usage() -> finance_query::Result<()> {
    use finance_query::{
        Capability, Interval, PriceAdjustment, Provider, Providers, StockBarsRequest,
        StockListingRequest,
    };
    let providers = Providers::builder()
        .providers([Provider::Polygon])
        .api_key(Provider::Polygon, "configured-key")
        .route(Capability::DISCOVERY, [Provider::Polygon])
        .route(Capability::CHART, [Provider::Polygon])
        .build()
        .await?;
    let request = StockListingRequest::new("2020-01-02", false)?.stock_type("CS");
    let page = providers
        .discovery()
        .stock_listings_page(&request, None)
        .await?;
    if let Some(next) = page.next {
        let _ = providers
            .discovery()
            .stock_listings_page(&request, Some(&next))
            .await?;
    }
    let request = StockBarsRequest::new("AAPL", "2020-01-02", "2020-01-02", Interval::OneMinute)?
        .adjustment(PriceAdjustment::Unadjusted);
    let _ = providers.market().stock_bars_page(&request, None).await?;
    let _ = providers
        .discovery()
        .details_at("AAPL", "2020-01-02")
        .await?;
    let _ = providers.discovery().stock_types("us").await?;
    let fmp = Providers::builder()
        .providers([Provider::Fmp])
        .api_key(Provider::Fmp, "configured-key")
        .route(Capability::FUNDAMENTALS, [Provider::Fmp])
        .build()
        .await?;
    let _ = fmp.ticker("BRK-B").build().await?.company_profile().await?;
    Ok(())
}
