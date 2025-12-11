//! Equity Performance Module
//!
//! Contains equity performance data comparing stock returns against a benchmark
//! over various time periods.

use serde::{Deserialize, Serialize};

/// Equity performance data comparing stock returns to benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquityPerformance {
    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Benchmark information
    #[serde(default)]
    pub benchmark: Option<Benchmark>,

    /// Stock performance overview across multiple time periods
    #[serde(default)]
    pub performance_overview: Option<PerformanceOverview>,

    /// Benchmark performance overview for comparison
    #[serde(default)]
    pub performance_overview_benchmark: Option<PerformanceOverview>,
}

/// Benchmark information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Benchmark {
    /// Benchmark symbol (e.g., "^GSPC" for S&P 500)
    #[serde(default)]
    pub symbol: Option<String>,

    /// Benchmark short name (e.g., "S&P 500")
    #[serde(default)]
    pub short_name: Option<String>,
}

/// Performance metrics across multiple time periods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceOverview {
    /// Date the performance data is as of (Unix timestamp)
    #[serde(default)]
    pub as_of_date: Option<crate::models::quote::FormattedValue<i64>>,

    /// 5-day return percentage
    #[serde(default)]
    pub five_days_return: Option<crate::models::quote::FormattedValue<f64>>,

    /// 1-month return percentage
    #[serde(default)]
    pub one_month_return: Option<crate::models::quote::FormattedValue<f64>>,

    /// 3-month return percentage
    #[serde(default)]
    pub three_month_return: Option<crate::models::quote::FormattedValue<f64>>,

    /// 6-month return percentage
    #[serde(default)]
    pub six_month_return: Option<crate::models::quote::FormattedValue<f64>>,

    /// Year-to-date return percentage
    #[serde(default)]
    pub ytd_return_pct: Option<crate::models::quote::FormattedValue<f64>>,

    /// 1-year total return percentage
    #[serde(default)]
    pub one_year_total_return: Option<crate::models::quote::FormattedValue<f64>>,

    /// 2-year total return percentage
    #[serde(default)]
    pub two_year_total_return: Option<crate::models::quote::FormattedValue<f64>>,

    /// 3-year total return percentage
    #[serde(default)]
    pub three_year_total_return: Option<crate::models::quote::FormattedValue<f64>>,

    /// 5-year total return percentage
    #[serde(default)]
    pub five_year_total_return: Option<crate::models::quote::FormattedValue<f64>>,

    /// 10-year total return percentage
    #[serde(default)]
    pub ten_year_total_return: Option<crate::models::quote::FormattedValue<f64>>,

    /// Maximum return percentage (all-time)
    #[serde(default)]
    pub max_return: Option<crate::models::quote::FormattedValue<f64>>,
}

impl EquityPerformance {
    /// Returns the stock's year-to-date return percentage
    pub fn ytd_return(&self) -> Option<f64> {
        self.performance_overview
            .as_ref()?
            .ytd_return_pct
            .as_ref()?
            .raw
    }

    /// Returns the benchmark's year-to-date return percentage
    pub fn benchmark_ytd_return(&self) -> Option<f64> {
        self.performance_overview_benchmark
            .as_ref()?
            .ytd_return_pct
            .as_ref()?
            .raw
    }

    /// Returns the stock's outperformance vs benchmark for YTD (positive = outperforming)
    pub fn ytd_vs_benchmark(&self) -> Option<f64> {
        let stock_ytd = self.ytd_return()?;
        let benchmark_ytd = self.benchmark_ytd_return()?;
        Some(stock_ytd - benchmark_ytd)
    }

    /// Returns the stock's 1-year total return percentage
    pub fn one_year_return(&self) -> Option<f64> {
        self.performance_overview
            .as_ref()?
            .one_year_total_return
            .as_ref()?
            .raw
    }

    /// Returns the benchmark's 1-year total return percentage
    pub fn benchmark_one_year_return(&self) -> Option<f64> {
        self.performance_overview_benchmark
            .as_ref()?
            .one_year_total_return
            .as_ref()?
            .raw
    }

    /// Returns the stock's outperformance vs benchmark for 1 year (positive = outperforming)
    pub fn one_year_vs_benchmark(&self) -> Option<f64> {
        let stock_return = self.one_year_return()?;
        let benchmark_return = self.benchmark_one_year_return()?;
        Some(stock_return - benchmark_return)
    }

    /// Returns the stock's 5-year total return percentage
    pub fn five_year_return(&self) -> Option<f64> {
        self.performance_overview
            .as_ref()?
            .five_year_total_return
            .as_ref()?
            .raw
    }

    /// Returns the benchmark's 5-year total return percentage
    pub fn benchmark_five_year_return(&self) -> Option<f64> {
        self.performance_overview_benchmark
            .as_ref()?
            .five_year_total_return
            .as_ref()?
            .raw
    }

