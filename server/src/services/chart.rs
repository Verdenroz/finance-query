use crate::cache::{self, Cache};
use finance_query::{Interval, Ticker, Tickers, TimeRange};
use tracing::info;

use super::{ServiceError, ServiceResult};

pub async fn get_chart(
    cache: &Cache,
    symbol: &str,
    interval: Interval,
    range: TimeRange,
    events: bool,
    patterns: bool,
) -> ServiceResult {
    let events_str = if events { "1" } else { "0" };
    let patterns_str = if patterns { "1" } else { "0" };
    let cache_key = Cache::key(
        "chart",
        &[
            &symbol.to_uppercase(),
            interval.as_str(),
            range.as_str(),
            events_str,
            patterns_str,
        ],
    );
    let symbol = symbol.to_string();

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::CHART,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol).await?;
                let chart = ticker.chart(interval, range).await?;
                let mut json =
                    serde_json::to_value(&chart).map_err(|e| Box::new(e) as ServiceError)?;

                if patterns && let serde_json::Value::Object(ref mut map) = json {
                    let signals = finance_query::patterns(&chart.candles);
                    map.insert(
                        "patterns".to_string(),
                        serde_json::to_value(signals).unwrap_or_default(),
                    );
                }

                if events && let serde_json::Value::Object(ref mut map) = json {
                    if let Ok(dividends) = ticker.dividends(range).await {
                        map.insert(
                            "dividends".to_string(),
                            serde_json::to_value(dividends).unwrap_or_default(),
                        );
                    }
                    if let Ok(splits) = ticker.splits(range).await {
                        map.insert(
                            "splits".to_string(),
                            serde_json::to_value(splits).unwrap_or_default(),
                        );
                    }
                    if let Ok(capital_gains) = ticker.capital_gains(range).await {
                        map.insert(
                            "capitalGains".to_string(),
                            serde_json::to_value(capital_gains).unwrap_or_default(),
                        );
                    }
                }
                Ok(json)
            },
        )
        .await
}

pub async fn get_batch_charts(
    cache: &Cache,
    symbols: Vec<&str>,
    interval: Interval,
    range: TimeRange,
    patterns: bool,
) -> ServiceResult {
    let mut symbols = symbols;
    symbols.sort();
    let patterns_str = if patterns { "1" } else { "0" };
    let symbols_key = symbols.join(",").to_uppercase();
    let cache_key = Cache::key(
        "charts",
        &[
            &symbols_key,
            interval.as_str(),
            range.as_str(),
            patterns_str,
        ],
    );

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::CHART,
            cache::is_market_open(),
            || async move {
                let tickers = Tickers::new(symbols).await?;
                let batch_response = tickers.charts(interval, range).await?;
                info!(
                    "Charts fetch complete: {} success, {} errors",
                    batch_response.success_count(),
                    batch_response.error_count()
                );

                let mut json = serde_json::to_value(&batch_response)
                    .map_err(|e| Box::new(e) as ServiceError)?;

                if patterns
                    && let serde_json::Value::Object(ref mut top) = json
                    && let Some(charts_val) = top.get_mut("charts")
                    && let serde_json::Value::Object(charts_map) = charts_val
                {
                    for (symbol, chart) in &batch_response.charts {
                        if let Some(chart_val) = charts_map.get_mut(symbol)
                            && let serde_json::Value::Object(m) = chart_val
                        {
                            let signals = finance_query::patterns(&chart.candles);
                            m.insert(
                                "patterns".to_string(),
                                serde_json::to_value(signals).unwrap_or_default(),
                            );
                        }
                    }
                }

                Ok(json)
            },
        )
        .await
}

pub async fn get_spark(
    cache: &Cache,
    symbols: Vec<&str>,
    interval: Interval,
    range: TimeRange,
) -> ServiceResult {
    let mut symbols = symbols;
    symbols.sort();
    let symbols_key = symbols.join(",").to_uppercase();
    let cache_key = Cache::key("spark", &[&symbols_key, interval.as_str(), range.as_str()]);

    cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SPARK,
            cache::is_market_open(),
            || async move {
                let tickers = Tickers::new(symbols).await?;
                let batch_response = tickers.spark(interval, range).await?;
                info!(
                    "Spark fetch complete: {} success, {} errors",
                    batch_response.success_count(),
                    batch_response.error_count()
                );
                serde_json::to_value(&batch_response).map_err(|e| Box::new(e) as ServiceError)
            },
        )
        .await
}
