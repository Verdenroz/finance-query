//! Quote module
//!
//! Contains the fully typed Quote struct for serialization and API responses.

use serde::{Deserialize, Serialize};

use super::{
    AssetProfile, BalanceSheetHistory, BalanceSheetHistoryQuarterly, CalendarEvents,
    CashflowStatementHistory, CashflowStatementHistoryQuarterly, DefaultKeyStatistics, Earnings,
    EarningsHistory, EarningsTrend, EquityPerformance, FinancialData, FundOwnership,
    FundPerformance, FundProfile, IncomeStatementHistory, IncomeStatementHistoryQuarterly,
    IndexTrend, IndustryTrend, InsiderHolders, InsiderTransactions, InstitutionOwnership,
    MajorHoldersBreakdown, NetSharePurchaseActivity, Price, QuoteSummaryResponse, QuoteTypeData,
    RecommendationTrend, SecFilings, SectorTrend, SummaryDetail, SummaryProfile, TopHoldings,
    UpgradeDowngradeHistory,
};

/// Flattened quote data with deduplicated fields
///
/// This is the primary data structure for stock quotes. It flattens scalar fields
/// from multiple Yahoo Finance modules while preserving complex nested objects.
///
/// # Creating Quote Instances
///
/// Quote instances can only be obtained through the Ticker API:
/// ```no_run
/// # use finance_query::Ticker;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let ticker = Ticker::new("AAPL")?;
/// let quote = ticker.quote(true)?;  // include_logo = true
/// println!("Price: {:?}", quote.regular_market_price);
/// # Ok(())
/// # }
/// ```
///
/// Note: This struct is marked `#[non_exhaustive]` and cannot be constructed manually.
/// Use `Ticker::quote()` or `AsyncTicker::quote()` instead.
///
/// # Field Precedence
///
/// For duplicate fields across Yahoo Finance modules:
/// - Price → SummaryDetail → DefaultKeyStatistics → FinancialData → AssetProfile
///
/// All fields are optional since Yahoo Finance may not return all data for every symbol.
///
/// # DataFrame Conversion
///
/// With the `dataframe` feature enabled, call `.to_dataframe()` to convert to a polars DataFrame:
/// ```ignore
/// let df = quote.to_dataframe()?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Quote {
    /// Stock symbol
    pub symbol: String,

    /// Company logo URL (50x50px)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,

    /// Alternative company logo URL (50x50px)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_logo_url: Option<String>,

    // ===== IDENTITY & METADATA =====
    /// Short name of the security
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,

    /// Long name of the security
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_name: Option<String>,

    /// Exchange code (e.g., "NMS" for NASDAQ)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange: Option<String>,

    /// Exchange name (e.g., "NasdaqGS")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_name: Option<String>,

    /// Quote type (e.g., "EQUITY", "ETF", "MUTUALFUND")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_type: Option<String>,

    /// Currency code (e.g., "USD")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    /// Currency symbol (e.g., "$")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_symbol: Option<String>,

    /// Underlying symbol (for derivatives/options)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlying_symbol: Option<String>,

    /// From currency (for forex pairs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_currency: Option<String>,

    /// To currency (for forex pairs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_currency: Option<String>,

    // ===== REAL-TIME PRICE DATA =====
    /// Current regular market price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_price: Option<super::FormattedValue<f64>>,

    /// Regular market change value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_change: Option<super::FormattedValue<f64>>,

    /// Regular market change percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_change_percent: Option<super::FormattedValue<f64>>,

    /// Regular market time as Unix timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_time: Option<i64>,

    /// Regular market day high
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_day_high: Option<super::FormattedValue<f64>>,

    /// Regular market day low
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_day_low: Option<super::FormattedValue<f64>>,

    /// Regular market open price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_open: Option<super::FormattedValue<f64>>,

    /// Regular market previous close
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_previous_close: Option<super::FormattedValue<f64>>,

    /// Regular market volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regular_market_volume: Option<super::FormattedValue<i64>>,

    /// Current market state (e.g., "REGULAR", "POST", "PRE")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_state: Option<String>,

    // ===== ALTERNATIVE TRADING METRICS (from summaryDetail) =====
    /// Day's high price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_high: Option<super::FormattedValue<f64>>,

    /// Day's low price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_low: Option<super::FormattedValue<f64>>,

    /// Opening price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open: Option<super::FormattedValue<f64>>,

    /// Previous close price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_close: Option<super::FormattedValue<f64>>,

    /// Trading volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<super::FormattedValue<i64>>,

    // ===== PRICE HISTORY =====
    /// All-time high price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_time_high: Option<super::FormattedValue<f64>>,

    /// All-time low price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_time_low: Option<super::FormattedValue<f64>>,

    // ===== PRE/POST MARKET DATA =====
    /// Pre-market price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_price: Option<super::FormattedValue<f64>>,

    /// Pre-market change value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_change: Option<super::FormattedValue<f64>>,

    /// Pre-market change percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_change_percent: Option<super::FormattedValue<f64>>,

    /// Pre-market time as Unix timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_market_time: Option<i64>,

    /// Post-market price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_price: Option<super::FormattedValue<f64>>,

    /// Post-market change value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_change: Option<super::FormattedValue<f64>>,

    /// Post-market change percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_change_percent: Option<super::FormattedValue<f64>>,

    /// Post-market time as Unix timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_market_time: Option<i64>,

    // ===== VOLUME DATA =====
    /// Average daily volume over 10 days
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_daily_volume10_day: Option<super::FormattedValue<i64>>,

    /// Average daily volume over 3 months
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_daily_volume3_month: Option<super::FormattedValue<i64>>,

    /// Average trading volume
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_volume: Option<super::FormattedValue<i64>>,

    /// Average trading volume (10 days)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_volume10days: Option<super::FormattedValue<i64>>,

    // ===== VALUATION METRICS =====
    /// Market capitalization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_cap: Option<super::FormattedValue<i64>>,

    /// Total enterprise value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_value: Option<super::FormattedValue<i64>>,

    /// Enterprise value to revenue ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_to_revenue: Option<super::FormattedValue<f64>>,

    /// Enterprise value to EBITDA ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_to_ebitda: Option<super::FormattedValue<f64>>,

    /// Price to book value ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_to_book: Option<super::FormattedValue<f64>>,

    /// Price to sales ratio (trailing 12 months)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_to_sales_trailing12_months: Option<super::FormattedValue<f64>>,

    // ===== PE RATIOS =====
    /// Forward price-to-earnings ratio
    #[serde(rename = "forwardPE", skip_serializing_if = "Option::is_none")]
    pub forward_pe: Option<super::FormattedValue<f64>>,

    /// Trailing price-to-earnings ratio
    #[serde(rename = "trailingPE", skip_serializing_if = "Option::is_none")]
    pub trailing_pe: Option<super::FormattedValue<f64>>,

    // ===== RISK METRICS =====
    /// Beta coefficient (volatility vs market)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beta: Option<super::FormattedValue<f64>>,

    // ===== 52-WEEK RANGE & MOVING AVERAGES =====
    /// 52-week high price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fifty_two_week_high: Option<super::FormattedValue<f64>>,

    /// 52-week low price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fifty_two_week_low: Option<super::FormattedValue<f64>>,

    /// 50-day moving average
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fifty_day_average: Option<super::FormattedValue<f64>>,

    /// 200-day moving average
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two_hundred_day_average: Option<super::FormattedValue<f64>>,

    /// 52-week price change percentage
    #[serde(rename = "52WeekChange", skip_serializing_if = "Option::is_none")]
    pub week_52_change: Option<super::FormattedValue<f64>>,

    /// S&P 500 52-week change percentage
    #[serde(rename = "SandP52WeekChange", skip_serializing_if = "Option::is_none")]
    pub sand_p_52_week_change: Option<super::FormattedValue<f64>>,

    // ===== DIVIDENDS =====
    /// Annual dividend rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dividend_rate: Option<super::FormattedValue<f64>>,

    /// Dividend yield percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dividend_yield: Option<super::FormattedValue<f64>>,

    /// Trailing annual dividend rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_annual_dividend_rate: Option<super::FormattedValue<f64>>,

    /// Trailing annual dividend yield
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_annual_dividend_yield: Option<super::FormattedValue<f64>>,

    /// 5-year average dividend yield
    #[serde(skip_serializing_if = "Option::is_none")]
    pub five_year_avg_dividend_yield: Option<super::FormattedValue<f64>>,

    /// Ex-dividend date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ex_dividend_date: Option<super::FormattedValue<i64>>,

    /// Dividend payout ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payout_ratio: Option<super::FormattedValue<f64>>,

    /// Last dividend value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_dividend_value: Option<super::FormattedValue<f64>>,

    /// Last dividend date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_dividend_date: Option<super::FormattedValue<i64>>,

    // ===== BID/ASK =====
    /// Current bid price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bid: Option<super::FormattedValue<f64>>,

    /// Bid size (shares)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bid_size: Option<super::FormattedValue<i64>>,

    /// Current ask price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask: Option<super::FormattedValue<f64>>,

    /// Ask size (shares)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask_size: Option<super::FormattedValue<i64>>,

    // ===== SHARES & OWNERSHIP =====
    /// Number of shares outstanding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shares_outstanding: Option<super::FormattedValue<i64>>,

    /// Number of floating shares
    #[serde(skip_serializing_if = "Option::is_none")]
    pub float_shares: Option<super::FormattedValue<i64>>,

    /// Implied shares outstanding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implied_shares_outstanding: Option<super::FormattedValue<i64>>,

    /// Percentage of shares held by insiders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub held_percent_insiders: Option<super::FormattedValue<f64>>,

    /// Percentage of shares held by institutions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub held_percent_institutions: Option<super::FormattedValue<f64>>,

    /// Number of shares short
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shares_short: Option<super::FormattedValue<i64>>,

    /// Number of shares short (prior month)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shares_short_prior_month: Option<super::FormattedValue<i64>>,

    /// Short ratio (days to cover)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_ratio: Option<super::FormattedValue<f64>>,

    /// Short interest as percentage of float
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_percent_of_float: Option<super::FormattedValue<f64>>,

    /// Short interest as percentage of shares outstanding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shares_percent_shares_out: Option<super::FormattedValue<f64>>,

    /// Date of short interest data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_short_interest: Option<super::FormattedValue<i64>>,

    // ===== FINANCIAL METRICS =====
    /// Current stock price (from financial data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_price: Option<super::FormattedValue<f64>>,

    /// Highest analyst price target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_high_price: Option<super::FormattedValue<f64>>,

    /// Lowest analyst price target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_low_price: Option<super::FormattedValue<f64>>,

    /// Mean analyst price target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_mean_price: Option<super::FormattedValue<f64>>,

    /// Median analyst price target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_median_price: Option<super::FormattedValue<f64>>,

    /// Mean analyst recommendation (1.0 = strong buy, 5.0 = sell)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation_mean: Option<super::FormattedValue<f64>>,

    /// Recommendation key (e.g., "buy", "hold", "sell")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation_key: Option<String>,

    /// Number of analyst opinions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_analyst_opinions: Option<super::FormattedValue<i64>>,

    /// Total cash and cash equivalents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cash: Option<super::FormattedValue<i64>>,

    /// Total cash per share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cash_per_share: Option<super::FormattedValue<f64>>,

    /// EBITDA (Earnings Before Interest, Taxes, Depreciation, and Amortization)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ebitda: Option<super::FormattedValue<i64>>,

    /// Total debt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_debt: Option<super::FormattedValue<i64>>,

    /// Total revenue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_revenue: Option<super::FormattedValue<i64>>,

    /// Net income to common shareholders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_income_to_common: Option<super::FormattedValue<i64>>,

    /// Debt to equity ratio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debt_to_equity: Option<super::FormattedValue<f64>>,

    /// Revenue per share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revenue_per_share: Option<super::FormattedValue<f64>>,

    /// Return on assets (ROA)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_on_assets: Option<super::FormattedValue<f64>>,

    /// Return on equity (ROE)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_on_equity: Option<super::FormattedValue<f64>>,

    /// Free cash flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_cashflow: Option<super::FormattedValue<i64>>,

    /// Operating cash flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_cashflow: Option<super::FormattedValue<i64>>,

    // ===== MARGINS =====
    /// Profit margins
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profit_margins: Option<super::FormattedValue<f64>>,

    /// Gross profit margins
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gross_margins: Option<super::FormattedValue<f64>>,

    /// EBITDA margins
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ebitda_margins: Option<super::FormattedValue<f64>>,

    /// Operating margins
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operating_margins: Option<super::FormattedValue<f64>>,

    /// Total gross profits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gross_profits: Option<super::FormattedValue<i64>>,

    // ===== GROWTH RATES =====
    /// Earnings growth rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings_growth: Option<super::FormattedValue<f64>>,

    /// Revenue growth rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revenue_growth: Option<super::FormattedValue<f64>>,

    /// Quarterly earnings growth rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings_quarterly_growth: Option<super::FormattedValue<f64>>,

    // ===== RATIOS =====
    /// Current ratio (current assets / current liabilities)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_ratio: Option<super::FormattedValue<f64>>,

    /// Quick ratio (quick assets / current liabilities)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quick_ratio: Option<super::FormattedValue<f64>>,

    // ===== EPS & BOOK VALUE =====
    /// Trailing earnings per share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_eps: Option<super::FormattedValue<f64>>,

    /// Forward earnings per share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_eps: Option<super::FormattedValue<f64>>,

    /// Book value per share
    #[serde(skip_serializing_if = "Option::is_none")]
    pub book_value: Option<super::FormattedValue<f64>>,

    // ===== COMPANY PROFILE =====
    /// Sector
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector: Option<String>,

    /// Sector key (machine-readable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector_key: Option<String>,

    /// Sector display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector_disp: Option<String>,

    /// Industry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<String>,

    /// Industry key (machine-readable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry_key: Option<String>,

    /// Industry display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry_disp: Option<String>,

    /// Long business summary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_business_summary: Option<String>,

    /// Company website
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,

    /// Investor relations website
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ir_website: Option<String>,

    /// Street address line 1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address1: Option<String>,

    /// City
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    /// State or province
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// Postal/ZIP code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip: Option<String>,

    /// Country
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// Phone number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// Number of full-time employees
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_time_employees: Option<i64>,

    /// Fund category (for mutual funds/ETFs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Fund family name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fund_family: Option<String>,

    // ===== RISK SCORES =====
    /// Audit risk score
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_risk: Option<i32>,

    /// Board risk score
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_risk: Option<i32>,

    /// Compensation risk score
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compensation_risk: Option<i32>,

    /// Shareholder rights risk score
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shareholder_rights_risk: Option<i32>,

    /// Overall risk score
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overall_risk: Option<i32>,

    // ===== TIMEZONE & EXCHANGE =====
    /// Full timezone name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone_full_name: Option<String>,

    /// Short timezone name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone_short_name: Option<String>,

    /// GMT offset in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gmt_off_set_milliseconds: Option<i64>,

    /// First trade date (Unix epoch UTC)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_trade_date_epoch_utc: Option<i64>,

    /// Message board ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_board_id: Option<String>,

    /// Exchange data delay in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange_data_delayed_by: Option<i32>,

    // ===== FUND-SPECIFIC =====
    /// Net asset value price (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nav_price: Option<super::FormattedValue<f64>>,

    /// Total assets (for funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_assets: Option<super::FormattedValue<i64>>,

    /// Yield (for bonds/funds)
    #[serde(rename = "yield", skip_serializing_if = "Option::is_none")]
    pub yield_value: Option<super::FormattedValue<f64>>,

    // ===== STOCK SPLITS & DATES =====
    /// Last stock split factor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_split_factor: Option<String>,

    /// Last stock split date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_split_date: Option<super::FormattedValue<i64>>,

    /// Last fiscal year end date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_fiscal_year_end: Option<super::FormattedValue<i64>>,

    /// Next fiscal year end date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_fiscal_year_end: Option<super::FormattedValue<i64>>,

    /// Most recent quarter date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub most_recent_quarter: Option<super::FormattedValue<i64>>,

    // ===== MISC =====
    /// Price hint for decimal places
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_hint: Option<super::FormattedValue<i64>>,

    /// Whether the security is tradeable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tradeable: Option<bool>,

    /// Currency code for financial data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub financial_currency: Option<String>,

    // ===== PRESERVED NESTED OBJECTS =====
    /// Company officers (executives and compensation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_officers: Option<Vec<super::CompanyOfficer>>,

    /// Earnings data (quarterly earnings vs estimates, revenue/earnings history)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings: Option<Earnings>,

    /// Calendar events (upcoming earnings dates, dividend dates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calendar_events: Option<CalendarEvents>,

    /// Analyst recommendation trends over time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation_trend: Option<RecommendationTrend>,

    /// Analyst upgrades/downgrades history
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_downgrade_history: Option<UpgradeDowngradeHistory>,

    /// Historical earnings data (actual vs estimate)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings_history: Option<EarningsHistory>,

    /// Earnings trend data (estimates and revisions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earnings_trend: Option<EarningsTrend>,

    /// Insider stock holdings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insider_holders: Option<InsiderHolders>,

    /// Insider transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insider_transactions: Option<InsiderTransactions>,

    /// Top institutional owners
    #[serde(skip_serializing_if = "Option::is_none")]
    pub institution_ownership: Option<InstitutionOwnership>,

    /// Top fund owners
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fund_ownership: Option<FundOwnership>,

    /// Major holders breakdown (insiders, institutions, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub major_holders_breakdown: Option<MajorHoldersBreakdown>,

    /// Net share purchase activity by insiders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_share_purchase_activity: Option<NetSharePurchaseActivity>,

    /// SEC filings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sec_filings: Option<SecFilings>,

    /// Balance sheet history (annual)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_sheet_history: Option<BalanceSheetHistory>,

    /// Balance sheet history (quarterly)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_sheet_history_quarterly: Option<BalanceSheetHistoryQuarterly>,

    /// Cash flow statement history (annual)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cashflow_statement_history: Option<CashflowStatementHistory>,

    /// Cash flow statement history (quarterly)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cashflow_statement_history_quarterly: Option<CashflowStatementHistoryQuarterly>,

    /// Income statement history (annual)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub income_statement_history: Option<IncomeStatementHistory>,

    /// Income statement history (quarterly)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub income_statement_history_quarterly: Option<IncomeStatementHistoryQuarterly>,

    /// Equity performance (returns vs benchmark across multiple time periods)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equity_performance: Option<EquityPerformance>,

    /// Index trend (PE and PEG ratios)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_trend: Option<IndexTrend>,

    /// Industry trend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry_trend: Option<IndustryTrend>,

    /// Sector trend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector_trend: Option<SectorTrend>,

    /// Fund profile (for ETFs and mutual funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fund_profile: Option<FundProfile>,

    /// Fund performance data (for ETFs and mutual funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fund_performance: Option<FundPerformance>,

    /// Top holdings and sector weightings (for ETFs and mutual funds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_holdings: Option<TopHoldings>,
}

