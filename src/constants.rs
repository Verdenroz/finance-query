/// Yahoo Finance API base URLs
pub mod urls {
    /// Base URL for Yahoo Finance API (query1)
    pub const YAHOO_FINANCE_QUERY1: &str = "https://query1.finance.yahoo.com";

    /// Base URL for Yahoo Finance API (query2)
    pub const YAHOO_FINANCE_QUERY2: &str = "https://query2.finance.yahoo.com";

    /// Yahoo authentication/cookie page
    pub const YAHOO_FC: &str = "https://fc.yahoo.com";
}

/// Yahoo Finance API endpoints
pub mod endpoints {
    use super::urls::*;

    /// Get crumb token (query1)
    pub const CRUMB_QUERY1: &str =
        const_format::concatcp!(YAHOO_FINANCE_QUERY1, "/v1/test/getcrumb");

    /// Quote summary endpoint (detailed quote data)
    pub fn quote_summary(symbol: &str) -> String {
        format!(
            "{}/v10/finance/quoteSummary/{}",
            YAHOO_FINANCE_QUERY1, symbol
        )
    }

    /// Batch quotes endpoint - fetch multiple symbols in one request
    pub const QUOTES: &str = const_format::concatcp!(YAHOO_FINANCE_QUERY1, "/v7/finance/quote");

    /// Historical chart data endpoint
    #[allow(dead_code)]
    pub fn chart(symbol: &str) -> String {
        format!("{}/v8/finance/chart/{}", YAHOO_FINANCE_QUERY1, symbol)
    }

    /// Search endpoint
    #[allow(dead_code)]
    pub const SEARCH: &str = const_format::concatcp!(YAHOO_FINANCE_QUERY1, "/v1/finance/search");

    /// Financial timeseries endpoint (financials)
    #[allow(dead_code)]
    pub fn financials(symbol: &str) -> String {
        format!(
            "{}/ws/fundamentals-timeseries/v1/finance/timeseries/{}",
            YAHOO_FINANCE_QUERY1, symbol
        )
    }

    /// Recommendations endpoint (similar stocks)
    #[allow(dead_code)]
    pub fn recommendations(symbol: &str) -> String {
        format!(
            "{}/v6/finance/recommendationsbysymbol/{}",
            YAHOO_FINANCE_QUERY2, symbol
        )
    }

    /// Quote type endpoint (get quartr ID)
    #[allow(dead_code)]
    pub fn quote_type(symbol: &str) -> String {
        format!("{}/v1/finance/quoteType/{}", YAHOO_FINANCE_QUERY1, symbol)
    }

    /// News endpoint
    #[allow(dead_code)]
    pub const NEWS: &str = const_format::concatcp!(YAHOO_FINANCE_QUERY2, "/v2/finance/news");

    /// Options endpoint
    #[allow(dead_code)]
    pub fn options(symbol: &str) -> String {
        format!("{}/v7/finance/options/{}", YAHOO_FINANCE_QUERY2, symbol)
    }
}

/// URL builders (functions that construct full URLs with query params)
pub mod url_builders {
    use super::urls::*;

    /// Movers/screener endpoint
    pub fn movers(screener_id: &str, count: u32) -> String {
        format!(
            "{}/v1/finance/screener/predefined/saved?count={}&formatted=true&scrIds={}",
            YAHOO_FINANCE_QUERY1, count, screener_id
        )
    }
}

/// Screener IDs for market movers
pub mod screener_ids {
    /// Most active stocks by volume
    pub const MOST_ACTIVES: &str = "MOST_ACTIVES";
    /// Top gaining stocks by percentage
    pub const DAY_GAINERS: &str = "DAY_GAINERS";
    /// Top losing stocks by percentage
    pub const DAY_LOSERS: &str = "DAY_LOSERS";
}

