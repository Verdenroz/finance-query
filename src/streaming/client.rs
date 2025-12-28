//! WebSocket client for Yahoo Finance real-time streaming
//!
//! Provides a Stream-based API for receiving real-time price updates.

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::SinkExt;
use futures::stream::Stream;
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::time::interval;
use tokio_stream::wrappers::BroadcastStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use super::pricing::{PriceUpdate, PricingData, PricingDecodeError};
use crate::error::YahooError;

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

impl From<StreamError> for YahooError {
    fn from(e: StreamError) -> Self {
        YahooError::ResponseStructureError {
            field: "streaming".to_string(),
            context: e.to_string(),
        }
    }
}

/// Yahoo Finance WebSocket URL
const YAHOO_WS_URL: &str = "wss://streamer.finance.yahoo.com/?version=2";

/// Heartbeat interval for subscription refresh
const HEARTBEAT_INTERVAL_SECS: u64 = 15;

/// Reconnection backoff duration
const RECONNECT_BACKOFF_SECS: u64 = 3;

/// Channel capacity for price updates
const CHANNEL_CAPACITY: usize = 1024;

/// A streaming price subscription that yields real-time price updates.
///
/// This provides a Flow-like API for receiving real-time price data from Yahoo Finance.
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
///         price.symbol(),
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

/// Handle to manage the WebSocket connection
struct StreamHandle {
    command_tx: mpsc::Sender<StreamCommand>,
    broadcast_tx: broadcast::Sender<PriceUpdate>,
}

/// Commands sent to the WebSocket task
enum StreamCommand {
    Subscribe(Vec<String>),
    Unsubscribe(Vec<String>),
    Close,
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
        let (broadcast_tx, broadcast_rx) = broadcast::channel(CHANNEL_CAPACITY);
        let (command_tx, command_rx) = mpsc::channel(32);

        let symbols: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        let initial_symbols = symbols.clone();

        let tx_clone = broadcast_tx.clone();

