//! DefiLlama provider implementation (keyless).
//!
//! Serves the DeFi slice of `CRYPTO`. `fetch_crypto_quote` is the trait's
//! required primary operation, but DefiLlama publishes TVL rather than prices,
//! so it reports `NotSupported` and dispatch falls through to an exchange or
//! aggregator route.

use super::{CryptoProvider, Operation, ProviderAdapter, ProviderCore};
use crate::error::Result;

pub(crate) struct DefiLlamaProvider;

impl ProviderCore for DefiLlamaProvider {
    fn id(&self) -> super::Provider {
        super::Provider::DefiLlama
    }
}

#[async_trait::async_trait]
impl CryptoProvider for DefiLlamaProvider {
    async fn fetch_crypto_quote(
        &self,
        _id: &str,
        _vs_currency: &str,
    ) -> Result<crate::models::crypto::CryptoQuote> {
        Err(self.not_supported(Operation::CryptoQuote))
    }

    async fn fetch_protocol_tvl(
        &self,
        protocol: &str,
    ) -> Result<crate::models::crypto::defi::ProtocolTvl> {
        crate::adapters::defillama::fetch_protocol_tvl_response(protocol).await
    }

    async fn fetch_protocol_tvl_history(
        &self,
        protocol: &str,
    ) -> Result<Vec<crate::models::crypto::defi::TvlPoint>> {
        crate::adapters::defillama::fetch_protocol_tvl_history_response(protocol).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for DefiLlamaProvider {
    fn as_crypto(&self) -> Option<&dyn CryptoProvider> {
        Some(self)
    }
}
