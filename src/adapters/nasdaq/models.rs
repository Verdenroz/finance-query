//! Wire DTOs for `api.nasdaq.com`'s calendar endpoints.
//!
//! Field names mirror the live response exactly — the dividend endpoint in
//! particular uses inconsistent casing (`dividend_Ex_Date`, not
//! `dividendExDate`), so most structs list `#[serde(rename)]` per field
//! rather than a blanket `rename_all`.

use serde::Deserialize;

/// Shared `{ "rows": [...] }` shape most Nasdaq calendar endpoints use.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct NasdaqRows<T> {
    pub(super) rows: Option<Vec<T>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct NasdaqEarningsEnvelope {
    pub(super) data: Option<NasdaqRows<NasdaqEarningsRow>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct NasdaqEarningsRow {
    pub(super) symbol: Option<String>,
    pub(super) time: Option<String>,
    pub(super) fiscal_quarter_ending: Option<String>,
    pub(super) eps_forecast: Option<String>,
    #[serde(default)]
    pub(super) eps: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct NasdaqSplitsEnvelope {
    pub(super) data: Option<NasdaqRows<NasdaqSplitRow>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct NasdaqSplitRow {
    pub(super) symbol: Option<String>,
    pub(super) ratio: Option<String>,
    #[serde(rename = "executionDate")]
    pub(super) execution_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct NasdaqDividendsEnvelope {
    pub(super) data: Option<NasdaqDividendData>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct NasdaqDividendData {
    pub(super) calendar: Option<NasdaqRows<NasdaqDividendRow>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct NasdaqDividendRow {
    pub(super) symbol: Option<String>,
    #[serde(rename = "dividend_Ex_Date")]
    pub(super) ex_date: Option<String>,
    #[serde(rename = "payment_Date")]
    pub(super) payment_date: Option<String>,
    #[serde(rename = "record_Date")]
    pub(super) record_date: Option<String>,
    #[serde(rename = "dividend_Rate")]
    pub(super) dividend_rate: Option<f64>,
    #[serde(rename = "announcement_Date")]
    pub(super) announcement_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct NasdaqIpoEnvelope {
    pub(super) data: Option<NasdaqIpoData>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct NasdaqIpoData {
    pub(super) priced: Option<NasdaqRows<NasdaqIpoRow>>,
    pub(super) upcoming: Option<NasdaqIpoUpcoming>,
    pub(super) filed: Option<NasdaqRows<NasdaqIpoRow>>,
    pub(super) withdrawn: Option<NasdaqRows<NasdaqIpoRow>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct NasdaqIpoUpcoming {
    pub(super) upcoming_table: NasdaqRows<NasdaqIpoRow>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct NasdaqIpoRow {
    pub(super) proposed_ticker_symbol: Option<String>,
    pub(super) company_name: Option<String>,
    pub(super) proposed_exchange: Option<String>,
    pub(super) proposed_share_price: Option<String>,
    pub(super) shares_offered: Option<String>,
    #[serde(default)]
    pub(super) priced_date: Option<String>,
    #[serde(default)]
    pub(super) expected_price_date: Option<String>,
    #[serde(default)]
    pub(super) filed_date: Option<String>,
    #[serde(default)]
    pub(super) withdraw_date: Option<String>,
}
