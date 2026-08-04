//! SEC EDGAR provider implementation.
//!
//! Provides free, keyless SEC filing access via the EDGAR adapter.
//! Always available — no API key required (needs `EDGAR_EMAIL` env var
//! or an `edgar::init()` call before use).

use super::{FilingsProvider, ProviderAdapter, ProviderCore};
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
}

#[async_trait::async_trait]
impl ProviderAdapter for EdgarProvider {
    fn as_filings(&self) -> Option<&dyn FilingsProvider> {
        Some(self)
    }
}
