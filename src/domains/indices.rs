//! Stock market index quote handle.
//!
//! Created via [`Providers::index`](crate::Providers::index).

use crate::error::Result;

domain_handle! {
    /// A stock market index backed by configured data providers.
    ///
    /// Created via [`Providers::index`](crate::Providers::index).
    pub struct Index { symbol, symbol }
    cache: crate::models::indices::IndexQuote
}

impl Index {
    /// Fetch the current quote for this index.
    pub async fn quote(&self) -> Result<crate::models::indices::IndexQuote> {
        fetch_via!(
            self,
            symbol,
            INDICES,
            fetch_indices_quote,
            crate::models::indices::IndexQuote
        )
    }
}
