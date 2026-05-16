//! Canonical intermediate types for multi-provider data normalization.
//!
//! These are `pub(crate)` types that each [`Provider`](super::Provider)
//! implementation populates from its native format. The [`ProviderSet`](super::ProviderSet)
//! then merges them into the public-facing types in [`crate::models`].
//!
//! Many types here are scaffolding for upcoming provider implementations.
//!
//! Most types are constructed only from feature-gated provider modules
//! (polygon, fmp, alphavantage, crypto, fred). Rust cannot express
//! per-struct `#[cfg(any(feature = "...", ...))]` dead_code suppression,
//! so we allow it at module level. Genuinely dead items (e.g.
//! `merge_vec_by_priority`) carry their own `#[allow(dead_code)]` so
//! they can be discovered by temporarily removing this attribute.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::Provider;

// ── Quote ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct QuoteData {
    pub symbol: String,
    pub provider_id: &'static str,
    pub short_name: Option<String>,
    pub long_name: Option<String>,
    pub exchange: Option<String>,
    pub quote_type: Option<String>,
    pub currency: Option<String>,
    pub regular_market_price: Option<f64>,
    pub regular_market_change: Option<f64>,
    pub regular_market_change_percent: Option<f64>,
    pub regular_market_volume: Option<u64>,
    pub regular_market_previous_close: Option<f64>,
    pub regular_market_open: Option<f64>,
    pub regular_market_day_high: Option<f64>,
    pub regular_market_day_low: Option<f64>,
    pub market_cap: Option<f64>,
    pub fifty_two_week_high: Option<f64>,
    pub fifty_two_week_low: Option<f64>,
    pub fifty_day_avg: Option<f64>,
    pub two_hundred_day_avg: Option<f64>,
    pub beta: Option<f64>,
    pub eps_ttm: Option<f64>,
    pub pe_ratio: Option<f64>,
    pub dividend_rate: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub description: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
    pub website: Option<String>,
    pub employees: Option<u64>,
    pub logo_url: Option<String>,
    pub extras: HashMap<String, serde_json::Value>,
}

// ── Chart / OHLCV ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CandleData {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ChartMetaData {
    pub currency: Option<String>,
    pub symbol: Option<String>,
    pub exchange_name: Option<String>,
    pub instrument_type: Option<String>,
    pub previous_close: Option<f64>,
    pub regular_market_price: Option<f64>,
    pub chart_previous_close: Option<f64>,
    pub data_granularity: Option<String>,
    pub valid_ranges: Option<Vec<String>>,
    pub extras: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChartData {
    pub symbol: String,
    pub provider_id: &'static str,
    pub candles: Vec<CandleData>,
    pub meta: ChartMetaData,
    pub extras: HashMap<String, serde_json::Value>,
}

// ── Events ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DividendData {
    pub date: i64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SplitData {
    pub date: i64,
    pub numerator: u32,
    pub denominator: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CapitalGainData {
    pub date: i64,
    pub amount: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct EventsData {
    pub provider_id: &'static str,
    pub dividends: Vec<DividendData>,
    pub splits: Vec<SplitData>,
    pub capital_gains: Vec<CapitalGainData>,
}

// ── Financials ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FinancialStatementData {
    pub provider_id: &'static str,
    pub symbol: String,
    pub statement_type: String,
    pub frequency: String,
    pub data: HashMap<String, HashMap<String, serde_json::Value>>,
}

// ── News ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NewsData {
    pub provider_id: &'static str,
    pub title: String,
    pub url: Option<String>,
    pub source: Option<String>,
    pub published_at: Option<String>,
    pub summary: Option<String>,
    pub extras: HashMap<String, serde_json::Value>,
}

// ── Options ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OptionsData {
    pub provider_id: &'static str,
    pub symbol: String,
    pub expiration_dates: Vec<i64>,
    pub calls: Vec<OptionContractData>,
    pub puts: Vec<OptionContractData>,
    pub extras: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OptionContractData {
    pub contract_symbol: String,
    pub strike: f64,
    pub currency: Option<String>,
    pub last_price: Option<f64>,
    pub change: Option<f64>,
    pub volume: Option<u64>,
    pub open_interest: Option<u64>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub expiration: i64,
    pub implied_volatility: Option<f64>,
    pub in_the_money: Option<bool>,
    pub extras: HashMap<String, serde_json::Value>,
}

