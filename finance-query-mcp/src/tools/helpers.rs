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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_statement_type_accepts_known_values() {
        assert_eq!(parse_statement_type("income"), Some(StatementType::Income));
        assert_eq!(
            parse_statement_type("balance"),
            Some(StatementType::Balance)
        );
        assert_eq!(
            parse_statement_type("cashflow"),
            Some(StatementType::CashFlow)
        );
    }

    #[test]
    fn parse_statement_type_rejects_unknown_values() {
        assert_eq!(parse_statement_type("bogus"), None);
        assert_eq!(parse_statement_type(""), None);
    }

    #[test]
    fn parse_frequency_accepts_known_values() {
        assert_eq!(parse_frequency("annual"), Some(Frequency::Annual));
        assert_eq!(parse_frequency("quarterly"), Some(Frequency::Quarterly));
    }

    #[test]
    fn parse_frequency_rejects_unknown_values_instead_of_defaulting() {
        // Deliberately fallible (see doc comment) — must not silently
        // default an unrecognized frequency to `Annual`.
        assert_eq!(parse_frequency("bogus"), None);
        assert_eq!(parse_frequency(""), None);
    }

    #[test]
    fn statement_to_gql_maps_every_variant() {
        assert_eq!(statement_to_gql(StatementType::Income), "INCOME");
        assert_eq!(statement_to_gql(StatementType::Balance), "BALANCE");
        assert_eq!(statement_to_gql(StatementType::CashFlow), "CASH_FLOW");
    }

    #[test]
    fn frequency_to_gql_maps_every_variant() {
        assert_eq!(frequency_to_gql(Frequency::Annual), "ANNUAL");
        assert_eq!(frequency_to_gql(Frequency::Quarterly), "QUARTERLY");
    }

    #[test]
    fn interval_to_gql_maps_every_known_value() {
        assert_eq!(interval_to_gql("1m"), "ONE_MINUTE");
        assert_eq!(interval_to_gql("5m"), "FIVE_MINUTES");
        assert_eq!(interval_to_gql("15m"), "FIFTEEN_MINUTES");
        assert_eq!(interval_to_gql("30m"), "THIRTY_MINUTES");
        assert_eq!(interval_to_gql("1h"), "ONE_HOUR");
        assert_eq!(interval_to_gql("1d"), "ONE_DAY");
        assert_eq!(interval_to_gql("1wk"), "ONE_WEEK");
        assert_eq!(interval_to_gql("1mo"), "ONE_MONTH");
        assert_eq!(interval_to_gql("3mo"), "THREE_MONTHS");
    }

    #[test]
    fn interval_to_gql_falls_back_to_one_day_for_unknown_input() {
        assert_eq!(interval_to_gql("not-a-real-interval"), "ONE_DAY");
        assert_eq!(interval_to_gql(""), "ONE_DAY");
    }

    #[test]
    fn range_to_gql_maps_every_known_value() {
        assert_eq!(range_to_gql("1d"), "ONE_DAY");
        assert_eq!(range_to_gql("5d"), "FIVE_DAYS");
        assert_eq!(range_to_gql("1mo"), "ONE_MONTH");
        assert_eq!(range_to_gql("3mo"), "THREE_MONTHS");
        assert_eq!(range_to_gql("6mo"), "SIX_MONTHS");
        assert_eq!(range_to_gql("1y"), "ONE_YEAR");
        assert_eq!(range_to_gql("2y"), "TWO_YEARS");
        assert_eq!(range_to_gql("5y"), "FIVE_YEARS");
        assert_eq!(range_to_gql("10y"), "TEN_YEARS");
        assert_eq!(range_to_gql("ytd"), "YEAR_TO_DATE");
        assert_eq!(range_to_gql("max"), "MAX");
    }

    #[test]
    fn range_to_gql_falls_back_to_one_month_for_unknown_input() {
        assert_eq!(range_to_gql("not-a-real-range"), "ONE_MONTH");
        assert_eq!(range_to_gql(""), "ONE_MONTH");
    }
}
