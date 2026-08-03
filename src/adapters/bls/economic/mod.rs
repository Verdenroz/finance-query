//! `ECONOMIC` capability for the BLS Public Data API.

use crate::adapters::common::numbers::parse_number;
use crate::adapters::common::periods;
use crate::error::Result;
use crate::models::economic::{EconomicSeries, MacroObservation};

use super::models::BlsSeries;

/// A BLS period label resolved to a date and a reporting frequency.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Period {
    /// `YYYY-MM-DD` start of the period.
    pub date: String,
    pub frequency: &'static str,
}

/// Resolve a `(year, period)` pair to the start date of that period.
///
/// Returns `None` for BLS's annual-aggregate periods (`M13`, `Q05`, `S03`).
/// Those rows are folded into an otherwise sub-annual series and would collide
/// with a real observation's date, so they are dropped rather than merged.
pub(super) fn resolve_period(year: &str, period: &str) -> Option<Period> {
    let (kind, n) = period.split_at_checked(1)?;
    let n: u32 = n.parse().ok()?;
    match kind {
        "M" if (1..=12).contains(&n) => Some(Period {
            date: periods::month_start(year, n),
            frequency: "Monthly",
        }),
        "Q" if (1..=4).contains(&n) => Some(Period {
            date: periods::quarter_start(year, n),
            frequency: "Quarterly",
        }),
        "S" if (1..=2).contains(&n) => Some(Period {
            date: periods::half_start(year, n),
            frequency: "Semiannual",
        }),
        "A" => Some(Period {
            date: periods::year_start(year),
            frequency: "Annual",
        }),
        // M13 / Q05 / S03 are annual aggregates, and anything else is a period
        // code this adapter does not know how to date.
        _ => None,
    }
}

/// Parse a BLS value string. `"-"` marks a figure BLS could not publish.
pub(super) fn parse_value(raw: &str) -> Option<f64> {
    parse_number(raw, &["-"])
}

/// Map a BLS series onto the canonical [`EconomicSeries`].
///
/// BLS returns newest-first; the canonical model documents chronological
/// order, so observations are sorted ascending here.
pub(super) fn to_canonical(series_id: &str, series: BlsSeries) -> EconomicSeries {
    let title = series.catalog.and_then(|c| c.series_title);

    let mut frequency: Option<&'static str> = None;
    let mut observations: Vec<MacroObservation> = series
        .data
        .into_iter()
        .filter_map(|point| {
            let period = resolve_period(&point.year, &point.period)?;
            // The first datable row settles the frequency; BLS never mixes
            // base frequencies within one series id.
            frequency.get_or_insert(period.frequency);
            Some(MacroObservation {
                date: period.date,
                value: parse_value(&point.value),
            })
        })
        .collect();
    observations.sort_by(|a, b| a.date.cmp(&b.date));

    EconomicSeries {
        series_id: series_id.to_string(),
        title,
        // BLS reports no unit field; the unit is implied by the series id and
        // stated in its title, so claiming one here would be a guess.
        units: None,
        frequency: frequency.map(str::to_string),
        observations,
    }
}

/// Fetch a BLS series as the canonical
/// [`EconomicSeries`](crate::models::economic::EconomicSeries).
pub(crate) async fn fetch_economic_series_response(series_id: &str) -> Result<EconomicSeries> {
    let series = super::client()?.series(series_id).await?;
    Ok(to_canonical(series_id, series))
}
