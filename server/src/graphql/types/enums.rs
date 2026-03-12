//! GraphQL enum wrappers mirroring library enums.
//!
//! Each wrapper derives `async_graphql::Enum` and provides bidirectional
//! conversion with the corresponding library type via `From` impls.

use async_graphql::Enum;
use finance_query::{
    Frequency, IndicesRegion, Interval, Screener, Sector, StatementType, TimeRange,
};

// ── Interval ─────────────────────────────────────────────────────────────────

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum GqlInterval {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    OneDay,
    OneWeek,
    OneMonth,
    ThreeMonths,
}

impl From<GqlInterval> for Interval {
    fn from(v: GqlInterval) -> Self {
        match v {
            GqlInterval::OneMinute => Interval::OneMinute,
            GqlInterval::FiveMinutes => Interval::FiveMinutes,
            GqlInterval::FifteenMinutes => Interval::FifteenMinutes,
            GqlInterval::ThirtyMinutes => Interval::ThirtyMinutes,
            GqlInterval::OneHour => Interval::OneHour,
            GqlInterval::OneDay => Interval::OneDay,
            GqlInterval::OneWeek => Interval::OneWeek,
            GqlInterval::OneMonth => Interval::OneMonth,
            GqlInterval::ThreeMonths => Interval::ThreeMonths,
        }
    }
}

impl GqlInterval {
    /// Return the string key used by the cache layer (e.g. `"1d"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneMinute => "1m",
            Self::FiveMinutes => "5m",
            Self::FifteenMinutes => "15m",
            Self::ThirtyMinutes => "30m",
            Self::OneHour => "1h",
            Self::OneDay => "1d",
            Self::OneWeek => "1wk",
            Self::OneMonth => "1mo",
            Self::ThreeMonths => "3mo",
        }
    }
}

// ── TimeRange ────────────────────────────────────────────────────────────────

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum GqlTimeRange {
    OneDay,
    FiveDays,
    OneMonth,
    ThreeMonths,
    SixMonths,
    OneYear,
    TwoYears,
    FiveYears,
    TenYears,
    YearToDate,
    Max,
}

impl From<GqlTimeRange> for TimeRange {
    fn from(v: GqlTimeRange) -> Self {
        match v {
            GqlTimeRange::OneDay => TimeRange::OneDay,
            GqlTimeRange::FiveDays => TimeRange::FiveDays,
            GqlTimeRange::OneMonth => TimeRange::OneMonth,
            GqlTimeRange::ThreeMonths => TimeRange::ThreeMonths,
            GqlTimeRange::SixMonths => TimeRange::SixMonths,
            GqlTimeRange::OneYear => TimeRange::OneYear,
            GqlTimeRange::TwoYears => TimeRange::TwoYears,
            GqlTimeRange::FiveYears => TimeRange::FiveYears,
            GqlTimeRange::TenYears => TimeRange::TenYears,
            GqlTimeRange::YearToDate => TimeRange::YearToDate,
            GqlTimeRange::Max => TimeRange::Max,
        }
    }
}

impl GqlTimeRange {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneDay => "1d",
            Self::FiveDays => "5d",
            Self::OneMonth => "1mo",
            Self::ThreeMonths => "3mo",
            Self::SixMonths => "6mo",
            Self::OneYear => "1y",
            Self::TwoYears => "2y",
            Self::FiveYears => "5y",
            Self::TenYears => "10y",
            Self::YearToDate => "ytd",
            Self::Max => "max",
        }
    }
}

// ── StatementType ────────────────────────────────────────────────────────────

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum GqlStatementType {
    Income,
    Balance,
    CashFlow,
}

impl From<GqlStatementType> for StatementType {
    fn from(v: GqlStatementType) -> Self {
        match v {
            GqlStatementType::Income => StatementType::Income,
            GqlStatementType::Balance => StatementType::Balance,
            GqlStatementType::CashFlow => StatementType::CashFlow,
        }
    }
}

impl GqlStatementType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Income => "income",
            Self::Balance => "balance",
            Self::CashFlow => "cashflow",
        }
    }
}

// ── Frequency ────────────────────────────────────────────────────────────────

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum GqlFrequency {
    Annual,
    Quarterly,
}

impl From<GqlFrequency> for Frequency {
    fn from(v: GqlFrequency) -> Self {
        match v {
            GqlFrequency::Annual => Frequency::Annual,
            GqlFrequency::Quarterly => Frequency::Quarterly,
        }
    }
}

impl GqlFrequency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Annual => "annual",
            Self::Quarterly => "quarterly",
        }
    }
}

