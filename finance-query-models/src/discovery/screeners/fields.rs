//! Typed screener field enums for equity and fund screener queries.
//!
//! Use these enums with [`ScreenerFieldExt`](super::condition::ScreenerFieldExt) to build
//! type-safe screener conditions with full IDE autocomplete:
//!
//! ```
//! use finance_query::{EquityField, ScreenerFieldExt};
//!
//! let pe_filter     = EquityField::PeRatio.between(10.0, 25.0);
//! let region_filter = EquityField::Region.eq_str("us");
//! let volume_filter = EquityField::AvgDailyVol3M.gt(500_000.0);
//! ```
use super::condition::ScreenerField;
use serde::Serialize;

// ============================================================================
// EquityField
// ============================================================================

/// Typed field names for equity custom screener queries.
///
/// Variants marked as *display-only* (`Ticker`, `CompanyShortName`) are used
/// in `include_fields` to request those columns in the response. They do not
/// support meaningful numeric or string filters via Yahoo's API.
///
/// All other variants support filtering via [`ScreenerFieldExt`](super::condition::ScreenerFieldExt)
/// methods. Categorical fields (`Region`, `Sector`, `Industry`, `Exchange`, `PeerGroup`) use
/// [`eq_str`](super::condition::ScreenerFieldExt::eq_str); all others use numeric operators.
///
/// # Example
///
/// ```
/// use finance_query::{EquityField, EquityScreenerQuery, ScreenerFieldExt};
///
/// let query = EquityScreenerQuery::new()
///     .sort_by(EquityField::IntradayMarketCap, false)
///     .add_condition(EquityField::Region.eq_str("us"))
///     .add_condition(EquityField::PeRatio.between(10.0, 25.0))
///     .add_condition(EquityField::AvgDailyVol3M.gt(200_000.0))
///     .include_fields(vec![
///         EquityField::Ticker,
///         EquityField::CompanyShortName,
///         EquityField::IntradayPrice,
///         EquityField::PeRatio,
///     ]);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquityField {
    // Display-only fields (for include_fields, not filterable)
    /// Ticker symbol — display only, use in `include_fields`.
    Ticker,
    /// Short company name — display only, use in `include_fields`.
    CompanyShortName,

    // Price / Market Cap
    /// End-of-day price (`"eodprice"`).
    EodPrice,
    /// Intraday price (`"intradayprice"`).
    IntradayPrice,
    /// Intraday price change (`"intradaypricechange"`).
    IntradayPriceChange,
    /// Percent change (`"percentchange"`).
    PercentChange,
    /// Last close 52-week high (`"lastclose52weekhigh.lasttwelvemonths"`).
    Lastclose52WkHigh,
    /// 52-week percent change (`"fiftytwowkpercentchange"`).
    FiftyTwoWkPctChange,
    /// Last close 52-week low (`"lastclose52weeklow.lasttwelvemonths"`).
    Lastclose52WkLow,
    /// Intraday market cap (`"intradaymarketcap"`).
    IntradayMarketCap,
    /// Last close market cap (`"lastclosemarketcap.lasttwelvemonths"`).
    LastcloseMarketCap,

    // Categorical filters (use eq_str)
    /// Geographic region — use `eq_str("us")`.
    Region,
    /// GICS sector — use `eq_str("Technology")` etc.
    Sector,
    /// Peer group (`"peer_group"`).
    PeerGroup,
    /// Industry (`"industry"`).
    Industry,
    /// Exchange (`"exchange"`).
    Exchange,

    // Trading
    /// Beta (`"beta"`).
    Beta,
    /// 3-month average daily volume (`"avgdailyvol3m"`).
    AvgDailyVol3M,
    /// Percent held by insiders (`"pctheldinsider"`).
    PctHeldInsider,
    /// Percent held by institutions (`"pctheldinst"`).
    PctHeldInst,
    /// Intraday volume (`"dayvolume"`).
    DayVolume,
    /// End-of-day volume (`"eodvolume"`).
    EodVolume,

    // Short Interest
    /// Short percentage of shares outstanding (`"short_percentage_of_shares_outstanding.value"`).
    ShortPctSharesOut,
    /// Short interest value (`"short_interest.value"`).
    ShortInterest,
    /// Short percentage of float (`"short_percentage_of_float.value"`).
    ShortPctFloat,
    /// Days to cover short (`"days_to_cover_short.value"`).
    DaysToCover,
    /// Short interest percent change (`"short_interest_percentage_change.value"`).
    ShortInterestPctChange,

    // Valuation
    /// Book value per share (`"bookvalueshare.lasttwelvemonths"`).
    BookValueShare,
    /// Market cap to revenue (`"lastclosemarketcaptotalrevenue.lasttwelvemonths"`).
    MarketCapToRevenue,
    /// TEV to revenue (`"lastclosetevtotalrevenue.lasttwelvemonths"`).
    TevToRevenue,
    /// Price-to-book ratio (`"pricebookratio.quarterly"`).
    PriceBookRatio,
    /// Trailing twelve months P/E ratio (`"peratio.lasttwelvemonths"`).
    PeRatio,
    /// Price to tangible book value (`"lastclosepricetangiblebookvalue.lasttwelvemonths"`).
    PriceTangibleBook,
    /// Price to earnings (`"lastclosepriceearnings.lasttwelvemonths"`).
    PriceEarnings,
    /// 5-year PEG ratio (`"pegratio_5y"`).
    PegRatio5Y,

    // Profitability / Dividends
    /// Consecutive years of dividend growth (`"consecutive_years_of_dividend_growth_count"`).
    ConsecutiveDivYears,
    /// Return on assets (`"returnonassets.lasttwelvemonths"`).
    Roa,
    /// Return on equity (`"returnonequity.lasttwelvemonths"`).
    Roe,
    /// Forward dividend per share (`"forward_dividend_per_share"`).
    ForwardDivPerShare,
    /// Forward dividend yield (`"forward_dividend_yield"`).
    ForwardDivYield,
    /// Return on total capital (`"returnontotalcapital.lasttwelvemonths"`).
    ReturnOnCapital,

    // Leverage
    /// TEV / EBIT (`"lastclosetevebit.lasttwelvemonths"`).
    TevEbit,
    /// Net debt / EBITDA (`"netdebtebitda.lasttwelvemonths"`).
    NetDebtEbitda,
    /// Total debt / equity (`"totaldebtequity.lasttwelvemonths"`).
    TotalDebtEquity,
    /// Long-term debt / equity (`"ltdebtequity.lasttwelvemonths"`).
    LtDebtEquity,
    /// EBIT / interest expense (`"ebitinterestexpense.lasttwelvemonths"`).
    EbitInterestExp,
    /// EBITDA / interest expense (`"ebitdainterestexpense.lasttwelvemonths"`).
    EbitdaInterestExp,
    /// TEV / EBITDA (`"lastclosetevebitda.lasttwelvemonths"`).
    TevEbitda,
    /// Total debt / EBITDA (`"totaldebtebitda.lasttwelvemonths"`).
    TotalDebtEbitda,

    // Liquidity
    /// Quick ratio (`"quickratio.lasttwelvemonths"`).
    QuickRatio,
    /// Altman Z-score (`"altmanzscoreusingtheaveragestockinformationforaperiod.lasttwelvemonths"`).
    AltmanZScore,
    /// Current ratio (`"currentratio.lasttwelvemonths"`).
    CurrentRatio,
    /// Operating cash flow to current liabilities (`"operatingcashflowtocurrentliabilities.lasttwelvemonths"`).
    OcfToCurrentLiab,

    // Income Statement
    /// Total revenues (`"totalrevenues.lasttwelvemonths"`).
    TotalRevenues,
    /// Net income margin (`"netincomemargin.lasttwelvemonths"`).
    NetIncomeMargin,
    /// Gross profit (`"grossprofit.lasttwelvemonths"`).
    GrossProfit,
    /// EBITDA 1-year growth (`"ebitda1yrgrowth.lasttwelvemonths"`).
    Ebitda1YrGrowth,
    /// Diluted EPS from continuing operations (`"dilutedepscontinuingoperations.lasttwelvemonths"`).
    DilutedEpsContOps,
    /// Quarterly revenue growth (`"quarterlyrevenuegrowth.quarterly"`).
    QuarterlyRevGrowth,
    /// EPS growth (`"epsgrowth.lasttwelvemonths"`).
    EpsGrowth,
    /// Net income (`"netincomeis.lasttwelvemonths"`).
    NetIncome,
    /// EBITDA (`"ebitda.lasttwelvemonths"`).
    Ebitda,
    /// Diluted EPS 1-year growth (`"dilutedeps1yrgrowth.lasttwelvemonths"`).
    DilutedEps1YrGrowth,
    /// Revenue 1-year growth (`"totalrevenues1yrgrowth.lasttwelvemonths"`).
    Revenue1YrGrowth,
    /// Operating income (`"operatingincome.lasttwelvemonths"`).
    OperatingIncome,
    /// Net income 1-year growth (`"netincome1yrgrowth.lasttwelvemonths"`).
    NetIncome1YrGrowth,
    /// Gross profit margin (`"grossprofitmargin.lasttwelvemonths"`).
    GrossProfitMargin,
    /// EBITDA margin (`"ebitdamargin.lasttwelvemonths"`).
    EbitdaMargin,
    /// EBIT (`"ebit.lasttwelvemonths"`).
    Ebit,
    /// Basic EPS from continuing operations (`"basicepscontinuingoperations.lasttwelvemonths"`).
    BasicEpsContOps,
    /// Basic EPS (`"netepsbasic.lasttwelvemonths"`).
    NetEpsBasic,
    /// Diluted EPS (`"netepsdiluted.lasttwelvemonths"`).
    NetEpsDiluted,

    // Balance Sheet
    /// Total assets (`"totalassets.lasttwelvemonths"`).
    TotalAssets,
    /// Common shares outstanding (`"totalcommonsharesoutstanding.lasttwelvemonths"`).
    CommonSharesOut,
    /// Total debt (`"totaldebt.lasttwelvemonths"`).
    TotalDebt,
    /// Total equity (`"totalequity.lasttwelvemonths"`).
    TotalEquity,
    /// Total current assets (`"totalcurrentassets.lasttwelvemonths"`).
    TotalCurrentAssets,
    /// Cash and short-term investments (`"totalcashandshortterminvestments.lasttwelvemonths"`).
    CashAndStInvestments,
    /// Total common equity (`"totalcommonequity.lasttwelvemonths"`).
    TotalCommonEquity,
    /// Total current liabilities (`"totalcurrentliabilities.lasttwelvemonths"`).
    TotalCurrentLiab,
    /// Total shares outstanding (`"totalsharesoutstanding"`).
    TotalSharesOut,

    // Cash Flow
    /// Levered free cash flow (`"leveredfreecashflow.lasttwelvemonths"`).
    LeveredFcf,
    /// Capital expenditure (`"capitalexpenditure.lasttwelvemonths"`).
    Capex,
    /// Cash from operations (`"cashfromoperations.lasttwelvemonths"`).
    CashFromOps,
    /// Levered FCF 1-year growth (`"leveredfreecashflow1yrgrowth.lasttwelvemonths"`).
    LeveredFcf1YrGrowth,
    /// Unlevered free cash flow (`"unleveredfreecashflow.lasttwelvemonths"`).
    UnleveredFcf,
    /// Cash from operations 1-year growth (`"cashfromoperations1yrgrowth.lasttwelvemonths"`).
    CashFromOps1YrGrowth,

    // ESG
    /// ESG score (`"esg_score"`).
    EsgScore,
    /// Environmental score (`"environmental_score"`).
    EnvironmentalScore,
    /// Governance score (`"governance_score"`).
    GovernanceScore,
    /// Social score (`"social_score"`).
    SocialScore,
    /// Highest controversy level (`"highest_controversy"`).
    HighestControversy,
}

