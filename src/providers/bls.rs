//! BLS (Bureau of Labor Statistics) provider implementation.
//!
//! Keyless by default; `BLS_API_KEY` upgrades every call to the v2 tier.

use super::{EconomicProvider, ProviderAdapter, ProviderCore};
use crate::error::Result;

pub(crate) struct BlsProvider;

impl ProviderCore for BlsProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Bls
    }
}

#[async_trait::async_trait]
impl EconomicProvider for BlsProvider {
    async fn fetch_economic_series(
        &self,
        series_id: &str,
    ) -> Result<crate::models::economic::EconomicSeries> {
        crate::adapters::bls::fetch_economic_series_response(series_id).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for BlsProvider {
    fn as_economic(&self) -> Option<&dyn EconomicProvider> {
        Some(self)
    }
}
