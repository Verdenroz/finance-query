//! US Treasury FiscalData provider implementation (keyless).

use super::{EconomicProvider, ProviderAdapter, ProviderCore};
use crate::error::Result;

pub(crate) struct FiscalDataProvider;

impl ProviderCore for FiscalDataProvider {
    fn id(&self) -> super::Provider {
        super::Provider::FiscalData
    }
}

#[async_trait::async_trait]
impl EconomicProvider for FiscalDataProvider {
    async fn fetch_economic_series(
        &self,
        series_id: &str,
    ) -> Result<crate::models::economic::EconomicSeries> {
        crate::adapters::fiscaldata::fetch_economic_series_response(series_id).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for FiscalDataProvider {
    fn as_economic(&self) -> Option<&dyn EconomicProvider> {
        Some(self)
    }
}