    /// Returns the stock's outperformance vs benchmark for 5 years (positive = outperforming)
    pub fn five_year_vs_benchmark(&self) -> Option<f64> {
        let stock_return = self.five_year_return()?;
        let benchmark_return = self.benchmark_five_year_return()?;
        Some(stock_return - benchmark_return)
    }

    /// Returns the benchmark name
    pub fn benchmark_name(&self) -> Option<&str> {
        self.benchmark.as_ref()?.short_name.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_equity_performance_deserialize() {
        let json = json!({
            "maxAge": 1,
            "benchmark": {
                "symbol": "^GSPC",
                "shortName": "S&P 500"
            },
            "performanceOverview": {
                "asOfDate": 1764892800,
                "fiveDaysReturn": 0.030622387,
                "oneMonthReturn": -0.06551842,
                "threeMonthReturn": 0.092266984,
                "sixMonthReturn": 0.30325815,
                "ytdReturnPct": 0.35870054,
                "oneYearTotalReturn": 0.2578236,
                "twoYearTotalReturn": 2.919418,
                "threeYearTotalReturn": 9.992929,
                "fiveYearTotalReturn": 12.491636,
                "tenYearTotalReturn": 220.57314,
                "maxReturn": 4168.3716
            },
            "performanceOverviewBenchmark": {
                "asOfDate": 1764892800,
                "fiveDaysReturn": 0.0031113708,
                "oneMonthReturn": 0.010904458,
                "threeMonthReturn": 0.060001526,
                "sixMonthReturn": 0.15676934,
                "ytdReturnPct": 0.16811156,
                "oneYearTotalReturn": 0.13090958,
                "twoYearTotalReturn": 0.504298,
                "threeYearTotalReturn": 0.71809816,
                "fiveYearTotalReturn": 0.85730654,
                "tenYearTotalReturn": 2.2846167,
                "maxReturn": 388.03735
            }
        });

        let equity_performance: EquityPerformance = serde_json::from_value(json).unwrap();
        assert_eq!(equity_performance.max_age, Some(1));
        assert_eq!(equity_performance.benchmark_name(), Some("S&P 500"));
        assert_eq!(equity_performance.ytd_return(), Some(0.35870054));
        assert_eq!(equity_performance.benchmark_ytd_return(), Some(0.16811156));
    }

    #[test]
    fn test_equity_performance_vs_benchmark() {
        use crate::models::quote::FormattedValue;

        let equity_performance = EquityPerformance {
            max_age: Some(1),
            benchmark: Some(Benchmark {
                symbol: Some("^GSPC".to_string()),
                short_name: Some("S&P 500".to_string()),
            }),
            performance_overview: Some(PerformanceOverview {
                as_of_date: Some(FormattedValue::new(1764892800)),
                five_days_return: Some(FormattedValue::new(0.030622387)),
                one_month_return: Some(FormattedValue::new(-0.06551842)),
                three_month_return: Some(FormattedValue::new(0.092266984)),
                six_month_return: Some(FormattedValue::new(0.30325815)),
                ytd_return_pct: Some(FormattedValue::new(0.35870054)),
                one_year_total_return: Some(FormattedValue::new(0.2578236)),
                two_year_total_return: Some(FormattedValue::new(2.919418)),
                three_year_total_return: Some(FormattedValue::new(9.992929)),
                five_year_total_return: Some(FormattedValue::new(12.491636)),
                ten_year_total_return: Some(FormattedValue::new(220.57314)),
                max_return: Some(FormattedValue::new(4168.3716)),
            }),
            performance_overview_benchmark: Some(PerformanceOverview {
                as_of_date: Some(FormattedValue::new(1764892800)),
                five_days_return: Some(FormattedValue::new(0.0031113708)),
                one_month_return: Some(FormattedValue::new(0.010904458)),
                three_month_return: Some(FormattedValue::new(0.060001526)),
                six_month_return: Some(FormattedValue::new(0.15676934)),
                ytd_return_pct: Some(FormattedValue::new(0.16811156)),
                one_year_total_return: Some(FormattedValue::new(0.13090958)),
                two_year_total_return: Some(FormattedValue::new(0.504298)),
                three_year_total_return: Some(FormattedValue::new(0.71809816)),
                five_year_total_return: Some(FormattedValue::new(0.85730654)),
                ten_year_total_return: Some(FormattedValue::new(2.2846167)),
                max_return: Some(FormattedValue::new(388.03735)),
            }),
        };

        // Check YTD outperformance (35.87% - 16.81% H 19.06%)
        let ytd_vs = equity_performance.ytd_vs_benchmark().unwrap();
        assert!((ytd_vs - 0.19058898).abs() < 0.0001);

        // Check 1-year outperformance (25.78% - 13.09% H 12.69%)
        let one_year_vs = equity_performance.one_year_vs_benchmark().unwrap();
        assert!((one_year_vs - 0.12691402).abs() < 0.0001);

        // Check 5-year outperformance (1249.16% - 85.73% H 1163.43%)
        let five_year_vs = equity_performance.five_year_vs_benchmark().unwrap();
        assert!((five_year_vs - 11.634329).abs() < 0.001);
    }
}
