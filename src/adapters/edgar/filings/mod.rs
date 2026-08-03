//! FILINGS capability endpoints layered on the EDGAR client.
//!
//! The submissions / company-facts / filing-index calls predate this directory
//! and still live in the parent module; new routed operations land here.

pub mod full_text;
pub mod insider;
pub mod thirteen_f;
pub(crate) mod xml;
