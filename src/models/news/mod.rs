//! News models.

mod article;
mod response;
mod scraped;
mod thumbnail;

pub use article::NewsArticle;
pub use response::NewsResponse;
pub use scraped::News;
pub use thumbnail::{NewsThumbnail, ThumbnailResolution};
