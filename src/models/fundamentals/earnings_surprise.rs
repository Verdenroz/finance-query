//! Earnings-surprise history.
//!
//! Served through the [`Capability::FUNDAMENTALS`](crate::Capability::FUNDAMENTALS)
//! route; FMP and Yahoo both implement it.

use serde::{Deserialize, Serialize};

/// One reported earnings result versus the analyst estimate.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
pub struct EarningsSurprise {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Earnings report date (`YYYY-MM-DD`).
    pub date: Option<String>,
    /// Actual reported EPS.
    pub actual_eps: Option<f64>,
    /// Analyst-estimated EPS.
    pub estimated_eps: Option<f64>,
    /// `actual_eps - estimated_eps`.
    pub surprise: Option<f64>,
    /// Surprise as a percentage of the estimate.
    pub surprise_percent: Option<f64>,
}

#[cfg(all(test, feature = "dataframe"))]
mod tests {
    use super::*;
    use polars::prelude::DataType;

    #[test]
    fn dataframe_columns_and_dtypes() {
        let surprise: EarningsSurprise = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL", "date": "2024-05-02", "actual_eps": 1.5, "estimated_eps": 1.4
        }))
        .unwrap();
        let df = EarningsSurprise::vec_to_dataframe(&[surprise]).unwrap();

        let names: Vec<_> = df
            .get_column_names()
            .iter()
            .map(|n| n.to_string())
            .collect();
        assert_eq!(
            names,
            [
                "symbol",
                "date",
                "actual_eps",
                "estimated_eps",
                "surprise",
                "surprise_percent"
            ]
        );
        assert_eq!(df.column("symbol").unwrap().dtype(), &DataType::String);
        assert_eq!(df.column("actual_eps").unwrap().dtype(), &DataType::Float64);
        assert_eq!(df.height(), 1);
    }
}
