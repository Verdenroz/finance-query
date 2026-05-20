//! Forex currency-pair query handle.
//!
//! Created via [`Providers::forex`](crate::Providers::forex).

use crate::error::Result;
use crate::providers::{Capability, ProviderSet};
use std::sync::Arc;

/// A foreign-exchange currency pair backed by configured data providers.
///
/// Created via [`Providers::forex`](crate::Providers::forex).
pub struct ForexPair {
    from: Arc<str>,
    to: Arc<str>,
    providers: Arc<ProviderSet>,
}

impl ForexPair {
    pub(crate) fn with_providers(
        from: Arc<str>,
        to: Arc<str>,
        providers: Arc<ProviderSet>,
    ) -> Self {
        Self {
            from,
            to,
            providers,
        }
    }

    /// The base (from) currency code (e.g., `"USD"`).
    pub fn from(&self) -> &str {
        &self.from
    }

    /// The quote (to) currency code (e.g., `"EUR"`).
    pub fn to(&self) -> &str {
        &self.to
    }

    /// Fetch the current exchange rate for this currency pair.
    pub async fn quote(&self) -> Result<crate::models::forex::ForexQuote> {
        let from = self.from.clone();
        let to = self.to.clone();
        self.providers
            .fetch(Capability::FOREX, move |p| {
                let f = from.clone();
                let t = to.clone();
                let p = p.clone();
                async move { p.fetch_forex_quote(&f, &t).await }
            })
            .await
    }
}
