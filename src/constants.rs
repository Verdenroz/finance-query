use serde::{Deserialize, Serialize};

/// Predefined screener selectors for Yahoo Finance
pub mod screeners {
    /// Predefined Yahoo Finance screener selector
    ///
    /// Passed to `finance::screener()` or `client.get_screener()` to select one of the
    /// 15 built-in Yahoo Finance screeners (equity or fund).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Screener {
        // Equity screeners
        /// Small caps with high EPS growth, sorted by volume
        AggressiveSmallCaps,
        /// Top gaining stocks (>3% change, >$2B market cap)
        DayGainers,
        /// Top losing stocks (<-2.5% change, >$2B market cap)
        DayLosers,
        /// Tech stocks with 25%+ revenue and EPS growth
        GrowthTechnologyStocks,
        /// Most actively traded stocks by volume
        MostActives,
        /// Stocks with highest short interest percentage
        MostShortedStocks,
        /// Small cap gainers (<$2B market cap)
        SmallCapGainers,
        /// Low P/E (<20), low PEG (<1), high EPS growth (25%+)
        UndervaluedGrowthStocks,
        /// Large caps ($10B-$100B) with low P/E and PEG
        UndervaluedLargeCaps,
        // Fund screeners
        /// Low-risk foreign large cap funds (4-5 star rated)
        ConservativeForeignFunds,
        /// High yield bond funds (4-5 star rated)
        HighYieldBond,
        /// Large blend core funds (4-5 star rated)
        PortfolioAnchors,
        /// Large growth funds (4-5 star rated)
        SolidLargeGrowthFunds,
        /// Mid-cap growth funds (4-5 star rated)
        SolidMidcapGrowthFunds,
        /// Top performing mutual funds by percent change
        TopMutualFunds,
    }

    impl Screener {
        /// Convert to Yahoo Finance scrId parameter value (SCREAMING_SNAKE_CASE)
        pub fn as_scr_id(&self) -> &'static str {
            match self {
                Screener::AggressiveSmallCaps => "aggressive_small_caps",
                Screener::DayGainers => "day_gainers",
                Screener::DayLosers => "day_losers",
                Screener::GrowthTechnologyStocks => "growth_technology_stocks",
                Screener::MostActives => "most_actives",
                Screener::MostShortedStocks => "most_shorted_stocks",
                Screener::SmallCapGainers => "small_cap_gainers",
                Screener::UndervaluedGrowthStocks => "undervalued_growth_stocks",
                Screener::UndervaluedLargeCaps => "undervalued_large_caps",
                Screener::ConservativeForeignFunds => "conservative_foreign_funds",
                Screener::HighYieldBond => "high_yield_bond",
                Screener::PortfolioAnchors => "portfolio_anchors",
                Screener::SolidLargeGrowthFunds => "solid_large_growth_funds",
                Screener::SolidMidcapGrowthFunds => "solid_midcap_growth_funds",
                Screener::TopMutualFunds => "top_mutual_funds",
            }
        }

        /// Parse from string, returns None on invalid input
        ///
        /// # Example
        /// ```
        /// use finance_query::Screener;
        ///
        /// assert_eq!(Screener::parse("most-actives"), Some(Screener::MostActives));
        /// assert_eq!(Screener::parse("day-gainers"), Some(Screener::DayGainers));
        /// ```
        pub fn parse(s: &str) -> Option<Self> {
            s.parse().ok()
        }

        /// List all valid screener types for error messages
        pub fn valid_types() -> &'static str {
            "aggressive-small-caps, day-gainers, day-losers, growth-technology-stocks, \
             most-actives, most-shorted-stocks, small-cap-gainers, undervalued-growth-stocks, \
             undervalued-large-caps, conservative-foreign-funds, high-yield-bond, \
             portfolio-anchors, solid-large-growth-funds, solid-midcap-growth-funds, \
             top-mutual-funds"
        }

        /// Get all screener types as an array
        pub fn all() -> &'static [Screener] {
            &[
                Screener::AggressiveSmallCaps,
                Screener::DayGainers,
                Screener::DayLosers,
                Screener::GrowthTechnologyStocks,
                Screener::MostActives,
                Screener::MostShortedStocks,
                Screener::SmallCapGainers,
                Screener::UndervaluedGrowthStocks,
                Screener::UndervaluedLargeCaps,
                Screener::ConservativeForeignFunds,
                Screener::HighYieldBond,
                Screener::PortfolioAnchors,
                Screener::SolidLargeGrowthFunds,
                Screener::SolidMidcapGrowthFunds,
                Screener::TopMutualFunds,
            ]
        }
    }

    impl std::str::FromStr for Screener {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_lowercase().replace('_', "-").as_str() {
                "aggressive-small-caps" => Ok(Screener::AggressiveSmallCaps),
                "day-gainers" | "gainers" => Ok(Screener::DayGainers),
                "day-losers" | "losers" => Ok(Screener::DayLosers),
                "growth-technology-stocks" | "growth-tech" => Ok(Screener::GrowthTechnologyStocks),
                "most-actives" | "actives" => Ok(Screener::MostActives),
                "most-shorted-stocks" | "most-shorted" => Ok(Screener::MostShortedStocks),
                "small-cap-gainers" => Ok(Screener::SmallCapGainers),
                "undervalued-growth-stocks" | "undervalued-growth" => {
                    Ok(Screener::UndervaluedGrowthStocks)
                }
                "undervalued-large-caps" | "undervalued-large" => {
                    Ok(Screener::UndervaluedLargeCaps)
                }
                "conservative-foreign-funds" => Ok(Screener::ConservativeForeignFunds),
                "high-yield-bond" => Ok(Screener::HighYieldBond),
                "portfolio-anchors" => Ok(Screener::PortfolioAnchors),
                "solid-large-growth-funds" => Ok(Screener::SolidLargeGrowthFunds),
                "solid-midcap-growth-funds" => Ok(Screener::SolidMidcapGrowthFunds),
                "top-mutual-funds" => Ok(Screener::TopMutualFunds),
                _ => Err(()),
            }
        }
    }
}

/// Yahoo Finance sector types
///
/// These are the 11 GICS sectors available on Yahoo Finance.
pub mod sectors {
    use serde::{Deserialize, Serialize};

    /// Market sector types available on Yahoo Finance
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum Sector {
        /// Technology sector (software, semiconductors, hardware)
        Technology,
        /// Financial Services sector (banks, insurance, asset management)
        FinancialServices,
        /// Consumer Cyclical sector (retail, automotive, leisure)
        ConsumerCyclical,
        /// Communication Services sector (telecom, media, entertainment)
        CommunicationServices,
        /// Healthcare sector (pharma, biotech, medical devices)
        Healthcare,
        /// Industrials sector (aerospace, machinery, construction)
        Industrials,
        /// Consumer Defensive sector (food, beverages, household products)
        ConsumerDefensive,
        /// Energy sector (oil, gas, renewable energy)
        Energy,
        /// Basic Materials sector (chemicals, metals, mining)
        BasicMaterials,
        /// Real Estate sector (REITs, property management)
        RealEstate,
        /// Utilities sector (electric, gas, water utilities)
        Utilities,
    }

    impl Sector {
        /// Convert to Yahoo Finance API path segment (lowercase with hyphens)
        pub fn as_api_path(&self) -> &'static str {
            match self {
                Sector::Technology => "technology",
                Sector::FinancialServices => "financial-services",
                Sector::ConsumerCyclical => "consumer-cyclical",
                Sector::CommunicationServices => "communication-services",
                Sector::Healthcare => "healthcare",
                Sector::Industrials => "industrials",
                Sector::ConsumerDefensive => "consumer-defensive",
                Sector::Energy => "energy",
                Sector::BasicMaterials => "basic-materials",
                Sector::RealEstate => "real-estate",
                Sector::Utilities => "utilities",
            }
        }

        /// Get human-readable display name
        pub fn display_name(&self) -> &'static str {
            match self {
                Sector::Technology => "Technology",
                Sector::FinancialServices => "Financial Services",
                Sector::ConsumerCyclical => "Consumer Cyclical",
                Sector::CommunicationServices => "Communication Services",
                Sector::Healthcare => "Healthcare",
                Sector::Industrials => "Industrials",
                Sector::ConsumerDefensive => "Consumer Defensive",
                Sector::Energy => "Energy",
                Sector::BasicMaterials => "Basic Materials",
                Sector::RealEstate => "Real Estate",
                Sector::Utilities => "Utilities",
            }
        }

        /// List all valid sector types for error messages
        pub fn valid_types() -> &'static str {
            "technology, financial-services, consumer-cyclical, communication-services, \
             healthcare, industrials, consumer-defensive, energy, basic-materials, \
             real-estate, utilities"
        }

        /// Get all sector types as an array
        pub fn all() -> &'static [Sector] {
            &[
                Sector::Technology,
                Sector::FinancialServices,
                Sector::ConsumerCyclical,
                Sector::CommunicationServices,
                Sector::Healthcare,
                Sector::Industrials,
                Sector::ConsumerDefensive,
                Sector::Energy,
                Sector::BasicMaterials,
                Sector::RealEstate,
                Sector::Utilities,
            ]
        }
    }

    impl std::str::FromStr for Sector {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_lowercase().replace('_', "-").as_str() {
                "technology" | "tech" => Ok(Sector::Technology),
                "financial-services" | "financials" | "financial" => Ok(Sector::FinancialServices),
                "consumer-cyclical" => Ok(Sector::ConsumerCyclical),
                "communication-services" | "communication" => Ok(Sector::CommunicationServices),
                "healthcare" | "health" => Ok(Sector::Healthcare),
                "industrials" | "industrial" => Ok(Sector::Industrials),
                "consumer-defensive" => Ok(Sector::ConsumerDefensive),
                "energy" => Ok(Sector::Energy),
                "basic-materials" | "materials" => Ok(Sector::BasicMaterials),
                "real-estate" | "realestate" => Ok(Sector::RealEstate),
                "utilities" | "utility" => Ok(Sector::Utilities),
                _ => Err(()),
            }
        }
    }

    impl std::fmt::Display for Sector {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.display_name())
        }
    }

    impl From<Sector> for String {
        /// Returns the display name used by Yahoo Finance screener (e.g. `"Technology"`).
        fn from(v: Sector) -> Self {
            v.display_name().to_string()
        }
    }
}

