//! `FinanceTools` and its supporting modules, kept out of `main.rs` (a bin
//! target only) so `cargo soothfast spec gen` can resolve their types from
//! this crate's rustdoc JSON.

pub mod error;
pub mod lang;
pub mod metrics;
pub mod tools;
