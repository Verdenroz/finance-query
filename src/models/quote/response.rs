//! Quote Summary Response
//!
//! Handles parsing of Yahoo Finance quoteSummary API responses

use crate::error::{Error, Result};
use crate::models::quote::*;
use serde_json::Value;

/// Response from the quoteSummary endpoint
///
/// Deserializes all requested modules once on construction to avoid repeated
/// JSON parsing on every accessor call. Uses Option<T> for each module since
/// Yahoo Finance may not return all modules for all symbols.
#[derive(Debug, Clone)]
pub(crate) struct QuoteSummaryResponse {
    /// The symbol this response is for
    pub symbol: String,

    // Pre-deserialized module data - populated once in from_json()
    pub price: Option<Price>,
    pub summary_detail: Option<SummaryDetail>,
    pub financial_data: Option<FinancialData>,
    pub default_key_statistics: Option<DefaultKeyStatistics>,
    pub asset_profile: Option<AssetProfile>,
    pub calendar_events: Option<CalendarEvents>,
    pub earnings: Option<Earnings>,
    pub earnings_trend: Option<EarningsTrend>,
    pub earnings_history: Option<EarningsHistory>,
    pub recommendation_trend: Option<RecommendationTrend>,
    pub insider_holders: Option<InsiderHolders>,
    pub insider_transactions: Option<InsiderTransactions>,
    pub institution_ownership: Option<InstitutionOwnership>,
    pub fund_ownership: Option<FundOwnership>,
    pub major_holders_breakdown: Option<MajorHoldersBreakdown>,
    pub net_share_purchase_activity: Option<NetSharePurchaseActivity>,
    pub quote_type: Option<QuoteTypeData>,
    pub summary_profile: Option<SummaryProfile>,
    pub sec_filings: Option<SecFilings>,
    pub upgrade_downgrade_history: Option<UpgradeDowngradeHistory>,
    pub fund_performance: Option<FundPerformance>,
    pub fund_profile: Option<FundProfile>,
    pub top_holdings: Option<TopHoldings>,
    pub index_trend: Option<IndexTrend>,
    pub industry_trend: Option<IndustryTrend>,
    pub sector_trend: Option<SectorTrend>,
    pub equity_performance: Option<EquityPerformance>,
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
    pub(crate) fn from_json(json: Value, symbol: &str) -> Result<Self> {
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

        let quote_summary =
            json.get("quoteSummary")
                .ok_or_else(|| Error::ResponseStructureError {
                    field: "quoteSummary".to_string(),
                    context: "Missing quoteSummary field".to_string(),
                })?;

        // Check for errors
        if let Some(error) = quote_summary.get("error")
            && !error.is_null()
        {
            return Err(Error::ApiError(format!("API error: {}", error)));
        }

        let result = quote_summary
            .get("result")
            .and_then(|r| r.as_array())
            .ok_or_else(|| Error::ResponseStructureError {
                field: "result".to_string(),
                context: "Missing or invalid result field".to_string(),
            })?;

        if result.is_empty() {
            return Err(Error::ApiError(format!(
                "No data found for symbol: {}",
                symbol
            )));
        }

        let data = &result[0];

        // Helper macro to deserialize a module, returning None on missing/error
        macro_rules! deserialize_module {
            ($name:expr) => {
                data.get($name)
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
            };
        }

        Ok(Self {
            symbol: symbol.to_string(),
            price: deserialize_module!("price"),
            summary_detail: deserialize_module!("summaryDetail"),
            financial_data: deserialize_module!("financialData"),
            default_key_statistics: deserialize_module!("defaultKeyStatistics"),
            asset_profile: deserialize_module!("assetProfile"),
            calendar_events: deserialize_module!("calendarEvents"),
            earnings: deserialize_module!("earnings"),
            earnings_trend: deserialize_module!("earningsTrend"),
            earnings_history: deserialize_module!("earningsHistory"),
            recommendation_trend: deserialize_module!("recommendationTrend"),
            insider_holders: deserialize_module!("insiderHolders"),
            insider_transactions: deserialize_module!("insiderTransactions"),
            institution_ownership: deserialize_module!("institutionOwnership"),
            fund_ownership: deserialize_module!("fundOwnership"),
            major_holders_breakdown: deserialize_module!("majorHoldersBreakdown"),
            net_share_purchase_activity: deserialize_module!("netSharePurchaseActivity"),
            quote_type: deserialize_module!("quoteType"),
            summary_profile: deserialize_module!("summaryProfile"),
            sec_filings: deserialize_module!("secFilings"),
            upgrade_downgrade_history: deserialize_module!("upgradeDowngradeHistory"),
            fund_performance: deserialize_module!("fundPerformance"),
            fund_profile: deserialize_module!("fundProfile"),
            top_holdings: deserialize_module!("topHoldings"),
            index_trend: deserialize_module!("indexTrend"),
            industry_trend: deserialize_module!("industryTrend"),
            sector_trend: deserialize_module!("sectorTrend"),
            equity_performance: deserialize_module!("equityPerformance"),
        })
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
                            "regularMarketPrice": {
                                "raw": 150.0,
                                "fmt": "150.00"
                            }
                        },
                        "summaryDetail": {
                            "previousClose": {
                                "raw": 149.0,
                                "fmt": "149.00"
                            }
                        }
                    }
                ],
                "error": null
            }
        });

        let response = QuoteSummaryResponse::from_json(json, "AAPL").unwrap();
        assert_eq!(response.symbol, "AAPL");
        assert!(response.price.is_some());
        assert!(response.summary_detail.is_some());
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
}
