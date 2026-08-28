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
/// Maps each [`Capability`] to an ordered list of [`Provider`]s to try.
/// When a capability has no entry, all providers declaring that capability are used.
#[derive(Debug)]
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
}
