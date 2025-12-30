//! Quote Summary Response
//!
//! Handles parsing of Yahoo Finance quoteSummary API responses

use crate::error::{Error, Result};
use serde_json::Value;
use std::collections::HashMap;

/// Response from the quoteSummary endpoint
///
/// Contains the raw JSON data for each requested module.
/// Individual modules can be extracted using type-safe methods.
#[derive(Debug, Clone)]
pub(crate) struct QuoteSummaryResponse {
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
    pub(crate) fn get_typed<T: serde::de::DeserializeOwned>(&self, module_name: &str) -> Result<T> {
        let module_data =
            self.modules
                .get(module_name)
                .ok_or_else(|| Error::ResponseStructureError {
                    field: module_name.to_string(),
                    context: format!("Module '{}' not found", module_name),
                })?;

        serde_json::from_value(module_data.clone()).map_err(|e| Error::ResponseStructureError {
            field: module_name.to_string(),
            context: format!("Failed to deserialize {}: {}", module_name, e),
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
        assert_eq!(response.modules.len(), 2);
        assert!(response.modules.contains_key("price"));
        assert!(response.modules.contains_key("summaryDetail"));
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
    fn test_get_typed() {
        use serde::Deserialize;

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct TestPrice {
            regular_market_price: f64,
        }

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
        let price: TestPrice = response.get_typed("price").unwrap();
        assert_eq!(price.regular_market_price, 150.0);
    }
}
