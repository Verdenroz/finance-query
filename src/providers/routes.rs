//! Per-capability routing: which providers serve a capability, and how they
//! are queried.

use std::collections::HashMap;

use super::{Capability, Provider};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How providers are queried.
pub enum Fetch {
    /// Try providers in priority order; first success wins.
    Sequential,
    /// Fire all providers concurrently; first success wins.
    Parallel,
}

/// Per-capability provider routing table.
///
/// Maps each [`Capability`] to an ordered list of [`Provider`]s to try. A
/// capability with no entry falls back to Yahoo, or to EDGAR then Yahoo for
/// [`Capability::FILINGS`].
#[derive(Debug)]
#[non_exhaustive]
pub struct Routes {
    pub(crate) map: HashMap<Capability, Vec<Provider>>,
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
    pub fn route(mut self, cap: Capability, providers: impl IntoIterator<Item = Provider>) -> Self {
        self.map.insert(cap, providers.into_iter().collect());
        self
    }

    /// The concurrency mode this table was built with.
    pub fn fetch_mode(&self) -> Fetch {
        self.fetch
    }

    /// The providers routed to `cap`, or `None` when it falls back to the default.
    pub fn providers_for(&self, cap: Capability) -> Option<&[Provider]> {
        self.map.get(&cap).map(Vec::as_slice)
    }
}
