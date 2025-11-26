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