/// Quote summary module names for the quoteSummary endpoint
pub mod quote_summary_modules {
    // Core modules
    /// Company profile information (officers, description, website, etc.)
    pub const ASSET_PROFILE: &str = "assetProfile";
    /// Company summary profile
    pub const SUMMARY_PROFILE: &str = "summaryProfile";
    /// Current price data (regular market, pre/post market prices)
    pub const PRICE: &str = "price";
    /// Summary detail information (market cap, P/E, dividend yield, etc.)
    pub const SUMMARY_DETAIL: &str = "summaryDetail";
    /// Key statistics (beta, shares outstanding, etc.)
    pub const DEFAULT_KEY_STATISTICS: &str = "defaultKeyStatistics";
    /// Calendar events (earnings dates, dividend dates)
    pub const CALENDAR_EVENTS: &str = "calendarEvents";
    /// Performance overview data
    pub const QUOTE_UNADJUSTED_PERFORMANCE: &str = "quoteUnadjustedPerformanceOverview";
    /// Equity performance metrics
    pub const EQUITY_PERFORMANCE: &str = "equityPerformance";

    // Analysis modules
    /// Analyst recommendation trend (buy/hold/sell ratings over time)
    pub const RECOMMENDATION_TREND: &str = "recommendationTrend";
    /// Analyst upgrade/downgrade history
    pub const UPGRADE_DOWNGRADE_HISTORY: &str = "upgradeDowngradeHistory";
    /// Financial data (price targets, profit margins, etc.)
    pub const FINANCIAL_DATA: &str = "financialData";
    /// Earnings and revenue estimates/trends
    pub const EARNINGS_TREND: &str = "earningsTrend";
    /// Historical earnings data
    pub const EARNINGS_HISTORY: &str = "earningsHistory";
    /// Base earnings data
    pub const EARNINGS: &str = "earnings";
    /// GAAP earnings data
    pub const EARNINGS_GAAP: &str = "earningsgaap";
    /// Non-GAAP earnings data
    pub const EARNINGS_NON_GAAP: &str = "earningsnongaap";
    /// Earnings call transcripts
    pub const EARNINGS_CALL_TRANSCRIPTS: &str = "earningsCallTranscripts";

    // ESG and sentiment
    /// Environmental, Social, Governance scores
    pub const ESG_SCORES: &str = "esgScores";

    // Financial statement modules
    /// Financial statement template/structure
    pub const FINANCIALS_TEMPLATE: &str = "financialsTemplate";

    // Holders modules
    /// Major holders breakdown (% held by institutions, insiders, etc.)
    pub const MAJOR_HOLDERS_BREAKDOWN: &str = "majorHoldersBreakdown";
    /// Institutional ownership details
    pub const INSTITUTION_OWNERSHIP: &str = "institutionOwnership";
    /// Mutual fund ownership details
    pub const FUND_OWNERSHIP: &str = "fundOwnership";
    /// Insider transactions history
    pub const INSIDER_TRANSACTIONS: &str = "insiderTransactions";
    /// Net share purchase activity by insiders
    pub const NET_SHARE_PURCHASE_ACTIVITY: &str = "netSharePurchaseActivity";
    /// Insider holders roster
    pub const INSIDER_HOLDERS: &str = "insiderHolders";
}

