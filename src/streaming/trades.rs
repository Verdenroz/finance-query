//! Tick-by-tick trade streaming.
//!
//! [`PriceStream`](super::PriceStream) coalesces activity into a last-price
//! tick; this stream pushes every individual print, which is what
//! execution-quality and microstructure work needs.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::client::StreamResult;
use super::handle::{RECONNECT_BACKOFF, SourceStream, stream_builder, stream_handle};
use super::polygon::{AssetClass, PolygonTradeSource};
use super::source::ReconnectConfig;

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

stream_handle! {
    /// A subscription to every trade print for the given symbols.
    ///
    /// Requires the `polygon` feature and the `POLYGON_API_KEY` environment
    /// variable set.
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
    TradeStream(TradeTick);
    add: add_symbols = "Add symbols to the subscription.",
    remove: remove_symbols = "Remove symbols from the subscription.",
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
}

/// Builder for a [`TradeStream`].
pub struct TradeStreamBuilder {
    symbols: Vec<String>,
    asset_class: AssetClass,
    retry_delay: Duration,
    max_reconnect_attempts: Option<u32>,
}

impl TradeStreamBuilder {
    /// Create a builder defaulting to US equities.
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            asset_class: AssetClass::Stocks,
            retry_delay: RECONNECT_BACKOFF,
            max_reconnect_attempts: None,
        }
    }

    /// Choose the asset class (default: [`AssetClass::Stocks`]).
    ///
    /// Forex and indices publish no per-trade prints; those classes are
    /// rejected at [`build`](Self::build).
    pub fn asset_class(mut self, class: AssetClass) -> Self {
        self.asset_class = class;
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
        let reconnect =
            ReconnectConfig::new(self.retry_delay).max_attempts(self.max_reconnect_attempts);
        Ok(TradeStream {
            inner: SourceStream::start(source, self.symbols, reconnect, CHANNEL_CAPACITY),
        })
    }
}

stream_builder!(TradeStreamBuilder, symbols = "Add symbols to subscribe to.");

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
