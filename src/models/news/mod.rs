//! News Module
//!
//! Contains all data structures for Yahoo Finance's news endpoint.
mod article;
mod response;
mod thumbnail;

pub use article::NewsArticle;
pub use response::NewsResponse;
pub use thumbnail::{NewsThumbnail, ThumbnailResolution};
