use super::FormattedValue;
use serde::{Deserialize, Serialize};

/// Index trend data (growth estimates for the index)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexTrend {
    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Index symbol
    #[serde(default)]
    pub symbol: Option<String>,

    /// Growth estimates for different periods
    #[serde(default)]
    pub estimates: Option<Vec<TrendEstimate>>,

    /// PE ratio (may be FormattedValue or plain number)
    #[serde(default)]
    pub pe_ratio: Option<FormattedValue<f64>>,

    /// PEG ratio (may be FormattedValue or plain number)
    #[serde(default)]
    pub peg_ratio: Option<FormattedValue<f64>>,
}

/// Industry trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndustryTrend {
    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Industry symbol
    #[serde(default)]
    pub symbol: Option<String>,

    /// Growth estimates for different periods
    #[serde(default)]
    pub estimates: Option<Vec<TrendEstimate>>,

    /// PE ratio (may be FormattedValue or plain number)
    #[serde(default)]
    pub pe_ratio: Option<FormattedValue<f64>>,

    /// PEG ratio (may be FormattedValue or plain number)
    #[serde(default)]
    pub peg_ratio: Option<FormattedValue<f64>>,
}

/// Sector trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorTrend {
    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Sector symbol
    #[serde(default)]
    pub symbol: Option<String>,

    /// Growth estimates for different periods
    #[serde(default)]
    pub estimates: Option<Vec<TrendEstimate>>,

    /// PE ratio (may be FormattedValue or plain number)
    #[serde(default)]
    pub pe_ratio: Option<FormattedValue<f64>>,

    /// PEG ratio (may be FormattedValue or plain number)
    #[serde(default)]
    pub peg_ratio: Option<FormattedValue<f64>>,
}

/// Growth estimate for a specific period
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendEstimate {
    /// Period (e.g., "0q", "+1q", "0y", "+1y", "LTG")
    #[serde(default)]
    pub period: Option<String>,

    /// Growth rate (formatted value)
    #[serde(default)]
    pub growth: Option<FormattedValue<f64>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_index_trend_deserialize() {
        let json = r#"{
          "maxAge": 1,
          "symbol": "SP5",
          "estimates": [
            {
              "period": "0q",
              "growth": {"raw": 0.1516, "fmt": "0.15"}
            },
            {
              "period": "+1q",
              "growth": {"raw": 0.067600004, "fmt": "0.07"}
            }
          ],
          "peRatio": {},
          "pegRatio": {}
        }"#;

        let result: Result<IndexTrend, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "IndexTrend should deserialize successfully: {:?}",
            result.err()
        );

        let trend = result.unwrap();
        assert_eq!(trend.max_age, Some(1));
        assert_eq!(trend.symbol, Some("SP5".to_string()));
        assert!(trend.estimates.is_some());
        assert_eq!(trend.estimates.as_ref().unwrap().len(), 2);

        // Verify empty peRatio and pegRatio deserialize as None
        assert_eq!(
            trend.pe_ratio,
            Some(FormattedValue {
                fmt: None,
                long_fmt: None,
                raw: None
            })
        );
        assert_eq!(
            trend.peg_ratio,
            Some(FormattedValue {
                fmt: None,
                long_fmt: None,
                raw: None
            })
        );
    }

    #[test]
    fn test_index_trend_empty() {
        let json = r#"{"maxAge": 1, "symbol": null, "estimates": []}"#;
        let result: Result<IndexTrend, _> = serde_json::from_str(json);
        assert!(result.is_ok());
    }
}
