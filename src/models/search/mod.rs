//! Search models.
//!
//! Contains all data structures for Yahoo Finance's search endpoint.
//! Provides symbol and news search functionality.

mod news;
mod quote;
mod response;
mod thumbnail;

pub use news::SearchNews;
pub use quote::SearchQuote;
pub use response::SearchResponse;
pub use thumbnail::{NewsThumbnail, ThumbnailResolution};
