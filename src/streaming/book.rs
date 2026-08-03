//! Order-book (level 2) depth streaming.
//!
//! Top-of-book bid/ask is all [`PriceUpdate`](super::PriceUpdate) can carry;
//! this stream pushes the full ladder of price levels per side.

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::stream::Stream;
use serde::{Deserialize, Serialize};

use super::client::StreamResult;
use super::handle::SourceStream;
use super::polygon::PolygonBookSource;

/// Reconnection backoff, matching [`PriceStream`](super::PriceStream).
const RECONNECT_BACKOFF_SECS: u64 = 3;

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
pub struct DepthStream {
    inner: SourceStream<OrderBookUpdate>,
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

    /// Create an independent receiver sharing this subscription's connection.
    pub fn resubscribe(&self) -> Self {
        DepthStream {
            inner: self.inner.resubscribe(),
        }
    }

    /// Add pairs to the subscription.
    pub async fn add_pairs<S, I>(&self, pairs: I)
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.inner.add(pairs).await;
    }

    /// Remove pairs from the subscription.
    pub async fn remove_pairs<S, I>(&self, pairs: I)
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.inner.remove(pairs).await;
    }

    /// Close the stream and disconnect.
    pub async fn close(&self) {
        self.inner.close().await;
    }
}

impl Stream for DepthStream {
    type Item = OrderBookUpdate;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// Builder for a [`DepthStream`].
pub struct DepthStreamBuilder {
    pairs: Vec<String>,
    retry_delay: Duration,
}

impl DepthStreamBuilder {
    /// Create a builder with no pairs.
    pub fn new() -> Self {
        Self {
            pairs: Vec::new(),
            retry_delay: Duration::from_secs(RECONNECT_BACKOFF_SECS),
        }
    }

    /// Add crypto pairs to subscribe to.
    pub fn pairs<S, I>(mut self, pairs: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.pairs.extend(pairs.into_iter().map(Into::into));
        self
    }

    /// Set the delay between reconnection attempts (default: 3s).
    pub fn retry(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Build and start the stream.
    pub async fn build(self) -> StreamResult<DepthStream> {
        Ok(DepthStream {
            inner: SourceStream::start(
                Arc::new(PolygonBookSource),
                self.pairs,
                self.retry_delay,
                CHANNEL_CAPACITY,
            ),
        })
    }
}

impl Default for DepthStreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

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
