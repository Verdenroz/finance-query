//! Yahoo Finance provider implementation.
//!
//! Wraps YahooClient and delegates conversion to adapter functions
//! to keep this file focused on routing and lifecycle.

use crate::adapters::yahoo::client::{ClientConfig, YahooClient};
use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use std::sync::Arc;

pub(crate) struct YahooProvider {
    client: Arc<YahooClient>,
}

impl YahooProvider {
    pub(crate) async fn new(config: &ClientConfig) -> Result<Self> {
        Ok(Self {
            client: Arc::new(YahooClient::new(config.clone()).await?),
        })
    }

    /// Wrap an existing authenticated client — no new auth handshake.
    pub(crate) fn from_client(client: Arc<YahooClient>) -> Self {
        Self { client }
    }

    pub(crate) fn client_arc(&self) -> Arc<YahooClient> {
        Arc::clone(&self.client)
    }
}

#[async_trait::async_trait]
impl super::ProviderAdapter for YahooProvider {
    fn id(&self) -> &'static str {
        "yahoo"
    }

    fn capabilities(&self) -> super::Capability {
        super::Capability::QUOTE
            | super::Capability::CHART
            | super::Capability::FUNDAMENTALS
            | super::Capability::CORPORATE
            | super::Capability::OPTIONS
    }

    // ── Quote ─────────────────────────────────────────────────────

    async fn fetch_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::quote::QuoteSummaryResponse> {
        crate::adapters::yahoo::quote::summary::fetch_summary(&self.client, symbol).await
    }

    // ── Chart ─────────────────────────────────────────────────────

    async fn fetch_chart(
        &self,
        symbol: &str,
        interval: Interval,
        range: TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        crate::adapters::yahoo::chart::fetch_chart(&self.client, symbol, interval, range).await
    }

    async fn fetch_chart_range(
        &self,
        symbol: &str,
        interval: Interval,
        start: i64,
        end: i64,
    ) -> Result<crate::models::chart::Chart> {
        crate::adapters::yahoo::chart::fetch_chart_with_dates(
            &self.client,
            symbol,
            interval,
            start,
            end,
        )
        .await
    }

    // ── Financials ────────────────────────────────────────────────

    async fn fetch_financials(
        &self,
        symbol: &str,
        stmt_type: crate::StatementType,
        frequency: crate::Frequency,
    ) -> Result<crate::models::fundamentals::FinancialStatement> {
        let mut stmt =
            crate::adapters::yahoo::fundamentals::fetch(&self.client, symbol, stmt_type, frequency)
                .await?;
        stmt.provider_id = Some(super::Provider::Yahoo);
        Ok(stmt)
    }

    // ── News ──────────────────────────────────────────────────────

    async fn fetch_news(&self, symbol: &str) -> Result<Vec<crate::models::corporate::news::News>> {
        crate::adapters::yahoo::corporate::news::fetch_news(symbol).await
    }

    // ── Recommendations ───────────────────────────────────────────

    async fn fetch_similar_symbols(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::corporate::recommendation::SimilarSymbol>> {
        crate::adapters::yahoo::corporate::recommendations::fetch(&self.client, symbol, limit).await
    }

    // ── Options ───────────────────────────────────────────────────

    async fn fetch_options(
        &self,
        symbol: &str,
        date: Option<i64>,
    ) -> Result<crate::models::options::Options> {
        crate::adapters::yahoo::options::fetch(&self.client, symbol, date).await
    }

    // ── Events ────────────────────────────────────────────────────

    async fn fetch_events(
        &self,
        symbol: &str,
    ) -> Result<crate::models::chart::events::ChartEvents> {
        crate::adapters::yahoo::chart::fetch_events(&self.client, symbol).await
    }

    // ── Batch ─────────────────────────────────────────────────────

    async fn fetch_quotes_batch(
        &self,
        symbols: &[&str],
    ) -> Result<Vec<(String, crate::models::quote::QuoteSummaryResponse)>> {
        crate::adapters::yahoo::quote::quotes::fetch_quotes_batch(&self.client, symbols).await
    }

    async fn fetch_spark(
        &self,
        symbols: &[&str],
        interval: Interval,
        range: TimeRange,
    ) -> Result<Vec<(String, crate::models::chart::spark::Spark)>> {
        use crate::models::chart::spark::Spark;
        use crate::models::chart::spark::response::SparkResponse;

        let json =
            crate::adapters::yahoo::quote::spark::fetch(&self.client, symbols, interval, range)
                .await?;
        let spark_response = SparkResponse::from_json(json)?;

        let mut out = Vec::new();
        if let Some(results) = spark_response.spark.result {
            for result in &results {
                if let Some(spark) = Spark::from_response(
                    result,
                    Some(interval.as_str().to_string()),
                    Some(range.as_str().to_string()),
                ) {
                    out.push((result.symbol.clone(), spark));
                }
            }
        }
        Ok(out)
    }
}
