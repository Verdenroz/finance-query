//! Forex market data endpoints.

#[allow(dead_code)] // unrouted: grouped-daily aggregates routed by #245
pub mod aggregates;
pub mod quotes;
#[allow(dead_code)] // unrouted: cross-market snapshots routed by #244
pub mod snapshots;
