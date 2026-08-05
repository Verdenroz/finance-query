//! GDELT DOC 2.0 provider implementation (keyless).
//!
//! Serves only the news half of `CORPORATE` — GDELT has no corporate
//! calendar (earnings/dividends/splits), so `fetch_events` reports
//! `NotSupported` and dispatch falls through to the next routed provider.

use super::{CorporateProvider, Operation, ProviderAdapter, ProviderCore};
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
impl ProviderAdapter for GdeltProvider {
    fn as_corporate(&self) -> Option<&dyn CorporateProvider> {
        Some(self)
    }
}
