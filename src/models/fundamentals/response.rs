//! Financial Statement Response Models
//!
//! Flattened, user-friendly financial statement response.

use crate::constants::{Frequency, StatementType};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Raw response structure from Yahoo Finance fundamentals-timeseries API
#[derive(Debug, Clone, Deserialize)]
struct RawTimeseriesResponse {
    timeseries: RawTimeseries,
}

#[derive(Debug, Clone, Deserialize)]
struct RawTimeseries {
    result: Vec<RawTimeseriesResult>,
    #[allow(dead_code)] // serde completeness field; never read
    error: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawTimeseriesResult {
    meta: RawMeta,
    #[serde(flatten)]
    data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawMeta {
    #[serde(rename = "type")]
    data_type: Vec<String>,
}

/// A flattened, user-friendly financial statement
///
/// Transforms Yahoo Finance's complex nested response into a simple structure:
/// ```json
/// {
///   "symbol": "AAPL",
///   "statementType": "income",
///   "frequency": "annual",
///   "statement": {
///     "TotalRevenue": { "2024-09-30": 391035000000, "2023-09-30": 383285000000 },
///     "NetIncome": { "2024-09-30": 100913000000, "2023-09-30": 96995000000 }
///   }
/// }
/// ```
///
/// This matches the Python finance-query API response format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialStatement {
    /// Stock symbol
    pub symbol: String,

    /// Type of financial statement (income, balance, cashflow)
    pub statement_type: String,

    /// Frequency (annual or quarterly)
    pub frequency: String,

    /// Financial data: metric name -> (date -> value)
    /// Example: { "TotalRevenue": { "2024-09-30": 391035000000 } }
    pub statement: HashMap<String, HashMap<String, f64>>,

    /// Which provider supplied this data (None = Yahoo Finance default)
    pub provider_id: Option<crate::providers::Provider>,
}

impl FinancialStatement {
    /// Parse from raw Yahoo Finance JSON response
    ///
    /// Converts the nested Yahoo Finance response structure into a clean,
    /// user-friendly format by extracting data from timeseries.result[].
    pub(crate) fn from_response(
        raw: serde_json::Value,
        symbol: &str,
        statement_type: StatementType,
        frequency: Frequency,
    ) -> Result<Self> {
        let raw_response: RawTimeseriesResponse = serde_json::from_value(raw).map_err(|e| {
            crate::error::FinanceError::ResponseStructureError {
                field: "timeseries".to_string(),
                context: format!("Failed to parse financials response: {}", e),
            }
        })?;

        if raw_response.timeseries.result.is_empty() {
            return Err(crate::error::FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: format!(
                    "No {} {} data found",
                    frequency.as_str(),
                    statement_type.as_str()
                ),
            });
        }

        let mut statement: HashMap<String, HashMap<String, f64>> = HashMap::new();

        for result in raw_response.timeseries.result {
            // Get the metric name from meta.type (e.g., "annualTotalRevenue")
            let metric_name_with_prefix =
                result.meta.data_type.first().cloned().unwrap_or_default();

            if metric_name_with_prefix.is_empty() {
                continue;
            }

            // Remove frequency prefix (annual/quarterly/trailing) for storage
            let metric_name = strip_frequency_prefix(&metric_name_with_prefix);

            // Get the data array using the full metric name as key
            let data_points = match result.data.get(&metric_name_with_prefix) {
                Some(serde_json::Value::Array(arr)) => arr,
                _ => continue,
            };

            let mut date_values: HashMap<String, f64> = HashMap::new();

            for point in data_points {
                if point.is_null() {
                    continue;
                }

                let as_of_date = point
                    .get("asOfDate")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                if as_of_date.is_empty() {
                    continue;
                }

                // Extract the raw value, handling Yahoo's nested structure
                let value = extract_value(point.get("reportedValue"));

                if let Some(v) = value {
                    date_values.insert(as_of_date.to_string(), v);
                }
            }

            if !date_values.is_empty() {
                statement.insert(metric_name, date_values);
            }
        }

        if statement.is_empty() {
            return Err(crate::error::FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: format!(
                    "No {} {} data found",
                    frequency.as_str(),
                    statement_type.as_str()
                ),
            });
        }

        Ok(Self {
            symbol: symbol.to_uppercase(),
            statement_type: statement_type.as_str().to_string(),
            frequency: frequency.as_str().to_string(),
            statement,
            provider_id: None,
        })
    }
}

#[cfg(feature = "dataframe")]
impl FinancialStatement {
    /// Converts the statement to a long-format polars DataFrame, one row per metric and date.
    ///
    /// Columns are `symbol`, `statement_type`, `frequency`, `metric`, `date` (strings) and
    /// `value` (f64). Rows are sorted by metric, then date.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        use polars::prelude::*;