impl Quote {
    /// Creates a Quote from a QuoteSummaryResponse
    ///
    /// Extracts and flattens all typed modules from the raw response.
    /// Field precedence for duplicates: Price → SummaryDetail → KeyStats → FinancialData → AssetProfile
    ///
    /// # Arguments
    ///
    /// * `response` - The quote summary response from Yahoo Finance
    /// * `logo_url` - Optional company logo URL (fetched separately from /v7/finance/quote)
    /// * `company_logo_url` - Optional alternative company logo URL (fetched separately from /v7/finance/quote)
    pub(crate) fn from_response(
        response: &QuoteSummaryResponse,
        logo_url: Option<String>,
        company_logo_url: Option<String>,
    ) -> Self {
        // Deserialize all modules first
        let price: Option<Price> = response.get_typed("price").ok();
        let quote_type: Option<QuoteTypeData> = response.get_typed("quoteType").ok();
        let summary_detail: Option<SummaryDetail> = response.get_typed("summaryDetail").ok();
        let financial_data: Option<FinancialData> = response.get_typed("financialData").ok();
        let key_stats: Option<DefaultKeyStatistics> =
            response.get_typed("defaultKeyStatistics").ok();
        let asset_profile: Option<AssetProfile> = response.get_typed("assetProfile").ok();
        let summary_profile: Option<SummaryProfile> = response.get_typed("summaryProfile").ok();

        Self {
            symbol: response.symbol.clone(),
            logo_url,
            company_logo_url,

            // ===== IDENTITY & METADATA =====
            // Price priority, fallback to QuoteTypeData
            short_name: price
                .as_ref()
                .and_then(|p| p.short_name.clone())
                .or_else(|| quote_type.as_ref().and_then(|q| q.short_name.clone())),

            long_name: price
                .as_ref()
                .and_then(|p| p.long_name.clone())
                .or_else(|| quote_type.as_ref().and_then(|q| q.long_name.clone())),

            exchange: price
                .as_ref()
                .and_then(|p| p.exchange.clone())
                .or_else(|| quote_type.as_ref().and_then(|q| q.exchange.clone())),

            exchange_name: price.as_ref().and_then(|p| p.exchange_name.clone()),

            quote_type: price
                .as_ref()
                .and_then(|p| p.quote_type.clone())
                .or_else(|| quote_type.as_ref().and_then(|q| q.quote_type.clone())),

            currency: price.as_ref().and_then(|p| p.currency.clone()).or_else(|| {
                summary_detail
                    .as_ref()
                    .and_then(|s| s.currency.clone())
                    .or_else(|| {
                        financial_data
                            .as_ref()
                            .and_then(|f| f.financial_currency.clone())
                    })
            }),

            currency_symbol: price.as_ref().and_then(|p| p.currency_symbol.clone()),

            underlying_symbol: price
                .as_ref()
                .and_then(|p| p.underlying_symbol.clone())
                .or_else(|| {
                    quote_type
                        .as_ref()
                        .and_then(|q| q.underlying_symbol.clone())
                }),
            from_currency: price
                .as_ref()
                .and_then(|p| p.from_currency.clone())
                .or_else(|| {
                    summary_detail
                        .as_ref()
                        .and_then(|s| s.from_currency.clone())
                }),
            to_currency: price
                .as_ref()
                .and_then(|p| p.to_currency.clone())
                .or_else(|| summary_detail.as_ref().and_then(|s| s.to_currency.clone())),

            // ===== REAL-TIME PRICE DATA (from Price only) =====
            regular_market_price: price.as_ref().and_then(|p| p.regular_market_price.clone()),
            regular_market_change: price.as_ref().and_then(|p| p.regular_market_change.clone()),
            regular_market_change_percent: price
                .as_ref()
                .and_then(|p| p.regular_market_change_percent.clone()),
            regular_market_time: price.as_ref().and_then(|p| p.regular_market_time),
            regular_market_day_high: price
                .as_ref()
                .and_then(|p| p.regular_market_day_high.clone()),
            regular_market_day_low: price
                .as_ref()
                .and_then(|p| p.regular_market_day_low.clone()),
            regular_market_open: price.as_ref().and_then(|p| p.regular_market_open.clone()),
            regular_market_previous_close: price
                .as_ref()
                .and_then(|p| p.regular_market_previous_close.clone()),
            regular_market_volume: price.as_ref().and_then(|p| p.regular_market_volume.clone()),
            market_state: price.as_ref().and_then(|p| p.market_state.clone()),

            // ===== ALTERNATIVE TRADING METRICS (from summaryDetail) =====
            day_high: summary_detail.as_ref().and_then(|s| s.day_high.clone()),
            day_low: summary_detail.as_ref().and_then(|s| s.day_low.clone()),
            open: summary_detail.as_ref().and_then(|s| s.open.clone()),
            previous_close: summary_detail
                .as_ref()
                .and_then(|s| s.previous_close.clone()),
            volume: summary_detail.as_ref().and_then(|s| s.volume.clone()),

            // ===== PRICE HISTORY =====
            all_time_high: summary_detail
                .as_ref()
                .and_then(|s| s.all_time_high.clone()),
            all_time_low: summary_detail.as_ref().and_then(|s| s.all_time_low.clone()),

            // ===== PRE/POST MARKET DATA =====
            pre_market_price: price.as_ref().and_then(|p| p.pre_market_price.clone()),
            pre_market_change: price.as_ref().and_then(|p| p.pre_market_change.clone()),
            pre_market_change_percent: price
                .as_ref()
                .and_then(|p| p.pre_market_change_percent.clone()),
            pre_market_time: price.as_ref().and_then(|p| p.pre_market_time),
            post_market_price: price.as_ref().and_then(|p| p.post_market_price.clone()),
            post_market_change: price.as_ref().and_then(|p| p.post_market_change.clone()),
            post_market_change_percent: price
                .as_ref()
                .and_then(|p| p.post_market_change_percent.clone()),
            post_market_time: price.as_ref().and_then(|p| p.post_market_time),

            // ===== VOLUME DATA =====
            // Price priority, fallback to SummaryDetail
            average_daily_volume10_day: price
                .as_ref()
                .and_then(|p| p.average_daily_volume10_day.clone())
                .or_else(|| {
                    summary_detail
                        .as_ref()
                        .and_then(|s| s.average_daily_volume10_day.clone())
                }),
            average_daily_volume3_month: price
                .as_ref()
                .and_then(|p| p.average_daily_volume3_month.clone()),
            average_volume: summary_detail
                .as_ref()
                .and_then(|s| s.average_volume.clone()),
            average_volume10days: summary_detail
                .as_ref()
                .and_then(|s| s.average_volume10days.clone()),

            // ===== VALUATION METRICS =====
            // Price priority for market_cap (real-time)
            market_cap: price.as_ref().and_then(|p| p.market_cap.clone()),
            enterprise_value: key_stats.as_ref().and_then(|k| k.enterprise_value.clone()),
            enterprise_to_revenue: key_stats
                .as_ref()
                .and_then(|k| k.enterprise_to_revenue.clone()),
            enterprise_to_ebitda: key_stats
                .as_ref()
                .and_then(|k| k.enterprise_to_ebitda.clone()),
            price_to_book: key_stats.as_ref().and_then(|k| k.price_to_book.clone()),
            price_to_sales_trailing12_months: summary_detail
                .as_ref()
                .and_then(|s| s.price_to_sales_trailing12_months.clone()),

            // ===== PE RATIOS =====
            // SummaryDetail priority, fallback to KeyStats
            forward_pe: summary_detail
                .as_ref()
                .and_then(|s| s.forward_pe.clone())
                .or_else(|| key_stats.as_ref().and_then(|k| k.forward_pe.clone())),
            trailing_pe: summary_detail.as_ref().and_then(|s| s.trailing_pe.clone()),

            // ===== RISK METRICS =====
            // SummaryDetail priority, fallback to KeyStats
            beta: summary_detail
                .as_ref()
                .and_then(|s| s.beta.clone())
                .or_else(|| key_stats.as_ref().and_then(|k| k.beta.clone())),

            // ===== 52-WEEK RANGE & MOVING AVERAGES =====
            fifty_two_week_high: summary_detail
                .as_ref()
                .and_then(|s| s.fifty_two_week_high.clone()),
            fifty_two_week_low: summary_detail
                .as_ref()
                .and_then(|s| s.fifty_two_week_low.clone()),
            fifty_day_average: summary_detail
                .as_ref()
                .and_then(|s| s.fifty_day_average.clone()),
            two_hundred_day_average: summary_detail
                .as_ref()
                .and_then(|s| s.two_hundred_day_average.clone()),
            week_52_change: key_stats.as_ref().and_then(|k| k.week_52_change.clone()),
            sand_p_52_week_change: key_stats
                .as_ref()
                .and_then(|k| k.sand_p_52_week_change.clone()),

            // ===== DIVIDENDS =====
            dividend_rate: summary_detail
                .as_ref()
                .and_then(|s| s.dividend_rate.clone()),
            dividend_yield: summary_detail
                .as_ref()
                .and_then(|s| s.dividend_yield.clone()),
            trailing_annual_dividend_rate: summary_detail
                .as_ref()
                .and_then(|s| s.trailing_annual_dividend_rate.clone()),
            trailing_annual_dividend_yield: summary_detail
                .as_ref()
                .and_then(|s| s.trailing_annual_dividend_yield.clone()),
            five_year_avg_dividend_yield: summary_detail
                .as_ref()
                .and_then(|s| s.five_year_avg_dividend_yield.clone()),
            ex_dividend_date: summary_detail
                .as_ref()
                .and_then(|s| s.ex_dividend_date.clone()),
            payout_ratio: summary_detail.as_ref().and_then(|s| s.payout_ratio.clone()),
            last_dividend_value: key_stats
                .as_ref()
                .and_then(|k| k.last_dividend_value.clone()),
            last_dividend_date: key_stats
                .as_ref()
                .and_then(|k| k.last_dividend_date.clone()),

            // ===== BID/ASK =====
            bid: summary_detail.as_ref().and_then(|s| s.bid.clone()),
            bid_size: summary_detail.as_ref().and_then(|s| s.bid_size.clone()),
            ask: summary_detail.as_ref().and_then(|s| s.ask.clone()),
            ask_size: summary_detail.as_ref().and_then(|s| s.ask_size.clone()),

            // ===== SHARES & OWNERSHIP =====
            shares_outstanding: key_stats
                .as_ref()
                .and_then(|k| k.shares_outstanding.clone()),
            float_shares: key_stats.as_ref().and_then(|k| k.float_shares.clone()),
            implied_shares_outstanding: key_stats
                .as_ref()
                .and_then(|k| k.implied_shares_outstanding.clone()),
            held_percent_insiders: key_stats
                .as_ref()
                .and_then(|k| k.held_percent_insiders.clone()),
            held_percent_institutions: key_stats
                .as_ref()
                .and_then(|k| k.held_percent_institutions.clone()),
            shares_short: key_stats.as_ref().and_then(|k| k.shares_short.clone()),
            shares_short_prior_month: key_stats
                .as_ref()
                .and_then(|k| k.shares_short_prior_month.clone()),
            short_ratio: key_stats.as_ref().and_then(|k| k.short_ratio.clone()),
            short_percent_of_float: key_stats
                .as_ref()
                .and_then(|k| k.short_percent_of_float.clone()),
            shares_percent_shares_out: key_stats
                .as_ref()
                .and_then(|k| k.shares_percent_shares_out.clone()),
            date_short_interest: key_stats
                .as_ref()
                .and_then(|k| k.date_short_interest.clone()),

            // ===== FINANCIAL METRICS =====
            current_price: financial_data
                .as_ref()
                .and_then(|f| f.current_price.clone()),
            target_high_price: financial_data
                .as_ref()
                .and_then(|f| f.target_high_price.clone()),
            target_low_price: financial_data
                .as_ref()
                .and_then(|f| f.target_low_price.clone()),
            target_mean_price: financial_data
                .as_ref()
                .and_then(|f| f.target_mean_price.clone()),
            target_median_price: financial_data
                .as_ref()
                .and_then(|f| f.target_median_price.clone()),
            recommendation_mean: financial_data
                .as_ref()
                .and_then(|f| f.recommendation_mean.clone()),
            recommendation_key: financial_data
                .as_ref()
                .and_then(|f| f.recommendation_key.clone()),
            number_of_analyst_opinions: financial_data
                .as_ref()
                .and_then(|f| f.number_of_analyst_opinions.clone()),
            total_cash: financial_data.as_ref().and_then(|f| f.total_cash.clone()),
            total_cash_per_share: financial_data
                .as_ref()
                .and_then(|f| f.total_cash_per_share.clone()),
            ebitda: financial_data.as_ref().and_then(|f| f.ebitda.clone()),
            total_debt: financial_data.as_ref().and_then(|f| f.total_debt.clone()),
            total_revenue: financial_data
                .as_ref()
                .and_then(|f| f.total_revenue.clone()),
            net_income_to_common: key_stats
                .as_ref()
                .and_then(|k| k.net_income_to_common.clone()),
            debt_to_equity: financial_data
                .as_ref()
                .and_then(|f| f.debt_to_equity.clone()),
            revenue_per_share: financial_data
                .as_ref()
                .and_then(|f| f.revenue_per_share.clone()),
            return_on_assets: financial_data
                .as_ref()
                .and_then(|f| f.return_on_assets.clone()),
            return_on_equity: financial_data
                .as_ref()
                .and_then(|f| f.return_on_equity.clone()),
            free_cashflow: financial_data
                .as_ref()
                .and_then(|f| f.free_cashflow.clone()),
            operating_cashflow: financial_data
                .as_ref()
                .and_then(|f| f.operating_cashflow.clone()),

            // ===== MARGINS =====
            // FinancialData priority
            profit_margins: financial_data
                .as_ref()
                .and_then(|f| f.profit_margins.clone()),
            gross_margins: financial_data
                .as_ref()
                .and_then(|f| f.gross_margins.clone()),
            ebitda_margins: financial_data
                .as_ref()
                .and_then(|f| f.ebitda_margins.clone()),
            operating_margins: financial_data
                .as_ref()
                .and_then(|f| f.operating_margins.clone()),
            gross_profits: financial_data
                .as_ref()
                .and_then(|f| f.gross_profits.clone()),

            // ===== GROWTH RATES =====
            earnings_growth: financial_data
                .as_ref()
                .and_then(|f| f.earnings_growth.clone()),
            revenue_growth: financial_data
                .as_ref()
                .and_then(|f| f.revenue_growth.clone()),
            earnings_quarterly_growth: key_stats
                .as_ref()
                .and_then(|k| k.earnings_quarterly_growth.clone()),

            // ===== RATIOS =====
            current_ratio: financial_data
                .as_ref()
                .and_then(|f| f.current_ratio.clone()),
            quick_ratio: financial_data.as_ref().and_then(|f| f.quick_ratio.clone()),

            // ===== EPS & BOOK VALUE =====
            trailing_eps: key_stats.as_ref().and_then(|k| k.trailing_eps.clone()),
            forward_eps: key_stats.as_ref().and_then(|k| k.forward_eps.clone()),
            book_value: key_stats.as_ref().and_then(|k| k.book_value.clone()),

            // ===== COMPANY PROFILE =====
            // AssetProfile priority, fallback to SummaryProfile
            sector: asset_profile
                .as_ref()
                .and_then(|a| a.sector.clone())
                .or_else(|| summary_profile.as_ref().and_then(|s| s.sector.clone())),
            sector_key: asset_profile.as_ref().and_then(|a| a.sector_key.clone()),
            sector_disp: asset_profile.as_ref().and_then(|a| a.sector_disp.clone()),
            industry: asset_profile
                .as_ref()
                .and_then(|a| a.industry.clone())
                .or_else(|| summary_profile.as_ref().and_then(|s| s.industry.clone())),
            industry_key: asset_profile.as_ref().and_then(|a| a.industry_key.clone()),
            industry_disp: asset_profile.as_ref().and_then(|a| a.industry_disp.clone()),
            long_business_summary: asset_profile
                .as_ref()
                .and_then(|a| a.long_business_summary.clone())
                .or_else(|| {
                    summary_profile
                        .as_ref()
                        .and_then(|s| s.long_business_summary.clone())
                }),
            address1: asset_profile
                .as_ref()
                .and_then(|a| a.address1.clone())
                .or_else(|| summary_profile.as_ref().and_then(|s| s.address1.clone())),
            city: asset_profile
                .as_ref()
                .and_then(|a| a.city.clone())
                .or_else(|| summary_profile.as_ref().and_then(|s| s.city.clone())),
            state: asset_profile
                .as_ref()
                .and_then(|a| a.state.clone())
                .or_else(|| summary_profile.as_ref().and_then(|s| s.state.clone())),
            zip: asset_profile
                .as_ref()
                .and_then(|a| a.zip.clone())
                .or_else(|| summary_profile.as_ref().and_then(|s| s.zip.clone())),
            country: asset_profile
                .as_ref()
                .and_then(|a| a.country.clone())
                .or_else(|| summary_profile.as_ref().and_then(|s| s.country.clone())),
            phone: asset_profile
                .as_ref()
                .and_then(|a| a.phone.clone())
                .or_else(|| summary_profile.as_ref().and_then(|s| s.phone.clone())),
            full_time_employees: asset_profile
                .as_ref()
                .and_then(|a| a.full_time_employees)
                .or_else(|| summary_profile.as_ref().and_then(|s| s.full_time_employees)),

            website: asset_profile
                .as_ref()
                .and_then(|a| a.website.clone())
                .or_else(|| summary_profile.as_ref().and_then(|s| s.website.clone())),
            ir_website: summary_profile.as_ref().and_then(|s| s.ir_website.clone()),

            category: key_stats.as_ref().and_then(|k| k.category.clone()),
            fund_family: key_stats.as_ref().and_then(|k| k.fund_family.clone()),

            // ===== RISK SCORES =====
            audit_risk: asset_profile.as_ref().and_then(|a| a.audit_risk),
            board_risk: asset_profile.as_ref().and_then(|a| a.board_risk),
            compensation_risk: asset_profile.as_ref().and_then(|a| a.compensation_risk),
            shareholder_rights_risk: asset_profile
                .as_ref()
                .and_then(|a| a.shareholder_rights_risk),
            overall_risk: asset_profile.as_ref().and_then(|a| a.overall_risk),

            // ===== TIMEZONE & EXCHANGE =====
            time_zone_full_name: quote_type
                .as_ref()
                .and_then(|q| q.time_zone_full_name.clone()),
            time_zone_short_name: quote_type
                .as_ref()
                .and_then(|q| q.time_zone_short_name.clone()),
            gmt_off_set_milliseconds: quote_type.as_ref().and_then(|q| q.gmt_off_set_milliseconds),
            first_trade_date_epoch_utc: quote_type
                .as_ref()
                .and_then(|q| q.first_trade_date_epoch_utc),
            message_board_id: quote_type.as_ref().and_then(|q| q.message_board_id.clone()),
            exchange_data_delayed_by: price.as_ref().and_then(|p| p.exchange_data_delayed_by),

            // ===== FUND-SPECIFIC =====
            nav_price: summary_detail.as_ref().and_then(|s| s.nav_price.clone()),
            total_assets: summary_detail.as_ref().and_then(|s| s.total_assets.clone()),
            yield_value: summary_detail.as_ref().and_then(|s| s.yield_value.clone()),

            // ===== STOCK SPLITS & DATES =====
            last_split_factor: key_stats.as_ref().and_then(|k| k.last_split_factor.clone()),
            last_split_date: key_stats.as_ref().and_then(|k| k.last_split_date.clone()),
            last_fiscal_year_end: key_stats
                .as_ref()
                .and_then(|k| k.last_fiscal_year_end.clone()),
            next_fiscal_year_end: key_stats
                .as_ref()
                .and_then(|k| k.next_fiscal_year_end.clone()),
            most_recent_quarter: key_stats
                .as_ref()
                .and_then(|k| k.most_recent_quarter.clone()),

            // ===== MISC =====
            // Price priority for price_hint
            price_hint: price.as_ref().and_then(|p| p.price_hint.clone()),
            tradeable: summary_detail.as_ref().and_then(|s| s.tradeable),
            financial_currency: financial_data
                .as_ref()
                .and_then(|f| f.financial_currency.clone()),

            // ===== PRESERVED NESTED OBJECTS =====
            company_officers: asset_profile.as_ref().map(|a| a.company_officers.clone()),
            earnings: response.get_typed("earnings").ok(),
            calendar_events: response.get_typed("calendarEvents").ok(),
            recommendation_trend: response.get_typed("recommendationTrend").ok(),
            upgrade_downgrade_history: response.get_typed("upgradeDowngradeHistory").ok(),
            earnings_history: response.get_typed("earningsHistory").ok(),
            earnings_trend: response.get_typed("earningsTrend").ok(),
            insider_holders: response.get_typed("insiderHolders").ok(),
            insider_transactions: response.get_typed("insiderTransactions").ok(),
            institution_ownership: response.get_typed("institutionOwnership").ok(),
            fund_ownership: response.get_typed("fundOwnership").ok(),
            major_holders_breakdown: response.get_typed("majorHoldersBreakdown").ok(),
            net_share_purchase_activity: response.get_typed("netSharePurchaseActivity").ok(),
            sec_filings: response.get_typed("secFilings").ok(),
            balance_sheet_history: response.get_typed("balanceSheetHistory").ok(),
            balance_sheet_history_quarterly: response
                .get_typed("balanceSheetHistoryQuarterly")
                .ok(),
            cashflow_statement_history: response.get_typed("cashflowStatementHistory").ok(),
            cashflow_statement_history_quarterly: response
                .get_typed("cashflowStatementHistoryQuarterly")
                .ok(),
            income_statement_history: response.get_typed("incomeStatementHistory").ok(),
            income_statement_history_quarterly: response
                .get_typed("incomeStatementHistoryQuarterly")
                .ok(),
            equity_performance: response.get_typed("equityPerformance").ok(),
            index_trend: response.get_typed("indexTrend").ok(),
            industry_trend: response.get_typed("industryTrend").ok(),
            sector_trend: response.get_typed("sectorTrend").ok(),
            fund_profile: response.get_typed("fundProfile").ok(),
            fund_performance: response.get_typed("fundPerformance").ok(),
            top_holdings: response.get_typed("topHoldings").ok(),
        }
    }

