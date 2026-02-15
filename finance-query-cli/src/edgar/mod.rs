//! Interactive TUI for browsing SEC EDGAR filings.
//!
//! Displays filing history with metadata and allows opening filings in browser.

mod render;
mod state;

pub use state::{run_empty, run_search, run_symbol};
