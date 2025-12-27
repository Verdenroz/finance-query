//! Market hours models.
//!
//! Types for market time data from Yahoo Finance's markettime endpoint.
//!
//! ## Usage
//!
//! The API returns a clean, flattened `HoursResponse`:
//! ```json
//! {
//!   "markets": [
//!     {
//!       "id": "us",
//!       "name": "U.S. markets",
//!       "status": "closed",
//!       "message": "U.S. markets closed",
//!       "open": "2025-12-26T14:30:00Z",
//!       "close": "2025-12-26T21:00:00Z",
//!       "time": "2025-12-27T00:56:07Z",
//!       "timezone": "America/New_York",
//!       "timezoneShort": "EST",
//!       "gmtOffset": -18000,
//!       "dst": false
//!     }
//!   ]
//! }
//! ```

mod response;

pub use response::{HoursResponse, MarketTime};
