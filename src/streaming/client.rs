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
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::BroadcastStream;
use tracing::warn;

use super::pricing::PriceUpdate;
use super::source::{StreamCommand, StreamSource, run_stream_loop};
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
/// let mut stream = PriceStream::subscribe(&["AAPL", "NVDA", "TSLA"]).await?;
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
    inner: BroadcastStream<PriceUpdate>,
    _handle: Arc<StreamHandle>,
}

/// Handle to manage the streaming session
struct StreamHandle {
    command_tx: mpsc::Sender<StreamCommand>,
    broadcast_tx: broadcast::Sender<PriceUpdate>,
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
    /// let stream = PriceStream::subscribe(&["AAPL", "GOOGL"]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe(symbols: &[&str]) -> StreamResult<Self> {
        Self::subscribe_with_source(
            Arc::new(YahooStreamSource),
            symbols,
            Duration::from_secs(RECONNECT_BACKOFF_SECS),
        )
        .await
    }

    /// Subscribe using a specific [`StreamSource`] backend.
    ///
    /// Yahoo is the default ([`subscribe`](Self::subscribe)); this is the
    /// generic entry point shared with [`PriceStreamBuilder`].
    pub(crate) async fn subscribe_with_source(
        source: Arc<dyn StreamSource>,
        symbols: &[&str],
        retry_delay: Duration,
    ) -> StreamResult<Self> {
        let (broadcast_tx, broadcast_rx) = broadcast::channel(CHANNEL_CAPACITY);
        let (command_tx, command_rx) = mpsc::channel(32);

        let initial_symbols: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();

        let tx_clone = broadcast_tx.clone();

        // Spawn the streaming task driving the chosen source.
        tokio::spawn(run_stream_loop(
            source,
            initial_symbols,
            broadcast_tx,
            command_rx,
            retry_delay,
        ));

        let handle = Arc::new(StreamHandle {
            command_tx,
            broadcast_tx: tx_clone,
        });

        Ok(PriceStream {
            inner: BroadcastStream::new(broadcast_rx),
            _handle: handle,
        })
    }

    /// Create a new receiver for this stream.
    ///
    /// Useful when you need multiple consumers of the same price data.
    pub fn resubscribe(&self) -> Self {
        PriceStream {
            inner: BroadcastStream::new(self._handle.broadcast_tx.subscribe()),
            _handle: Arc::clone(&self._handle),
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
    /// let stream = PriceStream::subscribe(&["AAPL"]).await?;
    /// stream.add_symbols(&["NVDA", "TSLA"]).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_symbols(&self, symbols: &[&str]) {
        let symbols: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        let _ = self
            ._handle
            .command_tx
            .send(StreamCommand::Subscribe(symbols))
            .await;
    }

    /// Remove symbols from the subscription.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::streaming::PriceStream;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = PriceStream::subscribe(&["AAPL", "NVDA"]).await?;
    /// stream.remove_symbols(&["NVDA"]).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_symbols(&self, symbols: &[&str]) {
        let symbols: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        let _ = self
            ._handle
            .command_tx
            .send(StreamCommand::Unsubscribe(symbols))
            .await;
    }

    /// Close the stream and disconnect from the WebSocket.
    pub async fn close(&self) {
        let _ = self._handle.command_tx.send(StreamCommand::Close).await;
    }
}

impl Stream for PriceStream {
    type Item = PriceUpdate;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(data))) => Poll::Ready(Some(data)),
            Poll::Ready(Some(Err(e))) => {
                warn!("Broadcast error: {:?}", e);
                // Try again on lag
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Builder for creating price streams with custom configuration
pub struct PriceStreamBuilder {
    symbols: Vec<String>,
    retry_delay: Duration,
}

impl PriceStreamBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            retry_delay: Duration::from_secs(RECONNECT_BACKOFF_SECS),
        }
    }

    /// Add symbols to subscribe to
    pub fn symbols(mut self, symbols: &[&str]) -> Self {
        self.symbols.extend(symbols.iter().map(|s| s.to_string()));
        self
    }

    /// Set the delay between reconnection attempts (default: 3s)
    pub fn retry(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Build and start the price stream (Yahoo-backed).
    pub async fn build(self) -> StreamResult<PriceStream> {
        let symbol_refs: Vec<&str> = self.symbols.iter().map(|s| s.as_str()).collect();
        PriceStream::subscribe_with_source(
            Arc::new(YahooStreamSource),
            &symbol_refs,
            self.retry_delay,
        )
        .await
    }
}

impl Default for PriceStreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}
