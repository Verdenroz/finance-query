//! Search models.
//!
//! Contains all data structures for Yahoo Finance's search endpoint.
//! Provides symbol, news, and research report search functionality.

mod news;
mod quote;
mod research;
mod response;
mod thumbnail;

pub use news::SearchNews;
pub use quote::SearchQuote;
pub use research::ResearchReport;
pub use response::SearchResults;
pub use thumbnail::{NewsThumbnail, ThumbnailResolution};
