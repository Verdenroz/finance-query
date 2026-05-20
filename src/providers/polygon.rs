//! Polygon.io provider implementation.

use crate::adapters::polygon;
use crate::error::FinanceError;
use crate::error::Result;
use crate::models::quote::QuoteSummaryResponse;

pub(crate) struct PolygonProvider;

#[async_trait::async_trait]
impl super::ProviderAdapter for PolygonProvider {
    fn id(&self) -> &'static str {
        "polygon"
    }
    fn capabilities(&self) -> super::Capability {
        super::Capability::QUOTE
            | super::Capability::CHART
            | super::Capability::FUNDAMENTALS
            | super::Capability::CORPORATE
            | super::Capability::OPTIONS
            | super::Capability::CRYPTO
            | super::Capability::FOREX
            | super::Capability::FUTURES
            | super::Capability::INDICES
            | super::Capability::FILINGS
            | super::Capability::ECONOMIC
    }

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

    async fn fetch_quote(&self, symbol: &str) -> Result<QuoteSummaryResponse> {
        polygon::fetch_quote_response(symbol).await
    }

    async fn fetch_chart(
        &self,
        symbol: &str,
        interval: crate::Interval,
        range: crate::TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        polygon::fetch_chart_response(symbol, interval, range).await
    }

    async fn fetch_news(&self, symbol: &str) -> Result<Vec<crate::models::corporate::news::News>> {
        polygon::fetch_news_response(symbol).await
    }

    async fn fetch_events(
        &self,
        symbol: &str,
    ) -> Result<crate::models::chart::events::ChartEvents> {
        polygon::fetch_events_response(symbol).await
    }

    // ── Financials ────────────────────────────────────────────────

    async fn fetch_financials(
        &self,
        symbol: &str,
        stmt_type: crate::StatementType,
        frequency: crate::Frequency,
    ) -> Result<crate::models::fundamentals::FinancialStatement> {
        polygon::fetch_financials_response(symbol, stmt_type, frequency).await
    }

    // ── Chart Range ───────────────────────────────────────────────

    async fn fetch_chart_range(
        &self,
        symbol: &str,
        interval: crate::Interval,
        start: i64,
        end: i64,
    ) -> Result<crate::models::chart::Chart> {
        polygon::fetch_chart_range_response(symbol, interval, start, end).await
    }

    // ── Options ───────────────────────────────────────────────────

    async fn fetch_options(
        &self,
        symbol: &str,
        date: Option<i64>,
    ) -> Result<crate::models::options::Options> {
        polygon::fetch_options_response(symbol, date).await
    }

    // ── Recommendations ───────────────────────────────────────────

    async fn fetch_similar_symbols(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::corporate::recommendation::SimilarSymbol>> {
        polygon::fetch_similar_symbols_response(symbol, limit).await
    }

    // ── Forex ─────────────────────────────────────────────────────

    async fn fetch_forex_quote(
        &self,
        from: &str,
        to: &str,
    ) -> Result<crate::models::forex::ForexQuote> {
        polygon::fetch_forex_quote_response(from, to).await
    }

    // ── Futures ───────────────────────────────────────────────────

    async fn fetch_futures_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::futures::FuturesQuote> {
        polygon::fetch_futures_quote_response(symbol).await
    }

    // ── Indices ───────────────────────────────────────────────────

    async fn fetch_indices_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::indices::IndexQuote> {
        polygon::fetch_indices_quote_response(symbol).await
    }

    // ── Filings ───────────────────────────────────────────────────

    async fn fetch_filings(&self, symbol: &str) -> Result<crate::models::filings::ProviderFilings> {
        polygon::fetch_filings_response(symbol).await
    }

    // ── Crypto ────────────────────────────────────────────────────

    async fn fetch_crypto_quote(
        &self,
        from: &str,
        to: &str,
    ) -> Result<crate::models::crypto::CryptoQuote> {
        polygon::fetch_crypto_quote_response(from, to).await
    }

    // ── Economic ──────────────────────────────────────────────────

    async fn fetch_economic_series(
        &self,
        series_id: &str,
    ) -> Result<crate::models::economic::EconomicSeries> {
        polygon::fetch_economic_series_response(series_id).await
    }
}