/// World market indices
pub mod indices {
    /// Region categories for world indices
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Region {
        /// North and South America
        Americas,
        /// European markets
        Europe,
        /// Asia and Pacific markets
        AsiaPacific,
        /// Middle East and Africa
        MiddleEastAfrica,
        /// Currency indices
        Currencies,
    }

    impl std::str::FromStr for Region {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_lowercase().replace(['-', '_'], "").as_str() {
                "americas" | "america" => Ok(Region::Americas),
                "europe" | "eu" => Ok(Region::Europe),
                "asiapacific" | "asia" | "apac" => Ok(Region::AsiaPacific),
                "middleeastafrica" | "mea" | "emea" => Ok(Region::MiddleEastAfrica),
                "currencies" | "currency" | "fx" => Ok(Region::Currencies),
                _ => Err(()),
            }
        }
    }

    impl Region {
        /// Parse from string, returns None on invalid input
        pub fn parse(s: &str) -> Option<Self> {
            s.parse().ok()
        }

        /// Get the symbols for this region
        pub fn symbols(&self) -> &'static [&'static str] {
            match self {
                Region::Americas => AMERICAS,
                Region::Europe => EUROPE,
                Region::AsiaPacific => ASIA_PACIFIC,
                Region::MiddleEastAfrica => MIDDLE_EAST_AFRICA,
                Region::Currencies => CURRENCIES,
            }
        }

        /// Convert to string representation
        pub fn as_str(&self) -> &'static str {
            match self {
                Region::Americas => "americas",
                Region::Europe => "europe",
                Region::AsiaPacific => "asia-pacific",
                Region::MiddleEastAfrica => "middle-east-africa",
                Region::Currencies => "currencies",
            }
        }

        /// All region variants
        pub fn all() -> &'static [Region] {
            &[
                Region::Americas,
                Region::Europe,
                Region::AsiaPacific,
                Region::MiddleEastAfrica,
                Region::Currencies,
            ]
        }
    }

    /// Americas indices
    pub const AMERICAS: &[&str] = &[
        "^GSPC",   // S&P 500
        "^DJI",    // Dow Jones Industrial Average
        "^IXIC",   // NASDAQ Composite
        "^NYA",    // NYSE Composite Index
        "^XAX",    // NYSE American Composite Index
        "^RUT",    // Russell 2000 Index
        "^VIX",    // CBOE Volatility Index
        "^GSPTSE", // S&P/TSX Composite (Canada)
        "^BVSP",   // IBOVESPA (Brazil)
        "^MXX",    // IPC MEXICO
        "^IPSA",   // S&P IPSA (Chile)
        "^MERV",   // MERVAL (Argentina)
    ];

    /// Europe indices
    pub const EUROPE: &[&str] = &[
        "^FTSE",            // FTSE 100 (UK)
        "^GDAXI",           // DAX (Germany)
        "^FCHI",            // CAC 40 (France)
        "^STOXX50E",        // EURO STOXX 50
        "^N100",            // Euronext 100 Index
        "^BFX",             // BEL 20 (Belgium)
        "^BUK100P",         // Cboe UK 100
        "MOEX.ME",          // Moscow Exchange
        "^125904-USD-STRD", // MSCI EUROPE
    ];

    /// Asia Pacific indices
    pub const ASIA_PACIFIC: &[&str] = &[
        "^N225",     // Nikkei 225 (Japan)
        "^HSI",      // Hang Seng Index (Hong Kong)
        "000001.SS", // SSE Composite Index (China)
        "^KS11",     // KOSPI (South Korea)
        "^TWII",     // Taiwan Weighted Index
        "^STI",      // STI Index (Singapore)
        "^AXJO",     // S&P/ASX 200 (Australia)
        "^AORD",     // All Ordinaries (Australia)
        "^NZ50",     // S&P/NZX 50 (New Zealand)
        "^BSESN",    // S&P BSE SENSEX (India)
        "^JKSE",     // IDX Composite (Indonesia)
        "^KLSE",     // FTSE Bursa Malaysia KLCI
    ];

    /// Middle East & Africa indices
    pub const MIDDLE_EAST_AFRICA: &[&str] = &[
        "^TA125.TA", // TA-125 (Israel)
        "^CASE30",   // EGX 30 (Egypt)
        "^JN0U.JO",  // Top 40 USD Net TRI (South Africa)
    ];

    /// Currency indices
    pub const CURRENCIES: &[&str] = &[
        "DX-Y.NYB", // US Dollar Index
        "^XDB",     // British Pound Currency Index
        "^XDE",     // Euro Currency Index
        "^XDN",     // Japanese Yen Currency Index
        "^XDA",     // Australian Dollar Currency Index
    ];

    /// All world indices (all regions combined)
    pub fn all_symbols() -> Vec<&'static str> {
        Region::all()
            .iter()
            .flat_map(|r| r.symbols().iter().copied())
            .collect()
    }
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

    /// Get the list of fields for this statement type
    ///
    /// Returns field names without frequency prefix (e.g., "TotalRevenue" not "annualTotalRevenue")
    pub fn get_fields(&self) -> &'static [&'static str] {
        match self {
            StatementType::Income => &INCOME_STATEMENT_FIELDS,
            StatementType::Balance => &BALANCE_SHEET_FIELDS,
            StatementType::CashFlow => &CASH_FLOW_FIELDS,
        }
    }
}

/// Income statement fields (without frequency prefix)
const INCOME_STATEMENT_FIELDS: [&str; 30] = [
    fundamental_types::TOTAL_REVENUE,
    fundamental_types::OPERATING_REVENUE,
    fundamental_types::COST_OF_REVENUE,
    fundamental_types::GROSS_PROFIT,
    fundamental_types::OPERATING_EXPENSE,
    fundamental_types::SELLING_GENERAL_AND_ADMIN,
    fundamental_types::RESEARCH_AND_DEVELOPMENT,
    fundamental_types::OPERATING_INCOME,
    fundamental_types::NET_INTEREST_INCOME,
    fundamental_types::INTEREST_EXPENSE,
    fundamental_types::INTEREST_INCOME,
    fundamental_types::NET_NON_OPERATING_INTEREST_INCOME_EXPENSE,
    fundamental_types::OTHER_INCOME_EXPENSE,
    fundamental_types::PRETAX_INCOME,
    fundamental_types::TAX_PROVISION,
    fundamental_types::NET_INCOME_COMMON_STOCKHOLDERS,
    fundamental_types::NET_INCOME,
    fundamental_types::DILUTED_EPS,
    fundamental_types::BASIC_EPS,
    fundamental_types::DILUTED_AVERAGE_SHARES,
    fundamental_types::BASIC_AVERAGE_SHARES,
    fundamental_types::EBIT,
    fundamental_types::EBITDA,
    fundamental_types::RECONCILED_COST_OF_REVENUE,
    fundamental_types::RECONCILED_DEPRECIATION,
    fundamental_types::NET_INCOME_FROM_CONTINUING_OPERATION_NET_MINORITY_INTEREST,
    fundamental_types::NORMALIZED_EBITDA,
    fundamental_types::TOTAL_EXPENSES,
    fundamental_types::TOTAL_OPERATING_INCOME_AS_REPORTED,
    fundamental_types::DEPRECIATION_AND_AMORTIZATION,
];

