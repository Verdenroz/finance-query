//! Polygon.io provider implementation.

use super::{
    CalendarProvider, ChartProvider, CorporateProvider, CryptoProvider, DiscoveryProvider,
    EconomicProvider, FilingsProvider, ForexProvider, FundamentalsProvider, FuturesProvider,
    IndicesProvider, OptionsProvider, ProviderAdapter, ProviderCore, QuoteProvider,
};
use crate::adapters::polygon;
use crate::error::FinanceError;
use crate::error::Result;
use crate::models::quote::QuoteSummaryResponse;

pub(crate) struct PolygonProvider;

impl ProviderCore for PolygonProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Polygon
    }
}

#[async_trait::async_trait]
impl QuoteProvider for PolygonProvider {
    async fn fetch_quote(&self, symbol: &str) -> Result<QuoteSummaryResponse> {
        polygon::fetch_quote_response(symbol).await
    }

    async fn fetch_quotes_batch(
        &self,
        symbols: &[&str],
    ) -> Result<Vec<(String, QuoteSummaryResponse)>> {
        polygon::fetch_quotes_batch_response(symbols).await
    }

    async fn fetch_unified_snapshot(
        &self,
        symbols: &[&str],
    ) -> Result<Vec<crate::models::quote::snapshot::MarketSnapshot>> {
        polygon::unified::fetch_unified_snapshot_response(symbols).await
    }
}

#[async_trait::async_trait]
impl ChartProvider for PolygonProvider {
    async fn fetch_chart(
        &self,
        symbol: &str,
        interval: crate::Interval,
        range: crate::TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        polygon::fetch_chart_response(symbol, interval, range).await
    }

    async fn fetch_chart_range(
        &self,
        symbol: &str,
        interval: crate::Interval,
        start: i64,
        end: i64,
    ) -> Result<crate::models::chart::Chart> {
        polygon::fetch_chart_range_response(symbol, interval, start, end).await
    }

    async fn fetch_grouped_daily(
        &self,
        date: &str,
    ) -> Result<Vec<(String, crate::models::chart::Candle)>> {
        polygon::fetch_grouped_daily_response(date).await
    }

    async fn fetch_crypto_grouped_daily(
        &self,
        date: &str,
    ) -> Result<Vec<(String, crate::models::chart::Candle)>> {
        polygon::fetch_crypto_grouped_daily_response(date).await
    }

    async fn fetch_forex_grouped_daily(
        &self,
        date: &str,
    ) -> Result<Vec<(String, crate::models::chart::Candle)>> {
        polygon::fetch_forex_grouped_daily_response(date).await
    }
}

#[async_trait::async_trait]
impl FundamentalsProvider for PolygonProvider {
    async fn fetch_financials(
        &self,
        symbol: &str,
        stmt_type: crate::StatementType,
        frequency: crate::Frequency,
    ) -> Result<crate::models::fundamentals::FinancialStatement> {
        polygon::fetch_financials_response(symbol, stmt_type, frequency).await
    }