impl EquityField {
    /// Returns the Yahoo Finance API field name string for this variant.
    pub fn as_str(&self) -> &'static str {
        match self {
            EquityField::Ticker => "ticker",
            EquityField::CompanyShortName => "companyshortname",
            EquityField::EodPrice => "eodprice",
            EquityField::IntradayPrice => "intradayprice",
            EquityField::IntradayPriceChange => "intradaypricechange",
            EquityField::PercentChange => "percentchange",
            EquityField::Lastclose52WkHigh => "lastclose52weekhigh.lasttwelvemonths",
            EquityField::FiftyTwoWkPctChange => "fiftytwowkpercentchange",
            EquityField::Lastclose52WkLow => "lastclose52weeklow.lasttwelvemonths",
            EquityField::IntradayMarketCap => "intradaymarketcap",
            EquityField::LastcloseMarketCap => "lastclosemarketcap.lasttwelvemonths",
            EquityField::Region => "region",
            EquityField::Sector => "sector",
            EquityField::PeerGroup => "peer_group",
            EquityField::Industry => "industry",
            EquityField::Exchange => "exchange",
            EquityField::Beta => "beta",
            EquityField::AvgDailyVol3M => "avgdailyvol3m",
            EquityField::PctHeldInsider => "pctheldinsider",
            EquityField::PctHeldInst => "pctheldinst",
            EquityField::DayVolume => "dayvolume",
            EquityField::EodVolume => "eodvolume",
            EquityField::ShortPctSharesOut => "short_percentage_of_shares_outstanding.value",
            EquityField::ShortInterest => "short_interest.value",
            EquityField::ShortPctFloat => "short_percentage_of_float.value",
            EquityField::DaysToCover => "days_to_cover_short.value",
            EquityField::ShortInterestPctChange => "short_interest_percentage_change.value",
            EquityField::BookValueShare => "bookvalueshare.lasttwelvemonths",
            EquityField::MarketCapToRevenue => "lastclosemarketcaptotalrevenue.lasttwelvemonths",
            EquityField::TevToRevenue => "lastclosetevtotalrevenue.lasttwelvemonths",
            EquityField::PriceBookRatio => "pricebookratio.quarterly",
            EquityField::PeRatio => "peratio.lasttwelvemonths",
            EquityField::PriceTangibleBook => "lastclosepricetangiblebookvalue.lasttwelvemonths",
            EquityField::PriceEarnings => "lastclosepriceearnings.lasttwelvemonths",
            EquityField::PegRatio5Y => "pegratio_5y",
            EquityField::ConsecutiveDivYears => "consecutive_years_of_dividend_growth_count",
            EquityField::Roa => "returnonassets.lasttwelvemonths",
            EquityField::Roe => "returnonequity.lasttwelvemonths",
            EquityField::ForwardDivPerShare => "forward_dividend_per_share",
            EquityField::ForwardDivYield => "forward_dividend_yield",
            EquityField::ReturnOnCapital => "returnontotalcapital.lasttwelvemonths",
            EquityField::TevEbit => "lastclosetevebit.lasttwelvemonths",
            EquityField::NetDebtEbitda => "netdebtebitda.lasttwelvemonths",
            EquityField::TotalDebtEquity => "totaldebtequity.lasttwelvemonths",
            EquityField::LtDebtEquity => "ltdebtequity.lasttwelvemonths",
            EquityField::EbitInterestExp => "ebitinterestexpense.lasttwelvemonths",
            EquityField::EbitdaInterestExp => "ebitdainterestexpense.lasttwelvemonths",
            EquityField::TevEbitda => "lastclosetevebitda.lasttwelvemonths",
            EquityField::TotalDebtEbitda => "totaldebtebitda.lasttwelvemonths",
            EquityField::QuickRatio => "quickratio.lasttwelvemonths",
            EquityField::AltmanZScore => {
                "altmanzscoreusingtheaveragestockinformationforaperiod.lasttwelvemonths"
            }
            EquityField::CurrentRatio => "currentratio.lasttwelvemonths",
            EquityField::OcfToCurrentLiab => {
                "operatingcashflowtocurrentliabilities.lasttwelvemonths"
            }
            EquityField::TotalRevenues => "totalrevenues.lasttwelvemonths",
            EquityField::NetIncomeMargin => "netincomemargin.lasttwelvemonths",
            EquityField::GrossProfit => "grossprofit.lasttwelvemonths",
            EquityField::Ebitda1YrGrowth => "ebitda1yrgrowth.lasttwelvemonths",
            EquityField::DilutedEpsContOps => "dilutedepscontinuingoperations.lasttwelvemonths",
            EquityField::QuarterlyRevGrowth => "quarterlyrevenuegrowth.quarterly",
            EquityField::EpsGrowth => "epsgrowth.lasttwelvemonths",
            EquityField::NetIncome => "netincomeis.lasttwelvemonths",
            EquityField::Ebitda => "ebitda.lasttwelvemonths",
            EquityField::DilutedEps1YrGrowth => "dilutedeps1yrgrowth.lasttwelvemonths",
            EquityField::Revenue1YrGrowth => "totalrevenues1yrgrowth.lasttwelvemonths",
            EquityField::OperatingIncome => "operatingincome.lasttwelvemonths",
            EquityField::NetIncome1YrGrowth => "netincome1yrgrowth.lasttwelvemonths",
            EquityField::GrossProfitMargin => "grossprofitmargin.lasttwelvemonths",
            EquityField::EbitdaMargin => "ebitdamargin.lasttwelvemonths",
            EquityField::Ebit => "ebit.lasttwelvemonths",
            EquityField::BasicEpsContOps => "basicepscontinuingoperations.lasttwelvemonths",
            EquityField::NetEpsBasic => "netepsbasic.lasttwelvemonths",
            EquityField::NetEpsDiluted => "netepsdiluted.lasttwelvemonths",
            EquityField::TotalAssets => "totalassets.lasttwelvemonths",
            EquityField::CommonSharesOut => "totalcommonsharesoutstanding.lasttwelvemonths",
            EquityField::TotalDebt => "totaldebt.lasttwelvemonths",
            EquityField::TotalEquity => "totalequity.lasttwelvemonths",
            EquityField::TotalCurrentAssets => "totalcurrentassets.lasttwelvemonths",
            EquityField::CashAndStInvestments => {
                "totalcashandshortterminvestments.lasttwelvemonths"
            }
            EquityField::TotalCommonEquity => "totalcommonequity.lasttwelvemonths",
            EquityField::TotalCurrentLiab => "totalcurrentliabilities.lasttwelvemonths",
            EquityField::TotalSharesOut => "totalsharesoutstanding",
            EquityField::LeveredFcf => "leveredfreecashflow.lasttwelvemonths",
            EquityField::Capex => "capitalexpenditure.lasttwelvemonths",
            EquityField::CashFromOps => "cashfromoperations.lasttwelvemonths",
            EquityField::LeveredFcf1YrGrowth => "leveredfreecashflow1yrgrowth.lasttwelvemonths",
            EquityField::UnleveredFcf => "unleveredfreecashflow.lasttwelvemonths",
            EquityField::CashFromOps1YrGrowth => "cashfromoperations1yrgrowth.lasttwelvemonths",
            EquityField::EsgScore => "esg_score",
            EquityField::EnvironmentalScore => "environmental_score",
            EquityField::GovernanceScore => "governance_score",
            EquityField::SocialScore => "social_score",
            EquityField::HighestControversy => "highest_controversy",
        }
    }

    /// Returns a slice of all `EquityField` variants.
    ///
    /// Useful for validation: `EquityField::all().iter().find(|f| f.as_str() == s)`.
    pub fn all() -> &'static [EquityField] {
        &[
            EquityField::Ticker,
            EquityField::CompanyShortName,
            EquityField::EodPrice,
            EquityField::IntradayPrice,
            EquityField::IntradayPriceChange,
            EquityField::PercentChange,
            EquityField::Lastclose52WkHigh,
            EquityField::FiftyTwoWkPctChange,
            EquityField::Lastclose52WkLow,
            EquityField::IntradayMarketCap,
            EquityField::LastcloseMarketCap,
            EquityField::Region,
            EquityField::Sector,
            EquityField::PeerGroup,
            EquityField::Industry,
            EquityField::Exchange,
            EquityField::Beta,
            EquityField::AvgDailyVol3M,
            EquityField::PctHeldInsider,
            EquityField::PctHeldInst,
            EquityField::DayVolume,
            EquityField::EodVolume,
            EquityField::ShortPctSharesOut,
            EquityField::ShortInterest,
            EquityField::ShortPctFloat,
            EquityField::DaysToCover,
            EquityField::ShortInterestPctChange,
            EquityField::BookValueShare,
            EquityField::MarketCapToRevenue,
            EquityField::TevToRevenue,
            EquityField::PriceBookRatio,
            EquityField::PeRatio,
            EquityField::PriceTangibleBook,
            EquityField::PriceEarnings,
            EquityField::PegRatio5Y,
            EquityField::ConsecutiveDivYears,
            EquityField::Roa,
            EquityField::Roe,
            EquityField::ForwardDivPerShare,
            EquityField::ForwardDivYield,
            EquityField::ReturnOnCapital,
            EquityField::TevEbit,
            EquityField::NetDebtEbitda,
            EquityField::TotalDebtEquity,
            EquityField::LtDebtEquity,
            EquityField::EbitInterestExp,
            EquityField::EbitdaInterestExp,
            EquityField::TevEbitda,
            EquityField::TotalDebtEbitda,
            EquityField::QuickRatio,
            EquityField::AltmanZScore,
            EquityField::CurrentRatio,
            EquityField::OcfToCurrentLiab,
            EquityField::TotalRevenues,
            EquityField::NetIncomeMargin,
            EquityField::GrossProfit,
            EquityField::Ebitda1YrGrowth,
            EquityField::DilutedEpsContOps,
            EquityField::QuarterlyRevGrowth,
            EquityField::EpsGrowth,
            EquityField::NetIncome,
            EquityField::Ebitda,
            EquityField::DilutedEps1YrGrowth,
            EquityField::Revenue1YrGrowth,
            EquityField::OperatingIncome,
            EquityField::NetIncome1YrGrowth,
            EquityField::GrossProfitMargin,
            EquityField::EbitdaMargin,
            EquityField::Ebit,
            EquityField::BasicEpsContOps,
            EquityField::NetEpsBasic,
            EquityField::NetEpsDiluted,
            EquityField::TotalAssets,
            EquityField::CommonSharesOut,
            EquityField::TotalDebt,
            EquityField::TotalEquity,
            EquityField::TotalCurrentAssets,
            EquityField::CashAndStInvestments,
            EquityField::TotalCommonEquity,
            EquityField::TotalCurrentLiab,
            EquityField::TotalSharesOut,
            EquityField::LeveredFcf,
            EquityField::Capex,
            EquityField::CashFromOps,
            EquityField::LeveredFcf1YrGrowth,
            EquityField::UnleveredFcf,
            EquityField::CashFromOps1YrGrowth,
            EquityField::EsgScore,
            EquityField::EnvironmentalScore,
            EquityField::GovernanceScore,
            EquityField::SocialScore,
            EquityField::HighestControversy,
        ]
    }
}