/// Balance sheet fields (without frequency prefix)
const BALANCE_SHEET_FIELDS: [&str; 48] = [
    fundamental_types::TOTAL_ASSETS,
    fundamental_types::CURRENT_ASSETS,
    fundamental_types::CASH_CASH_EQUIVALENTS_AND_SHORT_TERM_INVESTMENTS,
    fundamental_types::CASH_AND_CASH_EQUIVALENTS,
    fundamental_types::CASH_FINANCIAL,
    fundamental_types::RECEIVABLES,
    fundamental_types::ACCOUNTS_RECEIVABLE,
    fundamental_types::INVENTORY,
    fundamental_types::PREPAID_ASSETS,
    fundamental_types::OTHER_CURRENT_ASSETS,
    fundamental_types::TOTAL_NON_CURRENT_ASSETS,
    fundamental_types::NET_PPE,
    fundamental_types::GROSS_PPE,
    fundamental_types::ACCUMULATED_DEPRECIATION,
    fundamental_types::GOODWILL,
    fundamental_types::GOODWILL_AND_OTHER_INTANGIBLE_ASSETS,
    fundamental_types::OTHER_INTANGIBLE_ASSETS,
    fundamental_types::INVESTMENTS_AND_ADVANCES,
    fundamental_types::LONG_TERM_EQUITY_INVESTMENT,
    fundamental_types::OTHER_NON_CURRENT_ASSETS,
    fundamental_types::TOTAL_LIABILITIES_NET_MINORITY_INTEREST,
    fundamental_types::CURRENT_LIABILITIES,
    fundamental_types::PAYABLES_AND_ACCRUED_EXPENSES,
    fundamental_types::ACCOUNTS_PAYABLE,
    fundamental_types::CURRENT_DEBT,
    fundamental_types::CURRENT_DEFERRED_REVENUE,
    fundamental_types::OTHER_CURRENT_LIABILITIES,
    fundamental_types::TOTAL_NON_CURRENT_LIABILITIES_NET_MINORITY_INTEREST,
    fundamental_types::LONG_TERM_DEBT,
    fundamental_types::LONG_TERM_DEBT_AND_CAPITAL_LEASE_OBLIGATION,
    fundamental_types::NON_CURRENT_DEFERRED_REVENUE,
    fundamental_types::NON_CURRENT_DEFERRED_TAXES_LIABILITIES,
    fundamental_types::OTHER_NON_CURRENT_LIABILITIES,
    fundamental_types::STOCKHOLDERS_EQUITY,
    fundamental_types::COMMON_STOCK_EQUITY,
    fundamental_types::COMMON_STOCK,
    fundamental_types::RETAINED_EARNINGS,
    fundamental_types::ADDITIONAL_PAID_IN_CAPITAL,
    fundamental_types::TREASURY_STOCK,
    fundamental_types::TOTAL_EQUITY_GROSS_MINORITY_INTEREST,
    fundamental_types::WORKING_CAPITAL,
    fundamental_types::INVESTED_CAPITAL,
    fundamental_types::TANGIBLE_BOOK_VALUE,
    fundamental_types::TOTAL_DEBT,
    fundamental_types::NET_DEBT,
    fundamental_types::SHARE_ISSUED,
    fundamental_types::ORDINARY_SHARES_NUMBER,
    fundamental_types::DEPRECIATION_AND_AMORTIZATION,
];

/// Cash flow statement fields (without frequency prefix)
const CASH_FLOW_FIELDS: [&str; 47] = [
    fundamental_types::OPERATING_CASH_FLOW,
    fundamental_types::CASH_FLOW_FROM_CONTINUING_OPERATING_ACTIVITIES,
    fundamental_types::NET_INCOME_FROM_CONTINUING_OPERATIONS,
    fundamental_types::DEPRECIATION_AND_AMORTIZATION,
    fundamental_types::DEFERRED_INCOME_TAX,
    fundamental_types::CHANGE_IN_WORKING_CAPITAL,
    fundamental_types::CHANGE_IN_RECEIVABLES,
    fundamental_types::CHANGES_IN_ACCOUNT_RECEIVABLES,
    fundamental_types::CHANGE_IN_INVENTORY,
    fundamental_types::CHANGE_IN_ACCOUNT_PAYABLE,
    fundamental_types::CHANGE_IN_OTHER_WORKING_CAPITAL,
    fundamental_types::STOCK_BASED_COMPENSATION,
    fundamental_types::OTHER_NON_CASH_ITEMS,
    fundamental_types::INVESTING_CASH_FLOW,
    fundamental_types::CASH_FLOW_FROM_CONTINUING_INVESTING_ACTIVITIES,
    fundamental_types::NET_PPE_PURCHASE_AND_SALE,
    fundamental_types::PURCHASE_OF_PPE,
    fundamental_types::SALE_OF_PPE,
    fundamental_types::CAPITAL_EXPENDITURE,
    fundamental_types::NET_BUSINESS_PURCHASE_AND_SALE,
    fundamental_types::PURCHASE_OF_BUSINESS,
    fundamental_types::SALE_OF_BUSINESS,
    fundamental_types::NET_INVESTMENT_PURCHASE_AND_SALE,
    fundamental_types::PURCHASE_OF_INVESTMENT,
    fundamental_types::SALE_OF_INVESTMENT,
    fundamental_types::NET_OTHER_INVESTING_CHANGES,
    fundamental_types::FINANCING_CASH_FLOW,
    fundamental_types::CASH_FLOW_FROM_CONTINUING_FINANCING_ACTIVITIES,
    fundamental_types::NET_ISSUANCE_PAYMENTS_OF_DEBT,
    fundamental_types::NET_LONG_TERM_DEBT_ISSUANCE,
    fundamental_types::LONG_TERM_DEBT_ISSUANCE,
    fundamental_types::LONG_TERM_DEBT_PAYMENTS,
    fundamental_types::NET_SHORT_TERM_DEBT_ISSUANCE,
    fundamental_types::NET_COMMON_STOCK_ISSUANCE,
    fundamental_types::COMMON_STOCK_ISSUANCE,
    fundamental_types::COMMON_STOCK_PAYMENTS,
    fundamental_types::REPURCHASE_OF_CAPITAL_STOCK,
    fundamental_types::CASH_DIVIDENDS_PAID,
    fundamental_types::COMMON_STOCK_DIVIDEND_PAID,
    fundamental_types::NET_OTHER_FINANCING_CHARGES,
    fundamental_types::END_CASH_POSITION,
    fundamental_types::BEGINNING_CASH_POSITION,
    fundamental_types::CHANGESIN_CASH,
    fundamental_types::EFFECT_OF_EXCHANGE_RATE_CHANGES,
    fundamental_types::FREE_CASH_FLOW,
    fundamental_types::CAPITAL_EXPENDITURE_REPORTED,
    fundamental_types::DEPRECIATION_AND_AMORTIZATION,
];

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
    /// use finance_query::Frequency;
    ///
    /// let field = Frequency::Annual.prefix("TotalRevenue");
    /// assert_eq!(field, "annualTotalRevenue");
    /// ```
    pub fn prefix(&self, field: &str) -> String {
        format!("{}{}", self.as_str(), field)
    }
}

/// Chart intervals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Interval {
    /// 1 minute
    #[serde(rename = "1m")]
    OneMinute,
    /// 5 minutes
    #[serde(rename = "5m")]
    FiveMinutes,
    /// 15 minutes
    #[serde(rename = "15m")]
    FifteenMinutes,
    /// 30 minutes
    #[serde(rename = "30m")]
    ThirtyMinutes,
    /// 1 hour
    #[serde(rename = "1h")]
    OneHour,
    /// 1 day
    #[serde(rename = "1d")]
    OneDay,
    /// 1 week
    #[serde(rename = "1wk")]
    OneWeek,
    /// 1 month
    #[serde(rename = "1mo")]
    OneMonth,
    /// 3 months
    #[serde(rename = "3mo")]
    ThreeMonths,
}

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

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Time ranges for chart data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeRange {
    /// 1 day
    #[serde(rename = "1d")]
    OneDay,
    /// 5 days
    #[serde(rename = "5d")]
    FiveDays,
    /// 1 month
    #[serde(rename = "1mo")]
    OneMonth,
    /// 3 months
    #[serde(rename = "3mo")]
    ThreeMonths,
    /// 6 months
    #[serde(rename = "6mo")]
    SixMonths,
    /// 1 year
    #[serde(rename = "1y")]
    OneYear,
    /// 2 years
    #[serde(rename = "2y")]
    TwoYears,
    /// 5 years
    #[serde(rename = "5y")]
    FiveYears,
    /// 10 years
    #[serde(rename = "10y")]
    TenYears,
    /// Year to date
    #[serde(rename = "ytd")]
    YearToDate,
    /// Maximum available
    #[serde(rename = "max")]
    Max,
}

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

impl std::fmt::Display for TimeRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Supported regions for Yahoo Finance regional APIs
///
/// Each region has predefined language and region codes that work together.
/// Using the Region enum ensures correct lang/region pairing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Region {
    /// Argentina (es-AR, AR)
    Argentina,
    /// Australia (en-AU, AU)
    Australia,
    /// Brazil (pt-BR, BR)
    Brazil,
    /// Canada (en-CA, CA)
    Canada,
    /// China (zh-CN, CN)
    China,
    /// Denmark (da-DK, DK)
    Denmark,
    /// Finland (fi-FI, FI)
    Finland,
    /// France (fr-FR, FR)
    France,
    /// Germany (de-DE, DE)
    Germany,
    /// Greece (el-GR, GR)
    Greece,
    /// Hong Kong (zh-Hant-HK, HK)
    HongKong,
    /// India (en-IN, IN)
    India,
    /// Israel (he-IL, IL)
    Israel,
    /// Italy (it-IT, IT)
    Italy,
    /// Malaysia (ms-MY, MY)
    Malaysia,
    /// New Zealand (en-NZ, NZ)
    NewZealand,
    /// Norway (nb-NO, NO)
    Norway,
    /// Portugal (pt-PT, PT)
    Portugal,
    /// Russia (ru-RU, RU)
    Russia,
    /// Singapore (en-SG, SG)
    Singapore,
    /// Spain (es-ES, ES)
    Spain,
    /// Sweden (sv-SE, SE)
    Sweden,
    /// Taiwan (zh-TW, TW)
    Taiwan,
    /// Thailand (th-TH, TH)
    Thailand,
    /// Turkey (tr-TR, TR)
    Turkey,
    /// United Kingdom (en-GB, GB)
    UnitedKingdom,
    /// United States (en-US, US) - Default
    #[default]
    UnitedStates,
    /// Vietnam (vi-VN, VN)
    Vietnam,
}

