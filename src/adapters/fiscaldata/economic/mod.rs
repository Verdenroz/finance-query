//! `ECONOMIC` capability for US Treasury FiscalData.
//!
//! Resolves a series id to a dataset query, then folds the flat string rows
//! FiscalData returns into the canonical [`EconomicSeries`].

use crate::error::{FinanceError, Result};
use crate::models::economic::{EconomicSeries, MacroObservation};

use super::client::{DATE_FIELD, SeriesQuery};
use super::models::{FiscalMeta, FiscalRow};

/// A curated series: a stable short name for a `(dataset, column, filter)`
/// triple, so callers need not know FiscalData's table layout.
#[derive(Debug)]
pub(super) struct CuratedSeries {
    pub id: &'static str,
    pub dataset: &'static str,
    pub value_field: &'static str,
    /// Row filter for datasets that stack several series in one table.
    pub filter: Option<&'static str>,
    /// Stated here rather than derived from `meta.dataTypes`, which reports
    /// only `CURRENCY`/`PERCENTAGE` and so cannot distinguish dollars from
    /// millions of dollars.
    pub units: &'static str,
    pub frequency: &'static str,
}

/// The curated catalogue. Every entry is a single value per `record_date`.
pub(super) const CURATED: &[CuratedSeries] = &[
    CuratedSeries {
        id: "DEBT_TO_PENNY",
        dataset: "v2/accounting/od/debt_to_penny",
        value_field: "tot_pub_debt_out_amt",
        filter: None,
        units: "US Dollars",
        frequency: "Daily",
    },
    CuratedSeries {
        id: "DEBT_HELD_BY_PUBLIC",
        dataset: "v2/accounting/od/debt_to_penny",
        value_field: "debt_held_public_amt",
        filter: None,
        units: "US Dollars",
        frequency: "Daily",
    },
    CuratedSeries {
        id: "INTRAGOVERNMENTAL_HOLDINGS",
        dataset: "v2/accounting/od/debt_to_penny",
        value_field: "intragov_hold_amt",
        filter: None,
        units: "US Dollars",
        frequency: "Daily",
    },
    CuratedSeries {
        id: "AVG_INTEREST_RATE",
        dataset: "v2/accounting/od/avg_interest_rates",
        value_field: "avg_interest_rate_amt",
        filter: Some("security_desc:eq:Total Interest-bearing Debt"),
        units: "Percent",
        frequency: "Monthly",
    },
    CuratedSeries {
        id: "AVG_INTEREST_RATE_MARKETABLE",
        dataset: "v2/accounting/od/avg_interest_rates",
        value_field: "avg_interest_rate_amt",
        filter: Some("security_desc:eq:Total Marketable"),
        units: "Percent",
        frequency: "Monthly",
    },
    CuratedSeries {
        id: "OPERATING_CASH_BALANCE",
        dataset: "v1/accounting/dts/operating_cash_balance",
        value_field: "open_today_bal",
        filter: Some("account_type:eq:Treasury General Account (TGA) Opening Balance"),
        units: "Millions of US Dollars",
        frequency: "Daily",
    },
];

/// How a series id resolved: a curated entry, or a raw dataset passthrough.
#[derive(Debug)]
pub(super) enum Resolved<'a> {
    Curated(&'static CuratedSeries),
    /// `"<dataset path>:<value column>"` — the escape hatch for the ~50
    /// datasets the curated list does not name.
    Passthrough {
        dataset: &'a str,
        value_field: &'a str,
    },
}

impl Resolved<'_> {
    pub(super) fn query(&self) -> SeriesQuery<'_> {
        match self {
            Self::Curated(c) => SeriesQuery {
                dataset: c.dataset,
                value_field: c.value_field,
                filter: c.filter,
            },
            Self::Passthrough {
                dataset,
                value_field,
            } => SeriesQuery {
                dataset,
                value_field,
                filter: None,
            },
        }
    }
}

/// Resolve a series id against the curated catalogue, falling back to the
/// `"<dataset>:<column>"` passthrough form.
pub(super) fn resolve(series_id: &str) -> Result<Resolved<'_>> {
    let trimmed = series_id.trim();
    if let Some(curated) = CURATED.iter().find(|c| c.id.eq_ignore_ascii_case(trimmed)) {
        return Ok(Resolved::Curated(curated));
    }
    if let Some((dataset, value_field)) = trimmed.rsplit_once(':')
        && !dataset.is_empty()
        && !value_field.is_empty()
    {
        return Ok(Resolved::Passthrough {
            dataset,
            value_field,
        });
    }
    Err(FinanceError::InvalidParameter {
        param: "series_id".to_string(),
        reason: format!(
            "'{trimmed}' is not a FiscalData series. Use one of [{}], or the \
             passthrough form \"<dataset path>:<value column>\" \
             (e.g. \"v2/accounting/od/debt_to_penny:tot_pub_debt_out_amt\")",
            CURATED.iter().map(|c| c.id).collect::<Vec<_>>().join(", ")
        ),
    })
}

/// Parse one of FiscalData's string-encoded numbers.
///
/// A missing figure arrives as the literal string `"null"`, or as an empty
/// string, rather than as JSON `null`.
pub(super) fn parse_value(raw: &str) -> Option<f64> {
    let raw = raw.trim();
    if raw.is_empty() || raw.eq_ignore_ascii_case("null") {
        return None;
    }
    raw.parse::<f64>().ok()
}

/// Map raw rows onto the canonical [`EconomicSeries`].
pub(super) fn to_canonical(
    series_id: &str,
    resolved: &Resolved<'_>,
    rows: Vec<FiscalRow>,
    meta: &FiscalMeta,
) -> EconomicSeries {
    let value_field = resolved.query().value_field.to_string();

    let title = meta.labels.get(&value_field).cloned();
    let (units, frequency) = match resolved {
        Resolved::Curated(c) => (Some(c.units.to_string()), Some(c.frequency.to_string())),
        // Nothing better to go on for a passthrough than the column's type.
        Resolved::Passthrough { .. } => (
            meta.data_types.get(&value_field).map(|t| match t.as_str() {
                "CURRENCY" => "US Dollars".to_string(),
                "PERCENTAGE" => "Percent".to_string(),
                other => other.to_string(),
            }),
            None,
        ),
    };

    let observations: Vec<MacroObservation> = rows
        .into_iter()
        .filter_map(|mut row| {
            let date = row.remove(DATE_FIELD)?;
            let value = row.get(&value_field).and_then(|v| parse_value(v));
            Some(MacroObservation { date, value })
        })
        .collect();

    EconomicSeries {
        series_id: series_id.to_string(),
        title,
        units,
        frequency,
        observations,
    }
}

/// Fetch a FiscalData series as the canonical
/// [`EconomicSeries`](crate::models::economic::EconomicSeries).
pub(crate) async fn fetch_economic_series_response(series_id: &str) -> Result<EconomicSeries> {
    let resolved = resolve(series_id)?;
    let (rows, meta) = super::client()?.series(&resolved.query()).await?;
    Ok(to_canonical(series_id, &resolved, rows, &meta))
}
