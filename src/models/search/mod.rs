//! Search models.

mod news;
mod quote;
mod research;
mod response;
mod thumbnail;

pub use news::{SearchNews, SearchNewsList};
pub use quote::{SearchQuote, SearchQuotes};
pub use research::{ResearchReport, ResearchReports};
pub use response::SearchResults;
