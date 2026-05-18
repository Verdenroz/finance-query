//! Market sentiment models.
//!
//! Contains the Fear & Greed Index data from Alternative.me.

pub(crate) mod response;

pub use response::{FearAndGreed, FearGreedLabel};

#[cfg(feature = "python")]
pub use response::{PyFearAndGreed, PyFearGreedLabel};
