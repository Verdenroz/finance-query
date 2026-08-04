//! Binance public market-data provider implementation (keyless).

use super::{ChartProvider, CryptoProvider, ProviderAdapter, ProviderCore};
use crate::error::Result;

pub(crate) struct BinanceProvider;

impl ProviderCore for BinanceProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Binance
    }
}

#[async_trait::async_trait]
impl CryptoProvider for BinanceProvider {
    async fn fetch_crypto_quote(
        &self,
        id: &str,
        vs_currency: &str,
    ) -> Result<crate::models::crypto::CryptoQuote> {
        crate::adapters::binance::fetch_crypto_quote_response(id, vs_currency).await
    }
}

#[async_trait::async_trait]
impl ChartProvider for BinanceProvider {
    async fn fetch_chart(
        &self,
        symbol: &str,
        interval: crate::Interval,
        range: crate::TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        crate::adapters::binance::fetch_chart_response(symbol, interval, range).await
    }

    async fn fetch_chart_range(
        &self,
        symbol: &str,
        interval: crate::Interval,
        start: i64,
        end: i64,
    ) -> Result<crate::models::chart::Chart> {
        crate::adapters::binance::fetch_chart_range_response(symbol, interval, start, end).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for BinanceProvider {
    fn as_crypto(&self) -> Option<&dyn CryptoProvider> {
        Some(self)
    }

    fn as_chart(&self) -> Option<&dyn ChartProvider> {
        Some(self)
    }
}
