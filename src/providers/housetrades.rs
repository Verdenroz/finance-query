//! House of Representatives PTR disclosures provider implementation.
//!
//! Keyless — always available once the `housetrades` feature is compiled in.
//! Serves only [`FilingsProvider::fetch_congressional_trades`]; every other
//! `FilingsProvider` method stays the trait's `NotSupported` default, since
//! this adapter has no general filing/insider/holdings data.

use super::{FilingsProvider, Operation, Provider, ProviderAdapter, ProviderCore};
use crate::error::Result;

pub(crate) struct HouseTradesProvider;

impl ProviderCore for HouseTradesProvider {
    fn id(&self) -> Provider {
        Provider::HouseTrades
    }
}

#[async_trait::async_trait]
impl FilingsProvider for HouseTradesProvider {
    async fn fetch_filings(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::filings::ProviderFilings> {
        Err(self.not_supported(Operation::Filings))
    }

    async fn fetch_congressional_trades(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::models::filings::CongressionalTrade>> {
        crate::adapters::housetrades::fetch_congressional_trades_response(symbol).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for HouseTradesProvider {
    fn as_filings(&self) -> Option<&dyn FilingsProvider> {
        Some(self)
    }
}