// ── Recommendations ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SimilarSymbolData {
    pub symbol: String,
    pub score: Option<f64>,
}

// ── Market-wide ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MarketHoursData {
    pub provider_id: &'static str,
    pub symbol: String,
    pub exchange: Option<String>,
    pub market_state: Option<String>,
    pub extras: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TrendingData {
    pub provider_id: &'static str,
    pub symbol: String,
    pub name: Option<String>,
    pub extras: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MarketSummaryData {
    pub provider_id: &'static str,
    pub symbol: String,
    pub name: Option<String>,
    pub price: Option<f64>,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub extras: HashMap<String, serde_json::Value>,
}

// ── Crypto ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct CryptoQuoteData {
    pub provider_id: &'static str,
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub price: Option<f64>,
    pub market_cap: Option<f64>,
    pub volume_24h: Option<f64>,
    pub change_24h: Option<f64>,
    pub change_percent_24h: Option<f64>,
    pub high_24h: Option<f64>,
    pub low_24h: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
    pub ath: Option<f64>,
    pub ath_date: Option<String>,
    pub extras: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CryptoCoinData {
    pub provider_id: &'static str,
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub market_cap_rank: Option<u32>,
    pub price: Option<f64>,
    pub market_cap: Option<f64>,
    pub volume_24h: Option<f64>,
    pub change_percent_24h: Option<f64>,
    pub image_url: Option<String>,
    pub extras: HashMap<String, serde_json::Value>,
}

// ── Macro / Economic ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EconomicSeriesData {
    pub provider_id: &'static str,
    pub series_id: String,
    pub title: Option<String>,
    pub units: Option<String>,
    pub frequency: Option<String>,
    pub seasonal_adjustment: Option<String>,
    pub observations: Vec<EconomicObservationData>,
    pub extras: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EconomicObservationData {
    pub date: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TreasuryYieldData {
    pub provider_id: &'static str,
    pub date: String,
    pub month_1: Option<f64>,
    pub month_3: Option<f64>,
    pub month_6: Option<f64>,
    pub year_1: Option<f64>,
    pub year_2: Option<f64>,
    pub year_3: Option<f64>,
    pub year_5: Option<f64>,
    pub year_7: Option<f64>,
    pub year_10: Option<f64>,
    pub year_20: Option<f64>,
    pub year_30: Option<f64>,
    pub extras: HashMap<String, serde_json::Value>,
}

impl QuoteData {
    pub(crate) fn into_quote(self) -> crate::models::quote::Quote {
        crate::models::quote::Quote::from_quote_data(self)
    }
}

pub(crate) fn backfill_quote_data(mut primary: QuoteData, fallback: QuoteData) -> QuoteData {
    macro_rules! fill_none {
        ($field:ident) => {
            if primary.$field.is_none() {
                primary.$field = fallback.$field;
            }
        };
    }

    fill_none!(short_name);
    fill_none!(long_name);
    fill_none!(exchange);
    fill_none!(quote_type);
    fill_none!(currency);
    fill_none!(regular_market_price);
    fill_none!(regular_market_change);
    fill_none!(regular_market_change_percent);
    fill_none!(regular_market_volume);
    fill_none!(regular_market_previous_close);
    fill_none!(regular_market_open);
    fill_none!(regular_market_day_high);
    fill_none!(regular_market_day_low);
    fill_none!(market_cap);
    fill_none!(fifty_two_week_high);
    fill_none!(fifty_two_week_low);
    fill_none!(fifty_day_avg);
    fill_none!(two_hundred_day_avg);
    fill_none!(beta);
    fill_none!(eps_ttm);
    fill_none!(pe_ratio);
    fill_none!(dividend_rate);
    fill_none!(dividend_yield);
    fill_none!(description);
    fill_none!(sector);
    fill_none!(industry);
    fill_none!(country);
    fill_none!(website);
    fill_none!(employees);
    fill_none!(logo_url);

    for (key, value) in fallback.extras {
        primary.extras.entry(key).or_insert(value);
    }

    primary
}

impl ChartMetaData {
    fn into_chart_meta(
        self,
        fallback_symbol: String,
        provider_id: Option<crate::providers::Provider>,
    ) -> crate::models::chart::ChartMeta {
        crate::models::chart::ChartMeta {
            symbol: self.symbol.unwrap_or(fallback_symbol),
            currency: self.currency,
            exchange_name: self.exchange_name,
            instrument_type: self.instrument_type,
            previous_close: self.previous_close,
            regular_market_price: self.regular_market_price,
            chart_previous_close: self.chart_previous_close,
            data_granularity: self.data_granularity,
            provider_id,
            ..Default::default()
        }
    }
}

pub(crate) fn backfill_chart_data(mut primary: ChartData, fallback: ChartData) -> ChartData {
    // Backfill ChartMetaData optional fields from fallback
    macro_rules! fill_none {
        ($field:ident) => {
            if primary.meta.$field.is_none() {
                primary.meta.$field = fallback.meta.$field;
            }
        };
    }
    fill_none!(currency);
    fill_none!(symbol);
    fill_none!(exchange_name);
    fill_none!(instrument_type);
    fill_none!(previous_close);
    fill_none!(regular_market_price);
    fill_none!(chart_previous_close);
    fill_none!(data_granularity);
    fill_none!(valid_ranges);

    // Merge ChartMetaData extras
    for (key, value) in fallback.meta.extras {
        primary.meta.extras.entry(key).or_insert(value);
    }

    // Merge candles: deduplicate by timestamp, primary wins
    let mut seen: std::collections::HashSet<i64> =
        primary.candles.iter().map(|c| c.timestamp).collect();
    for candle in fallback.candles {
        if seen.insert(candle.timestamp) {
            primary.candles.push(candle);
        }
    }
    primary.candles.sort_by_key(|c| c.timestamp);

    // Merge ChartData extras
    for (key, value) in fallback.extras {
        primary.extras.entry(key).or_insert(value);
    }

    primary
}

impl ChartData {
    pub(crate) fn into_chart(
        self,
        interval: Option<crate::Interval>,
        range: Option<crate::TimeRange>,
    ) -> crate::models::chart::Chart {
        let provider_id: Provider = Provider::from(self.provider_id);
        let symbol = self.symbol;
        let meta = self.meta.into_chart_meta(symbol.clone(), Some(provider_id));

        let candles = self
            .candles
            .into_iter()
            .map(|candle| {
                let mut candle = crate::models::chart::Candle::from(candle);
                candle.provider_id = Some(provider_id);
                candle
            })
            .collect();

        crate::models::chart::Chart {
            symbol,
            meta,
            candles,
            interval,
            range,
            provider_id: Some(provider_id),
        }
    }
}

impl EventsData {
    pub(crate) fn into_chart_events(self) -> crate::models::chart::events::ChartEvents {
        let mut events = crate::models::chart::events::ChartEvents::default();

        events.dividends = self
            .dividends
            .into_iter()
            .map(|dividend| {
                (
                    dividend.date.to_string(),
                    crate::models::chart::events::DividendEvent {
                        amount: dividend.amount,
                        date: dividend.date,
                    },
                )
            })
            .collect();

        events.splits = self
            .splits
            .into_iter()
            .map(|split| {
                (
                    split.date.to_string(),
                    crate::models::chart::events::SplitEvent {
                        date: split.date,
                        numerator: split.numerator as f64,
                        denominator: split.denominator as f64,
                        split_ratio: format!("{}:{}", split.numerator, split.denominator),
                    },
                )
            })
            .collect();

        events.capital_gains = self
            .capital_gains
            .into_iter()
            .map(|gain| {
                (
                    gain.date.to_string(),
                    crate::models::chart::events::CapitalGainEvent {
                        amount: gain.amount,
                        date: gain.date,
                    },
                )
            })
            .collect();

        events
    }
}

impl FinancialStatementData {
    pub(crate) fn into_financial_statement(
        self,
    ) -> crate::models::fundamentals::FinancialStatement {
        let statement = self
            .data
            .into_iter()
            .filter_map(|(metric, values)| {
                let values: HashMap<String, f64> = values
                    .into_iter()
                    .filter_map(|(date, value)| json_value_to_f64(value).map(|v| (date, v)))
                    .collect();

                if values.is_empty() {
                    None
                } else {
                    Some((metric, values))
                }
            })
            .collect();

        crate::models::fundamentals::FinancialStatement {
            symbol: self.symbol,
            statement_type: self.statement_type,
            frequency: self.frequency,
            statement,
            provider_id: Some(self.provider_id.into()),
        }
    }
}

pub(crate) fn backfill_financial_data(
    mut primary: FinancialStatementData,
    fallback: FinancialStatementData,
) -> FinancialStatementData {
    for (metric, fallback_values) in fallback.data {
        match primary.data.entry(metric) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                for (date, value) in fallback_values {
                    entry.get_mut().entry(date).or_insert(value);
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(fallback_values);
            }
        }
    }
    primary
}

fn json_value_to_f64(value: serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_i64().map(|v| v as f64))
        .or_else(|| value.as_u64().map(|v| v as f64))
        .or_else(|| value.as_str().and_then(|s| s.parse::<f64>().ok()))
        .or_else(|| {
            value
                .get("raw")
                .and_then(|raw| raw.as_f64().or_else(|| raw.as_i64().map(|v| v as f64)))
        })
}

