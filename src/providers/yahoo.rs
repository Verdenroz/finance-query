//! Yahoo Finance provider implementation.
//!
//! Wraps YahooClient and delegates conversion to adapter functions
//! to keep this file focused on routing and lifecycle.

use super::{
    Capability, ChartProvider, CorporateProvider, FundamentalsProvider, OptionsProvider,
    ProviderAdapter, ProviderCore, QuoteProvider,
};
use crate::adapters::yahoo::client::{ClientConfig, YahooClient};
use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use std::sync::Arc;

/// Yahoo's capability set, declared rather than derived: `Provider::Yahoo`
/// can't construct a `YahooProvider` for derivation without a live auth
/// handshake. Keep in lockstep with the `as_*` overrides below — pinned by
/// `yahoo_caps_const_matches_declared_capabilities` in `super::tests`.
pub(crate) const CAPS: Capability = Capability::QUOTE
    .union(Capability::CHART)
    .union(Capability::FUNDAMENTALS)
    .union(Capability::CORPORATE)
    .union(Capability::OPTIONS);

pub(crate) struct YahooProvider {
    client: Arc<YahooClient>,
}

impl YahooProvider {
    pub(crate) async fn new(config: &ClientConfig) -> Result<Self> {
        Ok(Self {
            client: crate::adapters::yahoo::session::get_or_auth(config).await?,
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

impl ProviderCore for YahooProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Yahoo
    }
}

#[async_trait::async_trait]
impl QuoteProvider for YahooProvider {
    async fn fetch_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::quote::QuoteSummaryResponse> {
        crate::adapters::yahoo::quote::summary::fetch_summary(&self.client, symbol).await
    }

    async fn fetch_quotes_batch(
        &self,
        symbols: &[&str],
    ) -> Result<Vec<(String, crate::models::quote::QuoteSummaryResponse)>> {
        crate::adapters::yahoo::quote::quotes::fetch_quotes_batch(&self.client, symbols).await
    }
}

#[async_trait::async_trait]
impl ChartProvider for YahooProvider {
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

#[async_trait::async_trait]
impl FundamentalsProvider for YahooProvider {
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
}

#[async_trait::async_trait]
impl CorporateProvider for YahooProvider {
    async fn fetch_news(&self, symbol: &str) -> Result<Vec<crate::models::corporate::news::News>> {
        crate::adapters::yahoo::corporate::news::fetch_news(symbol).await
    }

    async fn fetch_events(
        &self,
        symbol: &str,
    ) -> Result<crate::models::chart::events::ChartEvents> {
        crate::adapters::yahoo::chart::fetch_events(&self.client, symbol).await
    }

    async fn fetch_similar_symbols(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::corporate::recommendation::SimilarSymbol>> {
        crate::adapters::yahoo::corporate::recommendations::fetch(&self.client, symbol, limit).await
    }
}

#[async_trait::async_trait]
impl OptionsProvider for YahooProvider {
    async fn fetch_options(
        &self,
        symbol: &str,
        date: Option<i64>,
    ) -> Result<crate::models::options::Options> {
        crate::adapters::yahoo::options::fetch(&self.client, symbol, date).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for YahooProvider {
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
}
