//! Kraken public market-data provider implementation (keyless).

use super::{ChartProvider, CryptoProvider, ProviderAdapter, ProviderCore};
use crate::error::Result;

pub(crate) struct KrakenProvider;

impl ProviderCore for KrakenProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Kraken
    }
}

#[async_trait::async_trait]
impl CryptoProvider for KrakenProvider {
    async fn fetch_crypto_quote(
        &self,
        id: &str,
        vs_currency: &str,
    ) -> Result<crate::models::crypto::CryptoQuote> {
        crate::adapters::kraken::fetch_crypto_quote_response(id, vs_currency).await
    }
}

#[async_trait::async_trait]
impl ChartProvider for KrakenProvider {
    async fn fetch_chart(
        &self,
        symbol: &str,
        interval: crate::Interval,
        range: crate::TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        crate::adapters::kraken::fetch_chart_response(symbol, interval, range).await
    }

    async fn fetch_chart_range(
        &self,
        symbol: &str,
        interval: crate::Interval,
        start: i64,
        end: i64,
    ) -> Result<crate::models::chart::Chart> {
        crate::adapters::kraken::fetch_chart_range_response(symbol, interval, start, end).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for KrakenProvider {
    fn as_crypto(&self) -> Option<&dyn CryptoProvider> {
        Some(self)
    }

    fn as_chart(&self) -> Option<&dyn ChartProvider> {
        Some(self)
    }
}
