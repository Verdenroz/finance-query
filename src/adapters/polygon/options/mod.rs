//! Options market data endpoints.

#[allow(dead_code)] // unrouted: grouped-daily aggregates routed by #245
pub mod aggregates;
pub mod reference;
pub mod snapshots;
#[allow(dead_code)] // unrouted: tick-level trades land with #250
pub mod trades;