        // Spawn the WebSocket task
        tokio::spawn(async move {
            if let Err(e) = run_websocket_loop(initial_symbols, broadcast_tx, command_rx).await {
                error!("WebSocket loop error: {}", e);
            }
        });

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

/// Run the WebSocket connection loop with automatic reconnection
async fn run_websocket_loop(
    initial_symbols: Vec<String>,
    broadcast_tx: broadcast::Sender<PriceUpdate>,
    mut command_rx: mpsc::Receiver<StreamCommand>,
) -> StreamResult<()> {
    let subscriptions = Arc::new(RwLock::new(HashSet::<String>::from_iter(initial_symbols)));

    loop {
        match connect_and_stream(&subscriptions, &broadcast_tx, &mut command_rx).await {
            Ok(()) => {
                info!("WebSocket connection closed gracefully");
                break;
            }
            Err(e) => {
                error!(
                    "WebSocket error: {}, reconnecting in {}s...",
                    e, RECONNECT_BACKOFF_SECS
                );
                tokio::time::sleep(Duration::from_secs(RECONNECT_BACKOFF_SECS)).await;
            }
        }
    }

    Ok(())
}

/// Connect to Yahoo WebSocket and stream data
async fn connect_and_stream(
    subscriptions: &Arc<RwLock<HashSet<String>>>,
    broadcast_tx: &broadcast::Sender<PriceUpdate>,
    command_rx: &mut mpsc::Receiver<StreamCommand>,
) -> StreamResult<()> {
    use futures::StreamExt;

    info!("Connecting to Yahoo Finance WebSocket...");

    let (ws_stream, _) = connect_async(YAHOO_WS_URL)
        .await
        .map_err(|e| StreamError::ConnectionFailed(e.to_string()))?;

    info!("Connected to Yahoo Finance WebSocket");

    let (mut write, mut read) = ws_stream.split();

    // Send initial subscriptions
    {
        let subs = subscriptions.read().await;
        if !subs.is_empty() {
            let symbols: Vec<&str> = subs.iter().map(|s| s.as_str()).collect();
            let msg = serde_json::json!({ "subscribe": symbols });
            write
                .send(Message::Text(msg.to_string().into()))
                .await
                .map_err(|e| StreamError::WebSocketError(e.to_string()))?;
            info!("Subscribed to {} symbols", symbols.len());
        }
    }

    // Heartbeat task - sends subscription refresh every 15 seconds
    let heartbeat_subs = Arc::clone(subscriptions);
    let (heartbeat_tx, mut heartbeat_rx) = mpsc::channel::<Message>(32);

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
        loop {
            interval.tick().await;
            let subs = heartbeat_subs.read().await;
            if !subs.is_empty() {
                let symbols: Vec<&str> = subs.iter().map(|s| s.as_str()).collect();
                let msg = serde_json::json!({ "subscribe": symbols });
                if heartbeat_tx
                    .send(Message::Text(msg.to_string().into()))
                    .await
                    .is_err()
                {
                    break;
                }
                debug!("Heartbeat subscription sent for {} symbols", symbols.len());
            }
        }
    });

    loop {
        tokio::select! {
            // Handle incoming WebSocket messages
            Some(msg) = read.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        debug!("Received text message: {} bytes", text.len());
                        if let Err(e) = handle_text_message(&text, broadcast_tx) {
                            warn!("Failed to handle message: {}", e);
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        debug!("Received binary message: {} bytes", data.len());
                    }
                    Ok(Message::Close(_)) => {
                        info!("Received close frame");
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        let _ = write.send(Message::Pong(data)).await;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("WebSocket read error: {}", e);
                        return Err(StreamError::WebSocketError(e.to_string()));
                    }
                }
            }

            // Handle heartbeat messages
            Some(msg) = heartbeat_rx.recv() => {
                if let Err(e) = write.send(msg).await {
                    error!("Failed to send heartbeat: {}", e);
                    return Err(StreamError::WebSocketError(e.to_string()));
                }
            }

            // Handle commands (subscribe/unsubscribe)
            Some(cmd) = command_rx.recv() => {
                match cmd {
                    StreamCommand::Subscribe(symbols) => {
                        let mut subs = subscriptions.write().await;
                        for s in &symbols {
                            subs.insert(s.clone());
                        }
                        let msg = serde_json::json!({ "subscribe": symbols });
                        let _ = write.send(Message::Text(msg.to_string().into())).await;
                        info!("Added subscriptions: {:?}", symbols);
                    }
                    StreamCommand::Unsubscribe(symbols) => {
                        let mut subs = subscriptions.write().await;
                        for s in &symbols {
                            subs.remove(s);
                        }
                        let msg = serde_json::json!({ "unsubscribe": symbols });
                        let _ = write.send(Message::Text(msg.to_string().into())).await;
                        info!("Removed subscriptions: {:?}", symbols);
                    }
                    StreamCommand::Close => {
                        info!("Received close command");
                        let _ = write.send(Message::Close(None)).await;
                        return Ok(());
                    }
                }
            }

            else => break,
        }
    }

    Ok(())
}

/// Handle incoming text message from Yahoo WebSocket
fn handle_text_message(
    text: &str,
    broadcast_tx: &broadcast::Sender<PriceUpdate>,
) -> std::result::Result<(), PricingDecodeError> {
    // Yahoo sends JSON with base64-encoded protobuf in "message" field
    let json: serde_json::Value =
        serde_json::from_str(text).map_err(|e| PricingDecodeError::Base64(e.to_string()))?;

    if let Some(encoded) = json.get("message").and_then(|v| v.as_str()) {
        let pricing_data = PricingData::from_base64(encoded)?;
        let price_update: PriceUpdate = pricing_data.into();
        debug!(
            "Decoded price: {} = ${:.2}",
            price_update.id, price_update.price
        );

        // Broadcast to all receivers
        if broadcast_tx.receiver_count() > 0 {
            let _ = broadcast_tx.send(price_update);
        }
    }

    Ok(())
}

/// Builder for creating price streams with custom configuration
pub struct PriceStreamBuilder {
    symbols: Vec<String>,
    reconnect_delay: Duration,
}

impl PriceStreamBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            reconnect_delay: Duration::from_secs(RECONNECT_BACKOFF_SECS),
        }
    }

    /// Add symbols to subscribe to
    pub fn symbols(mut self, symbols: &[&str]) -> Self {
        self.symbols.extend(symbols.iter().map(|s| s.to_string()));
        self
    }

    /// Set reconnection delay
    pub fn reconnect_delay(mut self, delay: Duration) -> Self {
        self.reconnect_delay = delay;
        self
    }

    /// Build and start the price stream
    pub async fn build(self) -> StreamResult<PriceStream> {
        let symbol_refs: Vec<&str> = self.symbols.iter().map(|s| s.as_str()).collect();
        PriceStream::subscribe(&symbol_refs).await
    }
}

impl Default for PriceStreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}
