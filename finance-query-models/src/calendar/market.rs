//! Market-wide calendar models.
//!
//! Returned by the [`Capability::CALENDAR`](finance_query_types::Capability::CALENDAR) route
//! via `Providers::calendar`. These span the whole
//! market over a date range, unlike [`CalendarEvent`](crate::calendar::CalendarEvent),
//! which builds a per-symbol timeline from already-fetched Yahoo quote data.

use finance_query_derive::FieldsInit;
use serde::{Deserialize, Serialize};

/// Which market-wide calendar to fetch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CalendarKind {
    /// Earnings releases.
    Earnings,
    /// Initial public offerings.
    Ipo,
    /// Dividend payments.
    Dividend,
    /// Stock splits.
    Split,
    /// Macro-economic releases.
    Economic,
    /// Market holidays and early closes.
    MarketHoliday,
    /// Live exchange open/closed status — a snapshot, not a dated event, so
    /// providers serving it ignore the `from`/`to` range.
    MarketStatus,
}

/// One market-wide calendar entry.
///
/// The `kind`-specific payload lives in [`MarketCalendarEntry::detail`].
#[derive(Debug, Clone, Serialize, Deserialize, FieldsInit)]
#[non_exhaustive]
pub struct MarketCalendarEntry {
    /// Ticker symbol. `None` for economic releases, which are market-wide.
    pub symbol: Option<String>,
    /// Event date as reported by the provider (`YYYY-MM-DD`, or a timestamp
    /// string for economic releases).
    pub date: Option<String>,
    /// Event-specific payload.
    pub detail: CalendarDetail,
}

/// The event-specific payload of a [`MarketCalendarEntry`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CalendarDetail {
    /// An earnings release, with actuals once reported.
    ///
    /// Providers report different subsets; anything a provider lacks is `None`.
    #[non_exhaustive]
    Earnings {
        /// Reported EPS, if already released.
        eps: Option<f64>,
        /// Consensus EPS estimate.
        eps_estimated: Option<f64>,
        /// Reported revenue, if already released.
        revenue: Option<f64>,
        /// Consensus revenue estimate.
        revenue_estimated: Option<f64>,
        /// End of the fiscal period being reported (`YYYY-MM-DD`).
        fiscal_date_ending: Option<String>,
        /// Time of day for the release (e.g. `"amc"`, `"bmo"`).
        time: Option<String>,
        /// Company name.
        company: Option<String>,
        /// Number of analyst estimates behind the consensus.
        estimate_count: Option<u32>,
        /// Reported EPS against the consensus, in percent, once released.
        surprise_percent: Option<f64>,
        /// EPS reported for the same quarter a year earlier.
        previous_eps: Option<f64>,
        /// Date of the same quarter's release a year earlier (`YYYY-MM-DD`).
        previous_report_date: Option<String>,
        /// Market capitalisation when the calendar was read.
        market_cap: Option<f64>,
    },
    /// An initial public offering.
    Ipo {
        /// Company name.
        company: Option<String>,
        /// Listing exchange.
        exchange: Option<String>,
        /// Corporate action description (e.g. `"expected"`, `"priced"`).
        actions: Option<String>,
        /// Shares offered.
        shares: Option<f64>,
        /// Offering price range as reported (e.g. `"17.00-19.00"`).
        price_range: Option<String>,
        /// Market capitalisation at offering.
        market_cap: Option<f64>,
    },
    /// A dividend payment.
    Dividend {
        /// Dividend amount per share.
        dividend: Option<f64>,
        /// Split-adjusted dividend amount per share.
        adj_dividend: Option<f64>,
        /// Record date (`YYYY-MM-DD`).
        record_date: Option<String>,
        /// Payment date (`YYYY-MM-DD`).
        payment_date: Option<String>,
        /// Declaration date (`YYYY-MM-DD`).
        declaration_date: Option<String>,
    },
    /// A stock split.
    Split {
        /// Split ratio numerator (new shares).
        numerator: Option<f64>,
        /// Split ratio denominator (old shares).
        denominator: Option<f64>,
    },
    /// A market holiday or early close.
    MarketHoliday {
        /// Holiday name (e.g. `"Thanksgiving"`).
        name: Option<String>,
        /// Exchange the holiday applies to (e.g. `"NYSE"`).
        exchange: Option<String>,
        /// Status (e.g. `"closed"`, `"early-close"`).
        status: Option<String>,
        /// Open time, when the exchange opens late or closes early.
        open: Option<String>,
        /// Close time, when the exchange closes early.
        close: Option<String>,
    },
    /// A macro-economic release.
    Economic {
        /// Event name (e.g. `"CPI m/m"`).
        event: Option<String>,
        /// Country code or name.
        country: Option<String>,
        /// Reported value, if already released.
        actual: Option<f64>,
        /// Previous period's value.
        previous: Option<f64>,
        /// Consensus estimate.
        estimate: Option<f64>,
        /// Absolute change from the previous value.
        change: Option<f64>,
        /// Percentage change from the previous value.
        change_percentage: Option<f64>,
        /// Provider-assigned impact rating (e.g. `"High"`).
        impact: Option<String>,
    },
}

