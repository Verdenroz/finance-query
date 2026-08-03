//! Cross-market snapshot handle.
//!
//! Created via [`Providers::snapshot`](crate::Providers::snapshot).

use std::sync::Arc;

use crate::error::Result;
use crate::models::quote::snapshot::MarketSnapshot;
use crate::providers::{Capability, Operation};

domain_handle! {
    /// Snapshots for a watchlist spanning several asset classes, from one request.
    ///
    /// Routes through [`Capability::QUOTE`]. Unlike
    /// [`Tickers`](crate::Tickers) — which is equity-shaped and returns full quote
    /// summaries — this takes provider-spelled symbols from any market (`"AAPL"`,
    /// `"X:BTCUSD"`, `"I:SPX"`, `"C:EURUSD"`, `"O:NCLH221014C00005000"`) and
    /// returns one flattened row each. Polygon is currently the only provider whose
    /// snapshot endpoint spans markets.
    ///
    /// Created via [`Providers::snapshot`](crate::Providers::snapshot).
    pub struct Snapshot
    caches: { cache: Vec<MarketSnapshot> }
}

impl Snapshot {
    /// Fetch snapshots for `symbols`, which may span asset classes.
    ///
    /// Cached per symbol list. Symbols the provider could not resolve come back
    /// as rows with [`MarketSnapshot::error`] set rather than being dropped, so
    /// the result can be aligned with the request.
    pub async fn get<S: AsRef<str>>(&self, symbols: &[S]) -> Result<Vec<MarketSnapshot>> {
        let owned: Vec<String> = symbols.iter().map(|s| s.as_ref().to_string()).collect();
        let key = owned.join("\u{1f}");
        let providers = Arc::clone(&self.providers);

        self.cache
            .get_or_try(key, move || async move {
                providers
                    .fetch(Capability::QUOTE, move |p| {
                        let owned = owned.clone();
                        let p = p.clone();
                        async move {
                            let refs: Vec<&str> = owned.iter().map(String::as_str).collect();
                            p.as_quote()
                                .ok_or_else(|| p.not_supported(Operation::UnifiedSnapshot))?
                                .fetch_unified_snapshot(&refs)
                                .await
                        }
                    })
                    .await
            })
            .await
    }
}
