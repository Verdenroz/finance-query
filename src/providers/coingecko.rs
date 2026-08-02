//! CoinGecko provider implementation.

use super::{CryptoProvider, ProviderAdapter, ProviderCore};
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
}

#[async_trait::async_trait]
impl ProviderAdapter for CoinGeckoProvider {
    fn as_crypto(&self) -> Option<&dyn CryptoProvider> {
        Some(self)
    }
}
