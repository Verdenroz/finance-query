//! SEC EDGAR provider implementation.
//!
//! Provides free, keyless SEC filing access via the EDGAR adapter.
//! Always available — no API key required (needs `EDGAR_EMAIL` env var
//! or an `edgar::init()` call before use).

use super::{
    CorporateProvider, DiscoveryProvider, FilingsProvider, Operation, ProviderAdapter, ProviderCore,
};
use crate::error::Result;
use crate::models::filings::ProviderFilings;

pub(crate) struct EdgarProvider;

impl ProviderCore for EdgarProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Edgar
    }
}

#[async_trait::async_trait]
impl FilingsProvider for EdgarProvider {
    async fn fetch_filings(&self, symbol: &str) -> Result<ProviderFilings> {
        crate::adapters::edgar::fetch_filings_response(symbol).await
    }

    async fn fetch_filing_search(
        &self,
        symbol: Option<&str>,
        query: &str,
        filters: &crate::models::filings::FilingSearchFilters,
    ) -> Result<Vec<crate::models::filings::FilingSearchHit>> {
        crate::adapters::edgar::filings::full_text::fetch_filing_search_response(
            symbol, query, filters,
        )
        .await
    }

    async fn fetch_insider_trades(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::filings::InsiderTrade>> {
        crate::adapters::edgar::filings::insider::fetch_insider_trades_response(symbol, limit).await
    }

    async fn fetch_institutional_holdings(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::models::filings::InstitutionalHolding>> {
        crate::adapters::edgar::filings::thirteen_f::fetch_institutional_holdings_response(symbol)
            .await
    }

    #[cfg(feature = "secftd")]
    async fn fetch_fails_to_deliver(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::models::filings::FailToDeliver>> {
        crate::adapters::edgar::filings::fails_to_deliver::fetch_fails_to_deliver_response(symbol)
            .await
    }

    async fn fetch_filing_sections(
        &self,
        accession_number: &str,
        form: crate::models::filings::FilingSectionForm,
    ) -> Result<Vec<crate::models::filings::FilingSection>> {
        crate::adapters::edgar::filings::sections::fetch_filing_sections_response(
            accession_number,
            form,
        )
        .await
    }

    async fn fetch_risk_factors(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::models::filings::RiskFactor>> {
        crate::adapters::edgar::filings::sections::fetch_risk_factors_response(symbol).await
    }
}

/// EDGAR has no general news feed — only 8-K exhibit press releases.
#[async_trait::async_trait]
impl CorporateProvider for EdgarProvider {
    async fn fetch_news(&self, _symbol: &str) -> Result<Vec<crate::models::corporate::news::News>> {
        Err(self.not_supported(Operation::News))
    }

    async fn fetch_events(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::chart::events::ChartEvents> {
        Err(self.not_supported(Operation::Events))
    }

    async fn fetch_press_releases(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::corporate::press_release::PressRelease>> {
        crate::adapters::edgar::filings::press_releases::fetch_press_releases_response(
            symbol, limit,
        )
        .await
    }

    async fn fetch_employee_count(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::models::corporate::governance::EmployeeCount>> {
        crate::adapters::edgar::filings::employee_count::fetch_employee_count_response(symbol).await
    }
}

/// EDGAR has no free-text symbol search — only the bulk ticker listing.
#[async_trait::async_trait]
impl DiscoveryProvider for EdgarProvider {
    async fn fetch_symbol_search(
        &self,
        _query: &str,
        _limit: u32,
    ) -> Result<Vec<crate::models::discovery::reference::SymbolMatch>> {
        Err(self.not_supported(Operation::SymbolSearch))
    }

    async fn fetch_listing_status(
        &self,
        active: bool,
    ) -> Result<Vec<crate::models::discovery::reference::SymbolMatch>> {
        crate::adapters::edgar::discovery::fetch_listing_status_response(active).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for EdgarProvider {
    fn as_filings(&self) -> Option<&dyn FilingsProvider> {
        Some(self)
    }

    fn as_corporate(&self) -> Option<&dyn CorporateProvider> {
        Some(self)
    }

    fn as_discovery(&self) -> Option<&dyn DiscoveryProvider> {
        Some(self)
    }
}
