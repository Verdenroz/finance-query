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
            YAHOO_FINANCE_QUERY2, symbol
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
            YAHOO_FINANCE_QUERY2, symbol
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

    /// Market hours/time endpoint
    pub const MARKET_TIME: &str =
        const_format::concatcp!(YAHOO_FINANCE_QUERY1, "/v6/finance/markettime");

    /// Currencies endpoint
    pub const CURRENCIES: &str =
        const_format::concatcp!(YAHOO_FINANCE_QUERY2, "/v1/finance/currencies");

    /// Market summary endpoint
    pub const MARKET_SUMMARY: &str =
        const_format::concatcp!(YAHOO_FINANCE_QUERY2, "/v6/finance/quote/marketSummary");

    /// Trending tickers endpoint (requires region suffix)
    pub fn trending(region: &str) -> String {
        format!("{}/v1/finance/trending/{}", YAHOO_FINANCE_QUERY2, region)
    }
}

/// URL builders (functions that construct full URLs with query params)
pub mod url_builders {
    use super::screener_types::ScreenerType;
    use super::urls::*;

    /// Screener endpoint for predefined screeners
    pub fn screener(screener_type: ScreenerType, count: u32) -> String {
        format!(
            "{}/v1/finance/screener/predefined/saved?count={}&formatted=true&scrIds={}",
            YAHOO_FINANCE_QUERY1,
            count,
            screener_type.as_scr_id()
        )
    }

    /// Custom screener endpoint (POST)
    pub fn custom_screener() -> String {
        format!(
            "{}/v1/finance/screener?formatted=true&useRecordsResponse=true&lang=en-US&region=US",
            YAHOO_FINANCE_QUERY1
        )
    }

    /// Sector details endpoint
    pub fn sector(sector_key: &str) -> String {
        format!(
            "{}/v1/finance/sectors/{}?formatted=true&withReturns=false&lang=en-US&region=US",
            YAHOO_FINANCE_QUERY1, sector_key
        )
    }

    /// Industries endpoint - detailed industry data
    pub fn industry(industry_key: &str) -> String {
        format!(
            "{}/v1/finance/industries/{}?formatted=true&withReturns=false&lang=en-US&region=US",
            YAHOO_FINANCE_QUERY1, industry_key
        )
    }
}

/// Predefined screener types for Yahoo Finance
pub mod screener_types {
    /// Enum of all predefined Yahoo Finance screeners
    ///
    /// These map to Yahoo Finance's predefined screener IDs and can be used
    /// to fetch filtered stock/fund lists based on various criteria.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum ScreenerType {
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

    impl ScreenerType {
        /// Convert to Yahoo Finance scrId parameter value (SCREAMING_SNAKE_CASE)
        pub fn as_scr_id(&self) -> &'static str {
            match self {
                ScreenerType::AggressiveSmallCaps => "aggressive_small_caps",
                ScreenerType::DayGainers => "day_gainers",
                ScreenerType::DayLosers => "day_losers",
                ScreenerType::GrowthTechnologyStocks => "growth_technology_stocks",
                ScreenerType::MostActives => "most_actives",
                ScreenerType::MostShortedStocks => "most_shorted_stocks",
                ScreenerType::SmallCapGainers => "small_cap_gainers",
                ScreenerType::UndervaluedGrowthStocks => "undervalued_growth_stocks",
                ScreenerType::UndervaluedLargeCaps => "undervalued_large_caps",
                ScreenerType::ConservativeForeignFunds => "conservative_foreign_funds",
                ScreenerType::HighYieldBond => "high_yield_bond",
                ScreenerType::PortfolioAnchors => "portfolio_anchors",
                ScreenerType::SolidLargeGrowthFunds => "solid_large_growth_funds",
                ScreenerType::SolidMidcapGrowthFunds => "solid_midcap_growth_funds",
                ScreenerType::TopMutualFunds => "top_mutual_funds",
            }
        }