        let mut rows: Vec<(&str, &str, f64)> = self
            .statement
            .iter()
            .flat_map(|(metric, by_date)| {
                by_date
                    .iter()
                    .map(move |(date, value)| (metric.as_str(), date.as_str(), *value))
            })
            .collect();
        rows.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));

        let height = rows.len();
        DataFrame::new(
            height,
            vec![
                Series::new("symbol".into(), vec![self.symbol.as_str(); height]).into(),
                Series::new(
                    "statement_type".into(),
                    vec![self.statement_type.as_str(); height],
                )
                .into(),
                Series::new("frequency".into(), vec![self.frequency.as_str(); height]).into(),
                Series::new(
                    "metric".into(),
                    rows.iter().map(|r| r.0).collect::<Vec<_>>(),
                )
                .into(),
                Series::new("date".into(), rows.iter().map(|r| r.1).collect::<Vec<_>>()).into(),
                Series::new("value".into(), rows.iter().map(|r| r.2).collect::<Vec<_>>()).into(),
            ],
        )
    }

    /// Converts the statement to a wide-format polars DataFrame, one row per date.
    ///
    /// The first column is `date` (string), followed by one nullable f64 column per metric.
    /// Dates are sorted ascending and metric columns are sorted by name. A metric with no
    /// value on a date is null.
    pub fn to_wide_dataframe(
        &self,
    ) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        use polars::prelude::*;

        let mut dates: Vec<&str> = self
            .statement
            .values()
            .flat_map(|by_date| by_date.keys().map(String::as_str))
            .collect();
        dates.sort();
        dates.dedup();

        let mut metrics: Vec<&String> = self.statement.keys().collect();
        metrics.sort();

        let mut columns: Vec<Column> = vec![Series::new("date".into(), dates.clone()).into()];
        for metric in metrics {
            let by_date = &self.statement[metric];
            let values: Vec<Option<f64>> = dates.iter().map(|d| by_date.get(*d).copied()).collect();
            columns.push(Series::new(metric.as_str().into(), values).into());
        }
        DataFrame::new(dates.len(), columns)
    }
}

/// Strip frequency prefix from metric name
/// "annualTotalRevenue" -> "TotalRevenue"
/// "quarterlyNetIncome" -> "NetIncome"
fn strip_frequency_prefix(name: &str) -> String {
    for prefix in &["annual", "quarterly", "trailing"] {
        if let Some(stripped) = name.strip_prefix(prefix) {
            return stripped.to_string();
        }
    }
    name.to_string()
}