/// Fundamental timeseries field types for financial statements
///
/// These constants represent field names that must be prefixed with frequency ("annual" or "quarterly")
/// Example: "annualTotalRevenue", "quarterlyTotalRevenue"
#[allow(missing_docs)]
pub mod fundamental_types {
    // ==================
    // INCOME STATEMENT (48 fields)
    // ==================
    pub const TOTAL_REVENUE: &str = "TotalRevenue";
    pub const OPERATING_REVENUE: &str = "OperatingRevenue";
    pub const COST_OF_REVENUE: &str = "CostOfRevenue";
    pub const GROSS_PROFIT: &str = "GrossProfit";
    pub const OPERATING_EXPENSE: &str = "OperatingExpense";
    pub const SELLING_GENERAL_AND_ADMIN: &str = "SellingGeneralAndAdministration";
    pub const RESEARCH_AND_DEVELOPMENT: &str = "ResearchAndDevelopment";
    pub const OPERATING_INCOME: &str = "OperatingIncome";
    pub const NET_INTEREST_INCOME: &str = "NetInterestIncome";
    pub const INTEREST_EXPENSE: &str = "InterestExpense";
    pub const INTEREST_INCOME: &str = "InterestIncome";
    pub const NET_NON_OPERATING_INTEREST_INCOME_EXPENSE: &str =
        "NetNonOperatingInterestIncomeExpense";
    pub const OTHER_INCOME_EXPENSE: &str = "OtherIncomeExpense";
    pub const PRETAX_INCOME: &str = "PretaxIncome";
    pub const TAX_PROVISION: &str = "TaxProvision";
    pub const NET_INCOME_COMMON_STOCKHOLDERS: &str = "NetIncomeCommonStockholders";
    pub const NET_INCOME: &str = "NetIncome";
    pub const DILUTED_EPS: &str = "DilutedEPS";
    pub const BASIC_EPS: &str = "BasicEPS";
    pub const DILUTED_AVERAGE_SHARES: &str = "DilutedAverageShares";
    pub const BASIC_AVERAGE_SHARES: &str = "BasicAverageShares";
    pub const EBIT: &str = "EBIT";
    pub const EBITDA: &str = "EBITDA";
    pub const RECONCILED_COST_OF_REVENUE: &str = "ReconciledCostOfRevenue";
    pub const RECONCILED_DEPRECIATION: &str = "ReconciledDepreciation";
    pub const NET_INCOME_FROM_CONTINUING_OPERATION_NET_MINORITY_INTEREST: &str =
        "NetIncomeFromContinuingOperationNetMinorityInterest";
    pub const NORMALIZED_EBITDA: &str = "NormalizedEBITDA";
    pub const TOTAL_EXPENSES: &str = "TotalExpenses";
    pub const TOTAL_OPERATING_INCOME_AS_REPORTED: &str = "TotalOperatingIncomeAsReported";
    pub const DILUTED_NI_AVAILTO_COM_STOCKHOLDERS: &str = "DilutedNIAvailtoComStockholders";
    pub const NET_INCOME_FROM_CONTINUING_AND_DISCONTINUED_OPERATION: &str =
        "NetIncomeFromContinuingAndDiscontinuedOperation";
    pub const NORMALIZED_INCOME: &str = "NormalizedIncome";
    pub const INTEREST_INCOME_NON_OPERATING: &str = "InterestIncomeNonOperating";
    pub const INTEREST_EXPENSE_NON_OPERATING: &str = "InterestExpenseNonOperating";
    pub const NET_INCOME_CONTINUOUS_OPERATIONS: &str = "NetIncomeContinuousOperations";
    pub const TAX_RATE_FOR_CALCS: &str = "TaxRateForCalcs";
    pub const TAX_EFFECT_OF_UNUSUAL_ITEMS: &str = "TaxEffectOfUnusualItems";
    pub const TAX_PROVISION_AS_REPORTED: &str = "TaxProvisionAsReported";
    pub const OTHER_NON_OPERATING_INCOME_EXPENSES: &str = "OtherNonOperatingIncomeExpenses";
    pub const OTHER_OPERATING_EXPENSES: &str = "OtherOperatingExpenses";
    pub const OTHER_TAXES: &str = "OtherTaxes";
    pub const PROVISION_FOR_DOUBTFUL_ACCOUNTS: &str = "ProvisionForDoubtfulAccounts";
    pub const DEPRECIATION_AMORTIZATION_DEPLETION_INCOME_STATEMENT: &str =
        "DepreciationAmortizationDepletionIncomeStatement";
    pub const DEPRECIATION_AND_AMORTIZATION_IN_INCOME_STATEMENT: &str =
        "DepreciationAndAmortizationInIncomeStatement";
    pub const DEPRECIATION: &str = "Depreciation";
    pub const AMORTIZATION_OF_INTANGIBLES_INCOME_STATEMENT: &str =
        "AmortizationOfIntangiblesIncomeStatement";
    pub const AMORTIZATION: &str = "Amortization";

