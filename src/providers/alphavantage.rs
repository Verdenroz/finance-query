//! Alpha Vantage provider implementation.
//!
//! Thin delegate — all DTO→canonical conversion logic lives
//! in the adapter functions under `crate::adapters::alphavantage::*`.

use super::{
    CalendarProvider, ChartProvider, CommoditiesProvider, CorporateProvider, CryptoProvider,
    DiscoveryProvider, EconomicProvider, FilingsProvider, ForexProvider, FundamentalsProvider,
    MarketProvider, Operation, OptionsProvider, ProviderAdapter, ProviderCore, QuoteProvider,
};
use crate::adapters::alphavantage as av;
use crate::error::Result;
use crate::models::quote::QuoteSummaryResponse;

pub(crate) struct AlphaVantageProvider;

impl ProviderCore for AlphaVantageProvider {
    fn id(&self) -> super::Provider {
        super::Provider::AlphaVantage
    }
}

#[async_trait::async_trait]
impl QuoteProvider for AlphaVantageProvider {
    async fn fetch_quote(&self, symbol: &str) -> Result<QuoteSummaryResponse> {
        av::fetch_quote_response(symbol).await
    }

    async fn fetch_quotes_batch(
        &self,
        symbols: &[&str],
    ) -> Result<Vec<(String, QuoteSummaryResponse)>> {
        av::fetch_quotes_batch_response(symbols).await
    }
}

#[async_trait::async_trait]
impl ChartProvider for AlphaVantageProvider {
    async fn fetch_chart(
        &self,
        symbol: &str,
        interval: crate::Interval,
        range: crate::TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        av::fetch_chart_response(symbol, interval, range).await
    }

    async fn fetch_chart_range(
        &self,
        symbol: &str,
        interval: crate::Interval,
        start: i64,
        end: i64,
    ) -> Result<crate::models::chart::Chart> {
        av::fetch_chart_range_response(symbol, interval, start, end).await
    }
}

#[async_trait::async_trait]
impl FundamentalsProvider for AlphaVantageProvider {
    async fn fetch_financials(
        &self,
        symbol: &str,
        stmt_type: crate::StatementType,
        frequency: crate::Frequency,
    ) -> Result<crate::models::fundamentals::FinancialStatement> {
        av::fetch_financials_response(symbol, stmt_type, frequency).await
    }

