//! FMP financial analysis endpoints (ratios, metrics, DCF, ratings, growth).

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::models::Period;

// ============================================================================
// Response types
// ============================================================================

/// Financial ratios from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FinancialRatios {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Calendar year.
    #[serde(rename = "calendarYear")]
    pub calendar_year: Option<String>,
    /// Fiscal period.
    pub period: Option<String>,
    /// Current ratio.
    #[serde(rename = "currentRatio")]
    pub current_ratio: Option<f64>,
    /// Quick ratio.
    #[serde(rename = "quickRatio")]
    pub quick_ratio: Option<f64>,
    /// Cash ratio.
    #[serde(rename = "cashRatio")]
    pub cash_ratio: Option<f64>,
    /// Gross profit margin.
    #[serde(rename = "grossProfitMargin")]
    pub gross_profit_margin: Option<f64>,
    /// Operating profit margin.
    #[serde(rename = "operatingProfitMargin")]
    pub operating_profit_margin: Option<f64>,
    /// Net profit margin.
    #[serde(rename = "netProfitMargin")]
    pub net_profit_margin: Option<f64>,
    /// Return on assets.
    #[serde(rename = "returnOnAssets")]
    pub return_on_assets: Option<f64>,
    /// Return on equity.
    #[serde(rename = "returnOnEquity")]
    pub return_on_equity: Option<f64>,
    /// Return on capital employed.
    #[serde(rename = "returnOnCapitalEmployed")]
    pub return_on_capital_employed: Option<f64>,
    /// Debt-to-equity ratio.
    #[serde(rename = "debtEquityRatio")]
    pub debt_equity_ratio: Option<f64>,
    /// Debt ratio.
    #[serde(rename = "debtRatio")]
    pub debt_ratio: Option<f64>,
    /// Price-to-earnings ratio.
    #[serde(rename = "priceEarningsRatio")]
    pub price_earnings_ratio: Option<f64>,
    /// Price-to-book ratio.
    #[serde(rename = "priceToBookRatio")]
    pub price_to_book_ratio: Option<f64>,
    /// Price-to-sales ratio.
    #[serde(rename = "priceToSalesRatio")]
    pub price_to_sales_ratio: Option<f64>,
    /// Price-to-free-cash-flow ratio.
    #[serde(rename = "priceToFreeCashFlowsRatio")]
    pub price_to_free_cash_flows_ratio: Option<f64>,
    /// EV to EBITDA.
    #[serde(rename = "enterpriseValueMultiple")]
    pub enterprise_value_multiple: Option<f64>,
    /// Dividend yield.
    #[serde(rename = "dividendYield")]
    pub dividend_yield: Option<f64>,
    /// Payout ratio.
    #[serde(rename = "payoutRatio")]
    pub payout_ratio: Option<f64>,
}

