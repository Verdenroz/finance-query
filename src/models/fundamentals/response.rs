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
    /// Columns are `symbol`, `statement_type`, `frequency`, `metric` (strings), `date` (Date)
    /// and `value` (f64). `metric` holds the provider's metric name in snake_case
    /// (`TotalRevenue` becomes `total_revenue`). Rows are sorted by metric, then date. Errors
    /// when two metric names map to the same snake_case name.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        use polars::prelude::*;

        let metrics = self.frame_metrics()?;
        let rows: Vec<(&str, &str, f64)> = metrics
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
    /// when two metric names map to the same column name, or a metric maps to `date`.
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
        for (metric, by_date) in self.frame_metrics()? {
            let values: Vec<Option<f64>> = dates.iter().map(|d| by_date.get(*d).copied()).collect();
            columns.push(Series::new(metric.into(), values).into());
        }
        DataFrame::new(dates.len(), columns)
    }

    fn frame_metrics(
        &self,
    ) -> ::polars::prelude::PolarsResult<Vec<(String, &BTreeMap<String, f64>)>> {
        let mut metrics: Vec<_> = self
            .statement
            .iter()
            .map(|(metric, by_date)| (snake_case(metric), by_date))
            .collect();
        metrics.sort_by(|a, b| a.0.cmp(&b.0));
        match metrics.windows(2).find(|pair| pair[0].0 == pair[1].0) {
            Some(pair) => Err(::polars::prelude::PolarsError::Duplicate(
                format!("two metrics map to the column name `{}`", pair[0].0).into(),
            )),
            None => Ok(metrics),
        }
    }
}

/// `TotalRevenue` -> `total_revenue`, `DilutedEPS` -> `diluted_eps`, `totalRevenue` ->
/// `total_revenue`; already snake_case names pass through unchanged.
#[cfg(feature = "dataframe")]
fn snake_case(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::with_capacity(name.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_ascii_uppercase() {
            out.push(c);
            continue;
        }
        let prev = i.checked_sub(1).map(|j| chars[j]);
        let next_is_lower = chars.get(i + 1).is_some_and(char::is_ascii_lowercase);
        let starts_word = match prev {
            Some(p) if p.is_ascii_lowercase() || p.is_ascii_digit() => true,
            Some(p) if p.is_ascii_uppercase() => next_is_lower,
            _ => false,
        };
        if starts_word && !out.ends_with('_') {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
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
                .iter()
                .flatten()
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
        fn snake_case_handles_provider_spellings() {
            for (raw, want) in [
                ("TotalRevenue", "total_revenue"),
                ("totalRevenue", "total_revenue"),
                ("total_revenue", "total_revenue"),
                ("DilutedEPS", "diluted_eps"),
                ("EBITDA", "ebitda"),
                ("NetPPE", "net_ppe"),
                ("EPSBasic", "eps_basic"),
                ("revenue", "revenue"),
            ] {
                assert_eq!(snake_case(raw), want, "{raw}");
            }
        }

        #[test]
        fn camel_and_snake_names_share_one_shape() {
            let df = with_metrics(&["netIncome", "operating_income"])
                .to_wide_dataframe()
                .unwrap();
            let names: Vec<_> = df
                .get_column_names()
                .iter()
                .map(|n| n.to_string())
                .collect();
            assert_eq!(names, ["date", "net_income", "operating_income"]);
        }

        #[test]
        fn colliding_metric_names_error() {
            let statement = with_metrics(&["TotalRevenue", "totalRevenue"]);
            assert!(statement.to_dataframe().is_err());
            assert!(statement.to_wide_dataframe().is_err());
            assert!(with_metrics(&["Date"]).to_wide_dataframe().is_err());
        }
    }
}
