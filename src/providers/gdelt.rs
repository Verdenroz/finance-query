//! GDELT DOC 2.0 provider (keyless) — news only for corporate, crypto, forex.

use super::{
    CorporateProvider, CryptoProvider, ForexProvider, Operation, ProviderAdapter, ProviderCore,
};
use crate::error::Result;

pub(crate) struct GdeltProvider;

impl ProviderCore for GdeltProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Gdelt
    }
}

#[async_trait::async_trait]
impl CorporateProvider for GdeltProvider {
    async fn fetch_news(&self, symbol: &str) -> Result<Vec<crate::models::corporate::news::News>> {
        crate::adapters::gdelt::fetch_news_response(symbol).await
    }

    async fn fetch_events(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::chart::events::ChartEvents> {
        Err(self.not_supported(Operation::Events))
    }
}

#[async_trait::async_trait]
impl CryptoProvider for GdeltProvider {
    async fn fetch_crypto_quote(
        &self,
        _id: &str,
        _vs_currency: &str,
    ) -> Result<crate::models::crypto::CryptoQuote> {
        Err(self.not_supported(Operation::CryptoQuote))
    }

    async fn fetch_crypto_news(
        &self,
        limit: u32,
    ) -> Result<Vec<crate::models::corporate::news::News>> {
        crate::adapters::gdelt::fetch_crypto_news_response(limit).await
    }
}

#[async_trait::async_trait]
impl ForexProvider for GdeltProvider {
    async fn fetch_forex_quote(
        &self,
        _from: &str,
        _to: &str,
    ) -> Result<crate::models::forex::ForexQuote> {
        Err(self.not_supported(Operation::ForexQuote))
    }

    async fn fetch_forex_news(
        &self,
        limit: u32,
    ) -> Result<Vec<crate::models::corporate::news::News>> {
        crate::adapters::gdelt::fetch_forex_news_response(limit).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for GdeltProvider {
    fn as_corporate(&self) -> Option<&dyn CorporateProvider> {
        Some(self)
    }
    fn as_crypto(&self) -> Option<&dyn CryptoProvider> {
        Some(self)
    }
    fn as_forex(&self) -> Option<&dyn ForexProvider> {
        Some(self)
    }
}