    // ==================
    // BALANCE SHEET (42 fields)
    // ==================
    pub const TOTAL_ASSETS: &str = "TotalAssets";
    pub const CURRENT_ASSETS: &str = "CurrentAssets";
    pub const CASH_CASH_EQUIVALENTS_AND_SHORT_TERM_INVESTMENTS: &str =
        "CashCashEquivalentsAndShortTermInvestments";
    pub const CASH_AND_CASH_EQUIVALENTS: &str = "CashAndCashEquivalents";
    pub const CASH_FINANCIAL: &str = "CashFinancial";
    pub const RECEIVABLES: &str = "Receivables";
    pub const ACCOUNTS_RECEIVABLE: &str = "AccountsReceivable";
    pub const INVENTORY: &str = "Inventory";
    pub const PREPAID_ASSETS: &str = "PrepaidAssets";
    pub const OTHER_CURRENT_ASSETS: &str = "OtherCurrentAssets";
    pub const TOTAL_NON_CURRENT_ASSETS: &str = "TotalNonCurrentAssets";
    pub const NET_PPE: &str = "NetPPE";
    pub const GROSS_PPE: &str = "GrossPPE";
    pub const ACCUMULATED_DEPRECIATION: &str = "AccumulatedDepreciation";
    pub const GOODWILL: &str = "Goodwill";
    pub const GOODWILL_AND_OTHER_INTANGIBLE_ASSETS: &str = "GoodwillAndOtherIntangibleAssets";
    pub const OTHER_INTANGIBLE_ASSETS: &str = "OtherIntangibleAssets";
    pub const INVESTMENTS_AND_ADVANCES: &str = "InvestmentsAndAdvances";
    pub const LONG_TERM_EQUITY_INVESTMENT: &str = "LongTermEquityInvestment";
    pub const OTHER_NON_CURRENT_ASSETS: &str = "OtherNonCurrentAssets";
    pub const TOTAL_LIABILITIES_NET_MINORITY_INTEREST: &str = "TotalLiabilitiesNetMinorityInterest";
    pub const CURRENT_LIABILITIES: &str = "CurrentLiabilities";
    pub const PAYABLES_AND_ACCRUED_EXPENSES: &str = "PayablesAndAccruedExpenses";
    pub const ACCOUNTS_PAYABLE: &str = "AccountsPayable";
    pub const CURRENT_DEBT: &str = "CurrentDebt";
    pub const CURRENT_DEFERRED_REVENUE: &str = "CurrentDeferredRevenue";
    pub const OTHER_CURRENT_LIABILITIES: &str = "OtherCurrentLiabilities";
    pub const TOTAL_NON_CURRENT_LIABILITIES_NET_MINORITY_INTEREST: &str =
        "TotalNonCurrentLiabilitiesNetMinorityInterest";
    pub const LONG_TERM_DEBT: &str = "LongTermDebt";
    pub const LONG_TERM_DEBT_AND_CAPITAL_LEASE_OBLIGATION: &str =
        "LongTermDebtAndCapitalLeaseObligation";
    pub const NON_CURRENT_DEFERRED_REVENUE: &str = "NonCurrentDeferredRevenue";
    pub const NON_CURRENT_DEFERRED_TAXES_LIABILITIES: &str = "NonCurrentDeferredTaxesLiabilities";
    pub const OTHER_NON_CURRENT_LIABILITIES: &str = "OtherNonCurrentLiabilities";
    pub const STOCKHOLDERS_EQUITY: &str = "StockholdersEquity";
    pub const COMMON_STOCK_EQUITY: &str = "CommonStockEquity";
    pub const COMMON_STOCK: &str = "CommonStock";
    pub const RETAINED_EARNINGS: &str = "RetainedEarnings";
    pub const ADDITIONAL_PAID_IN_CAPITAL: &str = "AdditionalPaidInCapital";
    pub const TREASURY_STOCK: &str = "TreasuryStock";
    pub const TOTAL_EQUITY_GROSS_MINORITY_INTEREST: &str = "TotalEquityGrossMinorityInterest";
    pub const WORKING_CAPITAL: &str = "WorkingCapital";
    pub const INVESTED_CAPITAL: &str = "InvestedCapital";
    pub const TANGIBLE_BOOK_VALUE: &str = "TangibleBookValue";
    pub const TOTAL_DEBT: &str = "TotalDebt";
    pub const NET_DEBT: &str = "NetDebt";
    pub const SHARE_ISSUED: &str = "ShareIssued";
    pub const ORDINARY_SHARES_NUMBER: &str = "OrdinarySharesNumber";

