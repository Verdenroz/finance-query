//! [`ProviderSet`]: the ordered adapters plus the routing table, and the
//! dispatch that walks them.

use std::sync::Arc;
use std::time::Duration;

use futures::stream::StreamExt;

use super::adapter::ProviderAdapter;
use super::health::HealthTracker;
use super::retry::RetryPolicy;
use super::{Capability, Fetch, Provider, ProviderHealth, Routes};
use crate::adapters::yahoo::client::YahooClient;
use crate::error::{FinanceError, Result};

/// Not part of the stable public API — exposed only so `benches/soothfast.rs`
/// can inject a canned, network-free adapter for gating `Ticker`/`Tickers`
/// hot paths. No semver guarantees; may change or move without notice.
#[doc(hidden)]
pub struct ProviderSet {
    providers: Vec<Arc<dyn ProviderAdapter>>,
    yahoo_client: Option<Arc<YahooClient>>,
    routes: Routes,
    /// Opt-in retry policy. `None` (the default) preserves the prior
    /// behavior exactly: a `RateLimited` error is treated like any
    /// other failure and dispatch moves straight to the next candidate.
    retry_policy: Option<RetryPolicy>,
    /// In-memory recent success/failure tracker, always active (cheap to
    /// maintain) and surfaced on request via [`ProviderSet::health`].
    health: HealthTracker,
}

impl std::fmt::Debug for ProviderSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderSet")
            .field(
                "providers",
                &self.providers.iter().map(|p| p.id()).collect::<Vec<_>>(),
            )
            .field("has_yahoo_client", &self.yahoo_client.is_some())
            .field("routes", &self.routes)
            .finish()
    }
}

impl ProviderSet {
    /// Assemble a set from adapters and a route table.
    ///
    /// Adapters are tried in the order the route for a capability names them.
    pub fn new(providers: Vec<Arc<dyn ProviderAdapter>>, routes: Routes) -> Self {
        Self {
            providers,
            yahoo_client: None,
            routes,
            retry_policy: None,
            health: HealthTracker::new(),
        }
    }

    /// Attach the shared Yahoo session that `Ticker::client_handle` and logo
    /// fetching reach for. `YahooClient` is crate-private, so this cannot be
    /// part of the public constructor.
    pub(crate) fn with_yahoo_client(mut self, client: Option<Arc<YahooClient>>) -> Self {
        self.yahoo_client = client;
        self
    }

    /// Opt into [`RetryPolicy`]-driven retry of `RateLimited` errors.
    /// `None` (the default from [`new`](Self::new)) keeps a `RateLimited`
    /// error treated like any other failure.
    pub fn with_retry_policy(mut self, policy: Option<RetryPolicy>) -> Self {
        self.retry_policy = policy;
        self
    }

    /// Snapshot recent health for every configured provider, in the order
    /// they were added (see [`crate::Providers::health`]).
    pub fn health(&self) -> Vec<ProviderHealth> {
        self.providers
            .iter()
            .map(|p| {
                let mut snapshot = self.health.snapshot(p.id());
                snapshot.requests_remaining_estimate = p.rate_limit_remaining();
                snapshot
            })
            .collect()
    }

    /// Returns the providers to use for a given capability, respecting any
    /// explicit route configured via `.route()`. When no route is configured,
    /// defaults to Yahoo for all capabilities and EDGAR for filings.
    fn candidates_for(&self, cap: Capability) -> Vec<&Arc<dyn ProviderAdapter>> {
        if let Some(provider_ids) = self.routes.map.get(&cap) {
            provider_ids
                .iter()
                .filter_map(|id| self.providers.iter().find(|p| p.id() == *id))
                .collect()
        } else if cap == Capability::FILINGS {
            // Default: EDGAR (keyless SEC filings) first, then Yahoo
            let mut v: Vec<&Arc<dyn ProviderAdapter>> = self
                .providers
                .iter()
                .filter(|p| p.id() == Provider::Edgar)
                .collect();
            v.extend(self.providers.iter().filter(|p| p.id() == Provider::Yahoo));
            v
        } else {
            // Default: Yahoo only
            self.providers
                .iter()
                .filter(|p| p.id() == Provider::Yahoo)
                .collect()
        }
    }

