//! A downstream adapter serving the stock-ingestion operations, built only from
//! public `finance_query::` items.

use std::sync::Arc;

use finance_query::{
    Capability, Interval, PageCursor, PriceAdjustment, Provider, ProviderAdapter, ProviderCore,
    ProviderPage, Providers, SortType, StockBar, StockBarsRequest, StockListing,
    StockListingRequest, StockType, TickerChange,
};

struct Paged;

impl ProviderCore for Paged {
    fn id(&self) -> Provider {
        Provider::custom("paged")
    }
}

#[finance_query::async_trait]
impl finance_query::ChartProvider for Paged {
    async fn fetch_chart(
        &self,
        _: &str,
        _: Interval,
        _: finance_query::TimeRange,
    ) -> finance_query::Result<finance_query::Chart> {
        Err(self.not_supported(finance_query::Operation::Chart))
    }

    async fn fetch_stock_bars_page(
        &self,
        request: &StockBarsRequest,
        cursor: Option<&PageCursor>,
    ) -> finance_query::Result<ProviderPage<StockBar>> {
        assert_eq!(request.symbol(), "ACME");
        assert_eq!(request.from(), "2020-01-02");
        assert_eq!(request.to(), "2020-01-03");
        assert_eq!(request.interval(), Interval::OneDay);
        assert_eq!(request.adjustment(), PriceAdjustment::Unadjusted);
        assert_eq!(request.sort_order(), SortType::Asc);
        assert_eq!(request.page_limit(), 1);
        let day: i64 = cursor.map_or(0, |c| c.target().parse().unwrap());
        let mut bar = StockBar::default();
        bar.timestamp_ms = 1_577_923_200_000 + day * 86_400_000;
        bar.close = 10.0 + day as f64;
        let next = (day == 0).then(|| PageCursor::continuation(self.id(), "1".into()));
        let mut page = ProviderPage::new(vec![bar], self.id(), next);
        page.reported_symbol = Some(request.symbol().into());
        Ok(page)
    }
}

#[finance_query::async_trait]
impl finance_query::DiscoveryProvider for Paged {
    async fn fetch_symbol_search(
        &self,
        _: &str,
        _: u32,
    ) -> finance_query::Result<Vec<finance_query::SymbolMatch>> {
        Ok(Vec::new())
    }

    async fn fetch_stock_listings_page(
        &self,
        request: &StockListingRequest,
        _: Option<&PageCursor>,
    ) -> finance_query::Result<ProviderPage<StockListing>> {
        assert_eq!(request.as_of(), "2020-01-02");
        assert!(!request.active());
        assert_eq!(request.stock_type_code(), Some("CS"));
        assert_eq!(request.market_locale(), "us");
        assert_eq!(request.page_limit(), 1000);
        let mut listing = StockListing::default();
        listing.symbol = "OLD".into();
        Ok(ProviderPage::new(vec![listing], self.id(), None))
    }

    async fn fetch_ticker_changes(&self, id: &str) -> finance_query::Result<Vec<TickerChange>> {
        let mut change = TickerChange::default();
        change.date = "2020-01-02".into();
        change.ticker = id.into();
        Ok(vec![change])
    }

    async fn fetch_stock_types(&self, locale: &str) -> finance_query::Result<Vec<StockType>> {
        let mut stock_type = StockType::default();
        stock_type.code = "CS".into();
        stock_type.locale = Some(locale.into());
        Ok(vec![stock_type])
    }
}

impl ProviderAdapter for Paged {
    fn as_chart(&self) -> Option<&dyn finance_query::ChartProvider> {
        Some(self)
    }

    fn as_discovery(&self) -> Option<&dyn finance_query::DiscoveryProvider> {
        Some(self)
    }
}

async fn providers() -> Providers {
    Providers::builder()
        .with_adapter(Arc::new(Paged))
        .route(Capability::CHART, [Provider::custom("paged")])
        .route(Capability::DISCOVERY, [Provider::custom("paged")])
        .build()
        .await
        .expect("builds")
}

#[tokio::test]
async fn a_custom_adapter_pages_stock_bars_through_a_persisted_cursor() {
    let providers = providers().await;
    let request = StockBarsRequest::new(
        "ACME",
        "2020-01-02",
        "2020-01-03",
        Interval::OneDay,
        PriceAdjustment::Unadjusted,
    )
    .unwrap()
    .page_size(1)
    .unwrap();

    let first = providers
        .market()
        .stock_bars_page(&request, None)
        .await
        .unwrap();
    assert_eq!(first.items[0].close, 10.0);
    assert_eq!(first.reported_symbol.as_deref(), Some("ACME"));
    let persisted = serde_json::to_string(&first.next.unwrap()).unwrap();
    let cursor: PageCursor = serde_json::from_str(&persisted).unwrap();

    let second = providers
        .market()
        .stock_bars_page(&request, Some(&cursor))
        .await
        .unwrap();
    assert_eq!(second.items[0].close, 11.0);
    assert!(second.next.is_none());

    let other = StockBarsRequest::new(
        "OTHER",
        "2020-01-02",
        "2020-01-03",
        Interval::OneDay,
        PriceAdjustment::Unadjusted,
    )
    .unwrap()
    .page_size(1)
    .unwrap();
    assert!(
        providers
            .market()
            .stock_bars_page(&other, Some(&cursor))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn a_custom_adapter_serves_listings_ticker_changes_and_stock_types() {
    let discovery = providers().await.discovery();
    let request = StockListingRequest::new("2020-01-02", false)
        .unwrap()
        .stock_type("CS");

    let listings = discovery.stock_listings_page(&request, None).await.unwrap();
    assert_eq!(listings.items[0].symbol, "OLD");
    assert_eq!(
        discovery.ticker_changes("ACME").await.unwrap()[0].ticker,
        "ACME"
    );
    assert_eq!(
        discovery.stock_types("us").await.unwrap()[0]
            .locale
            .as_deref(),
        Some("us")
    );
}