/// Key metrics from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct KeyMetrics {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Calendar year.
    #[serde(rename = "calendarYear")]
    pub calendar_year: Option<String>,
    /// Fiscal period.
    pub period: Option<String>,
    /// Revenue per share.
    #[serde(rename = "revenuePerShare")]
    pub revenue_per_share: Option<f64>,
    /// Net income per share.
    #[serde(rename = "netIncomePerShare")]
    pub net_income_per_share: Option<f64>,
    /// Operating cash flow per share.
    #[serde(rename = "operatingCashFlowPerShare")]
    pub operating_cash_flow_per_share: Option<f64>,
    /// Free cash flow per share.
    #[serde(rename = "freeCashFlowPerShare")]
    pub free_cash_flow_per_share: Option<f64>,
    /// Cash per share.
    #[serde(rename = "cashPerShare")]
    pub cash_per_share: Option<f64>,
    /// Book value per share.
    #[serde(rename = "bookValuePerShare")]
    pub book_value_per_share: Option<f64>,
    /// Tangible book value per share.
    #[serde(rename = "tangibleBookValuePerShare")]
    pub tangible_book_value_per_share: Option<f64>,
    /// Shareholders equity per share.
    #[serde(rename = "shareholdersEquityPerShare")]
    pub shareholders_equity_per_share: Option<f64>,
    /// Interest debt per share.
    #[serde(rename = "interestDebtPerShare")]
    pub interest_debt_per_share: Option<f64>,
    /// Market capitalization.
    #[serde(rename = "marketCap")]
    pub market_cap: Option<f64>,
    /// Enterprise value.
    #[serde(rename = "enterpriseValue")]
    pub enterprise_value: Option<f64>,
    /// PE ratio.
    #[serde(rename = "peRatio")]
    pub pe_ratio: Option<f64>,
    /// PB ratio.
    #[serde(rename = "pbRatio")]
    pub pb_ratio: Option<f64>,
    /// EV to sales.
    #[serde(rename = "evToSales")]
    pub ev_to_sales: Option<f64>,
    /// EV to EBITDA (enterprise value multiple).
    #[serde(rename = "enterpriseValueOverEBITDA")]
    pub enterprise_value_over_ebitda: Option<f64>,
    /// EV to operating cash flow.
    #[serde(rename = "evToOperatingCashFlow")]
    pub ev_to_operating_cash_flow: Option<f64>,
    /// EV to free cash flow.
    #[serde(rename = "evToFreeCashFlow")]
    pub ev_to_free_cash_flow: Option<f64>,
    /// Earnings yield.
    #[serde(rename = "earningsYield")]
    pub earnings_yield: Option<f64>,
    /// Free cash flow yield.
    #[serde(rename = "freeCashFlowYield")]
    pub free_cash_flow_yield: Option<f64>,
    /// Debt to equity.
    #[serde(rename = "debtToEquity")]
    pub debt_to_equity: Option<f64>,
    /// Debt to assets.
    #[serde(rename = "debtToAssets")]
    pub debt_to_assets: Option<f64>,
    /// Net debt to EBITDA.
    #[serde(rename = "netDebtToEBITDA")]
    pub net_debt_to_ebitda: Option<f64>,
    /// Current ratio.
    #[serde(rename = "currentRatio")]
    pub current_ratio: Option<f64>,
    /// Dividend yield.
    #[serde(rename = "dividendYield")]
    pub dividend_yield: Option<f64>,
    /// Payout ratio.
    #[serde(rename = "payoutRatio")]
    pub payout_ratio: Option<f64>,
}

/// Enterprise value from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EnterpriseValue {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Stock price.
    #[serde(rename = "stockPrice")]
    pub stock_price: Option<f64>,
    /// Number of shares.
    #[serde(rename = "numberOfShares")]
    pub number_of_shares: Option<f64>,
    /// Market capitalization.
    #[serde(rename = "marketCapitalization")]
    pub market_capitalization: Option<f64>,
    /// Minus cash and cash equivalents.
    #[serde(rename = "minusCashAndCashEquivalents")]
    pub minus_cash_and_cash_equivalents: Option<f64>,
    /// Add total debt.
    #[serde(rename = "addTotalDebt")]
    pub add_total_debt: Option<f64>,
    /// Enterprise value.
    #[serde(rename = "enterpriseValue")]
    pub enterprise_value: Option<f64>,
}

/// Discounted cash flow valuation from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DiscountedCashFlow {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// DCF value.
    pub dcf: Option<f64>,
    /// Stock price.
    #[serde(rename = "Stock Price")]
    pub stock_price: Option<f64>,
}

/// Historical discounted cash flow from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HistoricalDcf {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// DCF value.
    pub dcf: Option<f64>,
    /// Stock price.
    pub price: Option<f64>,
}

/// Company rating from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CompanyRating {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Overall rating.
    pub rating: Option<String>,
    /// Rating score.
    #[serde(rename = "ratingScore")]
    pub rating_score: Option<i32>,
    /// Rating recommendation (Buy, Sell, etc.).
    #[serde(rename = "ratingRecommendation")]
    pub rating_recommendation: Option<String>,
    /// Rating DCF score.
    #[serde(rename = "ratingDetailsDCFScore")]
    pub rating_details_dcf_score: Option<i32>,
    /// Rating DCF recommendation.
    #[serde(rename = "ratingDetailsDCFRecommendation")]
    pub rating_details_dcf_recommendation: Option<String>,
    /// Rating ROE score.
    #[serde(rename = "ratingDetailsROEScore")]
    pub rating_details_roe_score: Option<i32>,
    /// Rating ROE recommendation.
    #[serde(rename = "ratingDetailsROERecommendation")]
    pub rating_details_roe_recommendation: Option<String>,
    /// Rating ROA score.
    #[serde(rename = "ratingDetailsROAScore")]
    pub rating_details_roa_score: Option<i32>,
    /// Rating ROA recommendation.
    #[serde(rename = "ratingDetailsROARecommendation")]
    pub rating_details_roa_recommendation: Option<String>,
    /// Rating DE score.
    #[serde(rename = "ratingDetailsDEScore")]
    pub rating_details_de_score: Option<i32>,
    /// Rating DE recommendation.
    #[serde(rename = "ratingDetailsDERecommendation")]
    pub rating_details_de_recommendation: Option<String>,
    /// Rating PE score.
    #[serde(rename = "ratingDetailsPEScore")]
    pub rating_details_pe_score: Option<i32>,
    /// Rating PE recommendation.
    #[serde(rename = "ratingDetailsPERecommendation")]
    pub rating_details_pe_recommendation: Option<String>,
    /// Rating PB score.
    #[serde(rename = "ratingDetailsPBScore")]
    pub rating_details_pb_score: Option<i32>,
    /// Rating PB recommendation.
    #[serde(rename = "ratingDetailsPBRecommendation")]
    pub rating_details_pb_recommendation: Option<String>,
}

