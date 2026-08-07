//! Order-book (level 2) depth streaming.
//!
//! Top-of-book bid/ask is all [`PriceUpdate`](super::PriceUpdate) can carry;
//! this stream pushes the full ladder of price levels per side.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::client::StreamResult;
use super::handle::{RECONNECT_BACKOFF, SourceStream, stream_builder, stream_handle};
use super::polygon::PolygonBookSource;
use super::source::ReconnectConfig;

/// Channel capacity — book updates are large but less frequent than prints.
const CHANNEL_CAPACITY: usize = 1024;

/// One price level of an order book.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BookLevel {
    /// Level price.
    pub price: f64,
    /// Total size resting at this price.
    pub size: f64,
}

/// A depth-of-book update: both sides, best level first.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OrderBookUpdate {
    /// Symbol or pair this book belongs to.
    pub symbol: String,
    /// Bid levels, highest price first.
    pub bids: Vec<BookLevel>,
    /// Ask levels, lowest price first.
    pub asks: Vec<BookLevel>,
    /// Exchange identifier, where the venue reports one.
    pub exchange: Option<i32>,
    /// Update timestamp (milliseconds).
    pub time: i64,
}

impl OrderBookUpdate {
    /// Best (highest) bid level.
    pub fn best_bid(&self) -> Option<BookLevel> {
        self.bids.first().copied()
    }

    /// Best (lowest) ask level.
    pub fn best_ask(&self) -> Option<BookLevel> {
        self.asks.first().copied()
    }

    /// Difference between best ask and best bid.
    pub fn spread(&self) -> Option<f64> {
        Some(self.best_ask()?.price - self.best_bid()?.price)
    }

    /// Midpoint of the top of book.
    pub fn mid(&self) -> Option<f64> {
        Some((self.best_ask()?.price + self.best_bid()?.price) / 2.0)
    }

    /// Total size resting on each side, as `(bid_depth, ask_depth)`.
    pub fn depth(&self) -> (f64, f64) {
        (
            self.bids.iter().map(|l| l.size).sum(),
            self.asks.iter().map(|l| l.size).sum(),
        )
    }
}

stream_handle! {
    /// A subscription to level-2 order-book depth.
    ///
    /// Backed by Polygon's crypto level-2 feed (`XL2`) — the one cluster that
    /// publishes depth — so pairs are crypto pairs (`"BTC-USD"`). Requires the
    /// `polygon` feature and [`polygon::init`](crate::polygon::init).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::streaming::DepthStream;
    /// use futures::StreamExt;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut books = DepthStream::subscribe(["BTC-USD"]).await?;
    ///
    /// while let Some(book) = books.next().await {
    ///     println!("{} spread {:?}", book.symbol, book.spread());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    DepthStream(OrderBookUpdate);
    add: add_pairs = "Add pairs to the subscription.",
    remove: remove_pairs = "Remove pairs from the subscription.",
}

impl DepthStream {
    /// Subscribe to depth updates for the given crypto pairs.
    pub async fn subscribe<S, I>(pairs: I) -> StreamResult<Self>
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        DepthStreamBuilder::new().pairs(pairs).build().await
    }
}

/// Builder for a [`DepthStream`].
pub struct DepthStreamBuilder {
    pairs: Vec<String>,
    retry_delay: Duration,
    max_reconnect_attempts: Option<u32>,
}

impl DepthStreamBuilder {
    /// Create a builder with no pairs.
    pub fn new() -> Self {
        Self {
            pairs: Vec::new(),
            retry_delay: RECONNECT_BACKOFF,
            max_reconnect_attempts: None,
        }
    }

    /// Build and start the stream.
    pub async fn build(self) -> StreamResult<DepthStream> {
        let reconnect =
            ReconnectConfig::new(self.retry_delay).max_attempts(self.max_reconnect_attempts);
        Ok(DepthStream {
            inner: SourceStream::start(
                Arc::new(PolygonBookSource),
                self.pairs,
                reconnect,
                CHANNEL_CAPACITY,
            ),
        })
    }
}

stream_builder!(
    DepthStreamBuilder,
    pairs = "Add crypto pairs to subscribe to."
);

#[cfg(test)]
mod tests {
    use super::*;

    fn book() -> OrderBookUpdate {
        OrderBookUpdate {
            symbol: "BTC-USD".into(),
            bids: vec![
                BookLevel {
                    price: 100.0,
                    size: 2.0,
                },
                BookLevel {
                    price: 99.0,
                    size: 3.0,
                },
            ],
            asks: vec![
                BookLevel {
                    price: 101.0,
                    size: 1.0,
                },
                BookLevel {
                    price: 102.0,
                    size: 4.0,
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn top_of_book_helpers_use_the_first_level() {
        let book = book();
        assert_eq!(book.best_bid().unwrap().price, 100.0);
        assert_eq!(book.best_ask().unwrap().price, 101.0);
        assert!((book.spread().unwrap() - 1.0).abs() < 1e-9);
        assert!((book.mid().unwrap() - 100.5).abs() < 1e-9);
    }

    #[test]
    fn depth_sums_each_side() {
        let (bid_depth, ask_depth) = book().depth();
        assert!((bid_depth - 5.0).abs() < 1e-9);
        assert!((ask_depth - 5.0).abs() < 1e-9);
    }

    #[test]
    fn an_empty_side_has_no_spread() {
        let empty = OrderBookUpdate::default();
        assert!(empty.spread().is_none());
        assert!(empty.mid().is_none());
    }
}
