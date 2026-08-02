//! Futures market data endpoints.

#[allow(dead_code)] // unrouted: grouped-daily aggregates routed by #245
pub mod aggregates;
#[allow(dead_code)] // unrouted: futures contract reference data has no route yet
pub mod contracts;
pub mod snapshots;
#[allow(dead_code)] // unrouted: tick-level trades land with #250
pub mod trades;
