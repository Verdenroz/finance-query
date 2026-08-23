//! United States Senate Periodic Transaction Report (PTR) disclosures —
//! keyless, browser-driven, from the primary source.
//!
//! Requires the **`senatetrades`** feature flag.
//!
//! The STOCK Act of 2012 requires members to disclose trades over $1,000
//! within 45 days via a Periodic Transaction Report. The Senate publishes
//! these free of charge at `efdsearch.senate.gov`, but unlike the House
//! Clerk's static per-year archive, the disclaimer gate, search results, and
//! each filing's transaction table all render client-side behind Akamai bot
//! protection — so this adapter drives a real headless Chromium via
//! `chromiumoxide` instead of a plain HTTP client. See [`client`] for the
//! launch flags and the IP-reputation/rate-limit constraints that come with
//! them.
//!
//! Like the House adapter, there is no per-symbol query endpoint — search is
//! by filer type/date only — so this scans a bounded window of the most
//! recent Senator PTR filings rather than the full archive. Filings typed
//! through the site's e-filing system render an HTML transaction table;
//! older ("paper") filings don't, and are silently skipped rather than
//! OCR'd — see [`filings`] for that boundary.

pub(crate) mod client;
pub(crate) mod filings;
pub(crate) mod models;

pub(crate) use filings::fetch_congressional_trades_response;