/// Financial growth metrics from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FinancialGrowth {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Calendar year.
    #[serde(rename = "calendarYear")]
    pub calendar_year: Option<String>,
    /// Fiscal period.
    pub period: Option<String>,
    /// Revenue growth.
    #[serde(rename = "revenueGrowth")]
    pub revenue_growth: Option<f64>,
    /// Gross profit growth.
    #[serde(rename = "grossProfitGrowth")]
    pub gross_profit_growth: Option<f64>,
    /// EBITDA growth.
    #[serde(rename = "ebitgrowth")]
    pub ebit_growth: Option<f64>,
    /// Operating income growth.
    #[serde(rename = "operatingIncomeGrowth")]
    pub operating_income_growth: Option<f64>,
    /// Net income growth.
    #[serde(rename = "netIncomeGrowth")]
    pub net_income_growth: Option<f64>,
    /// EPS growth.
    #[serde(rename = "epsgrowth")]
    pub eps_growth: Option<f64>,
    /// EPS diluted growth.
    #[serde(rename = "epsdilutedGrowth")]
    pub eps_diluted_growth: Option<f64>,
    /// Weighted average shares growth.
    #[serde(rename = "weightedAverageSharesGrowth")]
    pub weighted_average_shares_growth: Option<f64>,
    /// Weighted average shares diluted growth.
    #[serde(rename = "weightedAverageSharesDilutedGrowth")]
    pub weighted_average_shares_diluted_growth: Option<f64>,
    /// Dividend per share growth.
    #[serde(rename = "dividendsperShareGrowth")]
    pub dividends_per_share_growth: Option<f64>,
    /// Operating cash flow growth.
    #[serde(rename = "operatingCashFlowGrowth")]
    pub operating_cash_flow_growth: Option<f64>,
    /// Free cash flow growth.
    #[serde(rename = "freeCashFlowGrowth")]
    pub free_cash_flow_growth: Option<f64>,
    /// Receivables growth.
    #[serde(rename = "receivablesGrowth")]
    pub receivables_growth: Option<f64>,
    /// Inventory growth.
    #[serde(rename = "inventoryGrowth")]
    pub inventory_growth: Option<f64>,
    /// Asset growth.
    #[serde(rename = "assetGrowth")]
    pub asset_growth: Option<f64>,
    /// Book value per share growth.
    #[serde(rename = "bookValueperShareGrowth")]
    pub book_value_per_share_growth: Option<f64>,
    /// Debt growth.
    #[serde(rename = "debtGrowth")]
    pub debt_growth: Option<f64>,
    /// R&D expense growth.
    #[serde(rename = "rdexpenseGrowth")]
    pub rd_expense_growth: Option<f64>,
    /// SGA expenses growth.
    #[serde(rename = "sgaexpensesGrowth")]
    pub sga_expenses_growth: Option<f64>,
}

// ============================================================================
// Query functions
// ============================================================================

/// Fetch financial ratios for a symbol.
pub async fn financial_ratios(
    symbol: &str,
    period: Period,
    limit: Option<u32>,
) -> Result<Vec<FinancialRatios>> {
    let client = super::build_client()?;
    let limit_str = limit.unwrap_or(4).to_string();
    client
        .get(
            &format!("/api/v3/ratios/{symbol}"),
            &[("period", period.as_str()), ("limit", &limit_str)],
        )
        .await
}

