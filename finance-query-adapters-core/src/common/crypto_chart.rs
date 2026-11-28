//! Canonical [`Chart`] assembly shared by the keyless crypto exchange adapters.
//!
//! Binance and Kraken return different candle wire shapes (each adapter
//! converts its own), but the surrounding [`ChartMeta`] is identical apart from
//! the exchange's identity, so it is built in one place.

use finance_query_models::chart::{Candle, Chart, ChartFieldsInit, ChartMeta, ChartMetaFieldsInit};
use finance_query_types::Provider;
use finance_query_types::constants::{Interval, TimeRange};

/// How an exchange names itself in canonical chart metadata.
pub struct CryptoExchange {
    /// The routed provider the candles came from.
    pub provider: Provider,
    /// Uppercase exchange code (e.g. `"BINANCE"`).
    pub code: &'static str,
    /// Display name (e.g. `"Binance"`).
    pub name: &'static str,
}

/// Assemble a [`Chart`] around a crypto candle series.
///
/// `quote_currency` is the pair's quote asset in the library's spelling — each
/// exchange derives it from its own market symbol before calling.
pub fn crypto_chart(
    exchange: &CryptoExchange,
    symbol: &str,
    quote_currency: Option<String>,
    candles: Vec<Candle>,
    interval: Option<Interval>,
    range: Option<TimeRange>,
) -> Chart {
    Chart::from(ChartFieldsInit {
        symbol: symbol.to_string(),
        meta: ChartMeta::from(ChartMetaFieldsInit {
            symbol: symbol.to_string(),
            currency: quote_currency,
            exchange_name: Some(exchange.code.to_string()),
            full_exchange_name: Some(exchange.name.to_string()),
            instrument_type: Some("CRYPTOCURRENCY".to_string()),
            first_trade_date: candles.first().map(|c| c.timestamp),
            regular_market_time: candles.last().map(|c| c.timestamp),
            regular_market_price: candles.last().map(|c| c.close),
            data_granularity: interval.map(|i| i.as_str().to_string()),
            range: range.map(|r| r.as_str().to_string()),
            provider_id: Some(exchange.provider),
            ..Default::default()
        }),
        candles,
        interval,
        range,
        provider_id: Some(exchange.provider),
    })
}