    /// Returns the most relevant current price based on market state
    ///
    /// Returns post-market price if in post-market, pre-market price if in pre-market,
    /// otherwise regular market price.
    pub fn live_price(&self) -> Option<f64> {
        if self.market_state.as_deref() == Some("POST") {
            self.post_market_price
                .as_ref()
                .and_then(|p| p.raw)
                .or_else(|| self.regular_market_price.as_ref()?.raw)
        } else if self.market_state.as_deref() == Some("PRE") {
            self.pre_market_price
                .as_ref()
                .and_then(|p| p.raw)
                .or_else(|| self.regular_market_price.as_ref()?.raw)
        } else {
            self.regular_market_price.as_ref()?.raw
        }
    }

    /// Returns the day's trading range as (low, high)
    pub fn day_range(&self) -> Option<(f64, f64)> {
        let low = self.regular_market_day_low.as_ref()?.raw?;
        let high = self.regular_market_day_high.as_ref()?.raw?;
        Some((low, high))
    }

    /// Returns the 52-week range as (low, high)
    pub fn week_52_range(&self) -> Option<(f64, f64)> {
        let low = self.fifty_two_week_low.as_ref()?.raw?;
        let high = self.fifty_two_week_high.as_ref()?.raw?;
        Some((low, high))
    }

