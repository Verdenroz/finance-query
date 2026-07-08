//! Alpha Vantage provider implementation.
//!
//! Thin delegate — all DTO→canonical conversion logic lives
//! in the adapter functions under `crate::adapters::alphavantage::*`.

use crate::adapters::alphavantage as av;
use crate::error::Result;
use crate::models::quote::QuoteSummaryResponse;

pub(crate) struct AlphaVantageProvider;

#[async_trait::async_trait]
impl super::ProviderAdapter for AlphaVantageProvider {
    fn id(&self) -> super::Provider {
        super::Provider::AlphaVantage
    }
    fn capabilities(&self) -> super::Capability {
        super::Capability::QUOTE
            | super::Capability::CHART
            | super::Capability::FUNDAMENTALS
            | super::Capability::CORPORATE
            | super::Capability::OPTIONS
            | super::Capability::CRYPTO
            | super::Capability::FOREX
            | super::Capability::COMMODITIES
            | super::Capability::ECONOMIC
    }

    async fn initialize(&self) -> Result<()> {
        let key = std::env::var("ALPHAVANTAGE_API_KEY").map_err(|_| {
            crate::error::FinanceError::InvalidParameter {
                param: "alphavantage".into(),
                reason: "ALPHAVANTAGE_API_KEY not set. Set the environment variable or call alphavantage::init(key)."
                    .into(),
            }
        })?;
        let _ = av::init(key);
        Ok(())
    }

    async fn fetch_quote(&self, symbol: &str) -> Result<QuoteSummaryResponse> {
        av::fetch_quote_response(symbol).await
    }

    async fn fetch_chart(
        &self,
        symbol: &str,
        interval: crate::Interval,
        range: crate::TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        av::fetch_chart_response(symbol, interval, range).await
    }

    async fn fetch_financials(
        &self,
        symbol: &str,
        stmt_type: crate::StatementType,
        frequency: crate::Frequency,
    ) -> Result<crate::models::fundamentals::FinancialStatement> {
        av::fetch_financials_response(symbol, stmt_type, frequency).await
    }

    async fn fetch_news(&self, symbol: &str) -> Result<Vec<crate::models::corporate::news::News>> {
        av::fetch_news_response(symbol).await
    }

    async fn fetch_events(
        &self,
        symbol: &str,
    ) -> Result<crate::models::chart::events::ChartEvents> {
        av::fetch_events_response(symbol).await
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

    async fn fetch_forex_quote(
        &self,
        from: &str,
        to: &str,
    ) -> Result<crate::models::forex::ForexQuote> {
        av::fetch_forex_quote_response(from, to).await
    }

    async fn fetch_commodities_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::commodities::CommodityQuote> {
        av::fetch_commodities_quote_response(symbol).await
    }

    // ── Options ──────────────────────────────────────────────

    async fn fetch_options(
        &self,
        symbol: &str,
        date: Option<i64>,
    ) -> Result<crate::models::options::Options> {
        av::fetch_options_response(symbol, date).await
    }

    // ── Crypto ────────────────────────────────────────────────

    async fn fetch_crypto_quote(
        &self,
        symbol: &str,
        market: &str,
    ) -> Result<crate::models::crypto::CryptoQuote> {
        av::fetch_crypto_quote_response(symbol, market).await
    }

    // ── Economic ──────────────────────────────────────────────

    async fn fetch_economic_series(
        &self,
        series_id: &str,
    ) -> Result<crate::models::economic::EconomicSeries> {
        av::fetch_economic_series_response(series_id).await
    }
}
