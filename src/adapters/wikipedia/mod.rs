//! Wikipedia — keyless index constituent lists.
//!
//! Requires the **`wikipedia`** feature flag.
//!
//! Table-scrapes "List of S&P 500 companies" for current constituents and
//! "Historical components of the S&P 500" for constituent-change history —
//! the only tracked index with inline tables for either (see `indices` for
//! why Nasdaq-100/Dow Jones stay unrouted). Reuses the crate's
//! dependency-free HTML element matcher (`crate::scrapers::html`), the same
//! one `scrapers::yahoo_exchanges` already scrapes a table with.

pub(crate) mod client;
pub(crate) mod indices;

use std::time::Duration;

use crate::adapters::singleton::keyless_limiter;
use crate::error::Result;
use client::WikipediaClient;

/// Self-imposed pacing. These are static, rarely-changing pages — there's no
/// reason to hit them often even discounting Wikipedia's own tolerance.
const WIKIPEDIA_RATE_PER_SEC: f64 = 1.0;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

keyless_limiter!(rate = WIKIPEDIA_RATE_PER_SEC);

/// Build a client against the live site, reusing the shared token bucket.
fn client() -> Result<WikipediaClient> {
    WikipediaClient::new(DEFAULT_TIMEOUT, shared_limiter(), client::WIKIPEDIA_BASE)
}

pub(crate) use indices::{
    fetch_index_constituent_changes_response, fetch_index_constituents_response,
};