        /// Parse from string, returns None on invalid input
        ///
        /// # Example
        /// ```
        /// use finance_query::ScreenerType;
        ///
        /// assert_eq!(ScreenerType::parse("most-actives"), Some(ScreenerType::MostActives));
        /// assert_eq!(ScreenerType::parse("day-gainers"), Some(ScreenerType::DayGainers));
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
        pub fn all() -> &'static [ScreenerType] {
            &[
                ScreenerType::AggressiveSmallCaps,
                ScreenerType::DayGainers,
                ScreenerType::DayLosers,
                ScreenerType::GrowthTechnologyStocks,
                ScreenerType::MostActives,
                ScreenerType::MostShortedStocks,
                ScreenerType::SmallCapGainers,
                ScreenerType::UndervaluedGrowthStocks,
                ScreenerType::UndervaluedLargeCaps,
                ScreenerType::ConservativeForeignFunds,
                ScreenerType::HighYieldBond,
                ScreenerType::PortfolioAnchors,
                ScreenerType::SolidLargeGrowthFunds,
                ScreenerType::SolidMidcapGrowthFunds,
                ScreenerType::TopMutualFunds,
            ]
        }
    }

    impl std::str::FromStr for ScreenerType {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_lowercase().replace('_', "-").as_str() {
                "aggressive-small-caps" => Ok(ScreenerType::AggressiveSmallCaps),
                "day-gainers" | "gainers" => Ok(ScreenerType::DayGainers),
                "day-losers" | "losers" => Ok(ScreenerType::DayLosers),
                "growth-technology-stocks" | "growth-tech" => {
                    Ok(ScreenerType::GrowthTechnologyStocks)
                }
                "most-actives" | "actives" => Ok(ScreenerType::MostActives),
                "most-shorted-stocks" | "most-shorted" => Ok(ScreenerType::MostShortedStocks),
                "small-cap-gainers" => Ok(ScreenerType::SmallCapGainers),
                "undervalued-growth-stocks" | "undervalued-growth" => {
                    Ok(ScreenerType::UndervaluedGrowthStocks)
                }
                "undervalued-large-caps" | "undervalued-large" => {
                    Ok(ScreenerType::UndervaluedLargeCaps)
                }
                "conservative-foreign-funds" => Ok(ScreenerType::ConservativeForeignFunds),
                "high-yield-bond" => Ok(ScreenerType::HighYieldBond),
                "portfolio-anchors" => Ok(ScreenerType::PortfolioAnchors),
                "solid-large-growth-funds" => Ok(ScreenerType::SolidLargeGrowthFunds),
                "solid-midcap-growth-funds" => Ok(ScreenerType::SolidMidcapGrowthFunds),
                "top-mutual-funds" => Ok(ScreenerType::TopMutualFunds),
                _ => Err(()),
            }
        }
    }
}

/// Yahoo Finance sector types
///
/// These are the 11 GICS sectors available on Yahoo Finance.
pub mod sector_types {
    use serde::{Deserialize, Serialize};

    /// Market sector types available on Yahoo Finance
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum SectorType {
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

    impl SectorType {
        /// Convert to Yahoo Finance API path segment (lowercase with hyphens)
        pub fn as_api_path(&self) -> &'static str {
            match self {
                SectorType::Technology => "technology",
                SectorType::FinancialServices => "financial-services",
                SectorType::ConsumerCyclical => "consumer-cyclical",
                SectorType::CommunicationServices => "communication-services",
                SectorType::Healthcare => "healthcare",
                SectorType::Industrials => "industrials",
                SectorType::ConsumerDefensive => "consumer-defensive",
                SectorType::Energy => "energy",
                SectorType::BasicMaterials => "basic-materials",
                SectorType::RealEstate => "real-estate",
                SectorType::Utilities => "utilities",
            }
        }

        /// Get human-readable display name
        pub fn display_name(&self) -> &'static str {
            match self {
                SectorType::Technology => "Technology",
                SectorType::FinancialServices => "Financial Services",
                SectorType::ConsumerCyclical => "Consumer Cyclical",
                SectorType::CommunicationServices => "Communication Services",
                SectorType::Healthcare => "Healthcare",
                SectorType::Industrials => "Industrials",
                SectorType::ConsumerDefensive => "Consumer Defensive",
                SectorType::Energy => "Energy",
                SectorType::BasicMaterials => "Basic Materials",
                SectorType::RealEstate => "Real Estate",
                SectorType::Utilities => "Utilities",
            }
        }

