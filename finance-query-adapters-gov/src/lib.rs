//! Provider adapters for fred, bls, fiscaldata, worldbank, finra, cftc, house and senate trades.
//!
//! Internal crate of `finance-query`. It has no stability guarantee; depend on
//! `finance-query` instead.

#[cfg(feature = "fred")]
pub mod fred;

#[cfg(feature = "bls")]
pub mod bls;

#[cfg(feature = "fiscaldata")]
pub mod fiscaldata;

#[cfg(feature = "worldbank")]
pub mod worldbank;

#[cfg(feature = "finra")]
pub mod finra;

#[cfg(feature = "cftc")]
pub mod cftc;

#[cfg(feature = "housetrades")]
pub mod housetrades;

#[cfg(feature = "senatetrades")]
pub mod senatetrades;
