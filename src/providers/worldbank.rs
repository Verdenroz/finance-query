//! World Bank Open Data provider implementation (keyless).

use super::{EconomicProvider, ProviderAdapter, ProviderCore};
use crate::error::Result;

pub(crate) struct WorldBankProvider;

impl ProviderCore for WorldBankProvider {
    fn id(&self) -> super::Provider {
        super::Provider::WorldBank
    }
}

#[async_trait::async_trait]
impl EconomicProvider for WorldBankProvider {
    async fn fetch_economic_series(
        &self,
        series_id: &str,
    ) -> Result<crate::models::economic::EconomicSeries> {
        crate::adapters::worldbank::fetch_economic_series_response(series_id).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for WorldBankProvider {
    fn as_economic(&self) -> Option<&dyn EconomicProvider> {
        Some(self)
    }
}
