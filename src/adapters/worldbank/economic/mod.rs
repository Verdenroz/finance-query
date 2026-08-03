//! `ECONOMIC` capability for World Bank Open Data.

use crate::adapters::common::periods;
use crate::error::Result;
use crate::models::economic::{EconomicSeries, MacroObservation};

use super::models::WorldBankObservation;

/// Country used when a series id names only an indicator — the World Bank's
/// world aggregate, so a bare `"NY.GDP.MKTP.CD"` is still a valid series.
pub(crate) const DEFAULT_COUNTRY: &str = "WLD";

/// Split a `"<COUNTRY>/<INDICATOR>"` series id into its parts.
///
/// A bare indicator (no `/`) resolves against [`DEFAULT_COUNTRY`].
pub(crate) fn split_series_id(series_id: &str) -> (String, String) {
    match series_id.split_once('/') {
        Some((country, indicator)) if !country.is_empty() && !indicator.is_empty() => {
            (country.trim().to_uppercase(), indicator.trim().to_string())
        }
        _ => (DEFAULT_COUNTRY.to_string(), series_id.trim().to_string()),
    }
}

/// Normalise a World Bank period label to the `YYYY-MM-DD` start of that
/// period, which is what [`MacroObservation::date`] documents.
///
/// Accepts `"2023"`, `"2023Q3"`, and `"2023M04"`; anything else is passed
/// through untouched rather than silently dropped.
pub(crate) fn normalize_date(period: &str) -> String {
    let period = period.trim();
    if let Some((year, quarter)) = period.split_once('Q')
        && let Ok(q) = quarter.parse::<u32>()
        && (1..=4).contains(&q)
    {
        return periods::quarter_start(year, q);
    }
    if let Some((year, month)) = period.split_once('M')
        && let Ok(m) = month.parse::<u32>()
        && (1..=12).contains(&m)
    {
        return periods::month_start(year, m);
    }
    if period.len() == 4 && period.chars().all(|c| c.is_ascii_digit()) {
        return periods::year_start(period);
    }
    period.to_string()
}

/// Infer the reporting frequency from the shape of a period label.
///
/// One label settles it — the World Bank never mixes base frequencies within
/// a single country/indicator series.
pub(crate) fn infer_frequency(period: Option<&str>) -> Option<String> {
    let first = period?;
    if first.contains('Q') {
        Some("Quarterly".to_string())
    } else if first.contains('M') {
        Some("Monthly".to_string())
    } else if first.len() == 4 {
        Some("Annual".to_string())
    } else {
        None
    }
}

/// Map raw observations onto the canonical [`EconomicSeries`].
///
/// The API returns newest-first; the canonical model documents chronological
/// order, so observations are reversed here.
pub(crate) fn to_canonical(series_id: &str, raw: Vec<WorldBankObservation>) -> EconomicSeries {
    let frequency = infer_frequency(raw.first().map(|o| o.date.as_str()));

    let title = raw
        .first()
        .and_then(|o| o.indicator.as_ref())
        .and_then(|i| i.value.clone());
    let country = raw
        .first()
        .and_then(|o| o.country.as_ref())
        .and_then(|c| c.value.clone());
    let units = raw
        .iter()
        .find_map(|o| o.unit.as_ref().filter(|u| !u.trim().is_empty()).cloned());

    let mut observations: Vec<MacroObservation> = raw
        .into_iter()
        .map(|o| MacroObservation {
            date: normalize_date(&o.date),
            value: o.value,
        })
        .collect();
    observations.reverse();

    EconomicSeries {
        series_id: series_id.to_string(),
        // The indicator name alone ("GDP (current US$)") doesn't say which
        // country it describes, and the series id may have defaulted to WLD.
        title: match (title, country) {
            (Some(t), Some(c)) => Some(format!("{t} — {c}")),
            (Some(t), None) => Some(t),
            (None, c) => c,
        },
        units,
        frequency,
        observations,
    }
}

/// Fetch a World Bank indicator series as the canonical
/// [`EconomicSeries`](crate::models::economic::EconomicSeries).
pub(crate) async fn fetch_economic_series_response(series_id: &str) -> Result<EconomicSeries> {
    let (country, indicator) = split_series_id(series_id);
    let raw = super::client()?.indicator(&country, &indicator).await?;
    Ok(to_canonical(series_id, raw))
}