    // ==================
    // CASH FLOW STATEMENT (48 fields)
    // ==================
    pub const OPERATING_CASH_FLOW: &str = "OperatingCashFlow";
    pub const CASH_FLOW_FROM_CONTINUING_OPERATING_ACTIVITIES: &str =
        "CashFlowFromContinuingOperatingActivities";
    pub const NET_INCOME_FROM_CONTINUING_OPERATIONS: &str = "NetIncomeFromContinuingOperations";
    pub const DEPRECIATION_AND_AMORTIZATION: &str = "DepreciationAndAmortization";
    pub const DEFERRED_INCOME_TAX: &str = "DeferredIncomeTax";
    pub const CHANGE_IN_WORKING_CAPITAL: &str = "ChangeInWorkingCapital";
    pub const CHANGE_IN_RECEIVABLES: &str = "ChangeInReceivables";
    pub const CHANGES_IN_ACCOUNT_RECEIVABLES: &str = "ChangesInAccountReceivables";
    pub const CHANGE_IN_INVENTORY: &str = "ChangeInInventory";
    pub const CHANGE_IN_ACCOUNT_PAYABLE: &str = "ChangeInAccountPayable";
    pub const CHANGE_IN_OTHER_WORKING_CAPITAL: &str = "ChangeInOtherWorkingCapital";
    pub const STOCK_BASED_COMPENSATION: &str = "StockBasedCompensation";
    pub const OTHER_NON_CASH_ITEMS: &str = "OtherNonCashItems";
    pub const INVESTING_CASH_FLOW: &str = "InvestingCashFlow";
    pub const CASH_FLOW_FROM_CONTINUING_INVESTING_ACTIVITIES: &str =
        "CashFlowFromContinuingInvestingActivities";
    pub const NET_PPE_PURCHASE_AND_SALE: &str = "NetPPEPurchaseAndSale";
    pub const PURCHASE_OF_PPE: &str = "PurchaseOfPPE";
    pub const SALE_OF_PPE: &str = "SaleOfPPE";
    pub const CAPITAL_EXPENDITURE: &str = "CapitalExpenditure";
    pub const NET_BUSINESS_PURCHASE_AND_SALE: &str = "NetBusinessPurchaseAndSale";
    pub const PURCHASE_OF_BUSINESS: &str = "PurchaseOfBusiness";
    pub const SALE_OF_BUSINESS: &str = "SaleOfBusiness";
    pub const NET_INVESTMENT_PURCHASE_AND_SALE: &str = "NetInvestmentPurchaseAndSale";
    pub const PURCHASE_OF_INVESTMENT: &str = "PurchaseOfInvestment";
    pub const SALE_OF_INVESTMENT: &str = "SaleOfInvestment";
    pub const NET_OTHER_INVESTING_CHANGES: &str = "NetOtherInvestingChanges";
    pub const FINANCING_CASH_FLOW: &str = "FinancingCashFlow";
    pub const CASH_FLOW_FROM_CONTINUING_FINANCING_ACTIVITIES: &str =
        "CashFlowFromContinuingFinancingActivities";
    pub const NET_ISSUANCE_PAYMENTS_OF_DEBT: &str = "NetIssuancePaymentsOfDebt";
    pub const NET_LONG_TERM_DEBT_ISSUANCE: &str = "NetLongTermDebtIssuance";
    pub const LONG_TERM_DEBT_ISSUANCE: &str = "LongTermDebtIssuance";
    pub const LONG_TERM_DEBT_PAYMENTS: &str = "LongTermDebtPayments";
    pub const NET_SHORT_TERM_DEBT_ISSUANCE: &str = "NetShortTermDebtIssuance";
    pub const NET_COMMON_STOCK_ISSUANCE: &str = "NetCommonStockIssuance";
    pub const COMMON_STOCK_ISSUANCE: &str = "CommonStockIssuance";
    pub const COMMON_STOCK_PAYMENTS: &str = "CommonStockPayments";
    pub const REPURCHASE_OF_CAPITAL_STOCK: &str = "RepurchaseOfCapitalStock";
    pub const CASH_DIVIDENDS_PAID: &str = "CashDividendsPaid";
    pub const COMMON_STOCK_DIVIDEND_PAID: &str = "CommonStockDividendPaid";
    pub const NET_OTHER_FINANCING_CHARGES: &str = "NetOtherFinancingCharges";
    pub const END_CASH_POSITION: &str = "EndCashPosition";
    pub const BEGINNING_CASH_POSITION: &str = "BeginningCashPosition";
    pub const CHANGESIN_CASH: &str = "ChangesinCash";
    pub const EFFECT_OF_EXCHANGE_RATE_CHANGES: &str = "EffectOfExchangeRateChanges";
    pub const FREE_CASH_FLOW: &str = "FreeCashFlow";
    pub const CAPITAL_EXPENDITURE_REPORTED: &str = "CapitalExpenditureReported";
}

