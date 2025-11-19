//! Quote Summary Response
//!
//! Handles parsing of Yahoo Finance quoteSummary API responses

use crate::error::{Error, Result};
use crate::models::quote_summary::Price;
use serde_json::Value;
use std::collections::HashMap;

/// Response from the quoteSummary endpoint
///
/// Contains the raw JSON data for each requested module.
/// Individual modules can be extracted using type-safe methods.
#[derive(Debug, Clone)]
pub struct QuoteSummaryResponse {
    /// The symbol this response is for
    pub symbol: String,
    /// Raw JSON data for each module, keyed by module name
    pub modules: HashMap<String, Value>,
}

impl QuoteSummaryResponse {
    /// Creates a QuoteSummaryResponse from raw JSON
    ///
    /// # Arguments
    ///
    /// * `json` - The raw JSON response from Yahoo Finance
    /// * `symbol` - The stock symbol this response is for
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The response structure is invalid
    /// - The symbol is not found in the response
    /// - Required fields are missing
    pub fn from_json(json: Value, symbol: &str) -> Result<Self> {
        // Yahoo Finance response structure:
        // {
        //   "quoteSummary": {
        //     "result": [
        //       {
        //         "price": { ... },
        //         "summaryDetail": { ... },
        //         ...
        //       }
        //     ],
        //     "error": null
        //   }
        // }

        let quote_summary = json
            .get("quoteSummary")
            .ok_or_else(|| Error::ParseError("Missing quoteSummary field".to_string()))?;

        // Check for errors
        if let Some(error) = quote_summary.get("error")
            && !error.is_null()
        {
            return Err(Error::ApiError(format!("API error: {}", error)));
        }

        let result = quote_summary
            .get("result")
            .and_then(|r| r.as_array())
            .ok_or_else(|| Error::ParseError("Missing or invalid result field".to_string()))?;

        if result.is_empty() {
            return Err(Error::ApiError(format!(
                "No data found for symbol: {}",
                symbol
            )));
        }

        let data = &result[0];

        // Extract all modules into a HashMap
        let mut modules = HashMap::new();
        if let Some(obj) = data.as_object() {
            for (key, value) in obj.iter() {
                modules.insert(key.clone(), value.clone());
            }
        }

        Ok(Self {
            symbol: symbol.to_string(),
            modules,
        })
    }

    /// Checks if a specific module is present in the response
    pub fn has_module(&self, module_name: &str) -> bool {
        self.modules.contains_key(module_name)
    }

    /// Gets raw JSON for a specific module
    pub fn get_module(&self, module_name: &str) -> Option<&Value> {
        self.modules.get(module_name)
    }

    /// Gets and deserializes a specific module
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize the module into
    ///
    /// # Arguments
    ///
    /// * `module_name` - The name of the module to retrieve
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The module is not present in the response
    /// - Deserialization fails
    pub fn get_typed<T: serde::de::DeserializeOwned>(&self, module_name: &str) -> Result<T> {
        let module_data = self
            .get_module(module_name)
            .ok_or_else(|| Error::ParseError(format!("Module '{}' not found", module_name)))?;

        serde_json::from_value(module_data.clone())
            .map_err(|e| Error::ParseError(format!("Failed to deserialize {}: {}", module_name, e)))
    }

    /// Returns a list of all module names present in this response
    pub fn module_names(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }

    /// Converts this response into structured data with typed modules
    pub fn into_data(self) -> Result<QuoteSummaryData> {
        QuoteSummaryData::from_response(self)
    }
}

/// Structured quote summary data with all available modules
///
/// This struct contains strongly-typed optional fields for all Yahoo Finance modules.
/// Modules that weren't requested or aren't available will be None.
#[derive(Debug, Clone)]
pub struct QuoteSummaryData {
    /// Symbol this data is for
    pub symbol: String,

    // Core modules (most commonly used)
    /// Detailed pricing data
    pub price: Option<Price>,

    /// Summary detail information (PE ratios, dividends, 52-week range)
    pub summary_detail: Option<Value>, // TODO: Implement SummaryDetail struct

    /// Financial data (revenue, margins, cash flow)
    pub financial_data: Option<Value>, // TODO: Implement FinancialData struct

    /// Key statistics (enterprise value, shares outstanding, beta)
    pub key_stats: Option<Value>, // TODO: Implement KeyStats struct