impl OptionsData {
    pub(crate) fn into_options(self) -> crate::models::options::Options {
        let mut chains_by_expiration: std::collections::BTreeMap<
            i64,
            (
                Vec<crate::models::options::OptionContract>,
                Vec<crate::models::options::OptionContract>,
            ),
        > = std::collections::BTreeMap::new();

        for contract in self.calls {
            let expiration = contract.expiration;
            let mapped = map_option_contract(contract);
            chains_by_expiration
                .entry(expiration)
                .or_default()
                .0
                .push(mapped);
        }

        for contract in self.puts {
            let expiration = contract.expiration;
            let mapped = map_option_contract(contract);
            chains_by_expiration
                .entry(expiration)
                .or_default()
                .1
                .push(mapped);
        }

        let option_chains: Vec<crate::models::options::response::OptionChainData> =
            chains_by_expiration
                .into_iter()
                .map(|(expiration, (calls, puts))| {
                    crate::models::options::response::OptionChainData {
                        expiration_date: expiration,
                        has_mini_options: None,
                        calls: Some(calls),
                        puts: Some(puts),
                    }
                })
                .collect();

        let expiration_dates = if self.expiration_dates.is_empty() {
            option_chains
                .iter()
                .map(|chain| chain.expiration_date)
                .collect()
        } else {
            dedupe_sorted_i64(self.expiration_dates)
        };

        let strikes = dedupe_sorted_f64(
            option_chains
                .iter()
                .flat_map(|chain| {
                    chain
                        .calls
                        .as_deref()
                        .unwrap_or_default()
                        .iter()
                        .map(|c| c.strike)
                        .chain(
                            chain
                                .puts
                                .as_deref()
                                .unwrap_or_default()
                                .iter()
                                .map(|p| p.strike),
                        )
                })
                .collect(),
        );

        let result = crate::models::options::response::OptionChainResult {
            underlying_symbol: Some(self.symbol),
            expiration_dates: Some(expiration_dates),
            strikes: Some(strikes),
            has_mini_options: None,
            quote: None,
            options: option_chains,
        };

        crate::models::options::Options {
            option_chain: crate::models::options::response::OptionChainContainer {
                result: vec![result],
                error: None,
            },
            provider_id: Some(self.provider_id.into()),
        }
    }
}