    fn no_provider(cap: Capability) -> FinanceError {
        FinanceError::NoProviderAvailable {
            operation: cap,
            candidates: cap.candidate_providers(),
        }
    }

    /// Real provider failures outrank `NotSupported` (which just means "next
    /// candidate"), but when *every* candidate lacked the operation, surface
    /// the precise per-operation `NotSupported` instead of collapsing to a
    /// capability-level `NoProviderAvailable`.
    fn finish_err(
        cap: Capability,
        last: Option<FinanceError>,
        unsupported: Option<FinanceError>,
    ) -> FinanceError {
        last.or(unsupported)
            .unwrap_or_else(|| Self::no_provider(cap))
    }

    /// Call `f(p)`, retrying in place on `FinanceError::RateLimited` per
    /// `self.retry_policy`. With no policy configured this is
    /// exactly `f(p).await` — zero behavior change for existing callers.
    async fn call_with_retry<T, F, Fut>(&self, p: &Arc<dyn ProviderAdapter>, f: &F) -> Result<T>
    where
        F: Fn(&Arc<dyn ProviderAdapter>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let Some(policy) = self.retry_policy.as_ref() else {
            return f(p).await;
        };
        let mut seed = crate::backoff::seed_from_time();
        let mut attempt: u32 = 0;
        loop {
            match f(p).await {
                Ok(v) => return Ok(v),
                Err(FinanceError::RateLimited { retry_after })
                    if attempt + 1 < policy.max_attempts =>
                {
                    let delay =
                        policy.delay_for(attempt, retry_after.map(Duration::from_secs), &mut seed);
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Record a candidate's outcome in the health tracker. `NotSupported`
    /// isn't a provider failure (it just means "try the next candidate"), so
    /// it's excluded from health accounting entirely.
    fn record_health<T>(&self, provider: Provider, result: &Result<T>) {
        match result {
            Ok(_) => self.health.record(provider, true, None),
            Err(FinanceError::NotSupported { .. }) => {}
            Err(e) => self.health.record(provider, false, Some(e.to_string())),
        }
    }

    pub(crate) async fn fetch<T, F, Fut>(&self, cap: Capability, f: F) -> Result<T>
    where
        F: Fn(&Arc<dyn ProviderAdapter>) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let candidates = self.candidates_for(cap);
        if candidates.is_empty() {
            return Err(Self::no_provider(cap));
        }
        match self.routes.fetch {
            Fetch::Sequential => {
                let mut last = None;
                let mut unsupported = None;
                for p in &candidates {
                    let result = self.call_with_retry(p, &f).await;
                    self.record_health(p.id(), &result);
                    match result {
                        Ok(v) => return Ok(v),
                        Err(e @ FinanceError::NotSupported { .. }) => unsupported = Some(e),
                        Err(e) => last = Some(e),
                    }
                }
                Err(Self::finish_err(cap, last, unsupported))
            }
            Fetch::Parallel => {
                let mut futs = futures::stream::FuturesUnordered::new();
                for p in &candidates {
                    let id = p.id();
                    // Capture `&f` (Copy) rather than `f` itself — `f: F` isn't
                    // necessarily `Copy`, and `async move` would otherwise try
                    // (and fail) to move it out on every loop iteration.
                    let f_ref = &f;
                    futs.push(async move {
                        let result = self.call_with_retry(p, f_ref).await;
                        self.record_health(id, &result);
                        result
                    });
                }
                let mut last = None;
                let mut unsupported = None;
                while let Some(r) = futs.next().await {
                    match r {
                        Ok(v) => return Ok(v),
                        Err(e @ FinanceError::NotSupported { .. }) => unsupported = Some(e),
                        Err(e) => last = Some(e),
                    }
                }
                Err(Self::finish_err(cap, last, unsupported))
            }
        }
    }

    pub(crate) fn first_yahoo(&self) -> Result<Arc<YahooClient>> {
        self.yahoo_client.as_ref().map(Arc::clone).ok_or_else(|| {
            FinanceError::NoProviderAvailable {
                operation: Capability::QUOTE,
                candidates: vec![Provider::Yahoo],
            }
        })
    }
}
