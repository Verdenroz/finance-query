//! Nasdaq — keyless market-wide earnings/IPO/dividend/split calendars.
//!
//! Requires the **`nasdaq`** feature flag.
//!
//! `api.nasdaq.com`'s public calendar endpoints back this adapter, the same
//! undocumented API several third-party calendar wrappers already rely on.
//! Earnings, dividends, and splits are queried per calendar day; IPOs per
//! calendar month, since Nasdaq takes no date-range parameter for any of
//! them — a `[from, to]` calendar request fans out into one request per
//! day/month in range (see `calendar` for the range-iteration and mapping
//! logic, and `client` for the browser-`User-Agent` requirement).

pub(crate) mod calendar;
pub(crate) mod client;
mod models;

use std::time::Duration;

use crate::adapters::singleton::keyless_limiter;
use crate::error::Result;
use client::NasdaqClient;

/// Self-imposed pacing. Nasdaq documents no quota; requests fan out per
/// day/month in a range so keep this conservative.
const NASDAQ_RATE_PER_SEC: f64 = 2.0;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

keyless_limiter!(rate = NASDAQ_RATE_PER_SEC);

/// Build a client against the live API, reusing the shared token bucket.
fn client() -> Result<NasdaqClient> {
    NasdaqClient::new(DEFAULT_TIMEOUT, shared_limiter(), client::NASDAQ_BASE)
}

pub(crate) use calendar::fetch_market_calendar_response;
