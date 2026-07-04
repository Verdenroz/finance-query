//! GraphQL types for the financial event calendar: earnings, dividends,
//! options expirations, and (with `fred`) market-wide economic releases.
//!
//! `EventKind` is an internally-tagged Rust enum with distinct fields per
//! variant — modeled here as a GraphQL union (one object type per variant)
//! rather than a flat object, since the variants don't share a field set.

use async_graphql::{Json, SimpleObject, Union};

/// Upcoming earnings report with analyst estimate data.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlEarningsEvent {
    pub eps_estimate_low: Option<f64>,
    pub eps_estimate_avg: Option<f64>,
    pub eps_estimate_high: Option<f64>,
    pub revenue_estimate_avg: Option<i64>,
    pub is_estimate: bool,
}

/// Ex-dividend date — shares must be held before this date to receive the dividend.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlExDividendEvent {
    pub amount: Option<f64>,
}

/// Dividend payment date — cash arrives in the account.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlDividendPaymentEvent {
    pub amount: Option<f64>,
}

/// Standard monthly options expiration (3rd Friday) for a ticker.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlOptionsExpirationEvent {
    pub contract_count: Option<i64>,
}

/// Economic data release (requires the `fred` feature).
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlEconomicReleaseEvent {
    pub name: String,
    pub series_id: String,
}

/// Fallback for `EventKind` variants added to the library after this schema
/// was written — `EventKind` is `#[non_exhaustive]`, so this keeps the
/// conversion total instead of panicking on a future variant.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlUnknownEvent {
    pub raw: Json<serde_json::Value>,
}

/// The kind of financial event, with its event-specific payload.
#[derive(Union, Debug, Clone)]
pub enum GqlEventKind {
    Earnings(GqlEarningsEvent),
    ExDividend(GqlExDividendEvent),
    DividendPayment(GqlDividendPaymentEvent),
    OptionsExpiration(GqlOptionsExpirationEvent),
    EconomicRelease(GqlEconomicReleaseEvent),
    Unknown(GqlUnknownEvent),
}

/// A single upcoming financial event.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlCalendarEvent {
    pub timestamp: i64,
    pub date: String,
    pub symbol: Option<String>,
    pub event: GqlEventKind,
}

impl From<finance_query::EventKind> for GqlEventKind {
    fn from(kind: finance_query::EventKind) -> Self {
        match kind {
            finance_query::EventKind::Earnings {
                eps_estimate_low,
                eps_estimate_avg,
                eps_estimate_high,
                revenue_estimate_avg,
                is_estimate,
            } => GqlEventKind::Earnings(GqlEarningsEvent {
                eps_estimate_low,
                eps_estimate_avg,
                eps_estimate_high,
                revenue_estimate_avg,
                is_estimate,
            }),
            finance_query::EventKind::ExDividend { amount } => {
                GqlEventKind::ExDividend(GqlExDividendEvent { amount })
            }
            finance_query::EventKind::DividendPayment { amount } => {
                GqlEventKind::DividendPayment(GqlDividendPaymentEvent { amount })
            }
            finance_query::EventKind::OptionsExpiration { contract_count } => {
                GqlEventKind::OptionsExpiration(GqlOptionsExpirationEvent {
                    contract_count: contract_count.map(|c| c as i64),
                })
            }
            finance_query::EventKind::EconomicRelease { name, series_id } => {
                GqlEventKind::EconomicRelease(GqlEconomicReleaseEvent { name, series_id })
            }
            other => GqlEventKind::Unknown(GqlUnknownEvent {
                raw: Json(serde_json::to_value(&other).unwrap_or(serde_json::Value::Null)),
            }),
        }
    }
}

impl From<finance_query::CalendarEvent> for GqlCalendarEvent {
    fn from(event: finance_query::CalendarEvent) -> Self {
        GqlCalendarEvent {
            timestamp: event.timestamp,
            date: event.date,
            symbol: event.symbol,
            event: event.event.into(),
        }
    }
}