/// Industry sector classifications
#[allow(missing_docs)]
pub mod sectors {
    pub const BASIC_MATERIALS: &str = "Basic Materials";
    pub const COMMUNICATION_SERVICES: &str = "Communication Services";
    pub const CONSUMER_CYCLICAL: &str = "Consumer Cyclical";
    pub const CONSUMER_DEFENSIVE: &str = "Consumer Defensive";
    pub const ENERGY: &str = "Energy";
    pub const FINANCIAL_SERVICES: &str = "Financial Services";
    pub const HEALTHCARE: &str = "Healthcare";
    pub const INDUSTRIALS: &str = "Industrials";
    pub const REAL_ESTATE: &str = "Real Estate";
    pub const TECHNOLOGY: &str = "Technology";
    pub const UTILITIES: &str = "Utilities";
}

/// Statement types for financial data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatementType {
    /// Income statement
    Income,
    /// Balance sheet
    Balance,
    /// Cash flow statement
    CashFlow,
}

impl StatementType {
    /// Convert statement type to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            StatementType::Income => "income",
            StatementType::Balance => "balance",
            StatementType::CashFlow => "cashflow",
        }
    }
}

/// Frequency for financial data (annual or quarterly)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Frequency {
    /// Annual financial data
    Annual,
    /// Quarterly financial data
    Quarterly,
}

impl Frequency {
    /// Convert frequency to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Frequency::Annual => "annual",
            Frequency::Quarterly => "quarterly",
        }
    }

    /// Build a fundamental type string with frequency prefix
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::constants::{Frequency, fundamental_types};
    ///
    /// let field = Frequency::Annual.prefix(fundamental_types::TOTAL_REVENUE);
    /// assert_eq!(field, "annualTotalRevenue");
    /// ```
    pub fn prefix(&self, field: &str) -> String {
        format!("{}{}", self.as_str(), field)
    }
}

/// HTTP headers
pub mod headers {
    /// User agent to use for requests (Chrome on Windows)
    pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

    /// Accept header
    #[allow(dead_code)]
    pub const ACCEPT: &str = "*/*";

    /// Accept language
    #[allow(dead_code)]
    pub const ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9";

    /// Accept encoding
    #[allow(dead_code)]
    pub const ACCEPT_ENCODING: &str = "gzip, deflate, br";
}

/// Chart intervals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum Interval {
    /// 1 minute
    OneMinute,
    /// 5 minutes
    FiveMinutes,
    /// 15 minutes
    FifteenMinutes,
    /// 30 minutes
    ThirtyMinutes,
    /// 1 hour
    OneHour,
    /// 1 day
    OneDay,
    /// 1 week
    OneWeek,
    /// 1 month
    OneMonth,
    /// 3 months
    ThreeMonths,
}

#[allow(dead_code)]
impl Interval {
    /// Convert interval to Yahoo Finance API format
    pub fn as_str(&self) -> &'static str {
        match self {
            Interval::OneMinute => "1m",
            Interval::FiveMinutes => "5m",
            Interval::FifteenMinutes => "15m",
            Interval::ThirtyMinutes => "30m",
            Interval::OneHour => "1h",
            Interval::OneDay => "1d",
            Interval::OneWeek => "1wk",
            Interval::OneMonth => "1mo",
            Interval::ThreeMonths => "3mo",
        }
    }
}