impl Region {
    /// Get the language code for this region
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::Region;
    ///
    /// assert_eq!(Region::France.lang(), "fr-FR");
    /// assert_eq!(Region::UnitedStates.lang(), "en-US");
    /// ```
    pub fn lang(&self) -> &'static str {
        match self {
            Region::Argentina => "es-AR",
            Region::Australia => "en-AU",
            Region::Brazil => "pt-BR",
            Region::Canada => "en-CA",
            Region::China => "zh-CN",
            Region::Denmark => "da-DK",
            Region::Finland => "fi-FI",
            Region::France => "fr-FR",
            Region::Germany => "de-DE",
            Region::Greece => "el-GR",
            Region::HongKong => "zh-Hant-HK",
            Region::India => "en-IN",
            Region::Israel => "he-IL",
            Region::Italy => "it-IT",
            Region::Malaysia => "ms-MY",
            Region::NewZealand => "en-NZ",
            Region::Norway => "nb-NO",
            Region::Portugal => "pt-PT",
            Region::Russia => "ru-RU",
            Region::Singapore => "en-SG",
            Region::Spain => "es-ES",
            Region::Sweden => "sv-SE",
            Region::Taiwan => "zh-TW",
            Region::Thailand => "th-TH",
            Region::Turkey => "tr-TR",
            Region::UnitedKingdom => "en-GB",
            Region::UnitedStates => "en-US",
            Region::Vietnam => "vi-VN",
        }
    }

    /// Get the region code for this region
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::Region;
    ///
    /// assert_eq!(Region::France.region(), "FR");
    /// assert_eq!(Region::UnitedStates.region(), "US");
    /// ```
    pub fn region(&self) -> &'static str {
        match self {
            Region::Argentina => "AR",
            Region::Australia => "AU",
            Region::Brazil => "BR",
            Region::Canada => "CA",
            Region::China => "CN",
            Region::Denmark => "DK",
            Region::Finland => "FI",
            Region::France => "FR",
            Region::Germany => "DE",
            Region::Greece => "GR",
            Region::HongKong => "HK",
            Region::India => "IN",
            Region::Israel => "IL",
            Region::Italy => "IT",
            Region::Malaysia => "MY",
            Region::NewZealand => "NZ",
            Region::Norway => "NO",
            Region::Portugal => "PT",
            Region::Russia => "RU",
            Region::Singapore => "SG",
            Region::Spain => "ES",
            Region::Sweden => "SE",
            Region::Taiwan => "TW",
            Region::Thailand => "TH",
            Region::Turkey => "TR",
            Region::UnitedKingdom => "GB",
            Region::UnitedStates => "US",
            Region::Vietnam => "VN",
        }
    }

    /// Get the CORS domain for this region
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::Region;
    ///
    /// assert_eq!(Region::UnitedStates.cors_domain(), "finance.yahoo.com");
    /// assert_eq!(Region::France.cors_domain(), "fr.finance.yahoo.com");
    /// ```
    pub fn cors_domain(&self) -> &'static str {
        match self {
            Region::Argentina => "ar.finance.yahoo.com",
            Region::Australia => "au.finance.yahoo.com",
            Region::Brazil => "br.financas.yahoo.com",
            Region::Canada => "ca.finance.yahoo.com",
            Region::China => "cn.finance.yahoo.com",
            Region::Denmark => "dk.finance.yahoo.com",
            Region::Finland => "fi.finance.yahoo.com",
            Region::France => "fr.finance.yahoo.com",
            Region::Germany => "de.finance.yahoo.com",
            Region::Greece => "gr.finance.yahoo.com",
            Region::HongKong => "hk.finance.yahoo.com",
            Region::India => "in.finance.yahoo.com",
            Region::Israel => "il.finance.yahoo.com",
            Region::Italy => "it.finance.yahoo.com",
            Region::Malaysia => "my.finance.yahoo.com",
            Region::NewZealand => "nz.finance.yahoo.com",
            Region::Norway => "no.finance.yahoo.com",
            Region::Portugal => "pt.finance.yahoo.com",
            Region::Russia => "ru.finance.yahoo.com",
            Region::Singapore => "sg.finance.yahoo.com",
            Region::Spain => "es.finance.yahoo.com",
            Region::Sweden => "se.finance.yahoo.com",
            Region::Taiwan => "tw.finance.yahoo.com",
            Region::Thailand => "th.finance.yahoo.com",
            Region::Turkey => "tr.finance.yahoo.com",
            Region::UnitedKingdom => "uk.finance.yahoo.com",
            Region::UnitedStates => "finance.yahoo.com",
            Region::Vietnam => "vn.finance.yahoo.com",
        }
    }
}

impl std::str::FromStr for Region {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "AR" => Ok(Region::Argentina),
            "AU" => Ok(Region::Australia),
            "BR" => Ok(Region::Brazil),
            "CA" => Ok(Region::Canada),
            "CN" => Ok(Region::China),
            "DK" => Ok(Region::Denmark),
            "FI" => Ok(Region::Finland),
            "FR" => Ok(Region::France),
            "DE" => Ok(Region::Germany),
            "GR" => Ok(Region::Greece),
            "HK" => Ok(Region::HongKong),
            "IN" => Ok(Region::India),
            "IL" => Ok(Region::Israel),
            "IT" => Ok(Region::Italy),
            "MY" => Ok(Region::Malaysia),
            "NZ" => Ok(Region::NewZealand),
            "NO" => Ok(Region::Norway),
            "PT" => Ok(Region::Portugal),
            "RU" => Ok(Region::Russia),
            "SG" => Ok(Region::Singapore),
            "ES" => Ok(Region::Spain),
            "SE" => Ok(Region::Sweden),
            "TW" => Ok(Region::Taiwan),
            "TH" => Ok(Region::Thailand),
            "TR" => Ok(Region::Turkey),
            "GB" | "UK" => Ok(Region::UnitedKingdom),
            "US" => Ok(Region::UnitedStates),
            "VN" => Ok(Region::Vietnam),
            _ => Err(()),
        }
    }
}

impl From<Region> for String {
    /// Returns the lowercase two-letter country code used by the Yahoo Finance screener
    /// (e.g. `"us"`, `"gb"`).
    fn from(v: Region) -> Self {
        v.region().to_lowercase()
    }
}

/// Value format for API responses
///
/// Controls how `FormattedValue<T>` fields are serialized in responses.
/// This allows API consumers to choose between raw numeric values,
/// human-readable formatted strings, or both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ValueFormat {
    /// Return only raw numeric values (e.g., `123.45`) - default
    /// Best for programmatic use, calculations, charts
    #[default]
    Raw,
    /// Return only formatted strings (e.g., `"$123.45"`, `"1.2B"`)
    /// Best for display purposes
    Pretty,
    /// Return both raw and formatted values
    /// Returns the full `{raw, fmt, longFmt}` object
    Both,
}

impl std::str::FromStr for ValueFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "raw" => Ok(ValueFormat::Raw),
            "pretty" | "fmt" => Ok(ValueFormat::Pretty),
            "both" | "full" => Ok(ValueFormat::Both),
            _ => Err(()),
        }
    }
}

impl ValueFormat {
    /// Parse from string (case-insensitive), returns None on invalid input
    pub fn parse(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ValueFormat::Raw => "raw",
            ValueFormat::Pretty => "pretty",
            ValueFormat::Both => "both",
        }
    }

    /// Transform a JSON value based on this format
    ///
    /// Recursively processes the JSON, detecting FormattedValue objects
    /// (objects with `raw` key and optionally `fmt`/`longFmt`) and
    /// transforming them according to the format setting.
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::ValueFormat;
    /// use serde_json::json;
    ///
    /// let data = json!({"price": {"raw": 123.45, "fmt": "$123.45"}});
    ///
    /// // Raw format extracts just the raw value (default)
    /// let raw = ValueFormat::default().transform(data.clone());
    /// assert_eq!(raw, json!({"price": 123.45}));
    ///
    /// // Pretty extracts just the formatted string
    /// let pretty = ValueFormat::Pretty.transform(data.clone());
    /// assert_eq!(pretty, json!({"price": "$123.45"}));
    ///
    /// // Both keeps the full object
    /// let both = ValueFormat::Both.transform(data);
    /// assert_eq!(both, json!({"price": {"raw": 123.45, "fmt": "$123.45"}}));
    /// ```
    pub fn transform(&self, value: serde_json::Value) -> serde_json::Value {
        match self {
            ValueFormat::Both => value, // No transformation needed
            _ => self.transform_recursive(value),
        }
    }

    fn transform_recursive(&self, value: serde_json::Value) -> serde_json::Value {
        use serde_json::Value;

        match value {
            Value::Object(map) => {
                // Check if this looks like a FormattedValue (has 'raw' key)
                if self.is_formatted_value(&map) {
                    return self.extract_value(&map);
                }

                // Otherwise, recursively transform all values
                let transformed: serde_json::Map<String, Value> = map
                    .into_iter()
                    .map(|(k, v)| (k, self.transform_recursive(v)))
                    .collect();
                Value::Object(transformed)
            }
            Value::Array(arr) => Value::Array(
                arr.into_iter()
                    .map(|v| self.transform_recursive(v))
                    .collect(),
            ),
            // Primitives pass through unchanged
            other => other,
        }
    }

    /// Check if an object looks like a FormattedValue
    fn is_formatted_value(&self, map: &serde_json::Map<String, serde_json::Value>) -> bool {
        // Must have 'raw' key (can be null)
        // May have 'fmt' and/or 'longFmt'
        // Should not have many other keys (FormattedValue only has these 3)
        if !map.contains_key("raw") {
            return false;
        }

        let known_keys = ["raw", "fmt", "longFmt"];
        let unknown_keys = map
            .keys()
            .filter(|k| !known_keys.contains(&k.as_str()))
            .count();

        // If there are unknown keys, it's probably not a FormattedValue
        unknown_keys == 0
    }

    /// Extract the appropriate value based on format
    fn extract_value(&self, map: &serde_json::Map<String, serde_json::Value>) -> serde_json::Value {
        match self {
            ValueFormat::Raw => {
                // Return raw value directly (or null if not present)
                map.get("raw").cloned().unwrap_or(serde_json::Value::Null)
            }
            ValueFormat::Pretty => {
                // Prefer fmt, fall back to longFmt, then null
                map.get("fmt")
                    .or_else(|| map.get("longFmt"))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null)
            }
            ValueFormat::Both => {
                // Keep as-is (shouldn't reach here, but handle anyway)
                serde_json::Value::Object(map.clone())
            }
        }
    }
}