// ── Screener ─────────────────────────────────────────────────────────────────

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum GqlScreener {
    AggressiveSmallCaps,
    DayGainers,
    DayLosers,
    GrowthTechnologyStocks,
    MostActives,
    MostShortedStocks,
    SmallCapGainers,
    UndervaluedGrowthStocks,
    UndervaluedLargeCaps,
    ConservativeForeignFunds,
    HighYieldBond,
    PortfolioAnchors,
    SolidLargeGrowthFunds,
    SolidMidcapGrowthFunds,
    TopMutualFunds,
}

impl From<GqlScreener> for Screener {
    fn from(v: GqlScreener) -> Self {
        match v {
            GqlScreener::AggressiveSmallCaps => Screener::AggressiveSmallCaps,
            GqlScreener::DayGainers => Screener::DayGainers,
            GqlScreener::DayLosers => Screener::DayLosers,
            GqlScreener::GrowthTechnologyStocks => Screener::GrowthTechnologyStocks,
            GqlScreener::MostActives => Screener::MostActives,
            GqlScreener::MostShortedStocks => Screener::MostShortedStocks,
            GqlScreener::SmallCapGainers => Screener::SmallCapGainers,
            GqlScreener::UndervaluedGrowthStocks => Screener::UndervaluedGrowthStocks,
            GqlScreener::UndervaluedLargeCaps => Screener::UndervaluedLargeCaps,
            GqlScreener::ConservativeForeignFunds => Screener::ConservativeForeignFunds,
            GqlScreener::HighYieldBond => Screener::HighYieldBond,
            GqlScreener::PortfolioAnchors => Screener::PortfolioAnchors,
            GqlScreener::SolidLargeGrowthFunds => Screener::SolidLargeGrowthFunds,
            GqlScreener::SolidMidcapGrowthFunds => Screener::SolidMidcapGrowthFunds,
            GqlScreener::TopMutualFunds => Screener::TopMutualFunds,
        }
    }
}

impl GqlScreener {
    pub fn as_scr_id(&self) -> &'static str {
        let s: Screener = (*self).into();
        s.as_scr_id()
    }
}

// ── Sector ───────────────────────────────────────────────────────────────────

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum GqlSector {
    Technology,
    FinancialServices,
    ConsumerCyclical,
    CommunicationServices,
    Healthcare,
    Industrials,
    ConsumerDefensive,
    Energy,
    RealEstate,
    Utilities,
    BasicMaterials,
}

impl From<GqlSector> for Sector {
    fn from(v: GqlSector) -> Self {
        match v {
            GqlSector::Technology => Sector::Technology,
            GqlSector::FinancialServices => Sector::FinancialServices,
            GqlSector::ConsumerCyclical => Sector::ConsumerCyclical,
            GqlSector::CommunicationServices => Sector::CommunicationServices,
            GqlSector::Healthcare => Sector::Healthcare,
            GqlSector::Industrials => Sector::Industrials,
            GqlSector::ConsumerDefensive => Sector::ConsumerDefensive,
            GqlSector::Energy => Sector::Energy,
            GqlSector::RealEstate => Sector::RealEstate,
            GqlSector::Utilities => Sector::Utilities,
            GqlSector::BasicMaterials => Sector::BasicMaterials,
        }
    }
}

// ── IndicesRegion ────────────────────────────────────────────────────────────

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum GqlIndicesRegion {
    Americas,
    Europe,
    AsiaPacific,
    MiddleEastAfrica,
    Currencies,
}

impl From<GqlIndicesRegion> for IndicesRegion {
    fn from(v: GqlIndicesRegion) -> Self {
        match v {
            GqlIndicesRegion::Americas => IndicesRegion::Americas,
            GqlIndicesRegion::Europe => IndicesRegion::Europe,
            GqlIndicesRegion::AsiaPacific => IndicesRegion::AsiaPacific,
            GqlIndicesRegion::MiddleEastAfrica => IndicesRegion::MiddleEastAfrica,
            GqlIndicesRegion::Currencies => IndicesRegion::Currencies,
        }
    }
}

// ── GqlValueFormat ────────────────────────────────────────────────────────────

/// Controls how `FormattedValue` fields are returned in GraphQL responses.
///
/// - `Raw` (default): returns the raw numeric/scalar value (e.g. `182.5`)
/// - `Fmt`: returns the human-readable formatted string (e.g. `"182.50"`)
/// - `Both`: returns the full object `{ raw, fmt, longFmt }`
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Default)]
pub enum GqlValueFormat {
    #[default]
    Raw,
    Pretty,
    Both,
}

impl From<GqlValueFormat> for finance_query::ValueFormat {
    fn from(v: GqlValueFormat) -> Self {
        match v {
            GqlValueFormat::Raw => finance_query::ValueFormat::Raw,
            GqlValueFormat::Pretty => finance_query::ValueFormat::Pretty,
            GqlValueFormat::Both => finance_query::ValueFormat::Both,
        }
    }
}
