use finance_query::{Frequency, Interval, StatementType, TimeRange};

fn parse_interval(s: &str) -> Interval {
    match s {
        "1m" => Interval::OneMinute,
        "5m" => Interval::FiveMinutes,
        "15m" => Interval::FifteenMinutes,
        "30m" => Interval::ThirtyMinutes,
        "1h" => Interval::OneHour,
        "1d" => Interval::OneDay,
        "1wk" => Interval::OneWeek,
        "1mo" => Interval::OneMonth,
        "3mo" => Interval::ThreeMonths,
        _ => Interval::OneDay,
    }
}

fn parse_range(s: &str) -> TimeRange {
    match s {
        "1d" => TimeRange::OneDay,
        "5d" => TimeRange::FiveDays,
        "1mo" => TimeRange::OneMonth,
        "3mo" => TimeRange::ThreeMonths,
        "6mo" => TimeRange::SixMonths,
        "1y" => TimeRange::OneYear,
        "2y" => TimeRange::TwoYears,
        "5y" => TimeRange::FiveYears,
        "10y" => TimeRange::TenYears,
        "ytd" => TimeRange::YearToDate,
        "max" => TimeRange::Max,
        _ => TimeRange::OneMonth,
    }
}

pub fn parse_statement_type(s: &str) -> Option<StatementType> {
    match s.to_lowercase().as_str() {
        "income" => Some(StatementType::Income),
        "balance" => Some(StatementType::Balance),
        "cashflow" | "cash-flow" => Some(StatementType::CashFlow),
        _ => None,
    }
}

pub fn parse_frequency(s: &str) -> Frequency {
    match s.to_lowercase().as_str() {
        "quarterly" | "q" => Frequency::Quarterly,
        _ => Frequency::Annual,
    }
}

/// Parse a raw interval string and map it straight to its GraphQL enum literal.
pub fn interval_to_gql(s: &str) -> &'static str {
    match parse_interval(s) {
        Interval::OneMinute => "ONE_MINUTE",
        Interval::FiveMinutes => "FIVE_MINUTES",
        Interval::FifteenMinutes => "FIFTEEN_MINUTES",
        Interval::ThirtyMinutes => "THIRTY_MINUTES",
        Interval::OneHour => "ONE_HOUR",
        Interval::OneDay => "ONE_DAY",
        Interval::OneWeek => "ONE_WEEK",
        Interval::OneMonth => "ONE_MONTH",
        Interval::ThreeMonths => "THREE_MONTHS",
    }
}

/// Parse a raw range string and map it straight to its GraphQL enum literal.
pub fn range_to_gql(s: &str) -> &'static str {
    match parse_range(s) {
        TimeRange::OneDay => "ONE_DAY",
        TimeRange::FiveDays => "FIVE_DAYS",
        TimeRange::OneMonth => "ONE_MONTH",
        TimeRange::ThreeMonths => "THREE_MONTHS",
        TimeRange::SixMonths => "SIX_MONTHS",
        TimeRange::OneYear => "ONE_YEAR",
        TimeRange::TwoYears => "TWO_YEARS",
        TimeRange::FiveYears => "FIVE_YEARS",
        TimeRange::TenYears => "TEN_YEARS",
        TimeRange::YearToDate => "YEAR_TO_DATE",
        TimeRange::Max => "MAX",
    }
}
