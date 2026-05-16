//! Alpha Vantage provider implementation.

use super::types;
use crate::adapters::alphavantage as av;
use crate::error::Result;

pub(crate) struct AlphaVantageProvider;

#[async_trait::async_trait]
impl super::ProviderAdapter for AlphaVantageProvider {
    fn id(&self) -> &'static str {
        "alphavantage"
    }
    fn name(&self) -> &'static str {
        "Alpha Vantage"
    }

    fn capabilities(&self) -> super::Capability {
        super::Capability::QUOTE
            | super::Capability::CHART
            | super::Capability::FUNDAMENTALS
            | super::Capability::CORPORATE
            | super::Capability::OPTIONS
            | super::Capability::CRYPTO
            | super::Capability::FOREX
            | super::Capability::COMMODITIES
            | super::Capability::ECONOMIC
            | super::Capability::TECHNICALS
    }

    async fn fetch_quote(&self, symbol: &str) -> Result<types::QuoteData> {
        let gq = av::global_quote(symbol).await?;
        Ok(types::QuoteData {
            symbol: symbol.to_string(),
            provider_id: "alphavantage",
            regular_market_price: Some(gq.price),
            regular_market_change: Some(gq.change),
            regular_market_change_percent: gq.change_percent.trim_end_matches('%').parse().ok(),
            regular_market_volume: Some(gq.volume as u64),
            regular_market_previous_close: Some(gq.previous_close),
            regular_market_open: Some(gq.open),
            regular_market_day_high: Some(gq.high),
            regular_market_day_low: Some(gq.low),
            ..Default::default()
        })
    }

    async fn fetch_chart(
        &self,
        symbol: &str,
        interval: crate::Interval,
        _range: crate::TimeRange,
    ) -> Result<types::ChartData> {
        let ts = match interval {
            crate::Interval::OneMinute => {
                av::time_series_intraday(symbol, av::AvInterval::OneMin, None).await?
            }
            crate::Interval::FiveMinutes => {
                av::time_series_intraday(symbol, av::AvInterval::FiveMin, None).await?
            }
            crate::Interval::FifteenMinutes => {
                av::time_series_intraday(symbol, av::AvInterval::FifteenMin, None).await?
            }
            crate::Interval::ThirtyMinutes => {
                av::time_series_intraday(symbol, av::AvInterval::ThirtyMin, None).await?
            }
            crate::Interval::OneHour => {
                av::time_series_intraday(symbol, av::AvInterval::SixtyMin, None).await?
            }
            _ => av::time_series_daily(symbol, None).await?,
        };

        let candles: Vec<types::CandleData> = ts
            .entries
            .into_iter()
            .map(|bar| {
                let ts_val = chrono::NaiveDateTime::parse_from_str(
                    &format!("{} 00:00:00", bar.timestamp),
                    "%Y-%m-%d %H:%M:%S",
                )
                .ok()
                .map(|dt| dt.and_utc().timestamp())
                .unwrap_or(0);
                types::CandleData {
                    timestamp: ts_val,
                    open: bar.open,
                    high: bar.high,
                    low: bar.low,
                    close: bar.close,
                    volume: bar.volume as u64,
                }
            })
            .collect();

        Ok(types::ChartData {
            symbol: symbol.to_string(),
            provider_id: "alphavantage",
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
        let stmts = match stmt_type {
            crate::StatementType::Income => av::income_statement(symbol).await?,
            crate::StatementType::Balance => av::balance_sheet(symbol).await?,
            crate::StatementType::CashFlow => av::cash_flow(symbol).await?,
        };

        let reports = match frequency {
            crate::Frequency::Annual => &stmts.annual_reports,
            crate::Frequency::Quarterly => &stmts.quarterly_reports,
        };

        let data = pivot_av_reports(reports);

        Ok(types::FinancialStatementData {
            provider_id: "alphavantage",
            symbol: symbol.to_string(),
            statement_type: stmt_type.as_str().to_string(),
            frequency: frequency.as_str().to_string(),
            data,
        })
    }

    async fn fetch_news(&self, symbol: &str) -> Result<Vec<types::NewsData>> {
        let articles = av::news_sentiment(Some(&[symbol]), None, Some(50)).await?;
        Ok(articles
            .into_iter()
            .map(|a| types::NewsData {
                provider_id: "alphavantage",
                title: a.title,
                url: Some(a.url),
                source: Some(a.source),
                published_at: Some(a.time_published),
                summary: Some(a.summary),
                extras: Default::default(),
            })
            .collect())
    }

    async fn fetch_events(&self, symbol: &str) -> Result<types::EventsData> {
        let divs = av::dividends(symbol).await?;
        let splits = av::splits(symbol).await?;

        let dividends: Vec<types::DividendData> = divs
            .into_iter()
            .filter_map(|d| {
                let ts = parse_av_date(d.ex_dividend_date.as_deref()?)?;
                Some(types::DividendData {
                    date: ts,
                    amount: d.amount.unwrap_or(0.0),
                })
            })
            .collect();

        let stock_splits: Vec<types::SplitData> = splits
            .into_iter()
            .filter_map(|s| {
                let ts = parse_av_date(s.effective_date.as_deref()?)?;
                let (num, den) = parse_split_ratio(s.split_ratio.as_deref().unwrap_or("1:1"));
                Some(types::SplitData {
                    date: ts,
                    numerator: num,
                    denominator: den,
                })
            })
            .collect();

        Ok(types::EventsData {
            provider_id: "alphavantage",
            dividends,
            splits: stock_splits,
            capital_gains: vec![],
        })
    }

    async fn fetch_chart_range(
        &self,
        symbol: &str,
        _interval: crate::Interval,
        start: i64,
        end: i64,
    ) -> Result<types::ChartData> {
        let from_date = timestamp_to_date_string(start);
        let to_date = timestamp_to_date_string(end);

        let ts = av::time_series_daily(symbol, Some(av::OutputSize::Full)).await?;

        let candles: Vec<types::CandleData> = ts
            .entries
            .into_iter()
            .filter(|bar| bar.timestamp >= from_date && bar.timestamp <= to_date)
            .map(|bar| {
                let ts_val = chrono::NaiveDate::parse_from_str(&bar.timestamp, "%Y-%m-%d")
                    .ok()
                    .and_then(|d| d.and_hms_opt(0, 0, 0))
                    .map(|dt| dt.and_utc().timestamp())
                    .unwrap_or(0);
                types::CandleData {
                    timestamp: ts_val,
                    open: bar.open,
                    high: bar.high,
                    low: bar.low,
                    close: bar.close,
                    volume: bar.volume as u64,
                }
            })
            .collect();

        Ok(types::ChartData {
            symbol: symbol.to_string(),
            provider_id: "alphavantage",
            candles,
            meta: types::ChartMetaData::default(),
            extras: Default::default(),
        })
    }
}

/// Pivot Alpha Vantage FinancialReportDTO entries into the canonical
/// `HashMap<String, HashMap<String, serde_json::Value>>` format.
fn pivot_av_reports(
    reports: &[av::FinancialReportDTO],
) -> std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>> {
    let mut data: std::collections::HashMap<
        String,
        std::collections::HashMap<String, serde_json::Value>,
    > = std::collections::HashMap::new();

    for report in reports {
        let date = &report.fiscal_date_ending;
        for (key, value) in &report.fields {
            // Only include numeric values
            if !value.is_number() {
                continue;
            }
            data.entry(key.clone())
                .or_default()
                .insert(date.clone(), value.clone());
        }
    }

    data
}

/// Parse an Alpha Vantage date string (YYYY-MM-DD) to a Unix timestamp.
/// Returns None if the string is empty or cannot be parsed.
fn parse_av_date(date_str: &str) -> Option<i64> {
    if date_str.is_empty() {
        return None;
    }
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc().timestamp())
}

/// Parse a split ratio string like "4:1" into (numerator, denominator).
fn parse_split_ratio(ratio: &str) -> (u32, u32) {
    let parts: Vec<&str> = ratio.split(':').collect();
    if parts.len() == 2 {
        let num = parts[0].parse::<u32>().unwrap_or(1);
        let den = parts[1].parse::<u32>().unwrap_or(1);
        (num, den)
    } else {
        (1, 1)
    }
}

/// Convert a Unix timestamp to a YYYY-MM-DD date string.
fn timestamp_to_date_string(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "1970-01-01".to_string())
}