/// Time ranges for chart data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum TimeRange {
    /// 1 day
    OneDay,
    /// 5 days
    FiveDays,
    /// 1 month
    OneMonth,
    /// 3 months
    ThreeMonths,
    /// 6 months
    SixMonths,
    /// 1 year
    OneYear,
    /// 2 years
    TwoYears,
    /// 5 years
    FiveYears,
    /// 10 years
    TenYears,
    /// Year to date
    YearToDate,
    /// Maximum available
    Max,
}

#[allow(dead_code)]
impl TimeRange {
    /// Convert time range to Yahoo Finance API format
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeRange::OneDay => "1d",
            TimeRange::FiveDays => "5d",
            TimeRange::OneMonth => "1mo",
            TimeRange::ThreeMonths => "3mo",
            TimeRange::SixMonths => "6mo",
            TimeRange::OneYear => "1y",
            TimeRange::TwoYears => "2y",
            TimeRange::FiveYears => "5y",
            TimeRange::TenYears => "10y",
            TimeRange::YearToDate => "ytd",
            TimeRange::Max => "max",
        }
    }
}

/// Supported countries for Yahoo Finance regional APIs
///
/// Each country has predefined language and region codes that work together.
/// Using the Country enum ensures correct lang/region pairing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Country {
    /// France (fr-FR, FR)
    France,
    /// India (en-IN, IN)
    India,
    /// Hong Kong (zh-Hant-HK, HK)
    HongKong,
    /// Germany (de-DE, DE)
    Germany,
    /// Canada (en-CA, CA)
    Canada,
    /// Spain (es-ES, ES)
    Spain,
    /// Italy (it-IT, IT)
    Italy,
    /// United States (en-US, US) - Default
    #[default]
    UnitedStates,
    /// Australia (en-AU, AU)
    Australia,
    /// United Kingdom (en-GB, GB)
    UnitedKingdom,
    /// Brazil (pt-BR, BR)
    Brazil,
    /// New Zealand (en-NZ, NZ)
    NewZealand,
    /// Singapore (en-SG, SG)
    Singapore,
    /// Taiwan (zh-tw, TW)
    Taiwan,
}

impl Country {
    /// Get the language code for this country
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::Country;
    ///
    /// assert_eq!(Country::Japan.lang(), "ja-JP");
    /// assert_eq!(Country::UnitedStates.lang(), "en-US");
    /// ```
    pub fn lang(&self) -> &'static str {
        match self {
            Country::France => "fr-FR",
            Country::India => "en-IN",
            Country::HongKong => "zh-Hant-HK",
            Country::Germany => "de-DE",
            Country::Canada => "en-CA",
            Country::Spain => "es-ES",
            Country::Italy => "it-IT",
            Country::UnitedStates => "en-US",
            Country::Australia => "en-AU",
            Country::UnitedKingdom => "en-GB",
            Country::Brazil => "pt-BR",
            Country::NewZealand => "en-NZ",
            Country::Singapore => "en-SG",
            Country::Taiwan => "zh-tw",
        }
    }

    /// Get the region code for this country
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::Country;
    ///
    /// assert_eq!(Country::Japan.region(), "JP");
    /// assert_eq!(Country::UnitedStates.region(), "US");
    /// ```
    pub fn region(&self) -> &'static str {
        match self {
            Country::France => "FR",
            Country::India => "IN",
            Country::HongKong => "HK",
            Country::Germany => "DE",
            Country::Canada => "CA",
            Country::Spain => "ES",
            Country::Italy => "IT",
            Country::UnitedStates => "US",
            Country::Australia => "AU",
            Country::UnitedKingdom => "GB",
            Country::Brazil => "BR",
            Country::NewZealand => "NZ",
            Country::Singapore => "SG",
            Country::Taiwan => "TW",
        }
    }

    /// Get the CORS domain for this country
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::Country;
    ///
    /// assert_eq!(Country::UnitedStates.cors_domain(), "finance.yahoo.com");
    /// assert_eq!(Country::Japan.cors_domain(), "finance.yahoo.co.jp");
    /// ```
    pub fn cors_domain(&self) -> &'static str {
        match self {
            Country::France => "fr.finance.yahoo.com",
            Country::India => "in.finance.yahoo.com",
            Country::HongKong => "hk.finance.yahoo.com",
            Country::Germany => "de.finance.yahoo.com",
            Country::Canada => "ca.finance.yahoo.com",
            Country::Spain => "es.finance.yahoo.com",
            Country::Italy => "it.finance.yahoo.com",
            Country::UnitedStates => "finance.yahoo.com",
            Country::Australia => "au.finance.yahoo.com",
            Country::UnitedKingdom => "uk.finance.yahoo.com",
            Country::Brazil => "br.financas.yahoo.com",
            Country::NewZealand => "nz.finance.yahoo.com",
            Country::Singapore => "sg.finance.yahoo.com",
            Country::Taiwan => "tw.finance.yahoo.com",
        }
    }
}

