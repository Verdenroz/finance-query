//! Bounded, read-only provider verification. Requires Polygon and FMP keys.
use finance_query::{
    Capability, Interval, PriceAdjustment, Provider, Providers, StockBarsRequest,
    StockListingRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let massive = Providers::builder()
        .providers([Provider::Polygon])
        .api_key(Provider::Polygon, std::env::var("POLYGON_API_KEY")?)
        .route(Capability::DISCOVERY, [Provider::Polygon])
        .route(Capability::CHART, [Provider::Polygon])
        .build()
        .await?;
    if std::env::args().any(|arg| arg == "--directory-only") {
        let request = StockListingRequest::new("2020-01-02", true)?
            .stock_type("CS")
            .page_size(2)?;
        let first = massive
            .discovery()
            .stock_listings_page(&request, None)
            .await?;
        let cursor = first.next.ok_or("expected a second stock-directory page")?;
        let saved = serde_json::to_string(&cursor)?;
        let restored = serde_json::from_str(&saved)?;
        let second = massive
            .discovery()
            .stock_listings_page(&request, Some(&restored))
            .await?;
        println!(
            "{}",
            serde_json::json!({"requests":2,"first_symbols":first.items.iter().map(|row| &row.symbol).collect::<Vec<_>>(),"second_symbols":second.items.iter().map(|row| &row.symbol).collect::<Vec<_>>(),"more_pages":second.next.is_some(),"cursor_roundtrip":true})
        );
        return Ok(());
    }
    let exchanges = massive.discovery().exchanges().await?;
    let types = massive.discovery().stock_types("us").await?;
    let details = massive.discovery().details_at("AAPL", "2020-01-02").await?;
    let minute = StockBarsRequest::new("AAPL", "2020-01-02", "2020-01-02", Interval::OneMinute)?
        .adjustment(PriceAdjustment::Unadjusted);
    let minute = massive.market().stock_bars_page(&minute, None).await?;
    let daily = StockBarsRequest::new("AAPL", "2020-01-02", "2020-01-02", Interval::OneDay)?
        .adjustment(PriceAdjustment::Unadjusted);
    let daily = massive.market().stock_bars_page(&daily, None).await?;
    let fmp = Providers::builder()
        .providers([Provider::Fmp])
        .api_key(Provider::Fmp, std::env::var("FMP_API_KEY")?)
        .route(Capability::FUNDAMENTALS, [Provider::Fmp])
        .build()
        .await?;
    let profile = fmp.ticker("AAPL").build().await?.company_profile().await?;
    println!(
        "{}",
        serde_json::json!({
            "requests":6, "exchanges":exchanges.len(), "stock_types":types.len(),
            "details_symbol":details.symbol, "details_cik":details.cik,
            "minute_rows":minute.items.len(), "minute_complete":minute.next.is_none(),
            "daily_rows":daily.items.len(), "daily_complete":daily.next.is_none(),
            "fmp_symbol":profile.symbol, "fmp_cik":profile.cik, "fmp_ipo_date":profile.ipo_date,
        })
    );
    Ok(())
}
