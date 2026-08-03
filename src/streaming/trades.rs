//! Tick-by-tick trade streaming.
//!
//! [`PriceStream`](super::PriceStream) coalesces activity into a last-price
//! tick; this stream pushes every individual print, which is what
//! execution-quality and microstructure work needs.

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::stream::Stream;
use serde::{Deserialize, Serialize};

use super::client::StreamResult;
use super::handle::SourceStream;
use super::polygon::{AssetClass, PolygonTradeSource};

/// Reconnection backoff, matching [`PriceStream`](super::PriceStream).
const RECONNECT_BACKOFF_SECS: u64 = 3;

/// Channel capacity — trade prints are the highest-volume feed here.
const CHANNEL_CAPACITY: usize = 4096;

/// A single executed trade.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TradeTick {
    /// Symbol or pair the trade executed on.
    pub symbol: String,
    /// Execution price.
    pub price: f64,
    /// Executed size (shares, contracts, or base-asset units).
    pub size: f64,
    /// Exchange identifier, where the venue reports one.
    pub exchange: Option<i32>,
    /// Trade condition codes.
    pub conditions: Vec<i32>,
    /// Provider trade identifier, where one is supplied.
    pub trade_id: Option<String>,
    /// Execution timestamp (milliseconds).
    pub time: i64,
}

impl TradeTick {
    /// Notional value of the print (`price * size`).
    pub fn notional(&self) -> f64 {
        self.price * self.size
    }
}

/// A subscription to every trade print for the given symbols.
///
/// Requires the `polygon` feature and [`polygon::init`](crate::polygon::init).
/// This is a companion to [`PriceStream`](super::PriceStream), not a
/// replacement — most consumers want the coalesced tick.
///
/// # Example
///
/// ```no_run
/// use finance_query::streaming::TradeStream;
/// use futures::StreamExt;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut trades = TradeStream::subscribe(["AAPL"]).await?;
///
/// while let Some(trade) = trades.next().await {
///     println!("{} {} @ {}", trade.symbol, trade.size, trade.price);
/// }
/// # Ok(())
/// # }
/// ```
pub struct TradeStream {
    inner: SourceStream<TradeTick>,
}

impl TradeStream {
    /// Subscribe to US equity trade prints for the given symbols.
    pub async fn subscribe<S, I>(symbols: I) -> StreamResult<Self>
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        TradeStreamBuilder::new().symbols(symbols).build().await
    }

    /// Create an independent receiver sharing this subscription's connection.
    pub fn resubscribe(&self) -> Self {
        TradeStream {
            inner: self.inner.resubscribe(),
        }
    }

    /// Add symbols to the subscription.
    pub async fn add_symbols<S, I>(&self, symbols: I)
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.inner.add(symbols).await;
    }

    /// Remove symbols from the subscription.
    pub async fn remove_symbols<S, I>(&self, symbols: I)
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

impl Stream for TradeStream {
    type Item = TradeTick;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// Builder for a [`TradeStream`].
pub struct TradeStreamBuilder {
    symbols: Vec<String>,
    asset_class: AssetClass,
    retry_delay: Duration,
}

impl TradeStreamBuilder {
    /// Create a builder defaulting to US equities.
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            asset_class: AssetClass::Stocks,
            retry_delay: Duration::from_secs(RECONNECT_BACKOFF_SECS),
        }
    }

    /// Add symbols to subscribe to.
    pub fn symbols<S, I>(mut self, symbols: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.symbols.extend(symbols.into_iter().map(Into::into));
        self
    }

    /// Choose the asset class (default: [`AssetClass::Stocks`]).
    ///
    /// Forex and indices publish no per-trade prints; those classes are
    /// rejected at [`build`](Self::build).
    pub fn asset_class(mut self, class: AssetClass) -> Self {
        self.asset_class = class;
        self
    }

    /// Set the delay between reconnection attempts (default: 3s).
    pub fn retry(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Build and start the stream.
    ///
    /// # Errors
    ///
    /// Returns [`StreamError::ConnectionFailed`](super::StreamError::ConnectionFailed)
    /// when the chosen asset class has no trade feed.
    pub async fn build(self) -> StreamResult<TradeStream> {
        let source = Arc::new(PolygonTradeSource::new(self.asset_class)?);
        Ok(TradeStream {
            inner: SourceStream::start(source, self.symbols, self.retry_delay, CHANNEL_CAPACITY),
        })
    }
}

impl Default for TradeStreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn classes_without_trade_prints_are_rejected() {
        for class in [AssetClass::Forex, AssetClass::Indices] {
            assert!(
                TradeStreamBuilder::new()
                    .symbols(["X"])
                    .asset_class(class)
                    .build()
                    .await
                    .is_err()
            );
        }
    }

    #[test]
    fn notional_multiplies_price_by_size() {
        let tick = TradeTick {
            price: 10.0,
            size: 25.0,
            ..Default::default()
        };
        assert!((tick.notional() - 250.0).abs() < 1e-9);
    }
}