impl std::str::FromStr for EquityField {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EquityField::all()
            .iter()
            .find(|f| f.as_str() == s)
            .copied()
            .ok_or(())
    }
}

impl Serialize for EquityField {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl ScreenerField for EquityField {
    fn as_str(&self) -> &'static str {
        self.as_str()
    }
}

// ============================================================================
// FundField
// ============================================================================

/// Typed field names for mutual fund custom screener queries.
///
/// Use with [`FundScreenerQuery`](super::query::FundScreenerQuery) and
/// [`ScreenerFieldExt`](super::condition::ScreenerFieldExt).
///
/// # Example
///
/// ```
/// use finance_query::{FundField, FundScreenerQuery, ScreenerFieldExt};
///
/// let query = FundScreenerQuery::new()
///     .sort_by(FundField::PerformanceRating, false)
///     .add_condition(FundField::RiskRating.lte(3.0))
///     .include_fields(vec![
///         FundField::Ticker,
///         FundField::CompanyShortName,
///         FundField::IntradayPrice,
///         FundField::PerformanceRating,
///     ]);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FundField {
    // Display-only fields (for include_fields)
    /// Ticker symbol — display only, use in `include_fields`.
    Ticker,
    /// Short company name — display only, use in `include_fields`.
    CompanyShortName,

    // Price fields
    /// End-of-day price (`"eodprice"`).
    EodPrice,
    /// Intraday price change (`"intradaypricechange"`).
    IntradayPriceChange,
    /// Intraday price (`"intradayprice"`).
    IntradayPrice,

    // Fund-specific fields
    /// Fund category name (`"categoryname"`).
    CategoryName,
    /// Overall performance rating (`"performanceratingoverall"`).
    PerformanceRating,
    /// Minimum initial investment (`"initialinvestment"`).
    InitialInvestment,
    /// Annual return rank within category (`"annualreturnnavy1categoryrank"`).
    AnnualReturnRank,
    /// Overall risk rating (`"riskratingoverall"`).
    RiskRating,
    /// Exchange (`"exchange"`).
    Exchange,
}

