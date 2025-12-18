//! Market movers models.
//!
//! Types for market movers data (most actives, day gainers, day losers).
//!
//! ## Usage
//!
//! The API returns a clean, flattened `MoversResponse`:
//! ```json
//! {
//!   "quotes": [...],
//!   "type": "MOST_ACTIVES",
//!   "lastUpdated": 1234567890
//! }
//! ```
//!
//! Each quote in the array is a `MoverQuote` with ~50 fields of market data.

mod quote;
mod response;

pub use quote::MoverQuote;
pub use response::MoversResponse;
