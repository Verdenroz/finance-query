//! Yahoo Finance API adapter.
//!
//! The canonical data source. Organized by capability:
//! QUOTE, CHART, FUNDAMENTALS, CORPORATE, OPTIONS, MARKET, DISCOVERY.

pub(crate) mod auth;
pub(crate) mod client;
pub(crate) mod common;
pub(crate) mod endpoints;

// Capability-mapped endpoint modules
pub(crate) mod chart; // CHART
pub(crate) mod corporate; // CORPORATE
pub(crate) mod discovery;
pub(crate) mod fundamentals; // FUNDAMENTALS
pub(crate) mod market; // MARKET
pub(crate) mod options; // OPTIONS
pub(crate) mod quote; // QUOTE // DISCOVERY