impl FundField {
    /// Returns the Yahoo Finance API field name string for this variant.
    pub fn as_str(&self) -> &'static str {
        match self {
            FundField::Ticker => "ticker",
            FundField::CompanyShortName => "companyshortname",
            FundField::EodPrice => "eodprice",
            FundField::IntradayPriceChange => "intradaypricechange",
            FundField::IntradayPrice => "intradayprice",
            FundField::CategoryName => "categoryname",
            FundField::PerformanceRating => "performanceratingoverall",
            FundField::InitialInvestment => "initialinvestment",
            FundField::AnnualReturnRank => "annualreturnnavy1categoryrank",
            FundField::RiskRating => "riskratingoverall",
            FundField::Exchange => "exchange",
        }
    }

    /// Returns a slice of all `FundField` variants.
    pub fn all() -> &'static [FundField] {
        &[
            FundField::Ticker,
            FundField::CompanyShortName,
            FundField::EodPrice,
            FundField::IntradayPriceChange,
            FundField::IntradayPrice,
            FundField::CategoryName,
            FundField::PerformanceRating,
            FundField::InitialInvestment,
            FundField::AnnualReturnRank,
            FundField::RiskRating,
            FundField::Exchange,
        ]
    }
}

impl std::str::FromStr for FundField {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FundField::all()
            .iter()
            .find(|f| f.as_str() == s)
            .copied()
            .ok_or(())
    }
}

