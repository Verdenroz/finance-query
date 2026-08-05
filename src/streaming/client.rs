//! Streaming client providing a Stream-based API for real-time price updates.
//!
//! Backed by a pluggable [`StreamSource`](super::source::StreamSource) — Yahoo
//! is the default implementation, with additional providers (e.g. Polygon)
//! supported through the same abstraction.

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::stream::Stream;

use super::handle::SourceStream;
use super::pricing::PriceUpdate;
use super::source::{ReconnectConfig, StreamSource};
use super::yahoo::YahooStreamSource;
use crate::error::FinanceError;

/// Result type for streaming operations
pub type StreamResult<T> = std::result::Result<T, StreamError>;

/// Errors that can occur during streaming
#[derive(Debug, Clone)]
pub enum StreamError {
    /// WebSocket connection failed
    ConnectionFailed(String),
    /// WebSocket send/receive error
    WebSocketError(String),
    /// Failed to decode message
    DecodeError(String),
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamError::ConnectionFailed(e) => write!(f, "Connection failed: {}", e),
            StreamError::WebSocketError(e) => write!(f, "WebSocket error: {}", e),
            StreamError::DecodeError(e) => write!(f, "Decode error: {}", e),
        }
    }
}

impl std::error::Error for StreamError {}

impl From<StreamError> for FinanceError {
    fn from(e: StreamError) -> Self {
        FinanceError::ResponseStructureError {
            field: "streaming".to_string(),
            context: e.to_string(),
        }
    }
}

/// Reconnection backoff duration
const RECONNECT_BACKOFF_SECS: u64 = 3;

/// Channel capacity for price updates
const CHANNEL_CAPACITY: usize = 1024;

/// A streaming price subscription that yields real-time price updates.
///
/// This provides a Flow-like API for receiving real-time price data.
/// Backed by a pluggable source (Yahoo by default).
///
/// # Example
///
/// ```no_run
/// use finance_query::streaming::PriceStream;
/// use futures::StreamExt;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Subscribe to multiple symbols
/// let mut stream = PriceStream::subscribe(["AAPL", "NVDA", "TSLA"]).await?;
///
/// // Receive price updates
/// while let Some(price) = stream.next().await {
///     println!("{}: ${:.2} ({:+.2}%)",
///         price.id,
///         price.price,
///         price.change_percent
///     );
/// }
/// # Ok(())
/// # }
/// ```
pub struct PriceStream {
    inner: SourceStream<PriceUpdate>,
}

impl PriceStream {
    /// Subscribe to real-time price updates for the given symbols.
    ///
    /// # Arguments
    ///
    /// * `symbols` - Ticker symbols to subscribe to (e.g., `["AAPL", "NVDA"]`)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::streaming::PriceStream;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = PriceStream::subscribe(["AAPL", "GOOGL"]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe<S, I>(symbols: I) -> StreamResult<Self>
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        Self::subscribe_with_source(
            Arc::new(YahooStreamSource),
            symbols,
            ReconnectConfig::new(Duration::from_secs(RECONNECT_BACKOFF_SECS)),
        )
        .await
    }

    /// Subscribe using a specific [`StreamSource`] backend.
    ///
    /// Yahoo is the default ([`subscribe`](Self::subscribe)); this is the
    /// generic entry point shared with [`PriceStreamBuilder`].
    pub(crate) async fn subscribe_with_source<S, I>(
        source: Arc<dyn StreamSource<PriceUpdate>>,
        symbols: I,
        reconnect: ReconnectConfig,
    ) -> StreamResult<Self>
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        let initial_symbols: Vec<String> = symbols.into_iter().map(Into::into).collect();

        Ok(PriceStream {
            inner: SourceStream::start(source, initial_symbols, reconnect, CHANNEL_CAPACITY),
        })
    }

    /// Create a new receiver for this stream.
    ///
    /// Useful when you need multiple consumers of the same price data.
    pub fn resubscribe(&self) -> Self {
        PriceStream {
            inner: self.inner.resubscribe(),
        }
    }

    /// Add more symbols to the subscription.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::streaming::PriceStream;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = PriceStream::subscribe(["AAPL"]).await?;
    /// stream.add_symbols(["NVDA", "TSLA"]).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_symbols<S, I>(&self, symbols: I)
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.inner.add(symbols).await;
    }

    /// Remove symbols from the subscription.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::streaming::PriceStream;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = PriceStream::subscribe(["AAPL", "NVDA"]).await?;
    /// stream.remove_symbols(["NVDA"]).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_symbols<S, I>(&self, symbols: I)
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.inner.remove(symbols).await;
    }

    /// Close the stream and disconnect from the WebSocket.
    pub async fn close(&self) {
        self.inner.close().await;
    }
}

impl Stream for PriceStream {
    type Item = PriceUpdate;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// Which upstream backend a [`PriceStream`] connects to.
///
/// Yahoo multiplexes every asset class onto one connection; Polygon runs a
/// separate cluster per asset class, so its variant carries the class to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum PriceSource {
    /// Yahoo Finance WebSocket — keyless, all asset classes (default).
    #[default]
    Yahoo,
    /// Polygon.io real-time cluster for one asset class. Requires
    /// [`polygon::init`](crate::polygon::init).
    #[cfg(feature = "polygon")]
    Polygon(crate::streaming::AssetClass),
}

/// Builder for creating price streams with custom configuration
pub struct PriceStreamBuilder {
    symbols: Vec<String>,
    retry_delay: Duration,
    max_reconnect_attempts: Option<u32>,
    source: PriceSource,
}

impl PriceStreamBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            retry_delay: Duration::from_secs(RECONNECT_BACKOFF_SECS),
            max_reconnect_attempts: None,
            source: PriceSource::Yahoo,
        }
    }

    /// Choose the upstream backend (default: [`PriceSource::Yahoo`]).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "polygon")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use finance_query::streaming::{AssetClass, PriceSource, PriceStreamBuilder};
    ///
    /// let stream = PriceStreamBuilder::new()
    ///     .symbols(["BTC-USD"])
    ///     .source(PriceSource::Polygon(AssetClass::Crypto))
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn source(mut self, source: PriceSource) -> Self {
        self.source = source;
        self
    }

    /// Add symbols to subscribe to
    pub fn symbols<S, I>(mut self, symbols: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.symbols.extend(symbols.into_iter().map(Into::into));
        self
    }

    /// Set the base delay before the first reconnection attempt (default:
    /// 3s). Later attempts grow exponentially from this, capped and
    /// jittered — see [`Self::max_reconnect_attempts`] to also cap how many
    /// attempts are made.
    pub fn retry(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Cap the number of consecutive reconnect attempts before the stream
    /// gives up and ends (default: unlimited, i.e. retry forever like before
    /// issue #276).
    pub fn max_reconnect_attempts(mut self, max: u32) -> Self {
        self.max_reconnect_attempts = Some(max);
        self
    }

    /// Build and start the price stream using the configured source.
    pub async fn build(self) -> StreamResult<PriceStream> {
        let source: Arc<dyn StreamSource<PriceUpdate>> = match self.source {
            PriceSource::Yahoo => Arc::new(YahooStreamSource),
            #[cfg(feature = "polygon")]
            PriceSource::Polygon(class) => Arc::new(super::polygon::PolygonPriceSource::new(class)),
        };
        let reconnect =
            ReconnectConfig::new(self.retry_delay).max_attempts(self.max_reconnect_attempts);
        PriceStream::subscribe_with_source(source, self.symbols, reconnect).await
    }
}

impl Default for PriceStreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}
