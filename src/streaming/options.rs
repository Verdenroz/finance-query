//! Live options-chain streaming.
//!
//! A dedicated stream type rather than a reuse of [`PriceUpdate`]: a chain is
//! many contracts per underlying, each with its own quote, open interest and
//! greeks — a shape a single-symbol price tick cannot carry.

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::stream::Stream;
use serde::{Deserialize, Serialize};

use super::client::StreamResult;
use super::handle::SourceStream;
use super::polygon::PolygonOptionsSource;
use super::pricing::OptionType;

/// Reconnection backoff, matching [`PriceStream`](super::PriceStream).
const RECONNECT_BACKOFF_SECS: u64 = 3;

/// Channel capacity — a wide chain fans out many contracts per tick.
const CHANNEL_CAPACITY: usize = 2048;

/// Default interval between greeks/open-interest snapshot refreshes.
const DEFAULT_GREEKS_REFRESH: Duration = Duration::from_secs(60);

/// Option greeks for a contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Greeks {
    /// Rate of change of price with respect to the underlying.
    pub delta: Option<f64>,
    /// Rate of change of delta with respect to the underlying.
    pub gamma: Option<f64>,
    /// Rate of change of price with respect to time.
    pub theta: Option<f64>,
    /// Rate of change of price with respect to volatility.
    pub vega: Option<f64>,
}

impl Greeks {
    /// `true` when no greek was populated.
    pub fn is_empty(&self) -> bool {
        self.delta.is_none() && self.gamma.is_none() && self.theta.is_none() && self.vega.is_none()
    }
}

/// A live update for one options contract.
///
/// Quote and trade fields arrive from the real-time WebSocket; `greeks`,
/// `implied_volatility` and `open_interest` come from the periodic chain
/// snapshot (see [`OptionsChainStreamBuilder::greeks_refresh`]) because the
/// real-time feed does not carry them.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OptionContractUpdate {
    /// OCC contract symbol (e.g. `"O:AAPL250117C00150000"`).
    pub contract_symbol: String,
    /// Underlying symbol parsed from the contract (e.g. `"AAPL"`).
    pub underlying: String,
    /// Expiration date as a Unix timestamp (seconds).
    pub expiration: Option<i64>,
    /// Strike price.
    pub strike: Option<f64>,
    /// Call or put.
    pub option_type: Option<OptionType>,
    /// Best bid.
    pub bid: Option<f64>,
    /// Best bid size.
    pub bid_size: Option<f64>,
    /// Best ask.
    pub ask: Option<f64>,
    /// Best ask size.
    pub ask_size: Option<f64>,
    /// Last traded price.
    pub last_price: Option<f64>,
    /// Last traded size.
    pub last_size: Option<f64>,
    /// Day volume, when the snapshot supplies it.
    pub volume: Option<i64>,
    /// Open interest, from the snapshot refresh.
    pub open_interest: Option<i64>,
    /// Implied volatility, from the snapshot refresh.
    pub implied_volatility: Option<f64>,
    /// Greeks, from the snapshot refresh.
    pub greeks: Option<Greeks>,
    /// Event timestamp (milliseconds).
    pub time: i64,
}

/// Contract metadata decoded from an OCC symbol.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ContractParts {
    pub(crate) underlying: String,
    pub(crate) expiration: i64,
    pub(crate) option_type: OptionType,
    pub(crate) strike: f64,
}

/// Decode an OCC-style contract symbol (`O:AAPL250117C00150000`).
///
/// Returns `None` for anything that does not match the layout, so a malformed
/// upstream ticker is skipped rather than mislabeled.
pub(crate) fn parse_contract_symbol(symbol: &str) -> Option<ContractParts> {
    let body = symbol.strip_prefix("O:").unwrap_or(symbol);
    // Trailing fixed-width fields: 6 date + 1 type + 8 strike.
    if body.len() < 16 {
        return None;
    }
    let split = body.len() - 15;
    let (underlying, rest) = body.split_at(split);
    if underlying.is_empty() || !underlying.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }

    let (date, rest) = rest.split_at(6);
    let (kind, strike) = rest.split_at(1);
    if !date.chars().all(|c| c.is_ascii_digit()) || !strike.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let option_type = match kind {
        "C" => OptionType::Call,
        "P" => OptionType::Put,
        _ => return None,
    };

    let year = 2000 + date[0..2].parse::<i32>().ok()?;
    let month = date[2..4].parse::<u32>().ok()?;
    let day = date[4..6].parse::<u32>().ok()?;
    let expiration = chrono::NaiveDate::from_ymd_opt(year, month, day)?
        .and_hms_opt(0, 0, 0)?
        .and_utc()
        .timestamp();

    Some(ContractParts {
        underlying: underlying.to_string(),
        expiration,
        option_type,
        strike: strike.parse::<f64>().ok()? / 1000.0,
    })
}