fn map_option_contract(data: OptionContractData) -> crate::models::options::OptionContract {
    crate::models::options::OptionContract {
        contract_symbol: data.contract_symbol,
        strike: data.strike,
        currency: data.currency,
        last_price: data.last_price,
        change: data.change,
        percent_change: None,
        volume: data.volume.map(|v| v as i64),
        open_interest: data.open_interest.map(|v| v as i64),
        bid: data.bid,
        ask: data.ask,
        contract_size: None,
        expiration: Some(data.expiration),
        last_trade_date: None,
        implied_volatility: data.implied_volatility,
        in_the_money: data.in_the_money,
    }
}

pub(crate) fn backfill_options_data(
    mut primary: OptionsData,
    fallback: OptionsData,
) -> OptionsData {
    // Merge expiration_dates (deduplicate)
    let mut expirations: std::collections::HashSet<i64> =
        primary.expiration_dates.iter().copied().collect();
    for date in fallback.expiration_dates {
        expirations.insert(date);
    }
    let mut expirations: Vec<i64> = expirations.into_iter().collect();
    expirations.sort_unstable();
    primary.expiration_dates = expirations;

    // Merge calls: deduplicate by contract_symbol, primary wins
    let mut seen_calls: std::collections::HashSet<String> = primary
        .calls
        .iter()
        .map(|c| c.contract_symbol.clone())
        .collect();
    for call in fallback.calls {
        if seen_calls.insert(call.contract_symbol.clone()) {
            primary.calls.push(call);
        }
    }

    // Merge puts: deduplicate by contract_symbol, primary wins
    let mut seen_puts: std::collections::HashSet<String> = primary
        .puts
        .iter()
        .map(|p| p.contract_symbol.clone())
        .collect();
    for put in fallback.puts {
        if seen_puts.insert(put.contract_symbol.clone()) {
            primary.puts.push(put);
        }
    }

    // Merge extras
    for (key, value) in fallback.extras {
        primary.extras.entry(key).or_insert(value);
    }

    primary
}