/// Typed industry identifiers shared between the industry endpoint and screener queries.
///
/// Use with [`finance::industry()`](crate::finance::industry) for the data endpoint, and with
/// [`EquityField::Industry`](crate::EquityField::Industry) +
/// [`ScreenerFieldExt::eq_str`](crate::ScreenerFieldExt::eq_str) for screener filtering.
///
/// - `as_slug()` → lowercase hyphenated key for `finance::industry()` (e.g. `"semiconductors"`)
/// - `From<Industry> for String` → screener display name (e.g. `"Semiconductors"`)
///
/// # Example
///
/// ```no_run
/// use finance_query::{finance, Industry, EquityField, EquityScreenerQuery, ScreenerFieldExt};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Industry endpoint
/// let data = finance::industry(Industry::Semiconductors).await?;
///
/// // Screener filter
/// let query = EquityScreenerQuery::new()
///     .add_condition(EquityField::Industry.eq_str(Industry::Semiconductors));
/// # Ok(())
/// # }
/// ```
pub mod industries {
    /// Typed industry identifier for the industry endpoint and custom screener queries.
    ///
    /// See the module-level doc for usage.
    #[non_exhaustive]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Industry {
        // ── Agriculture / Raw Materials ──────────────────────────────────────
        /// Agricultural inputs, fertilizers, and crop chemicals
        AgriculturalInputs,
        /// Aluminum production and processing companies
        Aluminum,
        /// Coal mining and processing companies
        Coal,
        /// Copper mining and processing companies
        Copper,
        /// Farm products including grains, livestock, and produce
        FarmProducts,
        /// Forest products including timber and paper pulp
        ForestProducts,
        /// Gold mining and royalty companies
        Gold,
        /// Lumber and wood production companies
        LumberAndWoodProduction,
        /// Other industrial metals and mining (zinc, nickel, etc.)
        OtherIndustrialMetalsAndMining,
        /// Other precious metals and mining (platinum, palladium, etc.)
        OtherPreciousMetalsAndMining,
        /// Silver mining and streaming companies
        Silver,
        /// Steel production and processing companies
        Steel,
        /// Thermal coal mining for electricity generation
        ThermalCoal,
        /// Uranium mining companies
        Uranium,
        // ── Consumer ─────────────────────────────────────────────────────────
        /// Clothing and apparel manufacturing companies
        ApparelManufacturing,
        /// Clothing and apparel retail chains
        ApparelRetail,
        /// Automotive and truck dealerships
        AutoAndTruckDealerships,
        /// Automobile manufacturers and assemblers
        AutoManufacturers,
        /// Automotive parts manufacturers and distributors
        AutoParts,
        /// Beer brewing and distribution companies
        BeveragesBrewers,
        /// Non-alcoholic beverages including soft drinks and juices
        BeveragesNonAlcoholic,
        /// Wineries, distilleries, and spirits producers
        BeveragesWineriesAndDistilleries,
        /// Candy, chocolate, and confectionery makers
        Confectioners,
        /// Traditional department store retailers
        DepartmentStores,
        /// Discount and value retail stores
        DiscountStores,
        /// Electronic gaming software and multimedia entertainment
        ElectronicGamingAndMultimedia,
        /// Wholesale food distribution companies
        FoodDistribution,
        /// Footwear, handbags, and fashion accessories
        FootwearAndAccessories,
        /// Furniture, fixtures, and household appliances
        FurnishingsFixturesAndAppliances,
        /// Casinos, online gambling, and gaming operators
        Gambling,
        /// Supermarkets and grocery retail chains
        GroceryStores,
        /// Home improvement retail stores
        HomeImprovementRetail,
        /// Household cleaning products and personal care items
        HouseholdAndPersonalProducts,
        /// Online retail and e-commerce marketplaces
        InternetRetail,
        /// Leisure, recreation, and entertainment companies
        Leisure,
        /// Hotels and lodging companies
        Lodging,
        /// Luxury goods, fashion, and premium consumer brands
        LuxuryGoods,
        /// Packaged and processed food manufacturers
        PackagedFoods,
        /// Personal care, laundry, and household services
        PersonalServices,
        /// Home builders and residential construction
        ResidentialConstruction,
        /// Resorts, integrated casinos, and hotel-casinos
        ResortsAndCasinos,
        /// Restaurant chains and food service operators
        Restaurants,
        /// Specialty retail stores (pets, books, electronics, etc.)
        SpecialtyRetail,
        /// Textile and fabric manufacturers
        TextileManufacturing,
        /// Tobacco product manufacturers
        Tobacco,
        /// Travel agencies, booking platforms, and tour operators
        TravelServices,
        // ── Energy ───────────────────────────────────────────────────────────
        /// Oil and gas contract drilling services
        OilAndGasDrilling,
        /// Oil and gas exploration and production companies
        OilAndGasEAndP,
        /// Oil field equipment, services, and engineering
        OilAndGasEquipmentAndServices,
        /// Vertically integrated oil and gas majors
        OilAndGasIntegrated,
        /// Oil and gas pipelines, storage, and transportation
        OilAndGasMidstream,
        /// Oil refining, wholesale fuel, and marketing
        OilAndGasRefiningAndMarketing,
        /// Solar panel manufacturers and solar energy producers
        Solar,
        // ── Financial Services ───────────────────────────────────────────────
        /// Asset managers, fund sponsors, and investment advisors
        AssetManagement,
        /// Large diversified national and international banks
        BanksDiversified,
        /// Regional and community banks
        BanksRegional,
        /// Investment banks, brokers, and financial exchanges
        CapitalMarkets,
        /// Credit card issuers and consumer credit services
        CreditServices,
        /// Financial data, analytics, and stock exchange operators
        FinancialDataAndStockExchanges,
        /// Insurance brokers and agencies
        InsuranceBrokers,
        /// Diversified multi-line insurance companies
        InsuranceDiversified,
        /// Life insurance and annuity providers
        InsuranceLife,
        /// Property and casualty insurance companies
        InsurancePropertyAndCasualty,
        /// Reinsurance companies
        InsuranceReinsurance,
        /// Specialty insurance lines (title, mortgage, etc.)
        InsuranceSpecialty,
        /// Mortgage banking and loan origination
        MortgageFinance,
        /// Blank-check and shell holding companies
        ShellCompanies,
        // ── Healthcare ───────────────────────────────────────────────────────
        /// Biotechnology drug development companies
        Biotechnology,
        /// Medical diagnostics labs and clinical research
        DiagnosticsAndResearch,
        /// Large branded pharmaceutical manufacturers
        DrugManufacturersGeneral,
        /// Specialty drugs, generics, and biosimilars
        DrugManufacturersSpecialtyAndGeneric,
        /// Healthcare IT, EHR, and health data services
        HealthInformationServices,
        /// Managed care organizations and health insurers
        HealthcarePlans,
        /// Hospitals, clinics, and outpatient care facilities
        MedicalCareFacilities,
        /// Medical device manufacturers (implants, diagnostics equipment)
        MedicalDevices,
        /// Medical product wholesalers and distributors
        MedicalDistribution,
        /// Surgical instruments, disposables, and medical supplies
        MedicalInstrumentsAndSupplies,
        /// Retail pharmacies and drug store chains
        PharmaceuticalRetailers,
        // ── Industrials ──────────────────────────────────────────────────────
        /// Defense contractors, aircraft, and space systems
        AerospaceAndDefense,
        /// Construction aggregates, cement, and building materials
        BuildingMaterials,
        /// HVAC, plumbing, windows, and building equipment
        BuildingProductsAndEquipment,
        /// Office supplies, commercial equipment, and printers
        BusinessEquipmentAndSupplies,
        /// Specialty chemical manufacturing for industrial use
        ChemicalManufacturing,
        /// Diversified commodity chemicals producers
        Chemicals,
        /// Diversified industrial holding companies
        Conglomerates,
        /// Management consulting and professional advisory services
        ConsultingServices,
        /// Electrical components, motors, and power equipment
        ElectricalEquipmentAndParts,
        /// Civil engineering, construction, and infrastructure projects
        EngineeringAndConstruction,
        /// Agricultural equipment and heavy construction machinery
        FarmAndHeavyConstructionMachinery,
        /// Industrial goods wholesalers and distributors
        IndustrialDistribution,
        /// Toll roads, airports, and infrastructure operators
        InfrastructureOperations,
        /// Third-party logistics and supply chain management
        IntegratedFreightAndLogistics,
        /// Diversified manufacturers across multiple industrial segments
        ManufacturingDiversified,
        /// Port operators and marine terminal services
        MarinePortsAndServices,
        /// Bulk cargo and tanker shipping companies
        MarineShipping,
        /// Custom metal fabrication and machined components
        MetalFabrication,
        /// Paper, packaging, and pulp product manufacturers
        PaperAndPaperProducts,
        /// Environmental controls, water treatment, and remediation
        PollutionAndTreatmentControls,
        /// Rail freight carriers and passenger rail operators
        Railroads,
        /// Equipment rental, leasing, and fleet management
        RentalAndLeasingServices,
        /// Security systems, guards, and monitoring services
        SecurityAndProtectionServices,
        /// Outsourced business services and BPO companies
        SpecialtyBusinessServices,
        /// High-value specialty chemicals and advanced materials
        SpecialtyChemicals,
        /// Specialized industrial machinery and equipment makers
        SpecialtyIndustrialMachinery,
        /// Staffing agencies and employment service providers
        StaffingAndEmploymentServices,
        /// Hand tools, power tools, and hardware accessories
        ToolsAndAccessories,
        /// Freight trucking and less-than-truckload carriers
        Trucking,
        /// Waste collection, recycling, and disposal services
        WasteManagement,
        // ── Real Estate ──────────────────────────────────────────────────────
        /// Real estate developers and homebuilders
        RealEstateDevelopment,
        /// Diversified real estate companies with mixed portfolios
        RealEstateDiversified,
        /// Real estate brokers, agents, and property managers
        RealEstateServices,
        /// Diversified REITs across multiple property types
        ReitDiversified,
        /// Healthcare and senior living facility REITs
        ReitHealthcareFacilities,
        /// Hotel and motel property REITs
        ReitHotelAndMotel,
        /// Industrial, warehouse, and logistics property REITs
        ReitIndustrial,
        /// Mortgage REITs investing in real estate debt
        ReitMortgage,
        /// Office building and commercial property REITs
        ReitOffice,
        /// Apartment, multifamily, and residential property REITs
        ReitResidential,
        /// Shopping center and retail property REITs
        ReitRetail,
        /// Specialty REITs (data centers, cell towers, self-storage)
        ReitSpecialty,
        // ── Technology ───────────────────────────────────────────────────────
        /// Networking hardware, routers, and communication equipment
        CommunicationEquipment,
        /// PCs, servers, and computer hardware manufacturers
        ComputerHardware,
        /// Smartphones, TVs, and consumer electronic devices
        ConsumerElectronics,
        /// Data analytics, business intelligence, and AI platforms
        DataAnalytics,
        /// Passive electronic components and circuit boards
        ElectronicComponents,
        /// Distributors of electronics and computer products
        ElectronicsAndComputerDistribution,
        /// Value-added resellers and software/hardware distributors
        HardwareAndSoftwareDistribution,
        /// IT services, outsourcing, and technology consulting
        InformationTechnologyServices,
        /// Online media, search engines, and digital content platforms
        InternetContentAndInformation,
        /// Precision instruments, sensors, and test equipment
        ScientificAndTechnicalInstruments,
        /// Semiconductor manufacturing equipment and materials
        SemiconductorEquipmentAndMaterials,
        /// Integrated circuit and chip designers and manufacturers
        Semiconductors,
        /// Business application software companies
        SoftwareApplication,
        /// Operating systems, middleware, and infrastructure software
        SoftwareInfrastructure,
        // ── Communication Services ───────────────────────────────────────────
        /// Television, radio, and broadcast media companies
        Broadcasting,
        /// Film studios, streaming, and live entertainment
        Entertainment,
        /// Book, magazine, newspaper, and digital media publishers
        Publishing,
        /// Wireless carriers and wireline telephone companies
        TelecomServices,
        // ── Utilities ────────────────────────────────────────────────────────
        /// Multi-utility companies serving electricity, gas, and water
        UtilitiesDiversified,
        /// Independent power producers and energy traders
        UtilitiesIndependentPowerProducers,
        /// Regulated electric utility companies
        UtilitiesRegulatedElectric,
        /// Regulated natural gas distribution utilities
        UtilitiesRegulatedGas,
        /// Regulated water and wastewater utilities
        UtilitiesRegulatedWater,
        /// Renewable energy generation companies (wind, solar, hydro)
        UtilitiesRenewable,
        // ── Special ──────────────────────────────────────────────────────────
        /// Closed-end funds investing in debt instruments
        ClosedEndFundDebt,
        /// Closed-end funds investing in equities
        ClosedEndFundEquity,
        /// Closed-end funds investing in foreign securities
        ClosedEndFundForeign,
        /// Exchange-traded fund products
        ExchangeTradedFund,
    }

    impl Industry {
        /// Returns the lowercase hyphenated slug used by `finance::industry()`.
        ///
        /// # Example
        ///
        /// ```
        /// use finance_query::Industry;
        /// assert_eq!(Industry::Semiconductors.as_slug(), "semiconductors");
        /// assert_eq!(Industry::SoftwareApplication.as_slug(), "software-application");
        /// ```
        pub fn as_slug(self) -> &'static str {
            match self {
                Industry::AgriculturalInputs => "agricultural-inputs",
                Industry::Aluminum => "aluminum",
                Industry::Coal => "coal",
                Industry::Copper => "copper",
                Industry::FarmProducts => "farm-products",
                Industry::ForestProducts => "forest-products",
                Industry::Gold => "gold",
                Industry::LumberAndWoodProduction => "lumber-wood-production",
                Industry::OtherIndustrialMetalsAndMining => "other-industrial-metals-mining",
                Industry::OtherPreciousMetalsAndMining => "other-precious-metals-mining",
                Industry::Silver => "silver",
                Industry::Steel => "steel",
                Industry::ThermalCoal => "thermal-coal",
                Industry::Uranium => "uranium",
                Industry::ApparelManufacturing => "apparel-manufacturing",
                Industry::ApparelRetail => "apparel-retail",
                Industry::AutoAndTruckDealerships => "auto-truck-dealerships",
                Industry::AutoManufacturers => "auto-manufacturers",
                Industry::AutoParts => "auto-parts",
                Industry::BeveragesBrewers => "beverages-brewers",
                Industry::BeveragesNonAlcoholic => "beverages-non-alcoholic",
                Industry::BeveragesWineriesAndDistilleries => "beverages-wineries-distilleries",
                Industry::Confectioners => "confectioners",
                Industry::DepartmentStores => "department-stores",
                Industry::DiscountStores => "discount-stores",
                Industry::ElectronicGamingAndMultimedia => "electronic-gaming-multimedia",
                Industry::FoodDistribution => "food-distribution",
                Industry::FootwearAndAccessories => "footwear-accessories",
                Industry::FurnishingsFixturesAndAppliances => "furnishings-fixtures-appliances",
                Industry::Gambling => "gambling",
                Industry::GroceryStores => "grocery-stores",
                Industry::HomeImprovementRetail => "home-improvement-retail",
                Industry::HouseholdAndPersonalProducts => "household-personal-products",
                Industry::InternetRetail => "internet-retail",
                Industry::Leisure => "leisure",
                Industry::Lodging => "lodging",
                Industry::LuxuryGoods => "luxury-goods",
                Industry::PackagedFoods => "packaged-foods",
                Industry::PersonalServices => "personal-services",
                Industry::ResidentialConstruction => "residential-construction",
                Industry::ResortsAndCasinos => "resorts-casinos",
                Industry::Restaurants => "restaurants",
                Industry::SpecialtyRetail => "specialty-retail",
                Industry::TextileManufacturing => "textile-manufacturing",
                Industry::Tobacco => "tobacco",
                Industry::TravelServices => "travel-services",
                Industry::OilAndGasDrilling => "oil-gas-drilling",
                Industry::OilAndGasEAndP => "oil-gas-ep",
                Industry::OilAndGasEquipmentAndServices => "oil-gas-equipment-services",
                Industry::OilAndGasIntegrated => "oil-gas-integrated",
                Industry::OilAndGasMidstream => "oil-gas-midstream",
                Industry::OilAndGasRefiningAndMarketing => "oil-gas-refining-marketing",
                Industry::Solar => "solar",
                Industry::AssetManagement => "asset-management",
                Industry::BanksDiversified => "banks-diversified",
                Industry::BanksRegional => "banks-regional",
                Industry::CapitalMarkets => "capital-markets",
                Industry::CreditServices => "credit-services",
                Industry::FinancialDataAndStockExchanges => "financial-data-stock-exchanges",
                Industry::InsuranceBrokers => "insurance-brokers",
                Industry::InsuranceDiversified => "insurance-diversified",
                Industry::InsuranceLife => "insurance-life",
                Industry::InsurancePropertyAndCasualty => "insurance-property-casualty",
                Industry::InsuranceReinsurance => "insurance-reinsurance",
                Industry::InsuranceSpecialty => "insurance-specialty",
                Industry::MortgageFinance => "mortgage-finance",
                Industry::ShellCompanies => "shell-companies",
                Industry::Biotechnology => "biotechnology",
                Industry::DiagnosticsAndResearch => "diagnostics-research",
                Industry::DrugManufacturersGeneral => "drug-manufacturers-general",
                Industry::DrugManufacturersSpecialtyAndGeneric => {
                    "drug-manufacturers-specialty-generic"
                }
                Industry::HealthInformationServices => "health-information-services",
                Industry::HealthcarePlans => "healthcare-plans",
                Industry::MedicalCareFacilities => "medical-care-facilities",
                Industry::MedicalDevices => "medical-devices",
                Industry::MedicalDistribution => "medical-distribution",
                Industry::MedicalInstrumentsAndSupplies => "medical-instruments-supplies",
                Industry::PharmaceuticalRetailers => "pharmaceutical-retailers",
                Industry::AerospaceAndDefense => "aerospace-defense",
                Industry::BuildingMaterials => "building-materials",
                Industry::BuildingProductsAndEquipment => "building-products-equipment",
                Industry::BusinessEquipmentAndSupplies => "business-equipment-supplies",
                Industry::ChemicalManufacturing => "chemical-manufacturing",
                Industry::Chemicals => "chemicals",
                Industry::Conglomerates => "conglomerates",
                Industry::ConsultingServices => "consulting-services",
                Industry::ElectricalEquipmentAndParts => "electrical-equipment-parts",
                Industry::EngineeringAndConstruction => "engineering-construction",
                Industry::FarmAndHeavyConstructionMachinery => "farm-heavy-construction-machinery",
                Industry::IndustrialDistribution => "industrial-distribution",
                Industry::InfrastructureOperations => "infrastructure-operations",
                Industry::IntegratedFreightAndLogistics => "integrated-freight-logistics",
                Industry::ManufacturingDiversified => "manufacturing-diversified",
                Industry::MarinePortsAndServices => "marine-ports-services",
                Industry::MarineShipping => "marine-shipping",
                Industry::MetalFabrication => "metal-fabrication",
                Industry::PaperAndPaperProducts => "paper-paper-products",
                Industry::PollutionAndTreatmentControls => "pollution-treatment-controls",
                Industry::Railroads => "railroads",
                Industry::RentalAndLeasingServices => "rental-leasing-services",
                Industry::SecurityAndProtectionServices => "security-protection-services",
                Industry::SpecialtyBusinessServices => "specialty-business-services",
                Industry::SpecialtyChemicals => "specialty-chemicals",
                Industry::SpecialtyIndustrialMachinery => "specialty-industrial-machinery",
                Industry::StaffingAndEmploymentServices => "staffing-employment-services",
                Industry::ToolsAndAccessories => "tools-accessories",
                Industry::Trucking => "trucking",
                Industry::WasteManagement => "waste-management",
                Industry::RealEstateDevelopment => "real-estate-development",
                Industry::RealEstateDiversified => "real-estate-diversified",
                Industry::RealEstateServices => "real-estate-services",
                Industry::ReitDiversified => "reit-diversified",
                Industry::ReitHealthcareFacilities => "reit-healthcare-facilities",
                Industry::ReitHotelAndMotel => "reit-hotel-motel",
                Industry::ReitIndustrial => "reit-industrial",
                Industry::ReitMortgage => "reit-mortgage",
                Industry::ReitOffice => "reit-office",
                Industry::ReitResidential => "reit-residential",
                Industry::ReitRetail => "reit-retail",
                Industry::ReitSpecialty => "reit-specialty",
                Industry::CommunicationEquipment => "communication-equipment",
                Industry::ComputerHardware => "computer-hardware",
                Industry::ConsumerElectronics => "consumer-electronics",
                Industry::DataAnalytics => "data-analytics",
                Industry::ElectronicComponents => "electronic-components",
                Industry::ElectronicsAndComputerDistribution => "electronics-computer-distribution",
                Industry::HardwareAndSoftwareDistribution => "hardware-software-distribution",
                Industry::InformationTechnologyServices => "information-technology-services",
                Industry::InternetContentAndInformation => "internet-content-information",
                Industry::ScientificAndTechnicalInstruments => "scientific-technical-instruments",
                Industry::SemiconductorEquipmentAndMaterials => "semiconductor-equipment-materials",
                Industry::Semiconductors => "semiconductors",
                Industry::SoftwareApplication => "software-application",
                Industry::SoftwareInfrastructure => "software-infrastructure",
                Industry::Broadcasting => "broadcasting",
                Industry::Entertainment => "entertainment",
                Industry::Publishing => "publishing",
                Industry::TelecomServices => "telecom-services",
                Industry::UtilitiesDiversified => "utilities-diversified",
                Industry::UtilitiesIndependentPowerProducers => {
                    "utilities-independent-power-producers"
                }
                Industry::UtilitiesRegulatedElectric => "utilities-regulated-electric",
                Industry::UtilitiesRegulatedGas => "utilities-regulated-gas",
                Industry::UtilitiesRegulatedWater => "utilities-regulated-water",
                Industry::UtilitiesRenewable => "utilities-renewable",
                Industry::ClosedEndFundDebt => "closed-end-fund-debt",
                Industry::ClosedEndFundEquity => "closed-end-fund-equity",
                Industry::ClosedEndFundForeign => "closed-end-fund-foreign",
                Industry::ExchangeTradedFund => "exchange-traded-fund",
            }
        }

        /// Returns the display name used by the Yahoo Finance screener.
        ///
        /// # Example
        ///
        /// ```
        /// use finance_query::Industry;
        /// assert_eq!(Industry::Semiconductors.screener_value(), "Semiconductors");
        /// assert_eq!(Industry::OilAndGasDrilling.screener_value(), "Oil & Gas Drilling");
        /// ```
        pub fn screener_value(self) -> &'static str {
            match self {
                Industry::AgriculturalInputs => "Agricultural Inputs",
                Industry::Aluminum => "Aluminum",
                Industry::Coal => "Coal",
                Industry::Copper => "Copper",
                Industry::FarmProducts => "Farm Products",
                Industry::ForestProducts => "Forest Products",
                Industry::Gold => "Gold",
                Industry::LumberAndWoodProduction => "Lumber & Wood Production",
                Industry::OtherIndustrialMetalsAndMining => "Other Industrial Metals & Mining",
                Industry::OtherPreciousMetalsAndMining => "Other Precious Metals & Mining",
                Industry::Silver => "Silver",
                Industry::Steel => "Steel",
                Industry::ThermalCoal => "Thermal Coal",
                Industry::Uranium => "Uranium",
                Industry::ApparelManufacturing => "Apparel Manufacturing",
                Industry::ApparelRetail => "Apparel Retail",
                Industry::AutoAndTruckDealerships => "Auto & Truck Dealerships",
                Industry::AutoManufacturers => "Auto Manufacturers",
                Industry::AutoParts => "Auto Parts",
                Industry::BeveragesBrewers => "Beverages - Brewers",
                Industry::BeveragesNonAlcoholic => "Beverages - Non-Alcoholic",
                Industry::BeveragesWineriesAndDistilleries => "Beverages - Wineries & Distilleries",
                Industry::Confectioners => "Confectioners",
                Industry::DepartmentStores => "Department Stores",
                Industry::DiscountStores => "Discount Stores",
                Industry::ElectronicGamingAndMultimedia => "Electronic Gaming & Multimedia",
                Industry::FoodDistribution => "Food Distribution",
                Industry::FootwearAndAccessories => "Footwear & Accessories",
                Industry::FurnishingsFixturesAndAppliances => "Furnishings, Fixtures & Appliances",
                Industry::Gambling => "Gambling",
                Industry::GroceryStores => "Grocery Stores",
                Industry::HomeImprovementRetail => "Home Improvement Retail",
                Industry::HouseholdAndPersonalProducts => "Household & Personal Products",
                Industry::InternetRetail => "Internet Retail",
                Industry::Leisure => "Leisure",
                Industry::Lodging => "Lodging",
                Industry::LuxuryGoods => "Luxury Goods",
                Industry::PackagedFoods => "Packaged Foods",
                Industry::PersonalServices => "Personal Services",
                Industry::ResidentialConstruction => "Residential Construction",
                Industry::ResortsAndCasinos => "Resorts & Casinos",
                Industry::Restaurants => "Restaurants",
                Industry::SpecialtyRetail => "Specialty Retail",
                Industry::TextileManufacturing => "Textile Manufacturing",
                Industry::Tobacco => "Tobacco",
                Industry::TravelServices => "Travel Services",
                Industry::OilAndGasDrilling => "Oil & Gas Drilling",
                Industry::OilAndGasEAndP => "Oil & Gas E&P",
                Industry::OilAndGasEquipmentAndServices => "Oil & Gas Equipment & Services",
                Industry::OilAndGasIntegrated => "Oil & Gas Integrated",
                Industry::OilAndGasMidstream => "Oil & Gas Midstream",
                Industry::OilAndGasRefiningAndMarketing => "Oil & Gas Refining & Marketing",
                Industry::Solar => "Solar",
                Industry::AssetManagement => "Asset Management",
                Industry::BanksDiversified => "Banks - Diversified",
                Industry::BanksRegional => "Banks - Regional",
                Industry::CapitalMarkets => "Capital Markets",
                Industry::CreditServices => "Credit Services",
                Industry::FinancialDataAndStockExchanges => "Financial Data & Stock Exchanges",
                Industry::InsuranceBrokers => "Insurance Brokers",
                Industry::InsuranceDiversified => "Insurance - Diversified",
                Industry::InsuranceLife => "Insurance - Life",
                Industry::InsurancePropertyAndCasualty => "Insurance - Property & Casualty",
                Industry::InsuranceReinsurance => "Insurance - Reinsurance",
                Industry::InsuranceSpecialty => "Insurance - Specialty",
                Industry::MortgageFinance => "Mortgage Finance",
                Industry::ShellCompanies => "Shell Companies",
                Industry::Biotechnology => "Biotechnology",
                Industry::DiagnosticsAndResearch => "Diagnostics & Research",
                Industry::DrugManufacturersGeneral => "Drug Manufacturers - General",
                Industry::DrugManufacturersSpecialtyAndGeneric => {
                    "Drug Manufacturers - Specialty & Generic"
                }
                Industry::HealthInformationServices => "Health Information Services",
                Industry::HealthcarePlans => "Healthcare Plans",
                Industry::MedicalCareFacilities => "Medical Care Facilities",
                Industry::MedicalDevices => "Medical Devices",
                Industry::MedicalDistribution => "Medical Distribution",
                Industry::MedicalInstrumentsAndSupplies => "Medical Instruments & Supplies",
                Industry::PharmaceuticalRetailers => "Pharmaceutical Retailers",
                Industry::AerospaceAndDefense => "Aerospace & Defense",
                Industry::BuildingMaterials => "Building Materials",
                Industry::BuildingProductsAndEquipment => "Building Products & Equipment",
                Industry::BusinessEquipmentAndSupplies => "Business Equipment & Supplies",
                Industry::ChemicalManufacturing => "Chemical Manufacturing",
                Industry::Chemicals => "Chemicals",
                Industry::Conglomerates => "Conglomerates",
                Industry::ConsultingServices => "Consulting Services",
                Industry::ElectricalEquipmentAndParts => "Electrical Equipment & Parts",
                Industry::EngineeringAndConstruction => "Engineering & Construction",
                Industry::FarmAndHeavyConstructionMachinery => {
                    "Farm & Heavy Construction Machinery"
                }
                Industry::IndustrialDistribution => "Industrial Distribution",
                Industry::InfrastructureOperations => "Infrastructure Operations",
                Industry::IntegratedFreightAndLogistics => "Integrated Freight & Logistics",
                Industry::ManufacturingDiversified => "Manufacturing - Diversified",
                Industry::MarinePortsAndServices => "Marine Ports & Services",
                Industry::MarineShipping => "Marine Shipping",
                Industry::MetalFabrication => "Metal Fabrication",
                Industry::PaperAndPaperProducts => "Paper & Paper Products",
                Industry::PollutionAndTreatmentControls => "Pollution & Treatment Controls",
                Industry::Railroads => "Railroads",
                Industry::RentalAndLeasingServices => "Rental & Leasing Services",
                Industry::SecurityAndProtectionServices => "Security & Protection Services",
                Industry::SpecialtyBusinessServices => "Specialty Business Services",
                Industry::SpecialtyChemicals => "Specialty Chemicals",
                Industry::SpecialtyIndustrialMachinery => "Specialty Industrial Machinery",
                Industry::StaffingAndEmploymentServices => "Staffing & Employment Services",
                Industry::ToolsAndAccessories => "Tools & Accessories",
                Industry::Trucking => "Trucking",
                Industry::WasteManagement => "Waste Management",
                Industry::RealEstateDevelopment => "Real Estate - Development",
                Industry::RealEstateDiversified => "Real Estate - Diversified",
                Industry::RealEstateServices => "Real Estate Services",
                Industry::ReitDiversified => "REIT - Diversified",
                Industry::ReitHealthcareFacilities => "REIT - Healthcare Facilities",
                Industry::ReitHotelAndMotel => "REIT - Hotel & Motel",
                Industry::ReitIndustrial => "REIT - Industrial",
                Industry::ReitMortgage => "REIT - Mortgage",
                Industry::ReitOffice => "REIT - Office",
                Industry::ReitResidential => "REIT - Residential",
                Industry::ReitRetail => "REIT - Retail",
                Industry::ReitSpecialty => "REIT - Specialty",
                Industry::CommunicationEquipment => "Communication Equipment",
                Industry::ComputerHardware => "Computer Hardware",
                Industry::ConsumerElectronics => "Consumer Electronics",
                Industry::DataAnalytics => "Data Analytics",
                Industry::ElectronicComponents => "Electronic Components",
                Industry::ElectronicsAndComputerDistribution => {
                    "Electronics & Computer Distribution"
                }
                Industry::HardwareAndSoftwareDistribution => "Hardware & Software Distribution",
                Industry::InformationTechnologyServices => "Information Technology Services",
                Industry::InternetContentAndInformation => "Internet Content & Information",
                Industry::ScientificAndTechnicalInstruments => "Scientific & Technical Instruments",
                Industry::SemiconductorEquipmentAndMaterials => {
                    "Semiconductor Equipment & Materials"
                }
                Industry::Semiconductors => "Semiconductors",
                Industry::SoftwareApplication => "Software - Application",
                Industry::SoftwareInfrastructure => "Software - Infrastructure",
                Industry::Broadcasting => "Broadcasting",
                Industry::Entertainment => "Entertainment",
                Industry::Publishing => "Publishing",
                Industry::TelecomServices => "Telecom Services",
                Industry::UtilitiesDiversified => "Utilities - Diversified",
                Industry::UtilitiesIndependentPowerProducers => {
                    "Utilities - Independent Power Producers"
                }
                Industry::UtilitiesRegulatedElectric => "Utilities - Regulated Electric",
                Industry::UtilitiesRegulatedGas => "Utilities - Regulated Gas",
                Industry::UtilitiesRegulatedWater => "Utilities - Regulated Water",
                Industry::UtilitiesRenewable => "Utilities - Renewable",
                Industry::ClosedEndFundDebt => "Closed-End Fund - Debt",
                Industry::ClosedEndFundEquity => "Closed-End Fund - Equity",
                Industry::ClosedEndFundForeign => "Closed-End Fund - Foreign",
                Industry::ExchangeTradedFund => "Exchange Traded Fund",
            }
        }
    }

    impl AsRef<str> for Industry {
        /// Returns the slug, enabling `finance::industry(Industry::Semiconductors)`.
        fn as_ref(&self) -> &str {
            self.as_slug()
        }
    }

    impl From<Industry> for String {
        /// Returns the screener display name, enabling `EquityField::Industry.eq_str(Industry::Semiconductors)`.
        fn from(v: Industry) -> Self {
            v.screener_value().to_string()
        }
    }
}

