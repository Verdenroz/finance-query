//! Provider adapters for coingecko, binance, kraken, frankfurter, nasdaq, defillama, gdelt, wikipedia, openfigi.
//!
//! Internal crate of `finance-query`. It has no stability guarantee; depend on
//! `finance-query` instead.

#[cfg(feature = "crypto")]
pub mod coingecko;

#[cfg(feature = "binance")]
pub mod binance;

#[cfg(feature = "kraken")]
pub mod kraken;

#[cfg(feature = "frankfurter")]
pub mod frankfurter;

#[cfg(feature = "nasdaq")]
pub mod nasdaq;

#[cfg(feature = "defi")]
pub mod defillama;

#[cfg(feature = "gdelt")]
pub mod gdelt;

#[cfg(feature = "wikipedia")]
pub mod wikipedia;

#[cfg(feature = "openfigi")]
pub mod openfigi;
