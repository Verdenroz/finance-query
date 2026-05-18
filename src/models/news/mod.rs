//! News models.

mod scraped;

pub use scraped::News;
#[cfg(feature = "python")]
pub use scraped::PyNews;
