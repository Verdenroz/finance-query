//! Financial Modeling Prep (FMP) provider implementation.

use super::{
    CalendarProvider, ChartProvider, CommoditiesProvider, CorporateProvider, CryptoProvider,
    DiscoveryProvider, ForexProvider, FundamentalsProvider, IndicesProvider, MarketProvider,
    Provider, ProviderAdapter, ProviderCore, QuoteProvider,
};
use crate::error::Result;

pub(crate) struct FmpProvider;

impl ProviderCore for FmpProvider {
    fn id(&self) -> Provider {
        Provider::Fmp
    }
}

#[async_trait::async_trait]
impl QuoteProvider for FmpProvider {
    async fn fetch_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::quote::QuoteSummaryResponse> {
        crate::adapters::fmp::quote::fetch_canonical_quote(symbol).await
    }

    async fn fetch_quotes_batch(
        &self,
        symbols: &[&str],
    ) -> Result<Vec<(String, crate::models::quote::QuoteSummaryResponse)>> {
        crate::adapters::fmp::quote::fetch_canonical_quotes_batch(symbols).await
    }
}

#[async_trait::async_trait]
impl ChartProvider for FmpProvider {
    async fn fetch_chart(
        &self,
        symbol: &str,
        interval: crate::Interval,
        range: crate::TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        let (from, to) = range_dates(range);
        let candles = fetch_candles(symbol, interval, from, to).await?;

        Ok(crate::models::chart::Chart {
            symbol: symbol.to_string(),
            meta: Default::default(),
            candles,
            interval: Some(interval),
            range: Some(range),
            provider_id: Some(Provider::Fmp),
        })
    }

    async fn fetch_chart_range(
        &self,
        symbol: &str,
        interval: crate::Interval,
        start: i64,
        end: i64,
    ) -> Result<crate::models::chart::Chart> {
        let from = timestamp_to_date_str(start);
        let to = timestamp_to_date_str(end);
        let candles = fetch_candles(symbol, interval, from, to).await?;

        Ok(crate::models::chart::Chart {
            symbol: symbol.to_string(),
            meta: Default::default(),
            candles,
            interval: Some(interval),
            range: None,
            provider_id: Some(Provider::Fmp),
        })
    }
}

#[async_trait::async_trait]
impl FundamentalsProvider for FmpProvider {
    async fn fetch_financials(
        &self,
        symbol: &str,
        stmt_type: crate::StatementType,
        frequency: crate::Frequency,
    ) -> Result<crate::models::fundamentals::FinancialStatement> {
        let period = match frequency {
            crate::Frequency::Annual => crate::adapters::fmp::Period::Annual,
            crate::Frequency::Quarterly => crate::adapters::fmp::Period::Quarter,
        };
        let limit = Some(50u32);

        let data = crate::adapters::fmp::fundamentals::fetch_financials_data(
            symbol, stmt_type, period, limit,
        )
        .await?;

        Ok(super::build_financial_statement(
            symbol.to_string(),
            stmt_type.as_str().to_string(),
            frequency.as_str().to_string(),
            Provider::Fmp,
            data,
        ))
    }