/// A live subscription to one or more options chains.
///
/// Subscribe by underlying (`"AAPL"`) to follow the whole chain, or by full
/// OCC symbol (`"O:AAPL250117C00150000"`) to follow single contracts.
/// Requires the `polygon` feature and [`polygon::init`](crate::polygon::init).
///
/// # Example
///
/// ```no_run
/// use finance_query::streaming::OptionsChainStream;
/// use futures::StreamExt;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut stream = OptionsChainStream::subscribe(["AAPL"]).await?;
///
/// while let Some(contract) = stream.next().await {
///     println!("{} {:?}/{:?}", contract.contract_symbol, contract.bid, contract.ask);
/// }
/// # Ok(())
/// # }
/// ```
pub struct OptionsChainStream {
    inner: SourceStream<OptionContractUpdate>,
}

impl OptionsChainStream {
    /// Subscribe to the chains of the given underlyings.
    pub async fn subscribe<S, I>(underlyings: I) -> StreamResult<Self>
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        OptionsChainStreamBuilder::new()
            .underlyings(underlyings)
            .build()
            .await
    }

    /// Create an independent receiver sharing this subscription's connection.
    pub fn resubscribe(&self) -> Self {
        OptionsChainStream {
            inner: self.inner.resubscribe(),
        }
    }

    /// Add underlyings or contracts to the subscription.
    pub async fn add<S, I>(&self, symbols: I)
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.inner.add(symbols).await;
    }

    /// Remove underlyings or contracts from the subscription.
    pub async fn remove<S, I>(&self, symbols: I)
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.inner.remove(symbols).await;
    }

    /// Close the stream and disconnect.
    pub async fn close(&self) {
        self.inner.close().await;
    }
}

impl Stream for OptionsChainStream {
    type Item = OptionContractUpdate;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// Builder for an [`OptionsChainStream`].
pub struct OptionsChainStreamBuilder {
    underlyings: Vec<String>,
    retry_delay: Duration,
    greeks_refresh: Option<Duration>,
}

impl OptionsChainStreamBuilder {
    /// Create a builder with no symbols and default timings.
    pub fn new() -> Self {
        Self {
            underlyings: Vec::new(),
            retry_delay: Duration::from_secs(RECONNECT_BACKOFF_SECS),
            greeks_refresh: Some(DEFAULT_GREEKS_REFRESH),
        }
    }

    /// Add underlyings (or full OCC contract symbols) to follow.
    pub fn underlyings<S, I>(mut self, symbols: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.underlyings.extend(symbols.into_iter().map(Into::into));
        self
    }

    /// Set the delay between reconnection attempts (default: 3s).
    pub fn retry(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Interval between greeks/open-interest snapshot refreshes.
    ///
    /// `None` disables them, leaving only WebSocket bid/ask/last (one REST
    /// call per underlying per interval otherwise). Default: 60s.
    pub fn greeks_refresh(mut self, interval: Option<Duration>) -> Self {
        self.greeks_refresh = interval;
        self
    }

    /// Build and start the stream.
    pub async fn build(self) -> StreamResult<OptionsChainStream> {
        let source = Arc::new(PolygonOptionsSource::new(self.greeks_refresh));
        Ok(OptionsChainStream {
            inner: SourceStream::start(
                source,
                self.underlyings,
                self.retry_delay,
                CHANNEL_CAPACITY,
            ),
        })
    }
}

impl Default for OptionsChainStreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_call_contract_symbol() {
        let parts = parse_contract_symbol("O:AAPL250117C00150000").expect("should parse");
        assert_eq!(parts.underlying, "AAPL");
        assert_eq!(parts.option_type, OptionType::Call);
        assert!((parts.strike - 150.0).abs() < 1e-9);
        // 2025-01-17T00:00:00Z
        assert_eq!(parts.expiration, 1737072000);
    }

    #[test]
    fn parses_a_put_and_a_fractional_strike() {
        let parts = parse_contract_symbol("O:SPY261218P00512500").expect("should parse");
        assert_eq!(parts.underlying, "SPY");
        assert_eq!(parts.option_type, OptionType::Put);
        assert!((parts.strike - 512.5).abs() < 1e-9);
    }

    #[test]
    fn parses_without_the_o_prefix() {
        assert_eq!(
            parse_contract_symbol("AAPL250117C00150000")
                .unwrap()
                .underlying,
            "AAPL"
        );
    }

    #[test]
    fn rejects_malformed_symbols() {
        for bad in [
            "O:AAPL",
            "AAPL250117X00150000",
            "O:AAPL2501I7C00150000",
            "",
            "O:250117C00150000",
        ] {
            assert!(
                parse_contract_symbol(bad).is_none(),
                "expected {bad} to be rejected"
            );
        }
    }

    #[test]
    fn greeks_report_emptiness() {
        assert!(Greeks::default().is_empty());
        assert!(
            !Greeks {
                delta: Some(0.5),
                ..Default::default()
            }
            .is_empty()
        );
    }
}
