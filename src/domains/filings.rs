//! SEC filing query handle.
//!
//! Created via [`Providers::filings`](crate::Providers::filings). Always
//! available — backed by EDGAR (keyless) with optional Polygon fallback.

use crate::error::Result;

domain_handle! {
    /// SEC filing data backed by configured data providers.
    ///
    /// Created via [`Providers::filings`](crate::Providers::filings).
    pub struct Filings { symbol, symbol }
}

impl Filings {
    /// Fetch SEC filings for this symbol.
    pub async fn get(&self) -> Result<crate::models::filings::ProviderFilings> {
        fetch_via!(
            self,
            symbol,
            FILINGS,
            fetch_filings,
            crate::models::filings::ProviderFilings
        )
    }
}