    async fn fetch_short_interest(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::models::fundamentals::ShortInterest>> {
        polygon::fetch_short_interest_response(symbol).await
    }

    async fn fetch_short_volume(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::models::fundamentals::ShortVolume>> {
        polygon::fetch_short_volume_response(symbol).await
    }

    async fn fetch_share_float(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::ShareFloat> {
        polygon::fetch_share_float_response(symbol).await
    }
}

#[async_trait::async_trait]
impl CorporateProvider for PolygonProvider {
    async fn fetch_news(&self, symbol: &str) -> Result<Vec<crate::models::corporate::news::News>> {
        polygon::fetch_news_response(symbol).await
    }

    async fn fetch_events(
        &self,
        symbol: &str,
    ) -> Result<crate::models::chart::events::ChartEvents> {
        polygon::fetch_events_response(symbol).await
    }

    async fn fetch_similar_symbols(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::corporate::recommendation::SimilarSymbol>> {
        polygon::fetch_similar_symbols_response(symbol, limit).await
    }
}

#[async_trait::async_trait]
impl OptionsProvider for PolygonProvider {
    async fn fetch_options(
        &self,
        symbol: &str,
        date: Option<i64>,
    ) -> Result<crate::models::options::Options> {
        polygon::fetch_options_response(symbol, date).await
    }
}

#[async_trait::async_trait]
impl DiscoveryProvider for PolygonProvider {
    async fn fetch_symbol_search(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::discovery::reference::SymbolMatch>> {
        polygon::fetch_symbol_search_response(query, limit).await
    }

    async fn fetch_symbol_details(
        &self,
        symbol: &str,
    ) -> Result<crate::models::discovery::reference::SymbolDetails> {
        polygon::fetch_symbol_details_response(symbol).await
    }

    async fn fetch_exchanges(
        &self,
    ) -> Result<Vec<crate::models::discovery::reference::ExchangeInfo>> {
        polygon::fetch_exchanges_response().await
    }
}

#[async_trait::async_trait]
impl ForexProvider for PolygonProvider {
    async fn fetch_forex_quote(
        &self,
        from: &str,
        to: &str,
    ) -> Result<crate::models::forex::ForexQuote> {
        polygon::fetch_forex_quote_response(from, to).await
    }
}

#[async_trait::async_trait]
impl FuturesProvider for PolygonProvider {
    async fn fetch_futures_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::futures::FuturesQuote> {
        polygon::fetch_futures_quote_response(symbol).await
    }
}

#[async_trait::async_trait]
impl IndicesProvider for PolygonProvider {
    async fn fetch_indices_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::indices::IndexQuote> {
        polygon::fetch_indices_quote_response(symbol).await
    }
}

#[async_trait::async_trait]
impl FilingsProvider for PolygonProvider {
    async fn fetch_filings(&self, symbol: &str) -> Result<crate::models::filings::ProviderFilings> {
        polygon::fetch_filings_response(symbol).await
    }

    async fn fetch_filing_sections(
        &self,
        accession_number: &str,
        form: crate::models::filings::FilingSectionForm,
    ) -> Result<Vec<crate::models::filings::FilingSection>> {
        polygon::fetch_filing_sections_response(accession_number, form).await
    }

    async fn fetch_risk_factors(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::models::filings::RiskFactor>> {
        polygon::fetch_risk_factors_response(symbol).await
    }
}

#[async_trait::async_trait]
impl CalendarProvider for PolygonProvider {
    /// Polygon serves the holiday calendar only; other kinds fall through to
    /// the next routed provider. Holidays are date-bounded upstream (Polygon
    /// returns the upcoming set), so `from`/`to` are ignored.
    async fn fetch_market_calendar(
        &self,
        kind: crate::models::calendar::market::CalendarKind,
        _from: &str,
        _to: &str,
    ) -> Result<Vec<crate::models::calendar::market::MarketCalendarEntry>> {
        if kind != crate::models::calendar::market::CalendarKind::MarketHoliday {
            return Err(self.not_supported(kind.operation()));
        }
        polygon::fetch_market_holidays_response().await
    }
}

#[async_trait::async_trait]
impl CryptoProvider for PolygonProvider {
    async fn fetch_crypto_quote(
        &self,
        from: &str,
        to: &str,
    ) -> Result<crate::models::crypto::CryptoQuote> {
        polygon::fetch_crypto_quote_response(from, to).await
    }
}

#[async_trait::async_trait]
impl EconomicProvider for PolygonProvider {
    async fn fetch_economic_series(
        &self,
        series_id: &str,
    ) -> Result<crate::models::economic::EconomicSeries> {
        polygon::fetch_economic_series_response(series_id).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for PolygonProvider {
    async fn initialize(&self) -> Result<()> {
        let key = std::env::var("POLYGON_API_KEY").map_err(|_| FinanceError::InvalidParameter {
            param: "polygon".into(),
            reason:
                "POLYGON_API_KEY not set. Set the environment variable or call polygon::init(key)."
                    .into(),
        })?;
        let _ = polygon::init(key);
        Ok(())
    }

    fn rate_limit_remaining(&self) -> Option<f64> {
        polygon::rate_limiter().and_then(|l| l.available_estimate())
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
    fn as_discovery(&self) -> Option<&dyn DiscoveryProvider> {
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
    fn as_indices(&self) -> Option<&dyn IndicesProvider> {
        Some(self)
    }
    fn as_futures(&self) -> Option<&dyn FuturesProvider> {
        Some(self)
    }
    fn as_filings(&self) -> Option<&dyn FilingsProvider> {
        Some(self)
    }
    fn as_calendar(&self) -> Option<&dyn CalendarProvider> {
        Some(self)
    }
}
