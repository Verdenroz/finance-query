//! Yahoo Finance provider implementation.
//!
//! Wraps YahooClient and converts all responses
//! to canonical intermediate types defined in [`super::types`].

use super::types;
use crate::adapters::yahoo::client::{ClientConfig, YahooClient};
use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use crate::models::chart::response::ChartResponse;
use crate::models::quote::QuoteSummaryResponse;
use std::sync::Arc;

pub(crate) struct YahooProvider {
    client: Arc<YahooClient>,
}

impl YahooProvider {
    pub(crate) async fn new(config: &ClientConfig) -> Result<Self> {
        Ok(Self {
            client: Arc::new(YahooClient::new(config.clone()).await?),
        })
    }

    /// Public access to the underlying YahooClient for finance module use.
    #[allow(dead_code)]
    pub(crate) fn client(&self) -> &Arc<YahooClient> {
        &self.client
    }
    pub(crate) fn client_arc(&self) -> Arc<YahooClient> {
        Arc::clone(&self.client)
    }
}

#[async_trait::async_trait]
impl super::ProviderAdapter for YahooProvider {
    fn id(&self) -> &'static str {
        "yahoo"
    }

    fn name(&self) -> &'static str {
        "Yahoo Finance"
    }

    fn capabilities(&self) -> super::Capability {
        super::Capability::QUOTE
            | super::Capability::CHART
            | super::Capability::FUNDAMENTALS
            | super::Capability::CORPORATE
            | super::Capability::OPTIONS
            | super::Capability::MARKET
            | super::Capability::DISCOVERY
    }

    // ── Quote ─────────────────────────────────────────────────────

    async fn fetch_quote(&self, symbol: &str) -> Result<types::QuoteData> {
        let url = crate::adapters::yahoo::endpoints::api::quote_summary(symbol);
        let response = self.client.request_with_crumb(&url).await?;
        let json: serde_json::Value = response.json().await?;
        let summary = QuoteSummaryResponse::from_json(json, symbol)?;

        let price = summary.price.as_ref();
        let detail = summary.summary_detail.as_ref();
        let profile = summary.asset_profile.as_ref();
        let stats = summary.default_key_statistics.as_ref();

        Ok(types::QuoteData {
            symbol: symbol.to_string(),
            provider_id: "yahoo",
            short_name: price.and_then(|p| p.short_name.clone()),
            long_name: price.and_then(|p| p.long_name.clone()),
            exchange: price.and_then(|p| p.exchange_name.clone()),
            quote_type: price.and_then(|p| p.quote_type.clone()),
            currency: price.and_then(|p| p.currency.clone()),
            regular_market_price: price
                .and_then(|p| p.regular_market_price.clone().and_then(|v| v.raw)),
            regular_market_change: price
                .and_then(|p| p.regular_market_change.clone().and_then(|v| v.raw)),
            regular_market_change_percent: price
                .and_then(|p| p.regular_market_change_percent.clone().and_then(|v| v.raw)),
            regular_market_volume: price.and_then(|p| {
                p.regular_market_volume
                    .clone()
                    .and_then(|v| v.raw.map(|r| r as u64))
            }),
            regular_market_previous_close: price
                .and_then(|p| p.regular_market_previous_close.clone().and_then(|v| v.raw)),
            regular_market_open: price
                .and_then(|p| p.regular_market_open.clone().and_then(|v| v.raw)),
            regular_market_day_high: price
                .and_then(|p| p.regular_market_day_high.clone().and_then(|v| v.raw)),
            regular_market_day_low: price
                .and_then(|p| p.regular_market_day_low.clone().and_then(|v| v.raw)),
            market_cap: price
                .and_then(|p| p.market_cap.clone().and_then(|v| v.raw.map(|r| r as f64))),
            fifty_two_week_high: detail
                .and_then(|d| d.fifty_two_week_high.clone().and_then(|v| v.raw)),
            fifty_two_week_low: detail
                .and_then(|d| d.fifty_two_week_low.clone().and_then(|v| v.raw)),
            fifty_day_avg: detail.and_then(|d| d.fifty_day_average.clone().and_then(|v| v.raw)),
            two_hundred_day_avg: detail
                .and_then(|d| d.two_hundred_day_average.clone().and_then(|v| v.raw)),
            beta: stats.and_then(|s| s.beta.clone().and_then(|v| v.raw)),
            eps_ttm: stats.and_then(|s| s.trailing_eps.clone().and_then(|v| v.raw)),
            pe_ratio: detail.and_then(|d| d.trailing_pe.clone().and_then(|v| v.raw)),
            dividend_rate: detail.and_then(|d| d.dividend_rate.clone().and_then(|v| v.raw)),
            dividend_yield: detail.and_then(|d| d.dividend_yield.clone().and_then(|v| v.raw)),
            description: profile.and_then(|a| a.long_business_summary.clone()),
            sector: profile.and_then(|a| a.sector.clone()),
            industry: profile.and_then(|a| a.industry.clone()),
            country: profile.and_then(|a| a.country.clone()),
            website: profile.and_then(|a| a.website.clone()),
            employees: profile.and_then(|a| a.full_time_employees.map(|v| v as u64)),
            logo_url: None,
            ..Default::default()
        })
    }

    // ── Chart ─────────────────────────────────────────────────────

    async fn fetch_chart(
        &self,
        symbol: &str,
        interval: Interval,
        range: TimeRange,
    ) -> Result<types::ChartData> {
        let json = self.client.get_chart(symbol, interval, range).await?;
        Self::parse_chart_data(json, symbol)
    }

    async fn fetch_chart_range(
        &self,
        symbol: &str,
        interval: Interval,
        start: i64,
        end: i64,
    ) -> Result<types::ChartData> {
        let json = self
            .client
            .get_chart_range(symbol, interval, start, end)
            .await?;
        Self::parse_chart_data(json, symbol)
    }

    // ── Financials ────────────────────────────────────────────────

    async fn fetch_financials(
        &self,
        symbol: &str,
        stmt_type: crate::StatementType,
        frequency: crate::Frequency,
    ) -> Result<types::FinancialStatementData> {
        let stmt = self
            .client
            .get_financials(symbol, stmt_type, frequency)
            .await?;
        Ok(types::FinancialStatementData {
            provider_id: "yahoo",
            symbol: symbol.to_string(),
            statement_type: stmt_type.as_str().to_string(),
            frequency: frequency.as_str().to_string(),
            data: stmt
                .statement
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        v.into_iter()
                            .map(|(date, val)| (date, serde_json::Value::from(val)))
                            .collect(),
                    )
                })
                .collect(),
        })
    }

    // ── News ──────────────────────────────────────────────────────

    async fn fetch_news(&self, symbol: &str) -> Result<Vec<types::NewsData>> {
        let news = crate::scrapers::stockanalysis::scrape_symbol_news(symbol).await?;
        Ok(news
            .into_iter()
            .map(|n| types::NewsData {
                provider_id: "yahoo",
                title: n.title,
                url: Some(n.link),
                source: Some(n.source),
                published_at: Some(n.time),
                summary: None,
                extras: Default::default(),
            })
            .collect())
    }

    // ── Recommendations ───────────────────────────────────────────

    async fn fetch_similar_symbols(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Vec<types::SimilarSymbolData>> {
        let json = self.client.get_recommendations(symbol, limit).await?;
        let recs: crate::models::corporate::recommendation::response::RecommendationResponse =
            serde_json::from_value(json)?;
        Ok(recs
            .finance
            .result
            .into_iter()
            .flat_map(|r| r.recommended_symbols)
            .take(limit as usize)
            .map(|s| types::SimilarSymbolData {
                symbol: s.symbol,
                score: Some(s.score),
            })
            .collect())
    }

    // ── Options ───────────────────────────────────────────────────

    async fn fetch_options(&self, symbol: &str, date: Option<i64>) -> Result<types::OptionsData> {
        let json = self.client.get_options(symbol, date).await?;
        let chain = json
            .get("optionChain")
            .and_then(|oc| oc.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first());

        let expiration_dates = chain
            .and_then(|c| c.get("expirationDates"))
            .and_then(|d| d.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
            .unwrap_or_default();

        fn map_contracts(arr: &serde_json::Value) -> Vec<types::OptionContractData> {
            arr.as_array()
                .map(|items| {
                    items
                        .iter()
                        .map(|c| types::OptionContractData {
                            contract_symbol: c["contractSymbol"].as_str().unwrap_or("").to_string(),
                            strike: c["strike"].as_f64().unwrap_or(0.0),
                            currency: c["currency"].as_str().map(String::from),
                            last_price: c["lastPrice"].as_f64(),
                            change: c["change"].as_f64(),
                            volume: c["volume"].as_u64(),
                            open_interest: c["openInterest"].as_u64(),
                            bid: c["bid"].as_f64(),
                            ask: c["ask"].as_f64(),
                            expiration: c["expiration"].as_i64().unwrap_or(0),
                            implied_volatility: c["impliedVolatility"].as_f64(),
                            in_the_money: c["inTheMoney"].as_bool(),
                            extras: Default::default(),
                        })
                        .collect()
                })
                .unwrap_or_default()
        }

        let calls = chain
            .and_then(|c| c.get("calls"))
            .map(map_contracts)
            .unwrap_or_default();

        let puts = chain
            .and_then(|c| c.get("puts"))
            .map(map_contracts)
            .unwrap_or_default();

        Ok(types::OptionsData {
            provider_id: "yahoo",
            symbol: symbol.to_string(),
            expiration_dates,
            calls,
            puts,
            extras: Default::default(),
        })
    }

    // ── Events ────────────────────────────────────────────────────

    async fn fetch_events(&self, symbol: &str) -> Result<types::EventsData> {
        let json = self
            .client
            .get_chart(symbol, Interval::OneDay, TimeRange::Max)
            .await?;
        let chart_response =
            ChartResponse::from_json(json).map_err(crate::error::FinanceError::JsonParseError)?;
        let events = chart_response
            .chart
            .result
            .and_then(|mut r| r.pop())
            .and_then(|r| r.events)
            .unwrap_or_default();

        Ok(types::EventsData {
            provider_id: "yahoo",
            dividends: events
                .dividends
                .into_iter()
                .filter_map(|(ts, d)| {
                    let date = ts.parse().ok()?;
                    Some(types::DividendData {
                        date,
                        amount: d.amount,
                    })
                })
                .collect(),
            splits: events
                .splits
                .into_iter()
                .filter_map(|(ts, s)| {
                    let date = ts.parse().ok()?;
                    Some(types::SplitData {
                        date,
                        numerator: s.numerator as u32,
                        denominator: s.denominator as u32,
                    })
                })
                .collect(),
            capital_gains: events
                .capital_gains
                .into_iter()
                .filter_map(|(ts, cg)| {
                    let date = ts.parse().ok()?;
                    Some(types::CapitalGainData {
                        date,
                        amount: cg.amount,
                    })
                })
                .collect(),
        })
    }

    // ── Batch ─────────────────────────────────────────────────────

    async fn fetch_quotes_batch(&self, symbols: &[&str]) -> Result<Vec<types::QuoteData>> {
        let json = crate::adapters::yahoo::quote::quotes::fetch(&self.client, symbols).await?;
        let result = json
            .get("quoteResponse")
            .and_then(|qr| qr.get("result"))
            .and_then(|r| r.as_array());

        let mut quotes = Vec::new();
        if let Some(results) = result {
            for item in results {
                let symbol = item["symbol"].as_str().unwrap_or("").to_string();
                let price = item["regularMarketPrice"].as_f64();
                let change = item["regularMarketChange"].as_f64();
                let change_pct = item["regularMarketChangePercent"].as_f64();
                let volume = item["regularMarketVolume"].as_u64();

                quotes.push(types::QuoteData {
                    symbol,
                    provider_id: "yahoo",
                    short_name: item["shortName"].as_str().map(String::from),
                    long_name: item["longName"].as_str().map(String::from),
                    exchange: item["fullExchangeName"].as_str().map(String::from),
                    quote_type: item["quoteType"].as_str().map(String::from),
                    currency: item["currency"].as_str().map(String::from),
                    regular_market_price: price,
                    regular_market_change: change,
                    regular_market_change_percent: change_pct,
                    regular_market_volume: volume,
                    regular_market_previous_close: item["regularMarketPreviousClose"].as_f64(),
                    regular_market_open: item["regularMarketOpen"].as_f64(),
                    regular_market_day_high: item["regularMarketDayHigh"].as_f64(),
                    regular_market_day_low: item["regularMarketDayLow"].as_f64(),
                    market_cap: item["marketCap"].as_f64(),
                    fifty_two_week_high: item["fiftyTwoWeekHigh"].as_f64(),
                    fifty_two_week_low: item["fiftyTwoWeekLow"].as_f64(),
                    fifty_day_avg: item["fiftyDayAverage"].as_f64(),
                    two_hundred_day_avg: item["twoHundredDayAverage"].as_f64(),
                    ..Default::default()
                });
            }
        }
        Ok(quotes)
    }

    // ── Market-wide ───────────────────────────────────────────────

    async fn fetch_market_hours(&self, symbol: &str) -> Result<types::MarketHoursData> {
        let hours = self.client.get_hours(Some(symbol)).await?;
        Ok(types::MarketHoursData {
            provider_id: "yahoo",
            symbol: symbol.to_string(),
            exchange: hours.markets.first().map(|m| m.name.clone()),
            market_state: hours.markets.first().map(|m| m.status.clone()),
            extras: Default::default(),
        })
    }

    async fn fetch_trending(&self) -> Result<Vec<types::TrendingData>> {
        let quotes = self.client.get_trending(None).await?;
        Ok(quotes
            .into_iter()
            .map(|q| types::TrendingData {
                provider_id: "yahoo",
                symbol: q.symbol,
                name: None,
                extras: Default::default(),
            })
            .collect())
    }

    async fn fetch_market_summary(&self) -> Result<Vec<types::MarketSummaryData>> {
        let quotes = self.client.get_market_summary(None).await?;
        Ok(quotes
            .into_iter()
            .map(|q| types::MarketSummaryData {
                provider_id: "yahoo",
                symbol: q.symbol,
                name: q.short_name,
                price: q.regular_market_price.and_then(|v| v.raw),
                change: q.regular_market_change.and_then(|v| v.raw),
                change_percent: q.regular_market_change_percent.and_then(|v| v.raw),
                extras: Default::default(),
            })
            .collect())
    }
}

impl YahooProvider {
    fn parse_chart_data(json: serde_json::Value, symbol: &str) -> Result<types::ChartData> {
        let chart_response =
            ChartResponse::from_json(json).map_err(crate::error::FinanceError::JsonParseError)?;
        let results = chart_response.chart.result.ok_or_else(|| {
            crate::error::FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: "no chart result".into(),
            }
        })?;
        let result = results
            .first()
            .ok_or_else(|| crate::error::FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: "empty chart results".into(),
            })?;

        let meta = &result.meta;
        let candles: Vec<types::CandleData> = result
            .to_candles()
            .into_iter()
            .map(|c| types::CandleData {
                timestamp: c.timestamp,
                open: c.open,
                high: c.high,
                low: c.low,
                close: c.close,
                volume: c.volume.max(0) as u64,
            })
            .collect();

        Ok(types::ChartData {
            symbol: symbol.to_string(),
            provider_id: "yahoo",
            candles,
            meta: types::ChartMetaData {
                currency: meta.currency.clone(),
                symbol: Some(meta.symbol.clone()),
                exchange_name: meta.exchange_name.clone(),
                instrument_type: meta.instrument_type.clone(),
                previous_close: meta.previous_close,
                regular_market_price: meta.regular_market_price,
                chart_previous_close: meta.chart_previous_close,
                data_granularity: meta.data_granularity.clone(),
                valid_ranges: None,
                extras: Default::default(),
            },
            extras: Default::default(),
        })
    }
}
