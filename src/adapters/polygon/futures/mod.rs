//! Futures market data endpoints.

pub mod aggregates;
pub mod contracts;
pub mod snapshots;
pub mod trades;

pub use aggregates::*;
pub use contracts::*;
pub use snapshots::*;
pub use trades::*;
