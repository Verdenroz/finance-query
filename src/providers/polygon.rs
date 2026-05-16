//! Polygon.io provider implementation.

use super::types;
use crate::adapters::polygon;
use crate::error::Result;

pub(crate) struct PolygonProvider;

#[async_trait::async_trait]
impl super::ProviderAdapter for PolygonProvider {
    fn id(&self) -> &'static str {
        "polygon"
    }
    fn name(&self) -> &'static str {
        "Polygon.io"
    }

    fn capabilities(&self) -> super::Capability {
        super::Capability::QUOTE
            | super::Capability::CHART
            | super::Capability::FUNDAMENTALS
            | super::Capability::CORPORATE
            | super::Capability::OPTIONS
            | super::Capability::CRYPTO
            | super::Capability::FOREX
            | super::Capability::FUTURES
            | super::Capability::INDICES
            | super::Capability::FILINGS
            | super::Capability::TECHNICALS
            | super::Capability::SENTIMENT
            | super::Capability::ECONOMIC
            | super::Capability::MARKET
            | super::Capability::DISCOVERY
    }

    async fn fetch_quote(&self, symbol: &str) -> Result<types::QuoteData> {
        let snap = polygon::stock_snapshot(symbol).await?;
        let day = snap.ticker.as_ref().and_then(|t| t.day.as_ref());
        Ok(types::QuoteData {
            symbol: symbol.to_string(),
            provider_id: "polygon",
            regular_market_price: day.and_then(|d| d.close),
            regular_market_volume: day.and_then(|d| d.volume).map(|v| v as u64),
            regular_market_open: day.and_then(|d| d.open),
            regular_market_day_high: day.and_then(|d| d.high),
            regular_market_day_low: day.and_then(|d| d.low),
            ..Default::default()
        })
    }

    async fn fetch_chart(
        &self,
        symbol: &str,
        _interval: crate::Interval,
        range: crate::TimeRange,
    ) -> Result<types::ChartData> {
        let (from, to) = range_bounds(range);
        let (mult, timespan) = interval_polygon(_interval);
        let aggs = polygon::stock_aggregates(symbol, mult, timespan, &from, &to, None).await?;
        let candles: Vec<types::CandleData> = aggs
            .results
            .into_iter()
            .flatten()
            .map(|r| types::CandleData {
                timestamp: r.timestamp,
                open: r.open,
                high: r.high,
                low: r.low,
                close: r.close,
                volume: r.volume as u64,
            })
            .collect();
        Ok(types::ChartData {
            symbol: symbol.to_string(),
            provider_id: "polygon",
            candles,
            meta: types::ChartMetaData::default(),
            extras: Default::default(),
        })
    }

    async fn fetch_news(&self, symbol: &str) -> Result<Vec<types::NewsData>> {
        let limit = "50".to_string();
        let paginated = polygon::stock_news(&[("ticker", symbol), ("limit", &limit)]).await?;
        Ok(paginated
            .results
            .into_iter()
            .flatten()
            .map(|a| types::NewsData {
                provider_id: "polygon",
                title: a.title.unwrap_or_default(),
                url: a.article_url,
                source: a.publisher.and_then(|p| p.name),
                published_at: a.published_utc,
                summary: a.description,
                extras: Default::default(),
            })
            .collect())
    }

    async fn fetch_events(&self, symbol: &str) -> Result<types::EventsData> {
        let dividends = polygon::stock_dividends(&[("ticker", symbol)]).await?;
        let splits = polygon::stock_splits(&[("ticker", symbol)]).await?;

        let parse_date = |d: &Option<String>| -> i64 {
            d.as_ref()
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .and_then(|dt| dt.and_hms_opt(0, 0, 0))
                .map(|dt| dt.and_utc().timestamp())
                .unwrap_or(0)
        };

        Ok(types::EventsData {
            provider_id: "polygon",
            dividends: dividends
                .results
                .into_iter()
                .flatten()
                .map(|d| types::DividendData {
                    date: parse_date(&d.pay_date),
                    amount: d.cash_amount.unwrap_or(0.0),
                })
                .collect(),
            splits: splits
                .results
                .into_iter()
                .flatten()
                .map(|s| types::SplitData {
                    date: parse_date(&s.execution_date),
                    numerator: s.split_to.unwrap_or(1.0) as u32,
                    denominator: s.split_from.unwrap_or(1.0) as u32,
                })
                .collect(),
            capital_gains: Vec::new(),
        })
    }

    // ── Financials ────────────────────────────────────────────────

    async fn fetch_financials(
        &self,
        symbol: &str,
        stmt_type: crate::StatementType,
        frequency: crate::Frequency,
    ) -> Result<types::FinancialStatementData> {
        let poly_type = match frequency {
            crate::Frequency::Annual => "Y",
            crate::Frequency::Quarterly => "Q",
        };
        let paginated =
            polygon::stock_financials(symbol, &[("type", poly_type), ("limit", "100")]).await?;
        let results = paginated.results.unwrap_or_default();

        let statement_key = match stmt_type {
            crate::StatementType::Income => "income_statement",
            crate::StatementType::Balance => "balance_sheet",
            crate::StatementType::CashFlow => "cash_flow_statement",
        };

        let mut data: std::collections::HashMap<
            String,
            std::collections::HashMap<String, serde_json::Value>,
        > = std::collections::HashMap::new();

        for result in &results {
            let period = result
                .period_of_report_date
                .as_deref()
                .or(result.filing_date.as_deref())
                .unwrap_or("unknown");

            if let Some(ref financials) = result.financials
                && let Some(stmt_section) = financials.get(statement_key)
                && let Some(section_obj) = stmt_section.as_object()
            {
                for (metric, metric_obj) in section_obj {
                    // Polygon returns metrics as {value, unit, label, order}
                    let metric_value = metric_obj
                        .get("value")
                        .cloned()
                        .unwrap_or_else(|| metric_obj.clone());
                    data.entry(metric.clone())
                        .or_default()
                        .insert(period.to_string(), metric_value);
                }
            }
        }

        Ok(types::FinancialStatementData {
            provider_id: "polygon",
            symbol: symbol.to_string(),
            statement_type: stmt_type.as_str().to_string(),
            frequency: frequency.as_str().to_string(),
            data,
        })
    }

    // ── Chart Range ───────────────────────────────────────────────

    async fn fetch_chart_range(
        &self,
        symbol: &str,
        interval: crate::Interval,
        start: i64,
        end: i64,
    ) -> Result<types::ChartData> {
        let from = timestamp_to_date(start);
        let to = timestamp_to_date(end);
        let (mult, timespan) = interval_polygon(interval);
        let aggs = polygon::stock_aggregates(symbol, mult, timespan, &from, &to, None).await?;
        let candles: Vec<types::CandleData> = aggs
            .results
            .into_iter()
            .flatten()
            .map(|r| types::CandleData {
                timestamp: r.timestamp,
                open: r.open,
                high: r.high,
                low: r.low,
                close: r.close,
                volume: r.volume as u64,
            })
            .collect();
        Ok(types::ChartData {
            symbol: symbol.to_string(),
            provider_id: "polygon",
            candles,
            meta: types::ChartMetaData::default(),
            extras: Default::default(),
        })
    }

    // ── Options ───────────────────────────────────────────────────

    async fn fetch_options(&self, symbol: &str, date: Option<i64>) -> Result<types::OptionsData> {
        let parse_date = |d: &Option<String>| -> i64 {
            d.as_ref()
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .and_then(|dt| dt.and_hms_opt(0, 0, 0))
                .map(|dt| dt.and_utc().timestamp())
                .unwrap_or(0)
        };

        let date_str_opt = date.map(timestamp_to_date);
        let mut params: Vec<(&str, &str)> = vec![("underlying_ticker", symbol), ("limit", "1000")];
        if let Some(ref ds) = date_str_opt {
            params.push(("expiration_date", ds.as_str()));
        }

        let paginated = polygon::options_contracts(&params).await?;
        let contracts = paginated.results.unwrap_or_default();

        // Collect unique expiration dates (sorted).
        let expiration_dates: Vec<i64> = contracts
            .iter()
            .filter_map(|c| {
                let ts = parse_date(&c.expiration_date);
                if ts > 0 { Some(ts) } else { None }
            })
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        let map_contract = |c: &polygon::OptionsContractDTO| -> types::OptionContractData {
            types::OptionContractData {
                contract_symbol: c.ticker.clone().unwrap_or_default(),
                strike: c.strike_price.unwrap_or(0.0),
                currency: None,
                last_price: None,
                change: None,
                volume: None,
                open_interest: None,
                bid: None,
                ask: None,
                expiration: parse_date(&c.expiration_date),
                implied_volatility: None,
                in_the_money: None,
                extras: Default::default(),
            }
        };

        let calls: Vec<types::OptionContractData> = contracts
            .iter()
            .filter(|c| c.contract_type.as_deref() == Some("call"))
            .map(map_contract)
            .collect();

        let puts: Vec<types::OptionContractData> = contracts
            .iter()
            .filter(|c| c.contract_type.as_deref() == Some("put"))
            .map(map_contract)
            .collect();

        Ok(types::OptionsData {
            provider_id: "polygon",
            symbol: symbol.to_string(),
            expiration_dates,
            calls,
            puts,
            extras: Default::default(),
        })
    }

    // ── Recommendations ───────────────────────────────────────────

    async fn fetch_similar_symbols(
        &self,
        symbol: &str,
        _limit: u32,
    ) -> Result<Vec<types::SimilarSymbolData>> {
        let paginated = polygon::related_tickers(symbol).await?;
        Ok(paginated
            .results
            .unwrap_or_default()
            .into_iter()
            .map(|r| types::SimilarSymbolData {
                symbol: r.ticker.unwrap_or_default(),
                score: None,
            })
            .collect())
    }
}

fn range_bounds(range: crate::TimeRange) -> (String, String) {
    super::range_to_dates(range)
}

/// Convert a Unix timestamp (seconds) to a "YYYY-MM-DD" date string.
fn timestamp_to_date(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "1970-01-01".to_string())
}

fn interval_polygon(interval: crate::Interval) -> (u32, polygon::Timespan) {
    match interval {
        crate::Interval::OneMinute => (1, polygon::Timespan::Minute),
        crate::Interval::FiveMinutes => (5, polygon::Timespan::Minute),
        crate::Interval::FifteenMinutes => (15, polygon::Timespan::Minute),
        crate::Interval::ThirtyMinutes => (30, polygon::Timespan::Minute),
        crate::Interval::OneHour => (1, polygon::Timespan::Hour),
        crate::Interval::OneDay => (1, polygon::Timespan::Day),
        crate::Interval::OneWeek => (1, polygon::Timespan::Week),
        crate::Interval::OneMonth => (1, polygon::Timespan::Month),
        crate::Interval::ThreeMonths => (3, polygon::Timespan::Month),
    }
}
