//! CoinGecko provider implementation.

use crate::error::Result;

pub(crate) struct CoinGeckoProvider;

#[async_trait::async_trait]
impl super::ProviderAdapter for CoinGeckoProvider {
    fn id(&self) -> &'static str {
        "coingecko"
    }
    fn capabilities(&self) -> super::Capability {
        super::Capability::CRYPTO
    }

    async fn fetch_crypto_quote(
        &self,
        id: &str,
        vs_currency: &str,
    ) -> Result<crate::models::crypto::CryptoQuote> {
        crate::adapters::coingecko::fetch_crypto_quote_response(id, vs_currency).await
    }
}
