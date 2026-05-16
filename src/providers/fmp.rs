//! Financial Modeling Prep (FMP) provider implementation.

use super::types;
use crate::adapters::fmp;
use crate::error::Result;

pub(crate) struct FmpProvider;

#[async_trait::async_trait]
impl super::ProviderAdapter for FmpProvider {
    fn id(&self) -> &'static str {
        "fmp"
    }
    fn name(&self) -> &'static str {
        "Financial Modeling Prep"
    }

    fn capabilities(&self) -> super::Capability {
        super::Capability::QUOTE
            | super::Capability::CHART
            | super::Capability::FUNDAMENTALS
            | super::Capability::CORPORATE
            | super::Capability::DISCOVERY
            | super::Capability::MARKET
            | super::Capability::INDICES
            | super::Capability::COMMODITIES
            | super::Capability::FOREX
            | super::Capability::CRYPTO
            | super::Capability::TECHNICALS
    }

    async fn fetch_quote(&self, symbol: &str) -> Result<types::QuoteData> {
        let quotes = fmp::quote(symbol).await?;
        let q = quotes.first();
        Ok(types::QuoteData {
            symbol: symbol.to_string(),
            provider_id: "fmp",
            regular_market_price: q.and_then(|q| q.price),
            regular_market_change: q.and_then(|q| q.change),
            regular_market_change_percent: q.and_then(|q| q.changes_percentage),
            regular_market_volume: q.and_then(|q| q.volume.map(|v| v as u64)),
            regular_market_day_high: q.and_then(|q| q.day_high),
            regular_market_day_low: q.and_then(|q| q.day_low),
            market_cap: q.and_then(|q| q.market_cap),
            fifty_two_week_high: q.and_then(|q| q.year_high),
            fifty_two_week_low: q.and_then(|q| q.year_low),
            exchange: q.and_then(|q| q.exchange.clone()),
            ..Default::default()
        })
    }

    async fn fetch_chart(
        &self,
        symbol: &str,
        _interval: crate::Interval,
        range: crate::TimeRange,
    ) -> Result<types::ChartData> {
        let (from, to) = range_dates(range);
        let params = fmp::HistoricalPriceParams {
            from: Some(from),
            to: Some(to),
        };
        let resp = fmp::historical_price_daily(symbol, Some(params)).await?;
        let candles: Vec<types::CandleData> = resp
            .historical
            .into_iter()
            .filter_map(|r| {
                let ts = chrono::NaiveDate::parse_from_str(r.date.as_deref()?, "%Y-%m-%d")
                    .ok()?
                    .and_hms_opt(0, 0, 0)?
                    .and_utc()
                    .timestamp();
                Some(types::CandleData {
                    timestamp: ts,
                    open: r.open?,
                    high: r.high?,
                    low: r.low?,
                    close: r.close?,
                    volume: r.volume.map(|v| v as u64).unwrap_or(0),
                })
            })
            .collect();
        Ok(types::ChartData {
            symbol: symbol.to_string(),
            provider_id: "fmp",
            candles,
            meta: types::ChartMetaData::default(),
            extras: Default::default(),
        })
    }

    async fn fetch_chart_range(
        &self,
        symbol: &str,
        interval: crate::Interval,
        start: i64,
        end: i64,
    ) -> Result<types::ChartData> {
        let from = timestamp_to_date_str(start);
        let to = timestamp_to_date_str(end);
        let params = fmp::HistoricalPriceParams {
            from: Some(from),
            to: Some(to),
        };

        let candles: Vec<types::CandleData> = match interval {
            crate::Interval::OneDay
            | crate::Interval::OneWeek
            | crate::Interval::OneMonth
            | crate::Interval::ThreeMonths => {
                let resp = fmp::historical_price_daily(symbol, Some(params)).await?;
                resp.historical
                    .into_iter()
                    .filter_map(|r| {
                        let ts = chrono::NaiveDate::parse_from_str(r.date.as_deref()?, "%Y-%m-%d")
                            .ok()?
                            .and_hms_opt(0, 0, 0)?
                            .and_utc()
                            .timestamp();
                        Some(types::CandleData {
                            timestamp: ts,
                            open: r.open?,
                            high: r.high?,
                            low: r.low?,
                            close: r.close?,
                            volume: r.volume.map(|v| v as u64).unwrap_or(0),
                        })
                    })
                    .collect()
            }
            _ => {
                let interval_str = fmp_interval_str(interval);
                let points =
                    fmp::historical_price_intraday(symbol, interval_str, Some(params)).await?;
                points
                    .into_iter()
                    .filter_map(|r| {
                        let ts = chrono::NaiveDateTime::parse_from_str(
                            r.date.as_deref()?,
                            "%Y-%m-%d %H:%M:%S",
                        )
                        .ok()?
                        .and_utc()
                        .timestamp();
                        Some(types::CandleData {
                            timestamp: ts,
                            open: r.open?,
                            high: r.high?,
                            low: r.low?,
                            close: r.close?,
                            volume: r.volume.map(|v| v as u64).unwrap_or(0),
                        })
                    })
                    .collect()
            }
        };

        Ok(types::ChartData {
            symbol: symbol.to_string(),
            provider_id: "fmp",
            candles,
            meta: types::ChartMetaData::default(),
            extras: Default::default(),
        })
    }

    async fn fetch_financials(
        &self,
        symbol: &str,
        stmt_type: crate::StatementType,
        frequency: crate::Frequency,
    ) -> Result<types::FinancialStatementData> {
        let period = match frequency {
            crate::Frequency::Annual => fmp::Period::Annual,
            crate::Frequency::Quarterly => fmp::Period::Quarter,
        };
        let limit = Some(50u32);

        let data = match stmt_type {
            crate::StatementType::Income => {
                let stmts = fmp::income_statement(symbol, period, limit).await?;
                pivot_financials(stmts)
            }
            crate::StatementType::Balance => {
                let stmts = fmp::balance_sheet(symbol, period, limit).await?;
                pivot_financials(stmts)
            }
            crate::StatementType::CashFlow => {
                let stmts = fmp::cash_flow(symbol, period, limit).await?;
                pivot_financials(stmts)
            }
        };

        Ok(types::FinancialStatementData {
            provider_id: "fmp",
            symbol: symbol.to_string(),
            statement_type: stmt_type.as_str().to_string(),
            frequency: frequency.as_str().to_string(),
            data,
        })
    }

    async fn fetch_events(&self, symbol: &str) -> Result<types::EventsData> {
        let divs = fmp::historical_dividends(symbol).await?;
        let splits = fmp::historical_splits(symbol).await?;

        let dividends: Vec<types::DividendData> = divs
            .historical
            .into_iter()
            .filter_map(|d| {
                let ts = chrono::NaiveDate::parse_from_str(d.date.as_deref()?, "%Y-%m-%d")
                    .ok()?
                    .and_hms_opt(0, 0, 0)?
                    .and_utc()
                    .timestamp();
                Some(types::DividendData {
                    date: ts,
                    amount: d.adj_dividend.or(d.dividend).unwrap_or(0.0),
                })
            })
            .collect();

        let stock_splits: Vec<types::SplitData> = splits
            .historical
            .into_iter()
            .filter_map(|s| {
                let ts = chrono::NaiveDate::parse_from_str(s.date.as_deref()?, "%Y-%m-%d")
                    .ok()?
                    .and_hms_opt(0, 0, 0)?
                    .and_utc()
                    .timestamp();
                Some(types::SplitData {
                    date: ts,
                    numerator: s.numerator.unwrap_or(1.0) as u32,
                    denominator: s.denominator.unwrap_or(1.0) as u32,
                })
            })
            .collect();

        Ok(types::EventsData {
            provider_id: "fmp",
            dividends,
            splits: stock_splits,
            capital_gains: vec![],
        })
    }

    async fn fetch_news(&self, symbol: &str) -> Result<Vec<types::NewsData>> {
        let articles = fmp::stock_news(symbol, 50).await?;
        Ok(articles
            .into_iter()
            .map(|a| types::NewsData {
                provider_id: "fmp",
                title: a.title.unwrap_or_default(),
                url: a.url,
                source: a.site,
                published_at: a.published_date,
                summary: a.text,
                extras: Default::default(),
            })
            .collect())
    }

    async fn fetch_similar_symbols(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Vec<types::SimilarSymbolData>> {
        let peers = fmp::stock_peers(symbol).await?;
        let mut symbols: Vec<types::SimilarSymbolData> = peers
            .into_iter()
            .filter_map(|p| p.peers_list)
            .flatten()
            .map(|s| types::SimilarSymbolData {
                symbol: s,
                score: None,
            })
            .collect();
        symbols.truncate(limit as usize);
        Ok(symbols)
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

/// Pivot a Vec of serde-serializable financial statements into the canonical
/// `HashMap<String, HashMap<String, serde_json::Value>>` format used by
/// `FinancialStatementData`.
///
/// Each statement should have a `date` field and arbitrary numeric metric fields.
fn pivot_financials<T: serde::Serialize>(
    stmts: Vec<T>,
) -> std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>> {
    let mut data: std::collections::HashMap<
        String,
        std::collections::HashMap<String, serde_json::Value>,
    > = std::collections::HashMap::new();

    for stmt in stmts {
        let val = match serde_json::to_value(&stmt) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let obj = match val.as_object() {
            Some(o) => o,
            None => continue,
        };

        let date = match obj.get("date").and_then(|v| v.as_str()) {
            Some(d) if !d.is_empty() => d.to_string(),
            _ => continue,
        };

        for (key, value) in obj {
            if key == "date" {
                continue;
            }
            // Only include numeric values
            match &value {
                serde_json::Value::Number(_) => {}
                _ => continue,
            }
            data.entry(key.clone())
                .or_default()
                .insert(date.clone(), value.clone());
        }
    }

    data
}