    /// Asset profile (company info, sector, industry, officers)
    pub asset_profile: Option<Value>, // TODO: Implement AssetProfile struct

    // Additional modules (will be implemented progressively)
    /// Calendar events (earnings date, dividend date)
    pub calendar_events: Option<Value>,

    /// Earnings data
    pub earnings: Option<Value>,

    /// Earnings trend
    pub earnings_trend: Option<Value>,

    /// Earnings history
    pub earnings_history: Option<Value>,

    /// ESG scores
    pub esg_scores: Option<Value>,

    /// Recommendation trend
    pub recommendation_trend: Option<Value>,

    /// Insider holders
    pub insider_holders: Option<Value>,

    /// Insider transactions
    pub insider_transactions: Option<Value>,

    /// Institution ownership
    pub institution_ownership: Option<Value>,

    /// Fund ownership
    pub fund_ownership: Option<Value>,

    /// Major holders breakdown
    pub major_holders: Option<Value>,

    /// Net share purchase activity
    pub share_purchase_activity: Option<Value>,

    /// Quote type information
    pub quote_type: Option<Value>,

    /// Summary profile
    pub summary_profile: Option<Value>,

    /// SEC filings
    pub sec_filings: Option<Value>,

    /// Upgrade/downgrade history
    pub grading_history: Option<Value>,

    /// Index trend
    pub index_trend: Option<Value>,

    /// Sector trend (deprecated by Yahoo)
    pub sector_trend: Option<Value>,

    /// Industry trend (deprecated by Yahoo)
    pub industry_trend: Option<Value>,

    /// Company officers
    pub company_officers: Option<Value>,

    /// Page views trend
    pub page_views: Option<Value>,

    // Fund-specific modules
    /// Fund profile
    pub fund_profile: Option<Value>,

    /// Fund performance
    pub fund_performance: Option<Value>,

    /// Top holdings
    pub fund_top_holdings: Option<Value>,

    /// Fund holding info
    pub fund_holding_info: Option<Value>,

    /// Fund bond holdings
    pub fund_bond_holdings: Option<Value>,

    /// Fund bond ratings
    pub fund_bond_ratings: Option<Value>,

    /// Fund equity holdings
    pub fund_equity_holdings: Option<Value>,

    /// Fund sector weightings
    pub fund_sector_weightings: Option<Value>,
}