/// Typed exchange codes for screener queries.
///
/// Use with [`EquityField::Exchange`](crate::EquityField::Exchange) or
/// [`FundField::Exchange`](crate::FundField::Exchange) and
/// [`ScreenerFieldExt::eq_str`](crate::ScreenerFieldExt::eq_str).
///
/// For mutual fund queries use [`ExchangeCode::Nas`].
///
/// # Example
///
/// ```
/// use finance_query::{EquityField, EquityScreenerQuery, ScreenerFieldExt, ExchangeCode};
///
/// let query = EquityScreenerQuery::new()
///     .add_condition(EquityField::Exchange.eq_str(ExchangeCode::Nms));
/// ```
pub mod exchange_codes {
    /// Typed exchange code for screener queries.
    #[non_exhaustive]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum ExchangeCode {
        // ── US Equity ─────────────────────────────────────────────────────────
        /// NYSE American / AMEX ("ASE")
        Ase,
        /// OTC Bulletin Board ("BTS")
        Bts,
        /// NASDAQ Capital Market ("NCM")
        Ncm,
        /// NASDAQ Global Market ("NGM")
        Ngm,
        /// NASDAQ Global Select Market ("NMS") — primary NASDAQ tier
        Nms,
        /// New York Stock Exchange ("NYQ")
        Nyq,
        /// NYSE Arca ("PCX")
        Pcx,
        /// OTC Pink Sheets / OTC Markets ("PNK")
        Pnk,
        // ── US Funds ──────────────────────────────────────────────────────────
        /// NASDAQ — used for US mutual funds ("NAS")
        Nas,
        // ── International ─────────────────────────────────────────────────────
        /// Australian Securities Exchange ("ASX")
        Asx,
        /// Bombay Stock Exchange ("BSE")
        Bse,
        /// Hong Kong Stock Exchange ("HKG")
        Hkg,
        /// Korea Exchange ("KRX")
        Krx,
        /// London Stock Exchange ("LSE")
        Lse,
        /// National Stock Exchange of India ("NSI")
        Nsi,
        /// Shanghai Stock Exchange ("SHH")
        Shh,
        /// Shenzhen Stock Exchange ("SHZ")
        Shz,
        /// Tokyo Stock Exchange ("TYO")
        Tyo,
        /// Toronto Stock Exchange ("TOR")
        Tor,
        /// XETRA / Deutsche Börse ("GER")
        Ger,
    }

    impl ExchangeCode {
        /// Returns the exchange code string used by Yahoo Finance.
        pub fn as_str(self) -> &'static str {
            match self {
                ExchangeCode::Ase => "ASE",
                ExchangeCode::Bts => "BTS",
                ExchangeCode::Ncm => "NCM",
                ExchangeCode::Ngm => "NGM",
                ExchangeCode::Nms => "NMS",
                ExchangeCode::Nyq => "NYQ",
                ExchangeCode::Pcx => "PCX",
                ExchangeCode::Pnk => "PNK",
                ExchangeCode::Nas => "NAS",
                ExchangeCode::Asx => "ASX",
                ExchangeCode::Bse => "BSE",
                ExchangeCode::Hkg => "HKG",
                ExchangeCode::Krx => "KRX",
                ExchangeCode::Lse => "LSE",
                ExchangeCode::Nsi => "NSI",
                ExchangeCode::Shh => "SHH",
                ExchangeCode::Shz => "SHZ",
                ExchangeCode::Tyo => "TYO",
                ExchangeCode::Tor => "TOR",
                ExchangeCode::Ger => "GER",
            }
        }
    }

    impl From<ExchangeCode> for String {
        fn from(v: ExchangeCode) -> Self {
            v.as_str().to_string()
        }
    }
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
}
