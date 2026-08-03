//! Period-label resolution shared by the macro-data adapters.
//!
//! BLS and the World Bank spell their period labels differently (`"Q01"`,
//! `"2023Q3"`, `"M04"`), but both resolve to the same thing:
//! [`MacroObservation::date`](crate::models::economic::MacroObservation::date),
//! documented as the `YYYY-MM-DD` **start** of the period. These helpers own
//! that convention so the adapters cannot drift from it.

/// `YYYY-MM-DD` start of the `month`th month (1-12) of `year`.
pub(crate) fn month_start(year: &str, month: u32) -> String {
    format!("{year}-{month:02}-01")
}

/// `YYYY-MM-DD` start of the `quarter`th quarter (1-4) of `year`.
pub(crate) fn quarter_start(year: &str, quarter: u32) -> String {
    month_start(year, (quarter - 1) * 3 + 1)
}

/// `YYYY-MM-DD` start of the `half`th half-year (1-2) of `year`.
#[cfg(feature = "bls")]
pub(crate) fn half_start(year: &str, half: u32) -> String {
    month_start(year, (half - 1) * 6 + 1)
}

/// `YYYY-01-01`, the start of `year`.
pub(crate) fn year_start(year: &str) -> String {
    format!("{year}-01-01")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarters_start_on_the_first_month_of_the_quarter() {
        assert_eq!(quarter_start("2026", 1), "2026-01-01");
        assert_eq!(quarter_start("2026", 2), "2026-04-01");
        assert_eq!(quarter_start("2026", 4), "2026-10-01");
    }

    #[test]
    fn months_and_years_are_zero_padded() {
        assert_eq!(month_start("2026", 4), "2026-04-01");
        assert_eq!(month_start("2026", 12), "2026-12-01");
        assert_eq!(year_start("2026"), "2026-01-01");
    }

    #[cfg(feature = "bls")]
    #[test]
    fn halves_start_in_january_and_july() {
        assert_eq!(half_start("2026", 1), "2026-01-01");
        assert_eq!(half_start("2026", 2), "2026-07-01");
    }
}
