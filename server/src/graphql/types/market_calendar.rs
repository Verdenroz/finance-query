//! GraphQL types for the market-wide event calendar (`Capability::CALENDAR`,
//! via `Providers::calendar()`). Unlike the per-symbol `calendar` field
//! (`Tickers::calendar()`), these span the whole market over a date range.
//!
//! `CalendarDetail` is an internally-tagged Rust enum with distinct fields
//! per variant — modeled here as a GraphQL union, mirroring the
//! `GqlEventKind` pattern in `types/calendar.rs`.

use async_graphql::{Enum, Json, SimpleObject, Union};

/// Which market-wide calendar to fetch.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum GqlCalendarKind {
    Earnings,
    Ipo,
    Dividend,
    Split,
    Economic,
    MarketHoliday,
    MarketStatus,
}

impl From<GqlCalendarKind> for finance_query::CalendarKind {
    fn from(v: GqlCalendarKind) -> Self {
        match v {
            GqlCalendarKind::Earnings => finance_query::CalendarKind::Earnings,
            GqlCalendarKind::Ipo => finance_query::CalendarKind::Ipo,
            GqlCalendarKind::Dividend => finance_query::CalendarKind::Dividend,
            GqlCalendarKind::Split => finance_query::CalendarKind::Split,
            GqlCalendarKind::Economic => finance_query::CalendarKind::Economic,
            GqlCalendarKind::MarketHoliday => finance_query::CalendarKind::MarketHoliday,
            GqlCalendarKind::MarketStatus => finance_query::CalendarKind::MarketStatus,
        }
    }
}

#[derive(SimpleObject, Debug, Clone, serde::Serialize)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlMarketEarningsDetail {
    pub eps: Option<f64>,
    pub eps_estimated: Option<f64>,
    pub revenue: Option<f64>,
    pub revenue_estimated: Option<f64>,
    pub fiscal_date_ending: Option<String>,
    pub time: Option<String>,
}

#[derive(SimpleObject, Debug, Clone, serde::Serialize)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlMarketIpoDetail {
    pub company: Option<String>,
    pub exchange: Option<String>,
    pub actions: Option<String>,
    pub shares: Option<f64>,
    pub price_range: Option<String>,
    pub market_cap: Option<f64>,
}

#[derive(SimpleObject, Debug, Clone, serde::Serialize)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlMarketDividendDetail {
    pub dividend: Option<f64>,
    pub adj_dividend: Option<f64>,
    pub record_date: Option<String>,
    pub payment_date: Option<String>,
    pub declaration_date: Option<String>,
}

#[derive(SimpleObject, Debug, Clone, serde::Serialize)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlMarketSplitDetail {
    pub numerator: Option<f64>,
    pub denominator: Option<f64>,
}

/// Also used for `MarketStatus` entries — providers report exchange
/// open/closed status through this same shape (`status` carries "open"/"closed").
#[derive(SimpleObject, Debug, Clone, serde::Serialize)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlMarketHolidayDetail {
    pub name: Option<String>,
    pub exchange: Option<String>,
    pub status: Option<String>,
    pub open: Option<String>,
    pub close: Option<String>,
}

#[derive(SimpleObject, Debug, Clone, serde::Serialize)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlMarketEconomicDetail {
    pub event: Option<String>,
    pub country: Option<String>,
    pub actual: Option<f64>,
    pub previous: Option<f64>,
    pub estimate: Option<f64>,
    pub change: Option<f64>,
    pub change_percentage: Option<f64>,
    pub impact: Option<String>,
}

/// Fallback for `CalendarDetail` variants added to the library after this
/// schema was written — `CalendarDetail` is `#[non_exhaustive]`, so this
/// keeps the conversion total instead of panicking on a future variant.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlMarketUnknownDetail {
    pub raw: Json<serde_json::Value>,
}

#[derive(Union, Debug, Clone)]
pub enum GqlMarketCalendarDetail {
    Earnings(GqlMarketEarningsDetail),
    Ipo(GqlMarketIpoDetail),
    Dividend(GqlMarketDividendDetail),
    Split(GqlMarketSplitDetail),
    MarketHoliday(GqlMarketHolidayDetail),
    Economic(GqlMarketEconomicDetail),
    Unknown(GqlMarketUnknownDetail),
}

impl From<finance_query::CalendarDetail> for GqlMarketCalendarDetail {
    fn from(detail: finance_query::CalendarDetail) -> Self {
        match detail {
            finance_query::CalendarDetail::Earnings {
                eps,
                eps_estimated,
                revenue,
                revenue_estimated,
                fiscal_date_ending,
                time,
            } => GqlMarketCalendarDetail::Earnings(GqlMarketEarningsDetail {
                eps,
                eps_estimated,
                revenue,
                revenue_estimated,
                fiscal_date_ending,
                time,
            }),
            finance_query::CalendarDetail::Ipo {
                company,
                exchange,
                actions,
                shares,
                price_range,
                market_cap,
            } => GqlMarketCalendarDetail::Ipo(GqlMarketIpoDetail {
                company,
                exchange,
                actions,
                shares,
                price_range,
                market_cap,
            }),
            finance_query::CalendarDetail::Dividend {
                dividend,
                adj_dividend,
                record_date,
                payment_date,
                declaration_date,
            } => GqlMarketCalendarDetail::Dividend(GqlMarketDividendDetail {
                dividend,
                adj_dividend,
                record_date,
                payment_date,
                declaration_date,
            }),
            finance_query::CalendarDetail::Split {
                numerator,
                denominator,
            } => GqlMarketCalendarDetail::Split(GqlMarketSplitDetail {
                numerator,
                denominator,
            }),
            finance_query::CalendarDetail::MarketHoliday {
                name,
                exchange,
                status,
                open,
                close,
            } => GqlMarketCalendarDetail::MarketHoliday(GqlMarketHolidayDetail {
                name,
                exchange,
                status,
                open,
                close,
            }),
            finance_query::CalendarDetail::Economic {
                event,
                country,
                actual,
                previous,
                estimate,
                change,
                change_percentage,
                impact,
            } => GqlMarketCalendarDetail::Economic(GqlMarketEconomicDetail {
                event,
                country,
                actual,
                previous,
                estimate,
                change,
                change_percentage,
                impact,
            }),
            other => GqlMarketCalendarDetail::Unknown(GqlMarketUnknownDetail {
                raw: Json(serde_json::to_value(&other).unwrap_or(serde_json::Value::Null)),
            }),
        }
    }
}

/// A single market-wide calendar entry.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlMarketCalendarEntry {
    pub symbol: Option<String>,
    pub date: Option<String>,
    pub detail: GqlMarketCalendarDetail,
}

impl From<finance_query::MarketCalendarEntry> for GqlMarketCalendarEntry {
    fn from(entry: finance_query::MarketCalendarEntry) -> Self {
        GqlMarketCalendarEntry {
            symbol: entry.symbol,
            date: entry.date,
            detail: entry.detail.into(),
        }
    }
}