/// Extract numeric value from Yahoo's reportedValue structure
/// Handles both simple: { "raw": 123.45 }
/// And nested: { "raw": { "parsedValue": 123456789 } }
fn extract_value(reported_value: Option<&serde_json::Value>) -> Option<f64> {
    let rv = reported_value?;

    // Try direct raw field first
    if let Some(raw) = rv.get("raw") {
        // Check if raw is a number
        if let Some(n) = raw.as_f64() {
            return Some(n);
        }
        // Check if raw is an object with parsedValue
        if let Some(parsed) = raw.get("parsedValue") {
            return parsed
                .as_f64()
                .or_else(|| parsed.as_i64().map(|i| i as f64));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_frequency_prefix() {
        assert_eq!(strip_frequency_prefix("annualTotalRevenue"), "TotalRevenue");
        assert_eq!(strip_frequency_prefix("quarterlyNetIncome"), "NetIncome");
        assert_eq!(strip_frequency_prefix("trailingMarketCap"), "MarketCap");
        assert_eq!(strip_frequency_prefix("SomeOther"), "SomeOther");
    }

    #[test]
    fn test_extract_value_simple() {
        let json: serde_json::Value = serde_json::json!({
            "raw": 123.45,
            "fmt": "123.45"
        });
        assert_eq!(extract_value(Some(&json)), Some(123.45));
    }

    #[test]
    fn test_extract_value_nested() {
        let json: serde_json::Value = serde_json::json!({
            "raw": {
                "source": "1.23E12",
                "parsedValue": 1230000000000_i64
            },
            "fmt": "1.23T"
        });
        assert_eq!(extract_value(Some(&json)), Some(1230000000000.0));
    }

    #[test]
    fn test_from_response() {
        let json: serde_json::Value = serde_json::json!({
            "timeseries": {
                "result": [
                    {
                        "meta": {
                            "symbol": ["AAPL"],
                            "type": ["annualTotalRevenue"]
                        },
                        "annualTotalRevenue": [
                            {
                                "asOfDate": "2024-09-30",
                                "periodType": "12M",
                                "currencyCode": "USD",
                                "reportedValue": {
                                    "raw": 391035000000.0,
                                    "fmt": "391.04B"
                                }
                            },
                            {
                                "asOfDate": "2023-09-30",
                                "periodType": "12M",
                                "currencyCode": "USD",
                                "reportedValue": {
                                    "raw": 383285000000.0,
                                    "fmt": "383.29B"
                                }
                            }
                        ]
                    },
                    {
                        "meta": {
                            "symbol": ["AAPL"],
                            "type": ["annualNetIncome"]
                        },
                        "annualNetIncome": [
                            {
                                "asOfDate": "2024-09-30",
                                "periodType": "12M",
                                "currencyCode": "USD",
                                "reportedValue": {
                                    "raw": 100913000000.0,
                                    "fmt": "100.91B"
                                }
                            }
                        ]
                    }
                ],
                "error": null
            }
        });

        let result = FinancialStatement::from_response(
            json,
            "AAPL",
            StatementType::Income,
            Frequency::Annual,
        );
        assert!(result.is_ok());

        let statement = result.unwrap();
        assert_eq!(statement.symbol, "AAPL");
        assert_eq!(statement.statement_type, "income");
        assert_eq!(statement.frequency, "annual");
        assert!(statement.statement.contains_key("TotalRevenue"));
        assert!(statement.statement.contains_key("NetIncome"));

        let revenue = statement.statement.get("TotalRevenue").unwrap();
        assert_eq!(revenue.get("2024-09-30"), Some(&391035000000.0));
        assert_eq!(revenue.get("2023-09-30"), Some(&383285000000.0));
    }

    #[test]
    fn test_from_response_empty() {
        let json: serde_json::Value = serde_json::json!({
            "timeseries": {
                "result": [],
                "error": null
            }
        });

        let result = FinancialStatement::from_response(
            json,
            "INVALID",
            StatementType::Income,
            Frequency::Annual,
        );
        assert!(result.is_err());
    }

    #[cfg(feature = "dataframe")]
    mod dataframe {
        use super::*;
        use polars::prelude::DataType;

        fn statement() -> FinancialStatement {
            serde_json::from_value(serde_json::json!({
                "symbol": "AAPL",
                "statementType": "income",
                "frequency": "annual",
                "statement": {
                    "TotalRevenue": {"2024-09-30": 391.0, "2023-09-30": 383.0},
                    "NetIncome": {"2024-09-30": 100.0}
                },
                "providerId": null
            }))
            .unwrap()
        }

        fn strings(df: &polars::prelude::DataFrame, name: &str) -> Vec<String> {
            df.column(name)
                .unwrap()
                .str()
                .unwrap()
                .into_no_null_iter()
                .map(String::from)
                .collect()
        }

        #[test]
        fn long_format_sorted_by_metric_then_date() {
            let df = statement().to_dataframe().unwrap();

            let names: Vec<_> = df
                .get_column_names()
                .iter()
                .map(|n| n.to_string())
                .collect();
            assert_eq!(
                names,
                [
                    "symbol",
                    "statement_type",
                    "frequency",
                    "metric",
                    "date",
                    "value"
                ]
            );
            assert_eq!(df.column("value").unwrap().dtype(), &DataType::Float64);
            assert_eq!(df.column("metric").unwrap().dtype(), &DataType::String);
            assert_eq!(
                strings(&df, "metric"),
                ["NetIncome", "TotalRevenue", "TotalRevenue"]
            );
            assert_eq!(
                strings(&df, "date"),
                ["2024-09-30", "2023-09-30", "2024-09-30"]
            );
            assert_eq!(strings(&df, "symbol"), ["AAPL"; 3]);
        }

        #[test]
        fn wide_format_has_sorted_dates_and_null_gaps() {
            let df = statement().to_wide_dataframe().unwrap();

            let names: Vec<_> = df
                .get_column_names()
                .iter()
                .map(|n| n.to_string())
                .collect();
            assert_eq!(names, ["date", "NetIncome", "TotalRevenue"]);
            assert_eq!(df.column("date").unwrap().dtype(), &DataType::String);
            assert_eq!(df.column("NetIncome").unwrap().dtype(), &DataType::Float64);
            assert_eq!(strings(&df, "date"), ["2023-09-30", "2024-09-30"]);
            let net_income = df.column("NetIncome").unwrap().f64().unwrap();
            assert_eq!(net_income.get(0), None);
            assert_eq!(net_income.get(1), Some(100.0));
        }
    }
}