/// Fetch key metrics for a symbol.
pub async fn key_metrics(
    symbol: &str,
    period: Period,
    limit: Option<u32>,
) -> Result<Vec<KeyMetrics>> {
    let client = super::build_client()?;
    let limit_str = limit.unwrap_or(4).to_string();
    client
        .get(
            &format!("/api/v3/key-metrics/{symbol}"),
            &[("period", period.as_str()), ("limit", &limit_str)],
        )
        .await
}

/// Fetch enterprise values for a symbol.
pub async fn enterprise_value(
    symbol: &str,
    period: Period,
    limit: Option<u32>,
) -> Result<Vec<EnterpriseValue>> {
    let client = super::build_client()?;
    let limit_str = limit.unwrap_or(4).to_string();
    client
        .get(
            &format!("/api/v3/enterprise-values/{symbol}"),
            &[("period", period.as_str()), ("limit", &limit_str)],
        )
        .await
}

/// Fetch current discounted cash flow valuation for a symbol.
pub async fn discounted_cash_flow(symbol: &str) -> Result<Vec<DiscountedCashFlow>> {
    let client = super::build_client()?;
    client
        .get(&format!("/api/v3/discounted-cash-flow/{symbol}"), &[])
        .await
}

/// Fetch historical discounted cash flow for a symbol.
pub async fn historical_dcf(
    symbol: &str,
    period: Period,
    limit: Option<u32>,
) -> Result<Vec<HistoricalDcf>> {
    let client = super::build_client()?;
    let limit_str = limit.unwrap_or(10).to_string();
    client
        .get(
            &format!("/api/v3/historical-discounted-cash-flow-statement/{symbol}"),
            &[("period", period.as_str()), ("limit", &limit_str)],
        )
        .await
}

/// Fetch company rating for a symbol.
pub async fn company_rating(symbol: &str) -> Result<Vec<CompanyRating>> {
    let client = super::build_client()?;
    client
        .get(&format!("/api/v3/rating/{symbol}"), &[])
        .await
}

/// Fetch historical ratings for a symbol.
pub async fn historical_rating(symbol: &str, limit: Option<u32>) -> Result<Vec<CompanyRating>> {
    let client = super::build_client()?;
    let limit_str = limit.unwrap_or(100).to_string();
    client
        .get(
            &format!("/api/v3/historical-rating/{symbol}"),
            &[("limit", &limit_str)],
        )
        .await
}

/// Fetch financial growth metrics for a symbol.
pub async fn financial_growth(
    symbol: &str,
    period: Period,
    limit: Option<u32>,
) -> Result<Vec<FinancialGrowth>> {
    let client = super::build_client()?;
    let limit_str = limit.unwrap_or(4).to_string();
    client
        .get(
            &format!("/api/v3/financial-growth/{symbol}"),
            &[("period", period.as_str()), ("limit", &limit_str)],
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_financial_ratios_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/ratios/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("period".into(), "annual".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "1".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([{
                    "symbol": "AAPL",
                    "date": "2024-09-28",
                    "calendarYear": "2024",
                    "period": "FY",
                    "currentRatio": 0.8673,
                    "quickRatio": 0.8268,
                    "grossProfitMargin": 0.4623,
                    "netProfitMargin": 0.2395,
                    "returnOnEquity": 1.6067,
                    "priceEarningsRatio": 34.12,
                    "dividendYield": 0.0044
                }])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result: Vec<FinancialRatios> = client
            .get(
                "/api/v3/ratios/AAPL",
                &[("period", "annual"), ("limit", "1")],
            )
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(result[0].gross_profit_margin, Some(0.4623));
        assert_eq!(result[0].price_earnings_ratio, Some(34.12));
    }

    #[tokio::test]
    async fn test_company_rating_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/rating/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([{
                    "symbol": "AAPL",
                    "date": "2024-12-01",
                    "rating": "S",
                    "ratingScore": 5,
                    "ratingRecommendation": "Strong Buy",
                    "ratingDetailsDCFScore": 5,
                    "ratingDetailsDCFRecommendation": "Strong Buy",
                    "ratingDetailsROEScore": 5,
                    "ratingDetailsROERecommendation": "Strong Buy"
                }])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result: Vec<CompanyRating> = client
            .get("/api/v3/rating/AAPL", &[])
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].rating.as_deref(), Some("S"));
        assert_eq!(result[0].rating_score, Some(5));
        assert_eq!(
            result[0].rating_recommendation.as_deref(),
            Some("Strong Buy")
        );
    }
}
