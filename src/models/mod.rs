//! Data models for Yahoo Finance responses.

/// Chart/historical data models.
pub mod chart;
/// Financials (fundamentals-timeseries) models.
pub mod financials;
/// Market hours models.
pub mod hours;
/// Technical indicators models.
pub mod indicators;
/// Industry models for market industry data.
pub mod industries;
/// News models.
pub mod news;
/// Options models.
pub mod options;
/// Quote models for detailed stock information.
pub mod quote;
/// Recommendation models.
pub mod recommendation;
/// Screener models for predefined Yahoo Finance screeners.
pub mod screeners;
/// Search models.
pub mod search;
/// Sector models for market sector data.
pub mod sectors;

// Re-exports for convenience
pub use chart::{Candle, Chart, ChartMeta};
pub use financials::FinancialStatement;
pub use hours::{HoursResponse, MarketTime};
pub use industries::{
    BenchmarkPerformance, GrowthCompany, IndustryCompany, IndustryOverview, IndustryPerformance,
    IndustryResponse, PerformingCompany, ResearchReport as IndustryResearchReport,
};
pub use news::{NewsArticle, NewsResponse, NewsThumbnail};
pub use options::{OptionChain, OptionContract, OptionsQuote, OptionsResponse};
pub use quote::{FormattedValue, Quote};
pub use recommendation::{Recommendation, SimilarSymbol};
pub use screeners::{ScreenerQuote, ScreenersResponse};
pub use search::{SearchQuote, SearchResponse};
pub use sectors::{
    Industry, ResearchReport, SectorCompany, SectorETF, SectorMutualFund, SectorOverview,
    SectorPerformance, SectorResponse,
};
