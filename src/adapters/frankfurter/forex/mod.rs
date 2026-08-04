//! `FOREX` capability for Frankfurter (ECB reference rates).

use crate::error::Result;
use crate::models::forex::ForexQuote;

/// Midnight UTC of an ISO `YYYY-MM-DD` date, as a unix timestamp.
///
/// ECB reference rates are a daily fix with no intraday time, so the date is
/// all the precision there is.
pub(super) fn date_to_timestamp(date: &str) -> Option<i64> {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .ok()?
        .and_hms_opt(0, 0, 0)
        .map(|dt| dt.and_utc().timestamp())
}

/// Build a [`ForexQuote`] from a chronological run of published rates.
///
/// `bid`/`ask` stay `None`: ECB rates are a reference fix, not a tradable
/// two-way price, and inventing a spread would misrepresent them.
pub(super) fn to_quote(from: &str, to: &str, series: &[(String, f64)]) -> Option<ForexQuote> {
    let (date, price) = series.last()?;
    let previous = series.len().checked_sub(2).and_then(|i| series.get(i));
    let change = previous.map(|(_, prev)| price - prev);
    let change_percent = previous
        .filter(|(_, prev)| *prev != 0.0)
        .map(|(_, prev)| (price - prev) / prev * 100.0);

    Some(ForexQuote {
        symbol: format!("{from}{to}"),
        base_currency: Some(from.to_string()),
        quote_currency: Some(to.to_string()),
        bid: None,
        ask: None,
        price: Some(*price),
        change,
        change_percent,
        timestamp: date_to_timestamp(date),
    })
}

/// A pair of identical currencies, which Frankfurter rejects outright (HTTP
/// 422) rather than answering with the trivially correct rate of 1.
fn identity_quote(currency: &str) -> ForexQuote {
    ForexQuote {
        symbol: format!("{currency}{currency}"),
        base_currency: Some(currency.to_string()),
        quote_currency: Some(currency.to_string()),
        bid: None,
        ask: None,
        price: Some(1.0),
        change: Some(0.0),
        change_percent: Some(0.0),
        timestamp: Some(chrono::Utc::now().timestamp()),
    }
}

/// Fetch the current ECB reference rate for `from`→`to`.
pub(crate) async fn fetch_forex_quote_response(from: &str, to: &str) -> Result<ForexQuote> {
    let from = from.trim().to_uppercase();
    let to = to.trim().to_uppercase();
    if from == to {
        return Ok(identity_quote(&from));
    }

    let series = super::client()?.recent_rates(&from, &to).await?;
    to_quote(&from, &to, &series).ok_or_else(|| crate::error::FinanceError::SymbolNotFound {
        symbol: Some(format!("{from}{to}")),
        context: "Frankfurter returned no usable rate for this pair".to_string(),
    })
}
