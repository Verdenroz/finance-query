use crate::models::format::{Both, Format};
use finance_query_derive::FormatConvert;
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use finance_query_derive::PyModel;

/// Financial data and key metrics
///
/// Contains financial ratios, margins, cash flow, and analyst recommendations.
///
/// The type parameter `F` controls how numeric fields are represented:
/// - `FinancialData` / `FinancialData<Both>` — **default**; fields hold `FormattedValue<T>`
/// - `FinancialData<Raw>` — fields hold `T` directly (e.g. `Option<f64>`)
/// - `FinancialData<Pretty>` — fields hold `Option<String>` (human-readable)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FormatConvert)]
#[cfg_attr(feature = "python", derive(PyModel))]
#[serde(rename_all = "camelCase", bound = "")]
pub struct FinancialData<F: Format = Both> {
    /// Current stock price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_price: Option<F::Value<f64>>,

    /// Current ratio (current assets / current liabilities)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_ratio: Option<F::Value<f64>>,

    /// Debt to equity ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debt_to_equity: Option<F::Value<f64>>,

    /// Earnings growth rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings_growth: Option<F::Value<f64>>,

    /// EBITDA (Earnings Before Interest, Taxes, Depreciation, and Amortization)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ebitda: Option<F::Value<i64>>,

    /// EBITDA margins
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ebitda_margins: Option<F::Value<f64>>,

    /// Currency code for financial data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub financial_currency: Option<String>,

    /// Free cash flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_cashflow: Option<F::Value<i64>>,

    /// Gross profit margins
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gross_margins: Option<F::Value<f64>>,

    /// Total gross profits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gross_profits: Option<F::Value<i64>>,

    /// Maximum age of data in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<i64>,

    /// Number of analyst opinions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_analyst_opinions: Option<F::Value<i64>>,

    /// Operating cash flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_cashflow: Option<F::Value<i64>>,

    /// Operating margins
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_margins: Option<F::Value<f64>>,

    /// Profit margins
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_margins: Option<F::Value<f64>>,

    /// Quick ratio (quick assets / current liabilities)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quick_ratio: Option<F::Value<f64>>,

    /// Recommendation key (e.g., "buy", "hold", "sell")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation_key: Option<String>,

    /// Mean analyst recommendation (1.0 = strong buy, 5.0 = sell)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation_mean: Option<F::Value<f64>>,

    /// Return on assets (ROA)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_on_assets: Option<F::Value<f64>>,

    /// Return on equity (ROE)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_on_equity: Option<F::Value<f64>>,

    /// Revenue growth rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revenue_growth: Option<F::Value<f64>>,

    /// Revenue per share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revenue_per_share: Option<F::Value<f64>>,

    /// Highest analyst price target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_high_price: Option<F::Value<f64>>,

    /// Lowest analyst price target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_low_price: Option<F::Value<f64>>,

    /// Mean analyst price target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_mean_price: Option<F::Value<f64>>,

    /// Median analyst price target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_median_price: Option<F::Value<f64>>,

    /// Total cash and cash equivalents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cash: Option<F::Value<i64>>,

    /// Total cash per share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cash_per_share: Option<F::Value<f64>>,

    /// Total debt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_debt: Option<F::Value<i64>>,

    /// Total revenue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_revenue: Option<F::Value<i64>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_financial_data() {
        let json = r#"{
            "currentPrice": {
                "fmt": "276.97",
                "raw": 276.97
            },
            "ebitda": {
                "fmt": "144.75B",
                "longFmt": "144,748,003,328",
                "raw": 144748003328
            },
            "financialCurrency": "USD",
            "recommendationKey": "buy"
        }"#;

        let data: FinancialData = serde_json::from_str(json).unwrap();
        assert_eq!(
            data.current_price.as_ref().map(|v| v.raw),
            Some(Some(276.97))
        );
        assert_eq!(
            data.ebitda.as_ref().map(|v| v.raw),
            Some(Some(144748003328))
        );
        assert_eq!(data.financial_currency.as_deref(), Some("USD"));
        assert_eq!(data.recommendation_key.as_deref(), Some("buy"));
    }

    #[test]
    fn test_into_raw() {
        let json = r#"{
            "currentPrice": {"fmt": "276.97", "raw": 276.97},
            "financialCurrency": "USD",
            "recommendationKey": "buy"
        }"#;

        let data: FinancialData = serde_json::from_str(json).unwrap();
        let raw = data.into_raw();
        assert_eq!(raw.current_price, Some(276.97));
        assert_eq!(raw.financial_currency.as_deref(), Some("USD"));
        assert_eq!(raw.recommendation_key.as_deref(), Some("buy"));
    }

    #[test]
    fn test_into_pretty() {
        let json = r#"{
            "currentPrice": {"fmt": "276.97", "raw": 276.97},
            "ebitda": {"fmt": "144.75B", "longFmt": "144,748,003,328", "raw": 144748003328}
        }"#;

        let data: FinancialData = serde_json::from_str(json).unwrap();
        let pretty = data.into_pretty();
        assert_eq!(pretty.current_price.as_deref(), Some("276.97"));
        assert_eq!(pretty.ebitda.as_deref(), Some("144.75B"));
    }
}
