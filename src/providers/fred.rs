//! FRED (Federal Reserve Economic Data) provider implementation.

use super::{CalendarProvider, EconomicProvider, ProviderAdapter, ProviderCore};
use crate::error::Result;
use crate::providers::Operation;

pub(crate) struct FredProvider;

impl ProviderCore for FredProvider {
    fn id(&self) -> super::Provider {
        super::Provider::Fred
    }
}

#[async_trait::async_trait]
impl EconomicProvider for FredProvider {
    async fn fetch_economic_series(
        &self,
        series_id: &str,
    ) -> Result<finance_query_models::economic::EconomicSeries> {
        finance_query_adapters_gov::fred::fetch_economic_series_response(series_id).await
    }

    async fn fetch_economic_series_as_of(
        &self,
        series_id: &str,
        date: &str,
    ) -> Result<finance_query_models::economic::EconomicSeries> {
        finance_query_adapters_gov::fred::economic::catalog::series_as_of(series_id, date).await
    }

    async fn fetch_economic_search(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<finance_query_models::economic::EconomicSeriesMatch>> {
        finance_query_adapters_gov::fred::economic::catalog::search(query, limit).await
    }

    async fn fetch_economic_categories(
        &self,
        parent_id: i64,
    ) -> Result<Vec<finance_query_models::economic::EconomicCategory>> {
        finance_query_adapters_gov::fred::economic::catalog::categories(parent_id).await
    }

    async fn fetch_economic_releases(
        &self,
    ) -> Result<Vec<finance_query_models::economic::EconomicRelease>> {
        finance_query_adapters_gov::fred::economic::catalog::releases().await
    }
}

/// FRED's release calendar covers `CalendarKind::Economic` only — it has no
/// earnings, IPO, dividend, split, or holiday data.
#[async_trait::async_trait]
impl CalendarProvider for FredProvider {
    async fn fetch_market_calendar(
        &self,
        kind: finance_query_models::calendar::market::CalendarKind,
        from: &str,
        to: &str,
    ) -> Result<Vec<finance_query_models::calendar::market::MarketCalendarEntry>> {
        if kind != finance_query_models::calendar::market::CalendarKind::Economic {
            return Err(self.not_supported(Operation::from_calendar(kind)));
        }
        finance_query_adapters_gov::fred::fetch_market_calendar_response(from, to).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for FredProvider {
    async fn initialize(&self) -> crate::error::Result<()> {
        let _ = finance_query_adapters_gov::fred::build_client()?;
        Ok(())
    }

    fn rate_limit_remaining(&self) -> Option<f64> {
        finance_query_adapters_gov::fred::rate_limiter().and_then(|l| l.available_estimate())
    }

    fn as_economic(&self) -> Option<&dyn EconomicProvider> {
        Some(self)
    }

    fn as_calendar(&self) -> Option<&dyn CalendarProvider> {
        Some(self)
    }
}
