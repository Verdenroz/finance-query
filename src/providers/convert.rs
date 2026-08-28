//! Shared DTO-to-model helpers used by more than one provider bridge.

use super::Provider;

#[allow(dead_code)] // used by fmp, polygon, alphavantage feature-gated providers
pub(crate) fn json_value_to_f64(value: serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_i64().map(|v| v as f64))
        .or_else(|| value.as_u64().map(|v| v as f64))
        .or_else(|| value.as_str().and_then(|s| s.parse::<f64>().ok()))
        .or_else(|| {
            value
                .get("raw")
                .and_then(|raw| raw.as_f64().or_else(|| raw.as_i64().map(|v| v as f64)))
        })
}

#[allow(dead_code)] // used by fmp, polygon, alphavantage feature-gated providers
pub(crate) fn build_financial_statement(
    symbol: String,
    statement_type: String,
    frequency: String,
    provider_id: Provider,
    data: std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>>,
) -> crate::models::fundamentals::FinancialStatement {
    let statement = data
        .into_iter()
        .filter_map(|(metric, values)| {
            let values: std::collections::HashMap<String, f64> = values
                .into_iter()
                .filter_map(|(date, value)| json_value_to_f64(value).map(|v| (date, v)))
                .collect();
            if values.is_empty() {
                None
            } else {
                Some((metric, values))
            }
        })
        .collect();
    crate::models::fundamentals::FinancialStatement {
        symbol,
        statement_type,
        frequency,
        statement,
        provider_id: Some(provider_id),
    }
}

pub(crate) fn build_options(
    symbol: String,
    provider_id: Provider,
    expiration_dates: Vec<i64>,
    calls: Vec<crate::models::options::OptionContract>,
    puts: Vec<crate::models::options::OptionContract>,
) -> crate::models::options::Options {
    use std::collections::BTreeMap;

    let mut chains_by_expiration: BTreeMap<
        i64,
        (
            Vec<crate::models::options::OptionContract>,
            Vec<crate::models::options::OptionContract>,
        ),
    > = BTreeMap::new();

    for contract in calls {
        let exp = contract.expiration.unwrap_or(0);
        chains_by_expiration
            .entry(exp)
            .or_default()
            .0
            .push(contract);
    }
    for contract in puts {
        let exp = contract.expiration.unwrap_or(0);
        chains_by_expiration
            .entry(exp)
            .or_default()
            .1
            .push(contract);
    }

    let option_chains: Vec<crate::models::options::response::OptionChainData> =
        chains_by_expiration
            .into_iter()
            .map(
                |(expiration, (c, p))| crate::models::options::response::OptionChainData {
                    expiration_date: expiration,
                    has_mini_options: None,
                    calls: Some(c),
                    puts: Some(p),
                },
            )
            .collect();

    let expiration_dates = if expiration_dates.is_empty() {
        option_chains
            .iter()
            .map(|chain| chain.expiration_date)
            .collect()
    } else {
        let mut v: Vec<i64> = expiration_dates;
        v.sort_unstable();
        v.dedup();
        v
    };

    let mut strikes: Vec<f64> = option_chains
        .iter()
        .flat_map(|chain| {
            chain
                .calls
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|c| c.strike)
                .chain(
                    chain
                        .puts
                        .as_deref()
                        .unwrap_or_default()
                        .iter()
                        .map(|p| p.strike),
                )
        })
        .collect();
    strikes.sort_by(|a, b| a.total_cmp(b));
    strikes.dedup_by(|a, b| a.total_cmp(b).is_eq());

    let result = crate::models::options::response::OptionChainResult {
        underlying_symbol: Some(symbol),
        expiration_dates: Some(expiration_dates),
        strikes: Some(strikes),
        has_mini_options: None,
        quote: None,
        options: option_chains,
    };

    crate::models::options::Options {
        option_chain: crate::models::options::response::OptionChainContainer {
            result: vec![result],
            error: None,
        },
        provider_id: Some(provider_id),
    }
}

#[allow(dead_code)] // used by fmp feature-gated provider
pub(crate) fn range_to_dates(range: crate::TimeRange) -> (String, String) {
    use chrono::{Datelike, Utc};
    let end = Utc::now();
    if range == crate::TimeRange::YearToDate {
        let year = end.year();
        let start = chrono::NaiveDate::from_ymd_opt(year, 1, 1)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|dt| dt.and_utc())
            .unwrap_or(end);
        return (
            start.format("%Y-%m-%d").to_string(),
            end.format("%Y-%m-%d").to_string(),
        );
    }
    let days = match range {
        crate::TimeRange::OneDay => 1,
        crate::TimeRange::FiveDays => 5,
        crate::TimeRange::OneMonth => 30,
        crate::TimeRange::ThreeMonths => 90,
        crate::TimeRange::SixMonths => 180,
        crate::TimeRange::OneYear => 365,
        crate::TimeRange::TwoYears => 730,
        crate::TimeRange::FiveYears => 1825,
        crate::TimeRange::TenYears => 3650,
        crate::TimeRange::Max => 36500,
        crate::TimeRange::YearToDate => unreachable!("YTD handled by early return above"),
    };
    let start = end - chrono::Duration::days(days);
    (
        start.format("%Y-%m-%d").to_string(),
        end.format("%Y-%m-%d").to_string(),
    )
}
