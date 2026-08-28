//! Quote Summary Response
//!
//! Handles parsing of Yahoo Finance quoteSummary API responses

use crate::error::{FinanceError, Result};
use crate::models::quote::*;
use serde_json::Value;

/// Response from the quoteSummary endpoint
///
/// Deserializes all requested modules once on construction to avoid repeated
/// JSON parsing on every accessor call. Uses `Option<T>` for each module since
/// Yahoo Finance may not return all modules for all symbols.
///
/// The return type of [`QuoteProvider::fetch_quote`](crate::QuoteProvider),
/// so an implementor populates the modules it can serve and leaves the rest
/// `None`. Each field mirrors one Yahoo `quoteSummary` module.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct QuoteSummaryResponse {
    /// The symbol this response is for
    pub symbol: String,

    /// Last price, change, and market state.
    pub price: Option<Price>,
    /// Day range, volume, yield, and valuation summary.
    pub summary_detail: Option<SummaryDetail>,
    /// Margins, returns, cash flow, and analyst targets.
    pub financial_data: Option<FinancialData>,
    /// Shares outstanding, float, beta, and per-share statistics.
    pub default_key_statistics: Option<DefaultKeyStatistics>,
    /// Company address, sector, industry, and officers.
    pub asset_profile: Option<AssetProfile>,
    /// Upcoming earnings and dividend dates.
    pub calendar_events: Option<CalendarEvents>,
    /// Quarterly and annual earnings history with estimates.
    pub earnings: Option<Earnings>,
    /// Analyst estimate trend by period.
    pub earnings_trend: Option<EarningsTrend>,
    /// Reported versus estimated EPS per quarter.
    pub earnings_history: Option<EarningsHistory>,
    /// Analyst buy, hold, and sell counts over time.
    pub recommendation_trend: Option<RecommendationTrend>,
    /// Insiders and their reported positions.
    pub insider_holders: Option<InsiderHolders>,
    /// Recent insider buys and sells.
    pub insider_transactions: Option<InsiderTransactions>,
    /// Institutional holders and position sizes.
    pub institution_ownership: Option<InstitutionOwnership>,
    /// Fund holders and position sizes.
    pub fund_ownership: Option<FundOwnership>,
    /// Insider and institutional ownership percentages.
    pub major_holders_breakdown: Option<MajorHoldersBreakdown>,
    /// Net insider share purchase totals.
    pub net_share_purchase_activity: Option<NetSharePurchaseActivity>,
    /// Instrument type, exchange, and naming metadata.
    pub quote_type: Option<QuoteTypeData>,
    /// Business description and contact details.
    pub summary_profile: Option<SummaryProfile>,
    /// Recent SEC filings with links.
    pub sec_filings: Option<SecFilings>,
    /// Analyst rating changes over time.
    pub upgrade_downgrade_history: Option<UpgradeDowngradeHistory>,
    /// Trailing and annual returns for a fund.
    pub fund_performance: Option<FundPerformance>,
    /// Fund category, family, and fee structure.
    pub fund_profile: Option<FundProfile>,
    /// A fund's largest positions and sector weights.
    pub top_holdings: Option<TopHoldings>,
    /// Index-level growth and valuation estimates.
    pub index_trend: Option<IndexTrend>,
    /// Industry-level growth and valuation estimates.
    pub industry_trend: Option<IndustryTrend>,
    /// Sector-level growth and valuation estimates.
    pub sector_trend: Option<SectorTrend>,
    /// Performance of the instrument against its peers.
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
                .ok_or_else(|| FinanceError::ResponseStructureError {
                    field: "quoteSummary".to_string(),
                    context: "Missing quoteSummary field".to_string(),
                })?;

        // Check for errors
        if let Some(error) = quote_summary.get("error")
            && !error.is_null()
        {
            return Err(FinanceError::ApiError(format!("API error: {}", error)));
        }

        let result = quote_summary
            .get("result")
            .and_then(|r| r.as_array())
            .ok_or_else(|| FinanceError::ResponseStructureError {
                field: "result".to_string(),
                context: "Missing or invalid result field".to_string(),
            })?;

        if result.is_empty() {
            return Err(FinanceError::ApiError(format!(
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
