//! CFTC Commitments of Traders provider implementation (keyless).
//!
//! CFTC publishes no price quotes, so `fetch_futures_quote` — `FuturesProvider`'s
//! required primary operation — reports `NotSupported` and dispatch falls
//! through to a quoting provider (e.g. Polygon); `fetch_commitments_of_traders`
//! is the operation this provider actually serves.

use super::{FuturesProvider, Operation, ProviderAdapter, ProviderCore};
use crate::error::Result;

pub(crate) struct CftcProvider;

impl ProviderCore for CftcProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Cftc
    }
}

#[async_trait::async_trait]
impl FuturesProvider for CftcProvider {
    async fn fetch_futures_quote(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::futures::FuturesQuote> {
        Err(self.not_supported(Operation::FuturesQuote))
    }

    async fn fetch_commitments_of_traders(
        &self,
        symbol: &str,
    ) -> Result<crate::models::futures::cot::CommitmentsOfTraders> {
        crate::adapters::cftc::fetch_commitments_of_traders_response(symbol).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for CftcProvider {
    fn as_futures(&self) -> Option<&dyn FuturesProvider> {
        Some(self)
    }
}