impl Serialize for FundField {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl ScreenerField for FundField {
    fn as_str(&self) -> &'static str {
        self.as_str()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equity_field_as_str() {
        assert_eq!(EquityField::PeRatio.as_str(), "peratio.lasttwelvemonths");
        assert_eq!(EquityField::AvgDailyVol3M.as_str(), "avgdailyvol3m");
        assert_eq!(EquityField::Region.as_str(), "region");
        assert_eq!(EquityField::IntradayMarketCap.as_str(), "intradaymarketcap");
        assert_eq!(EquityField::Ticker.as_str(), "ticker");
    }

    #[test]
    fn test_equity_field_from_str_round_trip() {
        for field in EquityField::all() {
            let s = field.as_str();
            let parsed: EquityField = s.parse().unwrap_or_else(|_| {
                panic!("Failed to parse EquityField from '{s}'");
            });
            assert_eq!(&parsed, field, "Round-trip failed for field '{s}'");
        }
    }

    #[test]
    fn test_equity_field_from_str_unknown_returns_err() {
        assert!("invalid_field_name".parse::<EquityField>().is_err());
        assert!("".parse::<EquityField>().is_err());
    }

    #[test]
    fn test_equity_field_serializes_as_string() {
        let json = serde_json::to_value(EquityField::PeRatio).unwrap();
        assert_eq!(json, "peratio.lasttwelvemonths");
    }

    #[test]
    fn test_fund_field_as_str() {
        assert_eq!(
            FundField::PerformanceRating.as_str(),
            "performanceratingoverall"
        );
        assert_eq!(FundField::RiskRating.as_str(), "riskratingoverall");
        assert_eq!(FundField::CategoryName.as_str(), "categoryname");
    }

    #[test]
    fn test_fund_field_from_str_round_trip() {
        for field in FundField::all() {
            let s = field.as_str();
            let parsed: FundField = s.parse().unwrap_or_else(|_| {
                panic!("Failed to parse FundField from '{s}'");
            });
            assert_eq!(&parsed, field, "Round-trip failed for fund field '{s}'");
        }
    }

    #[test]
    fn test_fund_field_from_str_unknown_returns_err() {
        assert!("intradaymarketcap".parse::<FundField>().is_err());
    }
}
