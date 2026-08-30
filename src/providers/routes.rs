//! Per-capability routing: which providers serve a capability, and how they
//! are queried.

use std::collections::HashMap;

use super::{Capability, Provider};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How providers are queried.
#[non_exhaustive]
pub enum Fetch {
    /// Try providers in priority order; first success wins.
    Sequential,
    /// Fire all providers concurrently; first success wins.
    Parallel,
}

#[derive(Debug)]
pub(crate) struct Route {
    pub(crate) providers: Vec<Provider>,
    pub(crate) fetch: Option<Fetch>,
}

/// Per-capability provider routing table.
///
/// Maps each [`Capability`] to an ordered list of [`Provider`]s to try. A
/// capability with no entry falls back to Yahoo, or to EDGAR then Yahoo for
/// [`Capability::FILINGS`].
///
/// Each route may carry its own [`Fetch`] mode, so a quota-limited capability
/// can stay sequential while another races its providers. Routes without one
/// use the table default.
#[derive(Debug)]
#[non_exhaustive]
pub struct Routes {
    pub(crate) map: HashMap<Capability, Route>,
    pub(crate) fetch: Fetch,
}

impl Routes {
    /// An empty route table (every capability falls back to its default
    /// candidate providers) using the given concurrency [`Fetch`] mode.
    pub fn new(fetch: Fetch) -> Self {
        Self {
            map: HashMap::new(),
            fetch,
        }
    }

    /// Route one capability to an ordered list of providers.
    ///
    /// Without this a hand-built table is empty and every capability falls
    /// back to its default, so a registered adapter would never be reached.
    #[must_use]
    pub fn route(self, cap: Capability, providers: impl IntoIterator<Item = Provider>) -> Self {
        self.insert(cap, providers, None)
    }

    /// Route one capability, overriding the table's [`Fetch`] mode for it.
    #[must_use]
    pub fn route_with(
        self,
        cap: Capability,
        providers: impl IntoIterator<Item = Provider>,
        fetch: Fetch,
    ) -> Self {
        self.insert(cap, providers, Some(fetch))
    }

    fn insert(
        mut self,
        cap: Capability,
        providers: impl IntoIterator<Item = Provider>,
        fetch: Option<Fetch>,
    ) -> Self {
        self.map.insert(
            cap,
            Route {
                providers: providers.into_iter().collect(),
                fetch,
            },
        );
        self
    }

    /// The default concurrency mode for capabilities that do not override it.
    pub fn fetch_mode(&self) -> Fetch {
        self.fetch
    }

    /// The concurrency mode `cap` resolves to.
    pub fn fetch_mode_for(&self, cap: Capability) -> Fetch {
        self.map
            .get(&cap)
            .and_then(|r| r.fetch)
            .unwrap_or(self.fetch)
    }

    /// The providers routed to `cap`, or `None` when it falls back to the default.
    pub fn providers_for(&self, cap: Capability) -> Option<&[Provider]> {
        self.map.get(&cap).map(|r| r.providers.as_slice())
    }
}
