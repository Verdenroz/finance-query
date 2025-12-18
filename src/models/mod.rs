//! Data models for Yahoo Finance responses.

/// Chart/historical data models.
pub mod chart;
/// Financials (fundamentals-timeseries) models.
pub mod financials;
/// Technical indicators models.
pub mod indicators;
/// Market movers models.
pub mod movers;
/// News models.
pub mod news;
/// Options models.
pub mod options;
/// Quote models for detailed stock information.
pub mod quote;
/// Recommendation models.
pub mod recommendation;
/// Search models.
pub mod search;

// Re-exports for convenience
pub use chart::{Candle, Chart, ChartMeta};
pub use financials::FinancialStatement;
pub use movers::{MoverQuote, MoversResponse};
pub use news::{NewsArticle, NewsResponse, NewsThumbnail};
pub use options::{OptionChain, OptionContract, OptionsQuote, OptionsResponse};
pub use quote::{FormattedValue, Quote};
pub use recommendation::{Recommendation, SimilarSymbol};
pub use search::{SearchQuote, SearchResponse};