impl QuoteSummaryData {
    /// Creates QuoteSummaryData from a QuoteSummaryResponse
    pub fn from_response(response: QuoteSummaryResponse) -> Result<Self> {
        let mut data = Self {
            symbol: response.symbol.clone(),
            price: None,
            summary_detail: None,
            financial_data: None,
            key_stats: None,
            asset_profile: None,
            calendar_events: None,
            earnings: None,
            earnings_trend: None,
            earnings_history: None,
            esg_scores: None,
            recommendation_trend: None,
            insider_holders: None,
            insider_transactions: None,
            institution_ownership: None,
            fund_ownership: None,
            major_holders: None,
            share_purchase_activity: None,
            quote_type: None,
            summary_profile: None,
            sec_filings: None,
            grading_history: None,
            index_trend: None,
            sector_trend: None,
            industry_trend: None,
            company_officers: None,
            page_views: None,
            fund_profile: None,
            fund_performance: None,
            fund_top_holdings: None,
            fund_holding_info: None,
            fund_bond_holdings: None,
            fund_bond_ratings: None,
            fund_equity_holdings: None,
            fund_sector_weightings: None,
        };

        // Parse typed modules
        if let Some(price_json) = response.get_module("price") {
            data.price = serde_json::from_value(price_json.clone()).ok();
        }

        // Store raw JSON for modules not yet implemented as structs
        if let Some(val) = response.get_module("summaryDetail") {
            data.summary_detail = Some(val.clone());
        }
        if let Some(val) = response.get_module("financialData") {
            data.financial_data = Some(val.clone());
        }
        if let Some(val) = response.get_module("defaultKeyStatistics") {
            data.key_stats = Some(val.clone());
        }
        if let Some(val) = response.get_module("assetProfile") {
            data.asset_profile = Some(val.clone());
        }
        if let Some(val) = response.get_module("calendarEvents") {
            data.calendar_events = Some(val.clone());
        }
        if let Some(val) = response.get_module("earnings") {
            data.earnings = Some(val.clone());
        }
        if let Some(val) = response.get_module("earningsTrend") {
            data.earnings_trend = Some(val.clone());
        }
        if let Some(val) = response.get_module("earningsHistory") {
            data.earnings_history = Some(val.clone());
        }
        if let Some(val) = response.get_module("esgScores") {
            data.esg_scores = Some(val.clone());
        }
        if let Some(val) = response.get_module("recommendationTrend") {
            data.recommendation_trend = Some(val.clone());
        }
        if let Some(val) = response.get_module("insiderHolders") {
            data.insider_holders = Some(val.clone());
        }
        if let Some(val) = response.get_module("insiderTransactions") {
            data.insider_transactions = Some(val.clone());
        }
        if let Some(val) = response.get_module("institutionOwnership") {
            data.institution_ownership = Some(val.clone());
        }
        if let Some(val) = response.get_module("fundOwnership") {
            data.fund_ownership = Some(val.clone());
        }
        if let Some(val) = response.get_module("majorHoldersBreakdown") {
            data.major_holders = Some(val.clone());
        }
        if let Some(val) = response.get_module("netSharePurchaseActivity") {
            data.share_purchase_activity = Some(val.clone());
        }
        if let Some(val) = response.get_module("quoteType") {
            data.quote_type = Some(val.clone());
        }
        if let Some(val) = response.get_module("summaryProfile") {
            data.summary_profile = Some(val.clone());
        }
        if let Some(val) = response.get_module("secFilings") {
            data.sec_filings = Some(val.clone());
        }
        if let Some(val) = response.get_module("upgradeDowngradeHistory") {
            data.grading_history = Some(val.clone());
        }
        if let Some(val) = response.get_module("indexTrend") {
            data.index_trend = Some(val.clone());
        }
        if let Some(val) = response.get_module("sectorTrend") {
            data.sector_trend = Some(val.clone());
        }
        if let Some(val) = response.get_module("industryTrend") {
            data.industry_trend = Some(val.clone());
        }
        if let Some(val) = response.get_module("companyOfficers") {
            data.company_officers = Some(val.clone());
        }
        if let Some(val) = response.get_module("pageViews") {
            data.page_views = Some(val.clone());
        }
        if let Some(val) = response.get_module("fundProfile") {
            data.fund_profile = Some(val.clone());
        }
        if let Some(val) = response.get_module("fundPerformance") {
            data.fund_performance = Some(val.clone());
        }
        if let Some(val) = response.get_module("topHoldings") {
            data.fund_top_holdings = Some(val.clone());
        }
        if let Some(val) = response.get_module("fundHoldingInfo") {
            data.fund_holding_info = Some(val.clone());
        }
        if let Some(val) = response.get_module("fundBondHoldings") {
            data.fund_bond_holdings = Some(val.clone());
        }
        if let Some(val) = response.get_module("fundBondRatings") {
            data.fund_bond_ratings = Some(val.clone());
        }
        if let Some(val) = response.get_module("fundEquityHoldings") {
            data.fund_equity_holdings = Some(val.clone());
        }
        if let Some(val) = response.get_module("fundSectorWeightings") {
            data.fund_sector_weightings = Some(val.clone());
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_from_json_valid() {
        let json = json!({
            "quoteSummary": {
                "result": [
                    {
                        "price": {
                            "regularMarketPrice": 150.0
                        },
                        "summaryDetail": {
                            "previousClose": 149.0
                        }
                    }
                ],
                "error": null
            }
        });

        let response = QuoteSummaryResponse::from_json(json, "AAPL").unwrap();
        assert_eq!(response.symbol, "AAPL");
        assert!(response.has_module("price"));
        assert!(response.has_module("summaryDetail"));
        assert_eq!(response.module_names().len(), 2);
    }

    #[test]
    fn test_from_json_error() {
        let json = json!({
            "quoteSummary": {
                "result": [],
                "error": null
            }
        });

        let response = QuoteSummaryResponse::from_json(json, "INVALID");
        assert!(response.is_err());
    }

    #[test]
    fn test_get_module() {
        let json = json!({
            "quoteSummary": {
                "result": [
                    {
                        "price": {
                            "regularMarketPrice": 150.0
                        }
                    }
                ],
                "error": null
            }
        });

        let response = QuoteSummaryResponse::from_json(json, "AAPL").unwrap();
        let price_data = response.get_module("price").unwrap();
        assert_eq!(price_data["regularMarketPrice"], 150.0);
    }
}
