//! SEC EDGAR provider implementation.
//!
//! Provides free, keyless SEC filing access via the EDGAR adapter.
//! Always available — no API key required (needs `EDGAR_EMAIL` env var
//! or an `edgar::init()` call before use).

use crate::error::Result;
use crate::models::filings::ProviderFilings;

pub(crate) struct EdgarProvider;

#[async_trait::async_trait]
impl super::ProviderAdapter for EdgarProvider {
    fn id(&self) -> &'static str {
        "edgar"
    }
    fn capabilities(&self) -> super::Capability {
        super::Capability::FILINGS
    }

    async fn fetch_filings(&self, symbol: &str) -> Result<ProviderFilings> {
        crate::adapters::edgar::fetch_filings_response(symbol).await
    }
}