    async fn fetch_price_target_consensus(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::PriceTargetConsensus> {
        crate::adapters::fmp::fundamentals::consensus::fetch_price_target_consensus_response(symbol)
            .await
    }

    async fn fetch_price_target_summary(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::PriceTargetSummary> {
        crate::adapters::fmp::fundamentals::consensus::fetch_price_target_summary_response(symbol)
            .await
    }

    async fn fetch_rating_consensus(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::RatingConsensus> {
        crate::adapters::fmp::fundamentals::consensus::fetch_rating_consensus_response(symbol).await
    }

    async fn fetch_key_metrics_ttm(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::KeyMetricsTtm> {
        crate::adapters::fmp::fundamentals::ttm::fetch_key_metrics_ttm_response(symbol).await
    }

    async fn fetch_ratios_ttm(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::FinancialRatiosTtm> {
        crate::adapters::fmp::fundamentals::ttm::fetch_ratios_ttm_response(symbol).await
    }

    async fn fetch_share_float(
        &self,
        symbol: &str,
    ) -> Result<crate::models::fundamentals::ShareFloat> {
        crate::adapters::fmp::fundamentals::float::fetch_share_float_response(symbol).await
    }
}

#[async_trait::async_trait]
impl CorporateProvider for FmpProvider {
    async fn fetch_news(&self, symbol: &str) -> Result<Vec<crate::models::corporate::news::News>> {
        crate::adapters::fmp::corporate::fetch_canonical_news(symbol, 50).await
    }

    async fn fetch_events(
        &self,
        symbol: &str,
    ) -> Result<crate::models::chart::events::ChartEvents> {
        crate::adapters::fmp::corporate::fetch_canonical_events(symbol).await
    }

    async fn fetch_similar_symbols(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::corporate::recommendation::SimilarSymbol>> {
        crate::adapters::fmp::quote::fetch_canonical_similar_symbols(symbol, limit).await
    }

    async fn fetch_press_releases(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::corporate::press_release::PressRelease>> {
        crate::adapters::fmp::corporate::fetch_press_releases_response(symbol, limit).await
    }

    async fn fetch_executive_compensation(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::models::corporate::governance::ExecutiveCompensation>> {
        crate::adapters::fmp::corporate::ownership::fetch_executive_compensation_response(symbol)
            .await
    }

    async fn fetch_employee_count(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::models::corporate::governance::EmployeeCount>> {
        crate::adapters::fmp::corporate::ownership::fetch_employee_count_response(symbol).await
    }
}

#[async_trait::async_trait]
impl DiscoveryProvider for FmpProvider {
    async fn fetch_symbol_search(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::discovery::reference::SymbolMatch>> {
        crate::adapters::fmp::discovery::screener::fetch_symbol_search_response(query, limit).await
    }

    async fn fetch_screener(
        &self,
        filters: &crate::models::discovery::reference::ScreenerFilters,
    ) -> Result<Vec<crate::models::discovery::reference::ScreenerMatch>> {
        crate::adapters::fmp::discovery::screener::fetch_screener_response(filters).await
    }
}

#[async_trait::async_trait]
impl CalendarProvider for FmpProvider {
    async fn fetch_market_calendar(
        &self,
        kind: crate::models::calendar::market::CalendarKind,
        from: &str,
        to: &str,
    ) -> Result<Vec<crate::models::calendar::market::MarketCalendarEntry>> {
        crate::adapters::fmp::market::calendars::fetch_market_calendar_response(kind, from, to)
            .await
    }
}

#[async_trait::async_trait]
impl MarketProvider for FmpProvider {
    async fn fetch_market_movers(
        &self,
        direction: crate::models::market::performance::MoverDirection,
    ) -> Result<Vec<crate::models::market::performance::MoverQuote>> {
        crate::adapters::fmp::market::market_performance::fetch_market_movers_response(direction)
            .await
    }

    async fn fetch_sector_performance_history(
        &self,
        limit: u32,
    ) -> Result<Vec<crate::models::market::performance::SectorPerformanceHistory>> {
        crate::adapters::fmp::market::market_performance::fetch_sector_performance_history_response(
            limit,
        )
        .await
    }

    async fn fetch_sector_performance(
        &self,
    ) -> Result<Vec<crate::models::market::performance::SectorPerformance>> {
        crate::adapters::fmp::market::market_performance::fetch_sector_performance_response().await
    }

    async fn fetch_sector_pe(&self) -> Result<Vec<crate::models::market::performance::SectorPe>> {
        crate::adapters::fmp::market::market_performance::fetch_sector_pe_response().await
    }

    async fn fetch_industry_pe(
        &self,
    ) -> Result<Vec<crate::models::market::performance::IndustryPe>> {
        crate::adapters::fmp::market::market_performance::fetch_industry_pe_response().await
    }
}

#[async_trait::async_trait]
impl ForexProvider for FmpProvider {
    async fn fetch_forex_quote(
        &self,
        from: &str,
        to: &str,
    ) -> Result<crate::models::forex::ForexQuote> {
        crate::adapters::fmp::forex::fetch_canonical_forex_quote(from, to).await
    }
}

#[async_trait::async_trait]
impl IndicesProvider for FmpProvider {
    async fn fetch_indices_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::indices::IndexQuote> {
        crate::adapters::fmp::indices::fetch_canonical_index_quote(symbol).await
    }

    async fn fetch_index_constituents(
        &self,
        index: crate::models::indices::MajorIndex,
    ) -> Result<Vec<crate::models::indices::IndexConstituent>> {
        crate::adapters::fmp::indices::fetch_index_constituents_response(index).await
    }

    async fn fetch_index_constituent_changes(
        &self,
        index: crate::models::indices::MajorIndex,
    ) -> Result<Vec<crate::models::indices::IndexConstituentChange>> {
        crate::adapters::fmp::indices::fetch_index_constituent_changes_response(index).await
    }
}

#[async_trait::async_trait]
impl CommoditiesProvider for FmpProvider {
    async fn fetch_commodities_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::commodities::CommodityQuote> {
        crate::adapters::fmp::commodities::fetch_canonical_commodity_quote(symbol).await
    }
}

#[async_trait::async_trait]
impl CryptoProvider for FmpProvider {
    async fn fetch_crypto_quote(
        &self,
        id: &str,
        vs_currency: &str,
    ) -> Result<crate::models::crypto::CryptoQuote> {
        crate::adapters::fmp::crypto::fetch_canonical_crypto_quote(id, vs_currency).await
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for FmpProvider {
    async fn initialize(&self) -> Result<()> {
        let _ = crate::adapters::fmp::build_client()?;
        Ok(())
    }

    fn as_quote(&self) -> Option<&dyn QuoteProvider> {
        Some(self)
    }
    fn as_chart(&self) -> Option<&dyn ChartProvider> {
        Some(self)
    }
    fn as_fundamentals(&self) -> Option<&dyn FundamentalsProvider> {
        Some(self)
    }
    fn as_corporate(&self) -> Option<&dyn CorporateProvider> {
        Some(self)
    }
    fn as_discovery(&self) -> Option<&dyn DiscoveryProvider> {
        Some(self)
    }
    fn as_calendar(&self) -> Option<&dyn CalendarProvider> {
        Some(self)
    }
    fn as_market(&self) -> Option<&dyn MarketProvider> {
        Some(self)
    }
    fn as_crypto(&self) -> Option<&dyn CryptoProvider> {
        Some(self)
    }
    fn as_forex(&self) -> Option<&dyn ForexProvider> {
        Some(self)
    }
    fn as_indices(&self) -> Option<&dyn IndicesProvider> {
        Some(self)
    }
    fn as_commodities(&self) -> Option<&dyn CommoditiesProvider> {
        Some(self)
    }
}

/// Fetch daily or intraday candles for `[from, to]`, picking the endpoint by
/// interval granularity.
async fn fetch_candles(
    symbol: &str,
    interval: crate::Interval,
    from: String,
    to: String,
) -> Result<Vec<crate::models::chart::Candle>> {
    let params = crate::adapters::fmp::quote::HistoricalPriceParams {
        from: Some(from),
        to: Some(to),
    };
    match interval {
        crate::Interval::OneDay
        | crate::Interval::OneWeek
        | crate::Interval::OneMonth
        | crate::Interval::ThreeMonths => {
            crate::adapters::fmp::quote::fetch_daily_chart_candles(symbol, Some(params)).await
        }
        _ => {
            crate::adapters::fmp::quote::fetch_intraday_chart_candles(
                symbol,
                fmp_interval_str(interval),
                Some(params),
            )
            .await
        }
    }
}

fn range_dates(range: crate::TimeRange) -> (String, String) {
    super::range_to_dates(range)
}

/// Convert a unix timestamp to YYYY-MM-DD date string.
fn timestamp_to_date_str(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "1970-01-01".to_string())
}

/// Map crate::Interval to FMP intraday interval string.
fn fmp_interval_str(interval: crate::Interval) -> &'static str {
    match interval {
        crate::Interval::OneMinute => "1min",
        crate::Interval::FiveMinutes => "5min",
        crate::Interval::FifteenMinutes => "15min",
        crate::Interval::ThirtyMinutes => "30min",
        crate::Interval::OneHour => "1hour",
        _ => "1hour", // fallback for daily/weekly/monthly
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Interval;

    #[test]
    fn timestamp_to_date_str_formats_utc_day() {
        // 2024-01-02T00:00:00Z.
        assert_eq!(timestamp_to_date_str(1_704_153_600), "2024-01-02");
        // Within the day still rounds to the same date string.
        assert_eq!(timestamp_to_date_str(1_704_153_600 + 3_600), "2024-01-02");
    }

    #[test]
    fn timestamp_to_date_str_clamps_invalid_to_epoch() {
        assert_eq!(timestamp_to_date_str(i64::MAX), "1970-01-01");
    }

    #[test]
    fn fmp_interval_str_maps_intraday_and_falls_back() {
        assert_eq!(fmp_interval_str(Interval::OneMinute), "1min");
        assert_eq!(fmp_interval_str(Interval::FiveMinutes), "5min");
        assert_eq!(fmp_interval_str(Interval::FifteenMinutes), "15min");
        assert_eq!(fmp_interval_str(Interval::ThirtyMinutes), "30min");
        assert_eq!(fmp_interval_str(Interval::OneHour), "1hour");
        // Daily/weekly/monthly aren't intraday intervals → documented fallback.
        assert_eq!(fmp_interval_str(Interval::OneDay), "1hour");
        assert_eq!(fmp_interval_str(Interval::OneWeek), "1hour");
    }

    #[test]
    fn range_dates_returns_well_formed_ordered_bounds() {
        let (from, to) = range_dates(crate::TimeRange::OneMonth);
        // YYYY-MM-DD shape.
        assert_eq!(from.len(), 10);
        assert_eq!(to.len(), 10);
        assert_eq!(from.matches('-').count(), 2);
        // Start precedes end (lexicographic works for ISO dates).
        assert!(from <= to, "from {from} should be <= to {to}");
    }
}
