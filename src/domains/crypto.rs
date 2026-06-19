//! Cryptocurrency coin query handle.
//!
//! Created via [`Providers::crypto`](crate::Providers::crypto).

use crate::error::Result;

domain_handle! {
    /// A cryptocurrency coin backed by configured data providers.
    ///
    /// Created via [`Providers::crypto`](crate::Providers::crypto).
    pub struct CryptoCoin { id, id }
    cache: crate::models::crypto::CryptoQuote
}

impl CryptoCoin {
    /// Fetch the current quote for this coin priced in `vs_currency` (e.g., `"usd"`).
    pub async fn quote(&self, vs_currency: &str) -> Result<crate::models::crypto::CryptoQuote> {
        fetch_via_with!(
            self,
            id,
            CRYPTO,
            fetch_crypto_quote,
            vs_currency,
            crate::models::crypto::CryptoQuote
        )
    }
}
