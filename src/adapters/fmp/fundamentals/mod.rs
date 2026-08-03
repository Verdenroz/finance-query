#[allow(dead_code)] // unrouted: period-based metrics/ratios have no capability route yet
pub mod analysis;
pub mod consensus;
pub mod core;
pub mod estimates;
#[allow(dead_code)] // unrouted: ETF/fund data has no capability route yet
pub mod etf_mutual_funds;
#[allow(dead_code)] // unrouted: ETF/fund data has no capability route yet
pub mod fund_holdings;
pub mod ttm;
pub use core::*;
