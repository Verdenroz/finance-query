// Quote summary models for detailed stock information
pub mod quote_summary;

// Response models for various endpoints
pub mod chart;
pub mod quote_type;
pub mod recommendation;
pub mod search;
pub mod timeseries;

// Re-exports for convenience
pub use chart::{Candle, ChartMeta, ChartResponse, ChartResult};
pub use quote_type::{QuoteTypeResponse, QuoteTypeResult};
pub use recommendation::{RecommendationResponse, RecommendedSymbol};
pub use search::{SearchQuote, SearchResponse};
pub use timeseries::{TimeseriesDataPoint, TimeseriesResponse};
