use super::formatted_value::FormattedValue;
/// Financial Data module
///
/// Contains key financial metrics and ratios for the company.
use serde::{Deserialize, Serialize};

/// Financial data and key metrics
///
/// Contains financial ratios, margins, cash flow, and analyst recommendations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialData {
    /// Current stock price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_price: Option<FormattedValue<f64>>,

    /// Current ratio (current assets / current liabilities)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_ratio: Option<FormattedValue<f64>>,

    /// Debt to equity ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debt_to_equity: Option<FormattedValue<f64>>,

    /// Earnings growth rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings_growth: Option<FormattedValue<f64>>,

    /// EBITDA (Earnings Before Interest, Taxes, Depreciation, and Amortization)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ebitda: Option<FormattedValue<i64>>,

    /// EBITDA margins
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ebitda_margins: Option<FormattedValue<f64>>,

    /// Currency code for financial data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub financial_currency: Option<String>,

    /// Free cash flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_cashflow: Option<FormattedValue<i64>>,

    /// Gross profit margins
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gross_margins: Option<FormattedValue<f64>>,

    /// Total gross profits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gross_profits: Option<FormattedValue<i64>>,

    /// Maximum age of data in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<i64>,

    /// Number of analyst opinions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_analyst_opinions: Option<FormattedValue<i64>>,

    /// Operating cash flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_cashflow: Option<FormattedValue<i64>>,

    /// Operating margins
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_margins: Option<FormattedValue<f64>>,

    /// Profit margins
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_margins: Option<FormattedValue<f64>>,

    /// Quick ratio (quick assets / current liabilities)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quick_ratio: Option<FormattedValue<f64>>,

    /// Recommendation key (e.g., "buy", "hold", "sell")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation_key: Option<String>,

    /// Mean analyst recommendation (1.0 = strong buy, 5.0 = sell)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation_mean: Option<FormattedValue<f64>>,

    /// Return on assets (ROA)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_on_assets: Option<FormattedValue<f64>>,

    /// Return on equity (ROE)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_on_equity: Option<FormattedValue<f64>>,

    /// Revenue growth rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revenue_growth: Option<FormattedValue<f64>>,

    /// Revenue per share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revenue_per_share: Option<FormattedValue<f64>>,

    /// Highest analyst price target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_high_price: Option<FormattedValue<f64>>,

    /// Lowest analyst price target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_low_price: Option<FormattedValue<f64>>,

    /// Mean analyst price target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_mean_price: Option<FormattedValue<f64>>,

    /// Median analyst price target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_median_price: Option<FormattedValue<f64>>,

    /// Total cash and cash equivalents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cash: Option<FormattedValue<i64>>,

    /// Total cash per share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cash_per_share: Option<FormattedValue<f64>>,

    /// Total debt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_debt: Option<FormattedValue<i64>>,

    /// Total revenue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_revenue: Option<FormattedValue<i64>>,
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
}
