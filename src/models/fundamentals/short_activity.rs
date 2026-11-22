//! Short interest, short volume, and share float models.
//!
//! Served through the [`Capability::FUNDAMENTALS`](crate::Capability::FUNDAMENTALS)
//! route; Polygon is currently the only provider.

use serde::{Deserialize, Serialize};

/// A short-interest data point (bi-monthly settlement report).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
pub struct ShortInterest {
    /// Settlement date (`YYYY-MM-DD`).
    pub settlement_date: Option<String>,
    /// Total shares held short at settlement.
    pub short_interest: Option<f64>,
    /// Average daily trading volume over the reporting period.
    pub avg_daily_volume: Option<f64>,
    /// Days to cover (short interest / average daily volume).
    pub days_to_cover: Option<f64>,
}

/// A daily short-volume data point.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
pub struct ShortVolume {
    /// Trade date (`YYYY-MM-DD`).
    pub date: Option<String>,
    /// Shares sold short.
    pub short_volume: Option<f64>,
    /// Shares sold short exempt from the uptick rule.
    pub short_exempt_volume: Option<f64>,
    /// Total volume.
    pub total_volume: Option<f64>,
}

/// Share float and shares outstanding.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[non_exhaustive]
pub struct ShareFloat {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Freely tradable shares.
    pub float_shares: Option<f64>,
    /// Total shares outstanding, as reported by the provider. `None` when the
    /// provider does not report it; it is never derived from `float_percent`.
    pub outstanding_shares: Option<f64>,
    /// Freely tradable shares as a percentage of shares outstanding (0-100).
    pub float_percent: Option<f64>,
    /// As-of date (`YYYY-MM-DD`).
    pub date: Option<String>,
}

#[cfg(all(test, feature = "dataframe"))]
mod tests {
    use super::*;
    use polars::prelude::DataType;

    #[test]
    fn short_volume_dataframe_keeps_row_order() {
        let rows: Vec<ShortVolume> = serde_json::from_value(serde_json::json!([
            {"date": "2024-01-02", "short_volume": 10.0, "total_volume": 100.0},
            {"date": "2024-01-03", "short_volume": 20.0, "total_volume": 200.0}
        ]))
        .unwrap();
        let df = ShortVolume::vec_to_dataframe(&rows).unwrap();

        let names: Vec<_> = df
            .get_column_names()
            .iter()
            .map(|n| n.to_string())
            .collect();
        assert_eq!(
            names[..4],
            [
                "date",
                "short_volume",
                "short_exempt_volume",
                "total_volume"
            ]
        );
        assert_eq!(df.column("date").unwrap().dtype(), &DataType::String);
        assert_eq!(
            df.column("total_volume").unwrap().dtype(),
            &DataType::Float64
        );
        let dates: Vec<_> = df
            .column("date")
            .unwrap()
            .str()
            .unwrap()
            .into_no_null_iter()
            .collect();
        assert_eq!(dates, ["2024-01-02", "2024-01-03"]);
    }

    #[test]
    fn short_interest_dataframe_columns() {
        let df = ShortInterest::default().to_dataframe().unwrap();

        let names: Vec<_> = df
            .get_column_names()
            .iter()
            .map(|n| n.to_string())
            .collect();
        assert_eq!(
            names[..4],
            [
                "settlement_date",
                "short_interest",
                "avg_daily_volume",
                "days_to_cover"
            ]
        );
        assert_eq!(
            df.column("days_to_cover").unwrap().dtype(),
            &DataType::Float64
        );
    }

    #[test]
    fn share_float_dataframe_columns() {
        let df = ShareFloat::default().to_dataframe().unwrap();

        let names: Vec<_> = df
            .get_column_names()
            .iter()
            .map(|n| n.to_string())
            .collect();
        assert_eq!(
            names[..5],
            [
                "symbol",
                "float_shares",
                "outstanding_shares",
                "float_percent",
                "date"
            ]
        );
        assert_eq!(
            df.column("float_shares").unwrap().dtype(),
            &DataType::Float64
        );
        assert_eq!(df.column("date").unwrap().dtype(), &DataType::String);
    }
}