        /// List all valid sector types for error messages
        pub fn valid_types() -> &'static str {
            "technology, financial-services, consumer-cyclical, communication-services, \
             healthcare, industrials, consumer-defensive, energy, basic-materials, \
             real-estate, utilities"
        }

        /// Get all sector types as an array
        pub fn all() -> &'static [SectorType] {
            &[
                SectorType::Technology,
                SectorType::FinancialServices,
                SectorType::ConsumerCyclical,
                SectorType::CommunicationServices,
                SectorType::Healthcare,
                SectorType::Industrials,
                SectorType::ConsumerDefensive,
                SectorType::Energy,
                SectorType::BasicMaterials,
                SectorType::RealEstate,
                SectorType::Utilities,
            ]
        }
    }

    impl std::str::FromStr for SectorType {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_lowercase().replace('_', "-").as_str() {
                "technology" | "tech" => Ok(SectorType::Technology),
                "financial-services" | "financials" | "financial" => {
                    Ok(SectorType::FinancialServices)
                }
                "consumer-cyclical" => Ok(SectorType::ConsumerCyclical),
                "communication-services" | "communication" => Ok(SectorType::CommunicationServices),
                "healthcare" | "health" => Ok(SectorType::Healthcare),
                "industrials" | "industrial" => Ok(SectorType::Industrials),
                "consumer-defensive" => Ok(SectorType::ConsumerDefensive),
                "energy" => Ok(SectorType::Energy),
                "basic-materials" | "materials" => Ok(SectorType::BasicMaterials),
                "real-estate" | "realestate" => Ok(SectorType::RealEstate),
                "utilities" | "utility" => Ok(SectorType::Utilities),
                _ => Err(()),
            }
        }
    }

    impl std::fmt::Display for SectorType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.display_name())
        }
    }
}

/// Custom screener query types and operators
///
/// Used to build custom screener queries with flexible filtering criteria.
pub mod screener_query {
    use serde::{Deserialize, Serialize};

