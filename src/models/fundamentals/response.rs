//! Financial Statement Response Models
//!
//! Flattened, user-friendly financial statement response.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A flattened, user-friendly financial statement
///
/// Metric names are the same whichever provider served the data: Yahoo's field names in
/// snake_case (`total_revenue`, `net_income`, `diluted_eps`, `operating_cash_flow`). A
/// provider metric with no canonical equivalent keeps its own name converted to snake_case,
/// so no data is dropped, but such names are provider-specific.
/// ```json
/// {
///   "symbol": "AAPL",
///   "statementType": "income",
///   "frequency": "annual",
///   "statement": {
///     "total_revenue": { "2024-09-30": 391035000000, "2023-09-30": 383285000000 },
///     "net_income": { "2024-09-30": 100913000000, "2023-09-30": 96995000000 }
///   }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialStatement {
    /// Stock symbol
    pub symbol: String,

    /// Type of financial statement (income, balance, cashflow)
    pub statement_type: String,

    /// Frequency (annual or quarterly)
    pub frequency: String,

    /// Financial data: canonical metric name -> (date -> value)
    /// Example: { "total_revenue": { "2024-09-30": 391035000000 } }
    pub statement: BTreeMap<String, BTreeMap<String, f64>>,

    /// Which provider supplied this data (None = Yahoo Finance default)
    pub provider_id: Option<finance_query_types::Provider>,
}

#[cfg(feature = "dataframe")]
impl FinancialStatement {
    /// Converts the statement to a long-format polars DataFrame, one row per metric and date.
    ///
    /// Columns are `symbol`, `statement_type`, `frequency`, `metric` (strings), `date` (Date)
    /// and `value` (f64). `metric` holds the statement's metric name
    /// (`total_revenue`). Rows are sorted by metric, then date.
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
                crate::__dataframe::str_date("date".into(), rows.iter().map(|r| Some(r.1))),
                Series::new("value".into(), rows.iter().map(|r| r.2).collect::<Vec<_>>()).into(),
            ],
        )
    }

    /// Converts the statement to a wide-format polars DataFrame, one row per date.
    ///
    /// The first column is `date` (Date), followed by one nullable f64 column per metric,
    /// named as in [`to_dataframe`](Self::to_dataframe)'s `metric` column. Dates are sorted
    /// ascending and metric columns by name. A metric with no value on a date is null. Errors
    /// when a metric is named `date`.
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

        let mut columns: Vec<Column> = vec![crate::__dataframe::str_date(
            "date".into(),
            dates.iter().map(|d| Some(*d)),
        )];
        for (metric, by_date) in &self.statement {
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
                    "total_revenue": {"2024-09-30": 391.0, "2023-09-30": 383.0},
                    "net_income": {"2024-09-30": 100.0}
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
                .iter()
                .map(Option::unwrap)
                .map(String::from)
                .collect()
        }

        fn dates(df: &polars::prelude::DataFrame) -> Vec<String> {
            df.column("date")
                .unwrap()
                .date()
                .unwrap()
                .as_date_iter()
                .map(|d| d.unwrap().to_string())
                .collect()
        }

        fn with_metrics(metrics: &[&str]) -> FinancialStatement {
            let by_metric = metrics
                .iter()
                .map(|m| {
                    (
                        m.to_string(),
                        BTreeMap::from([("2024-09-30".to_string(), 1.0)]),
                    )
                })
                .collect();
            FinancialStatement {
                statement: by_metric,
                ..statement()
            }
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
            assert_eq!(df.column("date").unwrap().dtype(), &DataType::Date);
            assert_eq!(
                strings(&df, "metric"),
                ["net_income", "total_revenue", "total_revenue"]
            );
            assert_eq!(dates(&df), ["2024-09-30", "2023-09-30", "2024-09-30"]);
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
            assert_eq!(names, ["date", "net_income", "total_revenue"]);
            assert_eq!(df.column("date").unwrap().dtype(), &DataType::Date);
            assert_eq!(df.column("net_income").unwrap().dtype(), &DataType::Float64);
            assert_eq!(dates(&df), ["2023-09-30", "2024-09-30"]);
            let net_income = df.column("net_income").unwrap().f64().unwrap();
            assert_eq!(net_income.get(0), None);
            assert_eq!(net_income.get(1), Some(100.0));
        }

        #[test]
        fn metric_named_date_errors() {
            assert!(with_metrics(&["date"]).to_wide_dataframe().is_err());
        }
    }
}