/// Field-complete initializer for [`CalendarDetail::Earnings`], whose variant is
/// `#[non_exhaustive]` and so cannot be built with a literal from another crate.
#[doc(hidden)]
pub struct EarningsDetailFieldsInit {
    pub eps: Option<f64>,
    pub eps_estimated: Option<f64>,
    pub revenue: Option<f64>,
    pub revenue_estimated: Option<f64>,
    pub fiscal_date_ending: Option<String>,
    pub time: Option<String>,
    pub company: Option<String>,
    pub estimate_count: Option<u32>,
    pub surprise_percent: Option<f64>,
    pub previous_eps: Option<f64>,
    pub previous_report_date: Option<String>,
    pub market_cap: Option<f64>,
}

impl From<EarningsDetailFieldsInit> for CalendarDetail {
    fn from(init: EarningsDetailFieldsInit) -> Self {
        let EarningsDetailFieldsInit {
            eps,
            eps_estimated,
            revenue,
            revenue_estimated,
            fiscal_date_ending,
            time,
            company,
            estimate_count,
            surprise_percent,
            previous_eps,
            previous_report_date,
            market_cap,
        } = init;
        Self::Earnings {
            eps,
            eps_estimated,
            revenue,
            revenue_estimated,
            fiscal_date_ending,
            time,
            company,
            estimate_count,
            surprise_percent,
            previous_eps,
            previous_report_date,
            market_cap,
        }
    }
}

#[cfg(feature = "dataframe")]
impl CalendarDetail {
    fn type_name(&self) -> &'static str {
        match self {
            Self::Earnings { .. } => "earnings",
            Self::Ipo { .. } => "ipo",
            Self::Dividend { .. } => "dividend",
            Self::Split { .. } => "split",
            Self::MarketHoliday { .. } => "market_holiday",
            Self::Economic { .. } => "economic",
        }
    }
}

#[cfg(feature = "dataframe")]
impl MarketCalendarEntry {
    /// Converts the entries to a polars DataFrame, one row per entry.
    ///
    /// Columns are `symbol` and `event_type` (strings) and `date` (`Date`,
    /// from the leading `YYYY-MM-DD`). The detail payload is not expanded
    /// because its fields differ per type.
    pub fn vec_to_dataframe(
        entries: &[Self],
    ) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        use polars::prelude::*;

        DataFrame::new(
            entries.len(),
            vec![
                Series::new(
                    "symbol".into(),
                    entries
                        .iter()
                        .map(|e| e.symbol.as_deref())
                        .collect::<Vec<_>>(),
                )
                .into(),
                crate::__dataframe::str_date(
                    "date".into(),
                    entries.iter().map(|e| e.date.as_deref()),
                ),
                Series::new(
                    "event_type".into(),
                    entries
                        .iter()
                        .map(|e| e.detail.type_name())
                        .collect::<Vec<_>>(),
                )
                .into(),
            ],
        )
    }

    /// Converts this entry to a one-row polars DataFrame; see [`vec_to_dataframe`](Self::vec_to_dataframe).
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        Self::vec_to_dataframe(std::slice::from_ref(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earnings_json_from_before_the_added_fields_still_deserializes() {
        let entry: MarketCalendarEntry = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "date": "2026-07-30",
            "detail": {
                "type": "earnings",
                "eps": 1.62,
                "eps_estimated": 1.5,
                "revenue": null,
                "revenue_estimated": null,
                "fiscal_date_ending": "2026-06-30",
                "time": "amc"
            }
        }))
        .unwrap();
        let CalendarDetail::Earnings {
            eps,
            time,
            company,
            market_cap,
            ..
        } = entry.detail
        else {
            panic!("expected Earnings detail");
        };
        assert_eq!((eps, time.as_deref()), (Some(1.62), Some("amc")));
        assert_eq!((company, market_cap), (None, None));
    }
}
