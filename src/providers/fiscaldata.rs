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

    async fn fetch_treasury_auctions(
        &self,
        query: &crate::models::economic::TreasuryAuctionQuery,
    ) -> Result<Vec<crate::models::economic::TreasuryAuction>> {
        crate::adapters::fiscaldata::fetch_treasury_auctions_response(query).await
    }

    async fn fetch_upcoming_auctions(
        &self,
    ) -> Result<Vec<crate::models::economic::UpcomingAuction>> {
        crate::adapters::fiscaldata::fetch_upcoming_auctions_response().await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for FiscalDataProvider {
    fn as_economic(&self) -> Option<&dyn EconomicProvider> {
        Some(self)
    }
}