    /// Quote type for custom screeners
    ///
    /// Yahoo Finance only supports EQUITY and MUTUALFUND for custom screener queries.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum QuoteType {
        /// Equity (stocks) - uses equity_fields for validation
        #[default]
        #[serde(rename = "EQUITY")]
        Equity,
        /// Mutual funds - uses fund_fields for validation
        #[serde(rename = "MUTUALFUND")]
        MutualFund,
    }

    impl QuoteType {
        /// Get valid values for this quote type
        pub fn valid_types() -> &'static str {
            "equity, mutualfund"
        }
    }

    impl std::str::FromStr for QuoteType {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_lowercase().replace(['-', '_'], "").as_str() {
                "equity" | "stock" | "stocks" => Ok(QuoteType::Equity),
                "mutualfund" | "fund" | "funds" => Ok(QuoteType::MutualFund),
                _ => Err(()),
            }
        }
    }

    /// Sort direction for custom screener results
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
    #[serde(rename_all = "UPPERCASE")]
    pub enum SortType {
        /// Sort ascending (smallest first)
        #[serde(rename = "ASC")]
        Asc,
        /// Sort descending (largest first)
        #[default]
        #[serde(rename = "DESC")]
        Desc,
    }

    impl std::str::FromStr for SortType {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_lowercase().as_str() {
                "asc" | "ascending" => Ok(SortType::Asc),
                "desc" | "descending" => Ok(SortType::Desc),
                _ => Err(()),
            }
        }
    }

    /// Comparison operator for query conditions
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Operator {
        /// Equal to
        #[serde(rename = "eq")]
        Eq,
        /// Greater than
        #[serde(rename = "gt")]
        Gt,
        /// Greater than or equal to
        #[serde(rename = "gte")]
        Gte,
        /// Less than
        #[serde(rename = "lt")]
        Lt,
        /// Less than or equal to
        #[serde(rename = "lte")]
        Lte,
        /// Between two values (inclusive)
        #[serde(rename = "btwn")]
        Between,
    }

    impl std::str::FromStr for Operator {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_lowercase().as_str() {
                "eq" | "=" | "==" => Ok(Operator::Eq),
                "gt" | ">" => Ok(Operator::Gt),
                "gte" | ">=" => Ok(Operator::Gte),
                "lt" | "<" => Ok(Operator::Lt),
                "lte" | "<=" => Ok(Operator::Lte),
                "btwn" | "between" => Ok(Operator::Between),
                _ => Err(()),
            }
        }
    }

    /// Logical operator for combining conditions
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum LogicalOperator {
        /// All conditions must match (AND)
        #[default]
        And,
        /// Any condition can match (OR)
        Or,
    }

    impl std::str::FromStr for LogicalOperator {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_lowercase().as_str() {
                "and" | "&&" => Ok(LogicalOperator::And),
                "or" | "||" => Ok(LogicalOperator::Or),
                _ => Err(()),
            }
        }
    }

    /// Valid screener fields for equity queries
    ///
    /// Based on Yahoo Finance screener API. Fields grouped by category.
    #[allow(missing_docs)]
    pub mod equity_fields {
        // Price fields
        pub const EOD_PRICE: &str = "eodprice";
        pub const INTRADAY_PRICE_CHANGE: &str = "intradaypricechange";
        pub const INTRADAY_PRICE: &str = "intradayprice";
        pub const PERCENT_CHANGE: &str = "percentchange";
        pub const LASTCLOSE_52WK_HIGH: &str = "lastclose52weekhigh.lasttwelvemonths";
        pub const FIFTY_TWO_WK_PCT_CHANGE: &str = "fiftytwowkpercentchange";
        pub const LASTCLOSE_52WK_LOW: &str = "lastclose52weeklow.lasttwelvemonths";
        pub const INTRADAY_MARKET_CAP: &str = "intradaymarketcap";
        pub const LASTCLOSE_MARKET_CAP: &str = "lastclosemarketcap.lasttwelvemonths";

        // Equality filter fields
        pub const REGION: &str = "region";
        pub const SECTOR: &str = "sector";
        pub const PEER_GROUP: &str = "peer_group";
        pub const INDUSTRY: &str = "industry";
        pub const EXCHANGE: &str = "exchange";

        // Trading fields
        pub const BETA: &str = "beta";
        pub const AVG_DAILY_VOL_3M: &str = "avgdailyvol3m";
        pub const PCT_HELD_INSIDER: &str = "pctheldinsider";
        pub const PCT_HELD_INST: &str = "pctheldinst";
        pub const DAY_VOLUME: &str = "dayvolume";
        pub const EOD_VOLUME: &str = "eodvolume";

        // Short interest fields
        pub const SHORT_PCT_SHARES_OUT: &str = "short_percentage_of_shares_outstanding.value";
        pub const SHORT_INTEREST: &str = "short_interest.value";
        pub const SHORT_PCT_FLOAT: &str = "short_percentage_of_float.value";
        pub const DAYS_TO_COVER: &str = "days_to_cover_short.value";
        pub const SHORT_INTEREST_PCT_CHANGE: &str = "short_interest_percentage_change.value";

        // Valuation fields
        pub const BOOK_VALUE_SHARE: &str = "bookvalueshare.lasttwelvemonths";
        pub const MARKET_CAP_TO_REVENUE: &str = "lastclosemarketcaptotalrevenue.lasttwelvemonths";
        pub const TEV_TO_REVENUE: &str = "lastclosetevtotalrevenue.lasttwelvemonths";
        pub const PRICE_BOOK_RATIO: &str = "pricebookratio.quarterly";
        pub const PE_RATIO: &str = "peratio.lasttwelvemonths";
        pub const PRICE_TANGIBLE_BOOK: &str = "lastclosepricetangiblebookvalue.lasttwelvemonths";
        pub const PRICE_EARNINGS: &str = "lastclosepriceearnings.lasttwelvemonths";
        pub const PEG_RATIO_5Y: &str = "pegratio_5y";

        // Profitability fields
        pub const CONSECUTIVE_DIV_YEARS: &str = "consecutive_years_of_dividend_growth_count";
        pub const ROA: &str = "returnonassets.lasttwelvemonths";
        pub const ROE: &str = "returnonequity.lasttwelvemonths";
        pub const FORWARD_DIV_PER_SHARE: &str = "forward_dividend_per_share";
        pub const FORWARD_DIV_YIELD: &str = "forward_dividend_yield";
        pub const RETURN_ON_CAPITAL: &str = "returnontotalcapital.lasttwelvemonths";

        // Leverage fields
        pub const TEV_EBIT: &str = "lastclosetevebit.lasttwelvemonths";
        pub const NET_DEBT_EBITDA: &str = "netdebtebitda.lasttwelvemonths";
        pub const TOTAL_DEBT_EQUITY: &str = "totaldebtequity.lasttwelvemonths";
        pub const LT_DEBT_EQUITY: &str = "ltdebtequity.lasttwelvemonths";
        pub const EBIT_INTEREST_EXP: &str = "ebitinterestexpense.lasttwelvemonths";
        pub const EBITDA_INTEREST_EXP: &str = "ebitdainterestexpense.lasttwelvemonths";
        pub const TEV_EBITDA: &str = "lastclosetevebitda.lasttwelvemonths";
        pub const TOTAL_DEBT_EBITDA: &str = "totaldebtebitda.lasttwelvemonths";

        // Liquidity fields
        pub const QUICK_RATIO: &str = "quickratio.lasttwelvemonths";
        pub const ALTMAN_Z_SCORE: &str =
            "altmanzscoreusingtheaveragestockinformationforaperiod.lasttwelvemonths";
        pub const CURRENT_RATIO: &str = "currentratio.lasttwelvemonths";
        pub const OCF_TO_CURRENT_LIAB: &str =
            "operatingcashflowtocurrentliabilities.lasttwelvemonths";

        // Income statement fields
        pub const TOTAL_REVENUES: &str = "totalrevenues.lasttwelvemonths";
        pub const NET_INCOME_MARGIN: &str = "netincomemargin.lasttwelvemonths";
        pub const GROSS_PROFIT: &str = "grossprofit.lasttwelvemonths";
        pub const EBITDA_1YR_GROWTH: &str = "ebitda1yrgrowth.lasttwelvemonths";
        pub const DILUTED_EPS_CONT_OPS: &str = "dilutedepscontinuingoperations.lasttwelvemonths";
        pub const QUARTERLY_REV_GROWTH: &str = "quarterlyrevenuegrowth.quarterly";
        pub const EPS_GROWTH: &str = "epsgrowth.lasttwelvemonths";
        pub const NET_INCOME: &str = "netincomeis.lasttwelvemonths";
        pub const EBITDA: &str = "ebitda.lasttwelvemonths";
        pub const DILUTED_EPS_1YR_GROWTH: &str = "dilutedeps1yrgrowth.lasttwelvemonths";
        pub const REVENUE_1YR_GROWTH: &str = "totalrevenues1yrgrowth.lasttwelvemonths";
        pub const OPERATING_INCOME: &str = "operatingincome.lasttwelvemonths";
        pub const NET_INCOME_1YR_GROWTH: &str = "netincome1yrgrowth.lasttwelvemonths";
        pub const GROSS_PROFIT_MARGIN: &str = "grossprofitmargin.lasttwelvemonths";
        pub const EBITDA_MARGIN: &str = "ebitdamargin.lasttwelvemonths";
        pub const EBIT: &str = "ebit.lasttwelvemonths";
        pub const BASIC_EPS_CONT_OPS: &str = "basicepscontinuingoperations.lasttwelvemonths";
        pub const NET_EPS_BASIC: &str = "netepsbasic.lasttwelvemonths";
        pub const NET_EPS_DILUTED: &str = "netepsdiluted.lasttwelvemonths";

        // Balance sheet fields
        pub const TOTAL_ASSETS: &str = "totalassets.lasttwelvemonths";
        pub const COMMON_SHARES_OUT: &str = "totalcommonsharesoutstanding.lasttwelvemonths";
        pub const TOTAL_DEBT: &str = "totaldebt.lasttwelvemonths";
        pub const TOTAL_EQUITY: &str = "totalequity.lasttwelvemonths";
        pub const TOTAL_CURRENT_ASSETS: &str = "totalcurrentassets.lasttwelvemonths";
        pub const CASH_AND_ST_INVESTMENTS: &str =
            "totalcashandshortterminvestments.lasttwelvemonths";
        pub const TOTAL_COMMON_EQUITY: &str = "totalcommonequity.lasttwelvemonths";
        pub const TOTAL_CURRENT_LIAB: &str = "totalcurrentliabilities.lasttwelvemonths";
        pub const TOTAL_SHARES_OUT: &str = "totalsharesoutstanding";

        // Cash flow fields
        pub const LEVERED_FCF: &str = "leveredfreecashflow.lasttwelvemonths";
        pub const CAPEX: &str = "capitalexpenditure.lasttwelvemonths";
        pub const CASH_FROM_OPS: &str = "cashfromoperations.lasttwelvemonths";
        pub const LEVERED_FCF_1YR_GROWTH: &str = "leveredfreecashflow1yrgrowth.lasttwelvemonths";
        pub const UNLEVERED_FCF: &str = "unleveredfreecashflow.lasttwelvemonths";
        pub const CASH_FROM_OPS_1YR_GROWTH: &str = "cashfromoperations1yrgrowth.lasttwelvemonths";

        // ESG fields
        pub const ESG_SCORE: &str = "esg_score";
        pub const ENVIRONMENTAL_SCORE: &str = "environmental_score";
        pub const GOVERNANCE_SCORE: &str = "governance_score";
        pub const SOCIAL_SCORE: &str = "social_score";
        pub const HIGHEST_CONTROVERSY: &str = "highest_controversy";
    }

    /// Valid screener fields for fund/mutual fund queries
    #[allow(missing_docs)]
    pub mod fund_fields {
        // Common price fields (shared with equity)
        pub const EOD_PRICE: &str = "eodprice";
        pub const INTRADAY_PRICE_CHANGE: &str = "intradaypricechange";
        pub const INTRADAY_PRICE: &str = "intradayprice";

        // Fund-specific fields
        pub const CATEGORY_NAME: &str = "categoryname";
        pub const PERFORMANCE_RATING: &str = "performanceratingoverall";
        pub const INITIAL_INVESTMENT: &str = "initialinvestment";
        pub const ANNUAL_RETURN_RANK: &str = "annualreturnnavy1categoryrank";
        pub const RISK_RATING: &str = "riskratingoverall";
        pub const EXCHANGE: &str = "exchange";
    }

    /// All valid equity screener fields (for validation)
    pub const VALID_EQUITY_FIELDS: &[&str] = &[
        equity_fields::EOD_PRICE,
        equity_fields::INTRADAY_PRICE_CHANGE,
        equity_fields::INTRADAY_PRICE,
        equity_fields::PERCENT_CHANGE,
        equity_fields::LASTCLOSE_52WK_HIGH,
        equity_fields::FIFTY_TWO_WK_PCT_CHANGE,
        equity_fields::LASTCLOSE_52WK_LOW,
        equity_fields::INTRADAY_MARKET_CAP,
        equity_fields::LASTCLOSE_MARKET_CAP,
        equity_fields::REGION,
        equity_fields::SECTOR,
        equity_fields::PEER_GROUP,
        equity_fields::INDUSTRY,
        equity_fields::EXCHANGE,
        equity_fields::BETA,
        equity_fields::AVG_DAILY_VOL_3M,
        equity_fields::PCT_HELD_INSIDER,
        equity_fields::PCT_HELD_INST,
        equity_fields::DAY_VOLUME,
        equity_fields::EOD_VOLUME,
        equity_fields::SHORT_PCT_SHARES_OUT,
        equity_fields::SHORT_INTEREST,
        equity_fields::SHORT_PCT_FLOAT,
        equity_fields::DAYS_TO_COVER,
        equity_fields::SHORT_INTEREST_PCT_CHANGE,
        equity_fields::BOOK_VALUE_SHARE,
        equity_fields::MARKET_CAP_TO_REVENUE,
        equity_fields::TEV_TO_REVENUE,
        equity_fields::PRICE_BOOK_RATIO,
        equity_fields::PE_RATIO,
        equity_fields::PRICE_TANGIBLE_BOOK,
        equity_fields::PRICE_EARNINGS,
        equity_fields::PEG_RATIO_5Y,
        equity_fields::CONSECUTIVE_DIV_YEARS,
        equity_fields::ROA,
        equity_fields::ROE,
        equity_fields::FORWARD_DIV_PER_SHARE,
        equity_fields::FORWARD_DIV_YIELD,
        equity_fields::RETURN_ON_CAPITAL,
        equity_fields::TEV_EBIT,
        equity_fields::NET_DEBT_EBITDA,
        equity_fields::TOTAL_DEBT_EQUITY,
        equity_fields::LT_DEBT_EQUITY,
        equity_fields::EBIT_INTEREST_EXP,
        equity_fields::EBITDA_INTEREST_EXP,
        equity_fields::TEV_EBITDA,
        equity_fields::TOTAL_DEBT_EBITDA,
        equity_fields::QUICK_RATIO,
        equity_fields::ALTMAN_Z_SCORE,
        equity_fields::CURRENT_RATIO,
        equity_fields::OCF_TO_CURRENT_LIAB,
        equity_fields::TOTAL_REVENUES,
        equity_fields::NET_INCOME_MARGIN,
        equity_fields::GROSS_PROFIT,
        equity_fields::EBITDA_1YR_GROWTH,
        equity_fields::DILUTED_EPS_CONT_OPS,
        equity_fields::QUARTERLY_REV_GROWTH,
        equity_fields::EPS_GROWTH,
        equity_fields::NET_INCOME,
        equity_fields::EBITDA,
        equity_fields::DILUTED_EPS_1YR_GROWTH,
        equity_fields::REVENUE_1YR_GROWTH,
        equity_fields::OPERATING_INCOME,
        equity_fields::NET_INCOME_1YR_GROWTH,
        equity_fields::GROSS_PROFIT_MARGIN,
        equity_fields::EBITDA_MARGIN,
        equity_fields::EBIT,
        equity_fields::BASIC_EPS_CONT_OPS,
        equity_fields::NET_EPS_BASIC,
        equity_fields::NET_EPS_DILUTED,
        equity_fields::TOTAL_ASSETS,
        equity_fields::COMMON_SHARES_OUT,
        equity_fields::TOTAL_DEBT,
        equity_fields::TOTAL_EQUITY,
        equity_fields::TOTAL_CURRENT_ASSETS,
        equity_fields::CASH_AND_ST_INVESTMENTS,
        equity_fields::TOTAL_COMMON_EQUITY,
        equity_fields::TOTAL_CURRENT_LIAB,
        equity_fields::TOTAL_SHARES_OUT,
        equity_fields::LEVERED_FCF,
        equity_fields::CAPEX,
        equity_fields::CASH_FROM_OPS,
        equity_fields::LEVERED_FCF_1YR_GROWTH,
        equity_fields::UNLEVERED_FCF,
        equity_fields::CASH_FROM_OPS_1YR_GROWTH,
        equity_fields::ESG_SCORE,
        equity_fields::ENVIRONMENTAL_SCORE,
        equity_fields::GOVERNANCE_SCORE,
        equity_fields::SOCIAL_SCORE,
        equity_fields::HIGHEST_CONTROVERSY,
    ];

    /// All valid fund screener fields (for validation)
    pub const VALID_FUND_FIELDS: &[&str] = &[
        fund_fields::EOD_PRICE,
        fund_fields::INTRADAY_PRICE_CHANGE,
        fund_fields::INTRADAY_PRICE,
        fund_fields::CATEGORY_NAME,
        fund_fields::PERFORMANCE_RATING,
        fund_fields::INITIAL_INVESTMENT,
        fund_fields::ANNUAL_RETURN_RANK,
        fund_fields::RISK_RATING,
        fund_fields::EXCHANGE,
    ];

    /// Check if a field is valid for equity screeners
    pub fn is_valid_equity_field(field: &str) -> bool {
        VALID_EQUITY_FIELDS.contains(&field)
    }

    /// Check if a field is valid for fund screeners
    pub fn is_valid_fund_field(field: &str) -> bool {
        VALID_FUND_FIELDS.contains(&field)
    }

    /// Check if a field is valid for the given quote type
    pub fn is_valid_field(field: &str, quote_type: QuoteType) -> bool {
        match quote_type {
            QuoteType::Equity => is_valid_equity_field(field),
            QuoteType::MutualFund => is_valid_fund_field(field),
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

impl Country {
    /// Get the language code for this country
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::Country;
    ///
    /// assert_eq!(Country::France.lang(), "fr-FR");
    /// assert_eq!(Country::UnitedStates.lang(), "en-US");
    /// ```
    pub fn lang(&self) -> &'static str {
        match self {
            Country::Argentina => "es-AR",
            Country::Australia => "en-AU",
            Country::Brazil => "pt-BR",
            Country::Canada => "en-CA",
            Country::China => "zh-CN",
            Country::Denmark => "da-DK",
            Country::Finland => "fi-FI",
            Country::France => "fr-FR",
            Country::Germany => "de-DE",
            Country::Greece => "el-GR",
            Country::HongKong => "zh-Hant-HK",
            Country::India => "en-IN",
            Country::Israel => "he-IL",
            Country::Italy => "it-IT",
            Country::Malaysia => "ms-MY",
            Country::NewZealand => "en-NZ",
            Country::Norway => "nb-NO",
            Country::Portugal => "pt-PT",
            Country::Russia => "ru-RU",
            Country::Singapore => "en-SG",
            Country::Spain => "es-ES",
            Country::Sweden => "sv-SE",
            Country::Taiwan => "zh-TW",
            Country::Thailand => "th-TH",
            Country::Turkey => "tr-TR",
            Country::UnitedKingdom => "en-GB",
            Country::UnitedStates => "en-US",
            Country::Vietnam => "vi-VN",
        }
    }

    /// Get the region code for this country
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::Country;
    ///
    /// assert_eq!(Country::France.region(), "FR");
    /// assert_eq!(Country::UnitedStates.region(), "US");
    /// ```
    pub fn region(&self) -> &'static str {
        match self {
            Country::Argentina => "AR",
            Country::Australia => "AU",
            Country::Brazil => "BR",
            Country::Canada => "CA",
            Country::China => "CN",
            Country::Denmark => "DK",
            Country::Finland => "FI",
            Country::France => "FR",
            Country::Germany => "DE",
            Country::Greece => "GR",
            Country::HongKong => "HK",
            Country::India => "IN",
            Country::Israel => "IL",
            Country::Italy => "IT",
            Country::Malaysia => "MY",
            Country::NewZealand => "NZ",
            Country::Norway => "NO",
            Country::Portugal => "PT",
            Country::Russia => "RU",
            Country::Singapore => "SG",
            Country::Spain => "ES",
            Country::Sweden => "SE",
            Country::Taiwan => "TW",
            Country::Thailand => "TH",
            Country::Turkey => "TR",
            Country::UnitedKingdom => "GB",
            Country::UnitedStates => "US",
            Country::Vietnam => "VN",
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
    /// assert_eq!(Country::France.cors_domain(), "fr.finance.yahoo.com");
    /// ```
    pub fn cors_domain(&self) -> &'static str {
        match self {
            Country::Argentina => "ar.finance.yahoo.com",
            Country::Australia => "au.finance.yahoo.com",
            Country::Brazil => "br.financas.yahoo.com",
            Country::Canada => "ca.finance.yahoo.com",
            Country::China => "cn.finance.yahoo.com",
            Country::Denmark => "dk.finance.yahoo.com",
            Country::Finland => "fi.finance.yahoo.com",
            Country::France => "fr.finance.yahoo.com",
            Country::Germany => "de.finance.yahoo.com",
            Country::Greece => "gr.finance.yahoo.com",
            Country::HongKong => "hk.finance.yahoo.com",
            Country::India => "in.finance.yahoo.com",
            Country::Israel => "il.finance.yahoo.com",
            Country::Italy => "it.finance.yahoo.com",
            Country::Malaysia => "my.finance.yahoo.com",
            Country::NewZealand => "nz.finance.yahoo.com",
            Country::Norway => "no.finance.yahoo.com",
            Country::Portugal => "pt.finance.yahoo.com",
            Country::Russia => "ru.finance.yahoo.com",
            Country::Singapore => "sg.finance.yahoo.com",
            Country::Spain => "es.finance.yahoo.com",
            Country::Sweden => "se.finance.yahoo.com",
            Country::Taiwan => "tw.finance.yahoo.com",
            Country::Thailand => "th.finance.yahoo.com",
            Country::Turkey => "tr.finance.yahoo.com",
            Country::UnitedKingdom => "uk.finance.yahoo.com",
            Country::UnitedStates => "finance.yahoo.com",
            Country::Vietnam => "vn.finance.yahoo.com",
        }
    }
}

impl std::str::FromStr for Country {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "AR" => Ok(Country::Argentina),
            "AU" => Ok(Country::Australia),
            "BR" => Ok(Country::Brazil),
            "CA" => Ok(Country::Canada),
            "CN" => Ok(Country::China),
            "DK" => Ok(Country::Denmark),
            "FI" => Ok(Country::Finland),
            "FR" => Ok(Country::France),
            "DE" => Ok(Country::Germany),
            "GR" => Ok(Country::Greece),
            "HK" => Ok(Country::HongKong),
            "IN" => Ok(Country::India),
            "IL" => Ok(Country::Israel),
            "IT" => Ok(Country::Italy),
            "MY" => Ok(Country::Malaysia),
            "NZ" => Ok(Country::NewZealand),
            "NO" => Ok(Country::Norway),
            "PT" => Ok(Country::Portugal),
            "RU" => Ok(Country::Russia),
            "SG" => Ok(Country::Singapore),
            "ES" => Ok(Country::Spain),
            "SE" => Ok(Country::Sweden),
            "TW" => Ok(Country::Taiwan),
            "TH" => Ok(Country::Thailand),
            "TR" => Ok(Country::Turkey),
            "GB" | "UK" => Ok(Country::UnitedKingdom),
            "US" => Ok(Country::UnitedStates),
            "VN" => Ok(Country::Vietnam),
            _ => Err(()),
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
