//! Financial Statement Response Models
//!
//! Flattened, user-friendly financial statement response.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    pub statement: BTreeMap<String, BTreeMap<String, f64>>,

    /// Which provider supplied this data (None = Yahoo Finance default)
    pub provider_id: Option<finance_query_types::Provider>,
}

#[cfg(feature = "dataframe")]
impl FinancialStatement {
    /// Converts the statement to a long-format polars DataFrame, one row per metric and date.
    ///
    /// Columns are `symbol`, `statement_type`, `frequency`, `metric`, `date` (strings) and
    /// `value` (f64). Rows are sorted by metric, then date.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        use polars::prelude::*;

        let rows: Vec<(&str, &str, f64)> = self
            .statement
            .iter()
            .flat_map(|(metric, by_date)| {
                by_date
                    .iter()
                    .map(move |(date, value)| (metric.as_str(), date.as_str(), *value))
            })
            .collect();

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
    /// value on a date is null. A metric named `date` collides with the first column and errors.
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

#[cfg(all(test, feature = "dataframe"))]
mod tests {
    use super::*;

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
