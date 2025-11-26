//! Quote module
//!
//! Contains the fully typed Quote struct for serialization and API responses.

use serde::{Deserialize, Serialize};

use super::{
    DefaultKeyStatistics, FinancialData, Price, QuoteSummaryResponse, QuoteTypeData, SummaryDetail,
    SummaryProfile,
};

/// Fully typed quote data
///
/// Aggregates all typed modules from the quoteSummary endpoint into a single
/// convenient structure. All fields are optional since Yahoo Finance may not
/// return all modules for every symbol.
///
/// This is the recommended type for serialization and API responses.
/// Used for both single quote and batch quotes endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    /// Stock symbol
    pub symbol: String,

    /// Current price and trading data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,

    /// Quote type metadata (exchange, company name, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_type: Option<QuoteTypeData>,

    /// Summary detail (trading metrics, market cap, volume)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_detail: Option<SummaryDetail>,

    /// Financial data (margins, cash flow, analyst recommendations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub financial_data: Option<FinancialData>,

    /// Key statistics (valuation metrics, shares, ratios)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_statistics: Option<DefaultKeyStatistics>,

    /// Company profile (address, sector, industry, business description)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_profile: Option<SummaryProfile>,
}

impl Quote {
    /// Creates a Quote from a QuoteSummaryResponse
    ///
    /// Extracts and deserializes all typed modules from the raw response.
    pub fn from_response(response: &QuoteSummaryResponse) -> Self {
        Self {
            symbol: response.symbol.clone(),
            price: response.get_typed("price").ok(),
            quote_type: response.get_typed("quoteType").ok(),
            summary_detail: response.get_typed("summaryDetail").ok(),
            financial_data: response.get_typed("financialData").ok(),
            key_statistics: response.get_typed("defaultKeyStatistics").ok(),
            summary_profile: response.get_typed("summaryProfile").ok(),
        }
    }
}
