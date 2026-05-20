//! Futures contract quote handle.
//!
//! Created via [`Providers::futures`](crate::Providers::futures).

use crate::error::Result;

domain_handle! {
    /// A futures contract backed by configured data providers.
    ///
    /// Created via [`Providers::futures`](crate::Providers::futures).
    pub struct FuturesContract { symbol, symbol }
}

impl FuturesContract {
    /// Fetch the current quote for this futures contract.
    pub async fn quote(&self) -> Result<crate::models::futures::FuturesQuote> {
        fetch_via!(
            self,
            symbol,
            FUTURES,
            fetch_futures_quote,
            crate::models::futures::FuturesQuote
        )
    }
}
