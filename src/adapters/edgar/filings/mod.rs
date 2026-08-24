//! FILINGS capability endpoints layered on the EDGAR client.
//!
//! The submissions / company-facts / filing-index calls predate this directory
//! and still live in the parent module; new routed operations land here.

pub mod employee_count;
#[cfg(feature = "secftd")]
pub mod fails_to_deliver;
pub mod full_text;
pub mod insider;
pub mod press_releases;
pub mod sections;
pub mod thirteen_f;
pub(crate) mod xml;
