/// Quote summary endpoint
///
/// Fetches full quote summary data for a single symbol.
/// Uses the /v10/finance/quoteSummary endpoint with all available modules.
use crate::adapters::yahoo::client::YahooClient;
use crate::adapters::yahoo::endpoints::api;
use crate::error::{FinanceError, Result};
use crate::models::quote::QuoteSummaryResponse;
use crate::models::quote::price::Price;
use serde_json::Value;
use tracing::info;

/// Fetch full quote summary for a symbol
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - Stock symbol (e.g., "AAPL")
pub(crate) async fn fetch_summary(
    client: &YahooClient,
    symbol: &str,
) -> Result<QuoteSummaryResponse> {
    info!("Fetching quote summary for: {}", symbol);

    let base_url = api::quote_summary(symbol);
    let modules = crate::models::quote::Module::all()
        .iter()
        .map(|m| m.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let url = format!("{base_url}?modules={modules}");
    let resp = client
        .request_with_crumb(&url)
        .await
        .map_err(|e| e.with_symbol(symbol))?;
    let json: serde_json::Value = resp.json().await?;
    let response = parse_summary(json, symbol)?;
    Ok(QuoteSummaryResponse {
        price: response.price.map(Price::with_percent_changes),
        ..response
    })
}

/// Creates a [`QuoteSummaryResponse`] from raw JSON
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
pub(crate) fn parse_summary(json: Value, symbol: &str) -> Result<QuoteSummaryResponse> {
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
            data.get($name).and_then(|v| {
                serde_json::from_value(v.clone())
                    .inspect_err(|e| {
                        tracing::warn!(symbol, module = $name, error = %e, "quoteSummary module failed to parse");
                    })
                    .ok()
            })
        };
    }

    Ok(QuoteSummaryResponse {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_summary_valid() {
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

        let response = parse_summary(json, "AAPL").unwrap();
        assert!(response.price.is_some());
        assert!(response.summary_detail.is_some());
    }

    #[test]
    fn test_parse_summary_error() {
        let json = json!({
            "quoteSummary": {
                "result": [],
                "error": null
            }
        });

        let response = parse_summary(json, "INVALID");
        assert!(response.is_err());
    }

    #[test]
    fn test_parse_summary_malformed_module_is_none() {
        let json = json!({
            "quoteSummary": {
                "result": [
                    {
                        "price": { "regularMarketPrice": { "raw": 150.0, "fmt": "150.00" } },
                        "financialData": "not an object"
                    }
                ],
                "error": null
            }
        });

        let response = parse_summary(json, "AAPL").unwrap();
        assert!(response.price.is_some());
        assert!(response.financial_data.is_none());
    }
}
