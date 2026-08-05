//! Forex market data endpoints.

pub mod aggregates;
pub mod quotes;
#[allow(dead_code)] // unrouted: cross-market snapshots routed by #244
pub mod snapshots;
