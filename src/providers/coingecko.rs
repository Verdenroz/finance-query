//! CoinGecko provider implementation.

use super::{ChartProvider, CryptoProvider, DiscoveryProvider, ProviderAdapter, ProviderCore};
use crate::error::Result;

pub(crate) struct CoinGeckoProvider;

impl ProviderCore for CoinGeckoProvider {
    fn id(&self) -> super::Provider {
        super::Provider::CoinGecko
    }
}

#[async_trait::async_trait]
impl CryptoProvider for CoinGeckoProvider {
    async fn fetch_crypto_quote(
        &self,
        id: &str,
        vs_currency: &str,
    ) -> Result<crate::models::crypto::CryptoQuote> {
        crate::adapters::coingecko::fetch_crypto_quote_response(id, vs_currency).await
    }

    async fn fetch_crypto_trending(&self) -> Result<Vec<crate::models::crypto::TrendingCoin>> {
        crate::adapters::coingecko::fetch_crypto_trending_response().await
    }

    async fn fetch_crypto_global(&self) -> Result<crate::models::crypto::GlobalCryptoStats> {
        crate::adapters::coingecko::fetch_crypto_global_response().await
    }
}

#[async_trait::async_trait]
impl DiscoveryProvider for CoinGeckoProvider {
    async fn fetch_symbol_search(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::discovery::reference::SymbolMatch>> {
        crate::adapters::coingecko::fetch_symbol_search_response(query, limit).await
    }
}

#[async_trait::async_trait]
impl ChartProvider for CoinGeckoProvider {
    /// Chart symbols arrive as `"{ID}-{VS}"` from
    /// [`CryptoCoin::chart`](crate::CryptoCoin::chart); CoinGecko wants the
    /// coin id and quote currency separately, so the symbol is split back apart.
    async fn fetch_chart(
        &self,
        symbol: &str,
        interval: crate::Interval,
        range: crate::TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        crate::adapters::coingecko::chart::fetch_chart_response(symbol, interval, range).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for CoinGeckoProvider {
    fn as_crypto(&self) -> Option<&dyn CryptoProvider> {
        Some(self)
    }
    fn as_chart(&self) -> Option<&dyn ChartProvider> {
        Some(self)
    }
    fn as_discovery(&self) -> Option<&dyn DiscoveryProvider> {
        Some(self)
    }
}
