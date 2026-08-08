use finance_query::{Frequency, Interval, StatementType, TimeRange};

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

/// Map an interval to its GraphQL enum literal.
pub fn interval_to_gql(interval: Interval) -> &'static str {
    match interval {
        Interval::OneMinute => "ONE_MINUTE",
        Interval::TwoMinutes => "TWO_MINUTES",
        Interval::FiveMinutes => "FIVE_MINUTES",
        Interval::FifteenMinutes => "FIFTEEN_MINUTES",
        Interval::ThirtyMinutes => "THIRTY_MINUTES",
        Interval::OneHour => "ONE_HOUR",
        Interval::NinetyMinutes => "NINETY_MINUTES",
        Interval::OneDay => "ONE_DAY",
        Interval::FiveDays => "FIVE_DAYS",
        Interval::OneWeek => "ONE_WEEK",
        Interval::OneMonth => "ONE_MONTH",
        Interval::ThreeMonths => "THREE_MONTHS",
    }
}

/// Map a time range to its GraphQL enum literal.
pub fn range_to_gql(range: TimeRange) -> &'static str {
    match range {
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
    fn interval_to_gql_maps_every_variant() {
        assert_eq!(interval_to_gql(Interval::OneMinute), "ONE_MINUTE");
        assert_eq!(interval_to_gql(Interval::TwoMinutes), "TWO_MINUTES");
        assert_eq!(interval_to_gql(Interval::FiveMinutes), "FIVE_MINUTES");
        assert_eq!(interval_to_gql(Interval::FifteenMinutes), "FIFTEEN_MINUTES");
        assert_eq!(interval_to_gql(Interval::ThirtyMinutes), "THIRTY_MINUTES");
        assert_eq!(interval_to_gql(Interval::OneHour), "ONE_HOUR");
        assert_eq!(interval_to_gql(Interval::NinetyMinutes), "NINETY_MINUTES");
        assert_eq!(interval_to_gql(Interval::OneDay), "ONE_DAY");
        assert_eq!(interval_to_gql(Interval::FiveDays), "FIVE_DAYS");
        assert_eq!(interval_to_gql(Interval::OneWeek), "ONE_WEEK");
        assert_eq!(interval_to_gql(Interval::OneMonth), "ONE_MONTH");
        assert_eq!(interval_to_gql(Interval::ThreeMonths), "THREE_MONTHS");
    }

    #[test]
    fn range_to_gql_maps_every_variant() {
        assert_eq!(range_to_gql(TimeRange::OneDay), "ONE_DAY");
        assert_eq!(range_to_gql(TimeRange::FiveDays), "FIVE_DAYS");
        assert_eq!(range_to_gql(TimeRange::OneMonth), "ONE_MONTH");
        assert_eq!(range_to_gql(TimeRange::ThreeMonths), "THREE_MONTHS");
        assert_eq!(range_to_gql(TimeRange::SixMonths), "SIX_MONTHS");
        assert_eq!(range_to_gql(TimeRange::OneYear), "ONE_YEAR");
        assert_eq!(range_to_gql(TimeRange::TwoYears), "TWO_YEARS");
        assert_eq!(range_to_gql(TimeRange::FiveYears), "FIVE_YEARS");
        assert_eq!(range_to_gql(TimeRange::TenYears), "TEN_YEARS");
        assert_eq!(range_to_gql(TimeRange::YearToDate), "YEAR_TO_DATE");
        assert_eq!(range_to_gql(TimeRange::Max), "MAX");
    }
}