    /// Returns whether the market is currently open
    pub fn is_market_open(&self) -> bool {
        self.market_state.as_deref() == Some("REGULAR")
    }

    /// Returns whether this is in pre-market trading
    pub fn is_pre_market(&self) -> bool {
        self.market_state.as_deref() == Some("PRE")
    }

    /// Returns whether this is in post-market trading
    pub fn is_post_market(&self) -> bool {
        self.market_state.as_deref() == Some("POST")
    }

    /// Creates a Quote from a batch /v7/finance/quote response item
    ///
    /// This parses the simpler flat structure from the batch quotes endpoint,
    /// which has fewer fields than the full quoteSummary response.
    pub(crate) fn from_batch_response(json: &serde_json::Value) -> Result<Self, String> {
        let symbol = json
            .get("symbol")
            .and_then(|v| v.as_str())
            .ok_or("Missing symbol")?
            .to_string();

        // Helper to extract FormattedValue<f64>
        // Handles both raw numbers and formatted objects: {"raw": 273.4, "fmt": "273.40"}
        let get_f64 = |key: &str| -> Option<super::FormattedValue<f64>> {
            json.get(key).and_then(|v| {
                if let Some(raw) = v.as_f64() {
                    // Raw number (formatted=false)
                    Some(super::FormattedValue {
                        raw: Some(raw),
                        fmt: None,
                        long_fmt: None,
                    })
                } else if v.is_object() {
                    // Formatted object (formatted=true)
                    Some(super::FormattedValue {
                        raw: v.get("raw").and_then(|r| r.as_f64()),
                        fmt: v.get("fmt").and_then(|f| f.as_str()).map(|s| s.to_string()),
                        long_fmt: v
                            .get("longFmt")
                            .and_then(|f| f.as_str())
                            .map(|s| s.to_string()),
                    })
                } else {
                    None
                }
            })
        };

        // Helper to extract FormattedValue<i64>
        // Handles both raw numbers and formatted objects
        let get_i64 = |key: &str| -> Option<super::FormattedValue<i64>> {
            json.get(key).and_then(|v| {
                if let Some(raw) = v.as_i64() {
                    // Raw number (formatted=false)
                    Some(super::FormattedValue {
                        raw: Some(raw),
                        fmt: None,
                        long_fmt: None,
                    })
                } else if v.is_object() {
                    // Formatted object (formatted=true)
                    Some(super::FormattedValue {
                        raw: v.get("raw").and_then(|r| r.as_i64()),
                        fmt: v.get("fmt").and_then(|f| f.as_str()).map(|s| s.to_string()),
                        long_fmt: v
                            .get("longFmt")
                            .and_then(|f| f.as_str())
                            .map(|s| s.to_string()),
                    })
                } else {
                    None
                }
            })
        };

        // Helper to extract String
        let get_str = |key: &str| -> Option<String> {
            json.get(key)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        };

        Ok(Self {
            symbol,
            logo_url: get_str("logoUrl"),
            company_logo_url: get_str("companyLogoUrl"),

            // Identity
            short_name: get_str("shortName"),
            long_name: get_str("longName"),
            exchange: get_str("exchange"),
            exchange_name: get_str("fullExchangeName"),
            quote_type: get_str("quoteType"),
            currency: get_str("currency"),
            currency_symbol: get_str("currencySymbol"),
            underlying_symbol: get_str("underlyingSymbol"),
            from_currency: get_str("fromCurrency"),
            to_currency: get_str("toCurrency"),

            // Real-time price
            regular_market_price: get_f64("regularMarketPrice"),
            regular_market_change: get_f64("regularMarketChange"),
            regular_market_change_percent: get_f64("regularMarketChangePercent"),
            regular_market_time: json.get("regularMarketTime").and_then(|v| v.as_i64()),
            regular_market_day_high: get_f64("regularMarketDayHigh"),
            regular_market_day_low: get_f64("regularMarketDayLow"),
            regular_market_open: get_f64("regularMarketOpen"),
            regular_market_previous_close: get_f64("regularMarketPreviousClose"),
            regular_market_volume: get_i64("regularMarketVolume"),
            market_state: get_str("marketState"),

            // Alternative trading metrics
            day_high: get_f64("dayHigh"),
            day_low: get_f64("dayLow"),
            open: get_f64("open"),
            previous_close: get_f64("previousClose"),
            volume: get_i64("volume"),

            // Price history
            all_time_high: None,
            all_time_low: None,

            // Pre/post market
            pre_market_price: get_f64("preMarketPrice"),
            pre_market_change: get_f64("preMarketChange"),
            pre_market_change_percent: get_f64("preMarketChangePercent"),
            pre_market_time: json.get("preMarketTime").and_then(|v| v.as_i64()),
            post_market_price: get_f64("postMarketPrice"),
            post_market_change: get_f64("postMarketChange"),
            post_market_change_percent: get_f64("postMarketChangePercent"),
            post_market_time: json.get("postMarketTime").and_then(|v| v.as_i64()),

            // Volume
            average_daily_volume10_day: get_i64("averageDailyVolume10Day"),
            average_daily_volume3_month: get_i64("averageDailyVolume3Month"),
            average_volume: get_i64("averageVolume"),
            average_volume10days: get_i64("averageVolume10days"),

            // Valuation
            market_cap: get_i64("marketCap"),
            enterprise_value: None,
            enterprise_to_revenue: None,
            enterprise_to_ebitda: None,
            price_to_book: get_f64("priceToBook"),
            price_to_sales_trailing12_months: None,

            // PE
            forward_pe: get_f64("forwardPE"),
            trailing_pe: get_f64("trailingPE"),

            // Risk
            beta: None,

            // 52-week
            fifty_two_week_high: get_f64("fiftyTwoWeekHigh"),
            fifty_two_week_low: get_f64("fiftyTwoWeekLow"),
            fifty_day_average: get_f64("fiftyDayAverage"),
            two_hundred_day_average: get_f64("twoHundredDayAverage"),
            week_52_change: get_f64("52WeekChange"),
            sand_p_52_week_change: None,

            // Dividends
            dividend_rate: get_f64("dividendRate"),
            dividend_yield: get_f64("dividendYield"),
            trailing_annual_dividend_rate: get_f64("trailingAnnualDividendRate"),
            trailing_annual_dividend_yield: get_f64("trailingAnnualDividendYield"),
            five_year_avg_dividend_yield: None,
            ex_dividend_date: None,
            payout_ratio: None,
            last_dividend_value: None,
            last_dividend_date: None,

            // Bid/Ask
            bid: get_f64("bid"),
            bid_size: get_i64("bidSize"),
            ask: get_f64("ask"),
            ask_size: get_i64("askSize"),

            // Shares
            shares_outstanding: get_i64("sharesOutstanding"),
            float_shares: None,
            implied_shares_outstanding: None,
            held_percent_insiders: None,
            held_percent_institutions: None,
            shares_short: None,
            shares_short_prior_month: None,
            short_ratio: None,
            short_percent_of_float: None,
            shares_percent_shares_out: None,
            date_short_interest: None,

            // Financial metrics
            current_price: None,
            target_high_price: None,
            target_low_price: None,
            target_mean_price: None,
            target_median_price: None,
            recommendation_mean: None,
            recommendation_key: None,
            number_of_analyst_opinions: None,
            total_cash: None,
            total_cash_per_share: None,
            ebitda: None,
            total_debt: None,
            total_revenue: None,
            net_income_to_common: None,
            debt_to_equity: None,
            revenue_per_share: None,
            return_on_assets: None,
            return_on_equity: None,
            free_cashflow: None,
            operating_cashflow: None,

            // Margins
            profit_margins: None,
            gross_margins: None,
            ebitda_margins: None,
            operating_margins: None,
            gross_profits: None,

            // Growth
            earnings_growth: None,
            revenue_growth: None,
            earnings_quarterly_growth: None,

            // Ratios
            current_ratio: None,
            quick_ratio: None,

            // EPS
            trailing_eps: get_f64("trailingEps"),
            forward_eps: get_f64("forwardEps"),
            book_value: get_f64("bookValue"),

            // Company profile
            sector: get_str("sector"),
            sector_key: get_str("sectorKey"),
            sector_disp: get_str("sectorDisp"),
            industry: get_str("industry"),
            industry_key: get_str("industryKey"),
            industry_disp: get_str("industryDisp"),
            long_business_summary: None,
            website: None,
            ir_website: None,
            address1: None,
            city: None,
            state: None,
            zip: None,
            country: None,
            phone: None,
            full_time_employees: None,
            category: None,
            fund_family: None,

            // Risk scores
            audit_risk: None,
            board_risk: None,
            compensation_risk: None,
            shareholder_rights_risk: None,
            overall_risk: None,

            // Timezone
            time_zone_full_name: get_str("exchangeTimezoneName"),
            time_zone_short_name: get_str("exchangeTimezoneShortName"),
            gmt_off_set_milliseconds: json.get("gmtOffSetMilliseconds").and_then(|v| v.as_i64()),
            first_trade_date_epoch_utc: json
                .get("firstTradeDateMilliseconds")
                .and_then(|v| v.as_i64()),
            message_board_id: get_str("messageBoardId"),
            exchange_data_delayed_by: json
                .get("exchangeDataDelayedBy")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32),

            // Fund-specific
            nav_price: get_f64("navPrice"),
            total_assets: get_i64("totalAssets"),
            yield_value: get_f64("yield"),

            // Stock splits
            last_split_factor: None,
            last_split_date: None,
            last_fiscal_year_end: None,
            next_fiscal_year_end: None,
            most_recent_quarter: None,

            // Misc
            price_hint: get_i64("priceHint"),
            tradeable: json.get("tradeable").and_then(|v| v.as_bool()),
            financial_currency: get_str("financialCurrency"),

            // Nested objects not available in batch response
            company_officers: None,
            earnings: None,
            calendar_events: None,
            recommendation_trend: None,
            upgrade_downgrade_history: None,
            earnings_history: None,
            earnings_trend: None,
            insider_holders: None,
            insider_transactions: None,
            institution_ownership: None,
            fund_ownership: None,
            major_holders_breakdown: None,
            net_share_purchase_activity: None,
            sec_filings: None,
            balance_sheet_history: None,
            balance_sheet_history_quarterly: None,
            cashflow_statement_history: None,
            cashflow_statement_history_quarterly: None,
            income_statement_history: None,
            income_statement_history_quarterly: None,
            equity_performance: None,
            index_trend: None,
            industry_trend: None,
            sector_trend: None,
            fund_profile: None,
            fund_performance: None,
            top_holdings: None,
        })
    }
}
