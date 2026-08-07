//! Cryptocurrency market data endpoints.

pub mod aggregates;
pub mod snapshots;
#[allow(dead_code)] // unrouted: tick-level trades land with #250
pub mod trades;