fn dedupe_sorted_i64(mut values: Vec<i64>) -> Vec<i64> {
    values.sort_unstable();
    values.dedup();
    values
}

fn dedupe_sorted_f64(mut values: Vec<f64>) -> Vec<f64> {
    values.sort_by(|a, b| a.total_cmp(b));
    values.dedup_by(|a, b| a.total_cmp(b).is_eq());
    values
}

impl From<NewsData> for crate::models::corporate::news::News {
    fn from(data: NewsData) -> Self {
        crate::models::corporate::news::News {
            title: data.title,
            link: data.url.unwrap_or_default(),
            source: data.source.unwrap_or_default(),
            img: String::new(),
            time: data.published_at.unwrap_or_default(),
            provider_id: Some(data.provider_id.into()),
        }
    }
}

impl From<SimilarSymbolData> for crate::models::corporate::recommendation::SimilarSymbol {
    fn from(data: SimilarSymbolData) -> Self {
        crate::models::corporate::recommendation::SimilarSymbol {
            symbol: data.symbol,
            score: data.score.unwrap_or(0.0),
        }
    }
}

pub(crate) fn recommendation_from_similar(
    symbol: impl Into<String>,
    provider_id: Option<Provider>,
    items: Vec<SimilarSymbolData>,
    limit: Option<u32>,
) -> crate::models::corporate::recommendation::Recommendation {
    let recommendations = if let Some(limit) = limit {
        items
            .into_iter()
            .take(limit as usize)
            .map(Into::into)
            .collect()
    } else {
        items.into_iter().map(Into::into).collect()
    };

    crate::models::corporate::recommendation::Recommendation {
        symbol: symbol.into(),
        recommendations,
        provider_id,
    }
}

// ── Merging helpers ───────────────────────────────────────────────

/// Merge strategy for list-type data (news, recommendations, etc.).
#[allow(dead_code)]
pub(crate) fn merge_vec_by_priority<T, K: Eq + std::hash::Hash>(
    results: Vec<(usize, Vec<T>)>,
    key_fn: impl Fn(&T) -> K,
) -> Vec<T> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut merged = Vec::new();
    let mut sorted_results = results;
    sorted_results.sort_by_key(|(priority, _)| *priority);
    for (_priority, items) in sorted_results {
        for item in items {
            let key = key_fn(&item);
            if seen.insert(key) {
                merged.push(item);
            }
        }
    }
    merged
}