/// Authentication constants
pub mod auth {
    use std::time::Duration;

    /// Minimum interval between auth refreshes (prevent excessive refreshing)
    #[allow(dead_code)]
    pub const MIN_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

    /// Maximum age of auth before considering it stale
    #[allow(dead_code)]
    pub const AUTH_MAX_AGE: Duration = Duration::from_secs(3600); // 1 hour
}

/// Default timeouts
pub mod timeouts {
    use std::time::Duration;

    /// Default HTTP request timeout
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

    /// Timeout for authentication requests
    pub const AUTH_TIMEOUT: Duration = Duration::from_secs(15);
}

/// Default values for API endpoints
pub mod defaults {
    /// Default number of similar stocks to return
    pub const SIMILAR_STOCKS_LIMIT: u32 = 5;

    /// Default number of search results
    pub const SEARCH_HITS: u32 = 6;

    /// Default server port
    pub const SERVER_PORT: u16 = 8000;

    /// Default number of news articles to return
    pub const NEWS_COUNT: u32 = 10;

    /// Default chart interval
    pub const DEFAULT_INTERVAL: &str = "1d";

    /// Default chart range
    pub const DEFAULT_RANGE: &str = "1mo";

    /// Default start period for timeseries (Unix timestamp)
    /// 0 = earliest available data
    pub const DEFAULT_PERIOD1: i64 = 0;

    /// Default end period for timeseries (Unix timestamp)
    /// 9999999999 = far future (essentially "now" for Yahoo Finance)
    pub const DEFAULT_PERIOD2: i64 = 9999999999;
}

/// API request parameters used across endpoints
pub mod api_params {
    /// CORS domain parameter value for Yahoo Finance
    pub const CORS_DOMAIN: &str = "finance.yahoo.com";

    /// Formatted parameter - disable formatting for raw data
    pub const FORMATTED: &str = "false";

    /// Merge parameter for timeseries - don't merge data
    pub const MERGE: &str = "false";

    /// Pad timeseries - fill gaps in data
    pub const PAD_TIMESERIES: &str = "true";

    /// Default language for API requests
    pub const DEFAULT_LANG: &str = "en-US";

    /// Default region for API requests
    pub const DEFAULT_REGION: &str = "US";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_as_str() {
        assert_eq!(Interval::OneMinute.as_str(), "1m");
        assert_eq!(Interval::FiveMinutes.as_str(), "5m");
        assert_eq!(Interval::OneDay.as_str(), "1d");
        assert_eq!(Interval::OneWeek.as_str(), "1wk");
    }

    #[test]
    fn test_time_range_as_str() {
        assert_eq!(TimeRange::OneDay.as_str(), "1d");
        assert_eq!(TimeRange::OneMonth.as_str(), "1mo");
        assert_eq!(TimeRange::OneYear.as_str(), "1y");
        assert_eq!(TimeRange::Max.as_str(), "max");
    }

    #[test]
    fn test_endpoint_construction() {
        assert_eq!(
            endpoints::chart("AAPL"),
            "https://query1.finance.yahoo.com/v8/finance/chart/AAPL"
        );
        assert_eq!(
            endpoints::quote_summary("NVDA"),
            "https://query2.finance.yahoo.com/v10/finance/quoteSummary/NVDA"
        );
    }
}