    async fn fetch_etf_profile(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::EtfProfile> {
        av::fundamentals::etf::fetch_etf_profile_response(symbol).await
    }

    async fn fetch_company_profile(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::CompanyProfile> {
        av::fundamentals::fetch_company_profile_response(symbol).await
    }

    async fn fetch_earnings_surprises(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::models::fundamentals::EarningsSurprise>> {
        av::fundamentals::fetch_earnings_surprises_response(symbol).await
    }
}

#[async_trait::async_trait]
impl DiscoveryProvider for AlphaVantageProvider {
    async fn fetch_symbol_search(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::discovery::reference::SymbolMatch>> {
        av::discovery::fetch_symbol_search_response(query, limit).await
    }

    /// Alpha Vantage has no exchange endpoint; the venue list is derived from
    /// the active listing universe, so only `name` is populated.
    async fn fetch_exchanges(
        &self,
    ) -> Result<Vec<crate::models::discovery::reference::ExchangeInfo>> {
        av::discovery::fetch_exchanges_response().await
    }

    async fn fetch_listing_status(
        &self,
        active: bool,
    ) -> Result<Vec<crate::models::discovery::reference::SymbolMatch>> {
        av::discovery::fetch_listing_status_response(active).await
    }
}

#[async_trait::async_trait]
impl CorporateProvider for AlphaVantageProvider {
    async fn fetch_news(&self, symbol: &str) -> Result<Vec<crate::models::corporate::news::News>> {
        av::fetch_news_response(symbol).await
    }

    async fn fetch_events(
        &self,
        symbol: &str,
    ) -> Result<crate::models::chart::events::ChartEvents> {
        av::fetch_events_response(symbol).await
    }

    async fn fetch_earnings_transcript(
        &self,
        symbol: &str,
        quarter: Option<&str>,
        year: Option<i32>,
    ) -> Result<crate::models::corporate::earnings_transcript::EarningsTranscript> {
        av::corporate::fetch_earnings_transcript_response(symbol, quarter, year).await
    }
}

#[async_trait::async_trait]
impl OptionsProvider for AlphaVantageProvider {
    async fn fetch_options(
        &self,
        symbol: &str,
        date: Option<i64>,
    ) -> Result<crate::models::options::Options> {
        av::fetch_options_response(symbol, date).await
    }
}

#[async_trait::async_trait]
impl ForexProvider for AlphaVantageProvider {
    async fn fetch_forex_quote(
        &self,
        from: &str,
        to: &str,
    ) -> Result<crate::models::forex::ForexQuote> {
        av::fetch_forex_quote_response(from, to).await
    }
}

#[async_trait::async_trait]
impl CommoditiesProvider for AlphaVantageProvider {
    async fn fetch_commodities_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::commodities::CommodityQuote> {
        av::fetch_commodities_quote_response(symbol).await
    }
}

#[async_trait::async_trait]
impl CryptoProvider for AlphaVantageProvider {
    async fn fetch_crypto_quote(
        &self,
        symbol: &str,
        market: &str,
    ) -> Result<crate::models::crypto::CryptoQuote> {
        av::fetch_crypto_quote_response(symbol, market).await
    }
}

#[async_trait::async_trait]
impl EconomicProvider for AlphaVantageProvider {
    async fn fetch_economic_series(
        &self,
        series_id: &str,
    ) -> Result<crate::models::economic::EconomicSeries> {
        av::fetch_economic_series_response(series_id).await
    }
}

#[async_trait::async_trait]
impl MarketProvider for AlphaVantageProvider {
    async fn fetch_market_movers(
        &self,
        direction: crate::models::market::performance::MoverDirection,
    ) -> Result<Vec<crate::models::market::performance::MoverQuote>> {
        av::fetch_market_movers_response(direction).await
    }
}

#[async_trait::async_trait]
impl CalendarProvider for AlphaVantageProvider {
    /// Alpha Vantage serves live exchange open/closed status only. Other
    /// kinds fall through to the next routed provider. `from`/`to` are
    /// ignored: this is a snapshot, not a dated event.
    async fn fetch_market_calendar(
        &self,
        kind: crate::models::calendar::market::CalendarKind,
        _from: &str,
        _to: &str,
    ) -> Result<Vec<crate::models::calendar::market::MarketCalendarEntry>> {
        if kind != crate::models::calendar::market::CalendarKind::MarketStatus {
            return Err(self.not_supported(kind.operation()));
        }
        av::fetch_market_status_response().await
    }
}

/// `fetch_filings` is the trait's required primary operation, but Alpha
/// Vantage publishes no raw SEC filing surface — only insider transactions —
/// so it reports `NotSupported` there and dispatch falls through to the next
/// routed provider (EDGAR).
#[async_trait::async_trait]
impl FilingsProvider for AlphaVantageProvider {
    async fn fetch_filings(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::filings::ProviderFilings> {
        Err(self.not_supported(Operation::Filings))
    }

    /// A second route for the ownership surface alongside EDGAR — Alpha
    /// Vantage's `INSIDER_TRANSACTIONS` is coarser (no accession number, form
    /// type, or derivative/non-derivative distinction) but covers the same
    /// canonical [`InsiderTrade`](crate::models::filings::InsiderTrade) model.
    async fn fetch_insider_trades(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::filings::InsiderTrade>> {
        av::corporate::fetch_insider_trades_response(symbol, limit).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for AlphaVantageProvider {
    async fn initialize(&self) -> Result<()> {
        let _ = av::build_client()?;
        Ok(())
    }

    fn rate_limit_remaining(&self) -> Option<f64> {
        av::rate_limiter().and_then(|l| l.available_estimate())
    }

    fn as_quote(&self) -> Option<&dyn QuoteProvider> {
        Some(self)
    }
    fn as_chart(&self) -> Option<&dyn ChartProvider> {
        Some(self)
    }
    fn as_fundamentals(&self) -> Option<&dyn FundamentalsProvider> {
        Some(self)
    }
    fn as_corporate(&self) -> Option<&dyn CorporateProvider> {
        Some(self)
    }
    fn as_options(&self) -> Option<&dyn OptionsProvider> {
        Some(self)
    }
    fn as_crypto(&self) -> Option<&dyn CryptoProvider> {
        Some(self)
    }
    fn as_economic(&self) -> Option<&dyn EconomicProvider> {
        Some(self)
    }
    fn as_forex(&self) -> Option<&dyn ForexProvider> {
        Some(self)
    }
    fn as_commodities(&self) -> Option<&dyn CommoditiesProvider> {
        Some(self)
    }
    fn as_market(&self) -> Option<&dyn MarketProvider> {
        Some(self)
    }
    fn as_discovery(&self) -> Option<&dyn DiscoveryProvider> {
        Some(self)
    }
    fn as_calendar(&self) -> Option<&dyn CalendarProvider> {
        Some(self)
    }
    fn as_filings(&self) -> Option<&dyn FilingsProvider> {
        Some(self)
    }
}
