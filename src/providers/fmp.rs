//! Financial Modeling Prep (FMP) provider implementation.

use super::Provider;
use crate::error::Result;

pub(crate) struct FmpProvider;

#[async_trait::async_trait]
impl super::ProviderAdapter for FmpProvider {
    fn id(&self) -> &'static str {
        "fmp"
    }
    fn capabilities(&self) -> super::Capability {
        super::Capability::QUOTE
            | super::Capability::CHART
            | super::Capability::FUNDAMENTALS
            | super::Capability::CORPORATE
            | super::Capability::INDICES
            | super::Capability::COMMODITIES
            | super::Capability::FOREX
            | super::Capability::CRYPTO
    }

    async fn initialize(&self) -> Result<()> {
        let key = std::env::var("FMP_API_KEY").map_err(|_| {
            crate::error::FinanceError::InvalidParameter {
                param: "fmp".into(),
                reason: "FMP_API_KEY not set. Set the environment variable or call fmp::init(key)."
                    .into(),
            }
        })?;
        let _ = crate::adapters::fmp::init(key);
        Ok(())
    }

    async fn fetch_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::quote::QuoteSummaryResponse> {
        crate::adapters::fmp::quote::fetch_canonical_quote(symbol).await
    }

    async fn fetch_chart(
        &self,
        symbol: &str,
        interval: crate::Interval,
        range: crate::TimeRange,
    ) -> Result<crate::models::chart::Chart> {
        let (from, to) = range_dates(range);

        let candles: Vec<crate::models::chart::Candle> = match interval {
            crate::Interval::OneDay
            | crate::Interval::OneWeek
            | crate::Interval::OneMonth
            | crate::Interval::ThreeMonths => {
                let params = crate::adapters::fmp::quote::HistoricalPriceParams {
                    from: Some(from),
                    to: Some(to),
                };
                crate::adapters::fmp::quote::fetch_daily_chart_candles(symbol, Some(params)).await?
            }
            _ => {
                let interval_str = fmp_interval_str(interval);
                let params = crate::adapters::fmp::quote::HistoricalPriceParams {
                    from: Some(from),
                    to: Some(to),
                };
                crate::adapters::fmp::quote::fetch_intraday_chart_candles(
                    symbol,
                    interval_str,
                    Some(params),
                )
                .await?
            }
        };

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

        let candles: Vec<crate::models::chart::Candle> = match interval {
            crate::Interval::OneDay
            | crate::Interval::OneWeek
            | crate::Interval::OneMonth
            | crate::Interval::ThreeMonths => {
                let params = crate::adapters::fmp::quote::HistoricalPriceParams {
                    from: Some(from),
                    to: Some(to),
                };
                crate::adapters::fmp::quote::fetch_daily_chart_candles(symbol, Some(params)).await?
            }
            _ => {
                let interval_str = fmp_interval_str(interval);
                let params = crate::adapters::fmp::quote::HistoricalPriceParams {
                    from: Some(from),
                    to: Some(to),
                };
                crate::adapters::fmp::quote::fetch_intraday_chart_candles(
                    symbol,
                    interval_str,
                    Some(params),
                )
                .await?
            }
        };

        Ok(crate::models::chart::Chart {
            symbol: symbol.to_string(),
            meta: Default::default(),
            candles,
            interval: Some(interval),
            range: None,
            provider_id: Some(Provider::Fmp),
        })
    }

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

    async fn fetch_events(
        &self,
        symbol: &str,
    ) -> Result<crate::models::chart::events::ChartEvents> {
        crate::adapters::fmp::corporate::fetch_canonical_events(symbol).await
    }

    async fn fetch_news(&self, symbol: &str) -> Result<Vec<crate::models::corporate::news::News>> {
        crate::adapters::fmp::corporate::fetch_canonical_news(symbol, 50).await
    }

    async fn fetch_similar_symbols(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::corporate::recommendation::SimilarSymbol>> {
        crate::adapters::fmp::quote::fetch_canonical_similar_symbols(symbol, limit).await
    }

    async fn fetch_forex_quote(
        &self,
        from: &str,
        to: &str,
    ) -> Result<crate::models::forex::ForexQuote> {
        crate::adapters::fmp::forex::fetch_canonical_forex_quote(from, to).await
    }

    async fn fetch_indices_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::indices::IndexQuote> {
        crate::adapters::fmp::indices::fetch_canonical_index_quote(symbol).await
    }

    async fn fetch_commodities_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::commodities::CommodityQuote> {
        crate::adapters::fmp::commodities::fetch_canonical_commodity_quote(symbol).await
    }

    // ── Crypto ─────────────────────────────────────────────────

    async fn fetch_crypto_quote(
        &self,
        id: &str,
        vs_currency: &str,
    ) -> Result<crate::models::crypto::CryptoQuote> {
        crate::adapters::fmp::crypto::fetch_canonical_crypto_quote(id, vs_currency).await
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
