//! Nasdaq provider implementation (keyless).
//!
//! Serves `CalendarProvider` only — earnings, IPO, dividend, and split
//! calendars. Economic releases and market holidays/status aren't Nasdaq
//! calendar data and fall through to the next routed provider.

use super::{CalendarProvider, ProviderAdapter, ProviderCore};
use crate::error::Result;

pub(crate) struct NasdaqProvider;

impl ProviderCore for NasdaqProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Nasdaq
    }
}

#[async_trait::async_trait]
impl CalendarProvider for NasdaqProvider {
    async fn fetch_market_calendar(
        &self,
        kind: crate::models::calendar::market::CalendarKind,
        from: &str,
        to: &str,
    ) -> Result<Vec<crate::models::calendar::market::MarketCalendarEntry>> {
        use crate::models::calendar::market::CalendarKind;

        if !matches!(
            kind,
            CalendarKind::Earnings
                | CalendarKind::Ipo
                | CalendarKind::Dividend
                | CalendarKind::Split
        ) {
            return Err(self.not_supported(kind.operation()));
        }
        crate::adapters::nasdaq::fetch_market_calendar_response(kind, from, to).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for NasdaqProvider {
    fn as_calendar(&self) -> Option<&dyn CalendarProvider> {
        Some(self)
    }
}
