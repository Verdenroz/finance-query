//! Frankfurter (ECB reference rates) provider implementation (keyless).

use super::{ForexProvider, ProviderAdapter, ProviderCore};
use crate::error::Result;

pub(crate) struct FrankfurterProvider;

impl ProviderCore for FrankfurterProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Frankfurter
    }
}

#[async_trait::async_trait]
impl ForexProvider for FrankfurterProvider {
    async fn fetch_forex_quote(
        &self,
        from: &str,
        to: &str,
    ) -> Result<crate::models::forex::ForexQuote> {
        crate::adapters::frankfurter::fetch_forex_quote_response(from, to).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for FrankfurterProvider {
    fn as_forex(&self) -> Option<&dyn ForexProvider> {
        Some(self)
    }
}
