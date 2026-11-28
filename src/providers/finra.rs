//! FINRA provider implementation (keyless).
//!
//! Serves the short-volume slice of `FUNDAMENTALS` only. `fetch_financials`
//! is the trait's required primary operation but FINRA publishes no financial
//! statements, so it reports `NotSupported` and dispatch falls through to the
//! next routed provider.

use super::{FundamentalsProvider, Operation, ProviderAdapter, ProviderCore};
use crate::error::Result;

pub(crate) struct FinraProvider;

impl ProviderCore for FinraProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Finra
    }
}

#[async_trait::async_trait]
impl FundamentalsProvider for FinraProvider {
    async fn fetch_financials(
        &self,
        _symbol: &str,
        _stmt_type: crate::StatementType,
        _frequency: crate::Frequency,
    ) -> Result<finance_query_models::fundamentals::FinancialStatement> {
        Err(self.not_supported(Operation::Financials))
    }

    async fn fetch_short_volume(
        &self,
        symbol: &str,
    ) -> Result<Vec<finance_query_models::fundamentals::ShortVolume>> {
        finance_query_adapters_gov::finra::fetch_short_volume_response(symbol).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for FinraProvider {
    fn as_fundamentals(&self) -> Option<&dyn FundamentalsProvider> {
        Some(self)
    }
}
