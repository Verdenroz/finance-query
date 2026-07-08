use finance_query::{Frequency, Interval, StatementType, TimeRange};

fn parse_interval(s: &str) -> Interval {
    s.parse().unwrap_or(Interval::OneDay)
}

fn parse_range(s: &str) -> TimeRange {
    s.parse().unwrap_or(TimeRange::OneMonth)
}

pub fn parse_statement_type(s: &str) -> Option<StatementType> {
    s.parse().ok()
}

/// Fallible on purpose: silently defaulting an unrecognized frequency to
/// `Annual` would mask typos, so invalid input must be rejected by the caller.
pub fn parse_frequency(s: &str) -> Option<Frequency> {
    s.parse().ok()
}

pub fn statement_to_gql(statement: StatementType) -> &'static str {
    match statement {
        StatementType::Income => "INCOME",
        StatementType::Balance => "BALANCE",
        StatementType::CashFlow => "CASH_FLOW",
    }
}

pub fn frequency_to_gql(frequency: Frequency) -> &'static str {
    match frequency {
        Frequency::Annual => "ANNUAL",
        Frequency::Quarterly => "QUARTERLY",
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
