//! Yahoo Finance WebSocket streaming source.
//!
//! Connects to Yahoo's real-time WebSocket, manages subscriptions with a
//! periodic heartbeat refresh, and decodes the base64 protobuf payloads into
//! [`PriceUpdate`]s. Implements [`StreamSource`] so it plugs into the generic
//! reconnect/stream machinery.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use futures::SinkExt;
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::time::interval;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use super::client::{StreamError, StreamResult};
use super::pricing::{PriceUpdate, PricingData, PricingDecodeError};
use super::source::{StreamCommand, StreamSource};

/// Yahoo Finance WebSocket URL.
const YAHOO_WS_URL: &str = "wss://streamer.finance.yahoo.com/?version=2";

/// Heartbeat interval for subscription refresh.
const HEARTBEAT_INTERVAL_SECS: u64 = 15;

/// Real-time price source backed by Yahoo Finance's WebSocket.
pub(crate) struct YahooStreamSource;

#[async_trait::async_trait]
impl StreamSource for YahooStreamSource {
    fn id(&self) -> &'static str {
        "yahoo"
    }

    async fn run_session(
        &self,
        subscriptions: &Arc<RwLock<HashSet<String>>>,
        broadcast_tx: &broadcast::Sender<PriceUpdate>,
        command_rx: &mut mpsc::Receiver<StreamCommand>,
    ) -> StreamResult<()> {
        connect_and_stream(subscriptions, broadcast_tx, command_rx).await
    }
}

/// Connect to Yahoo WebSocket and stream data until close/disconnect.
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
                        let mut newly_added = Vec::new();
                        {
                            let mut subs = subscriptions.write().await;
                            for s in &symbols {
                                if subs.insert(s.clone()) {
                                    newly_added.push(s.clone());
                                }
                            }
                        }
                        if !newly_added.is_empty() {
                            let msg = serde_json::json!({ "subscribe": newly_added });
                            let _ = write.send(Message::Text(msg.to_string().into())).await;
                            info!("Added subscriptions: {:?}", newly_added);
                        }
                    }
                    StreamCommand::Unsubscribe(symbols) => {
                        let mut actually_removed = Vec::new();
                        {
                            let mut subs = subscriptions.write().await;
                            for s in &symbols {
                                if subs.remove(s) {
                                    actually_removed.push(s.clone());
                                }
                            }
                        }
                        if !actually_removed.is_empty() {
                            let msg = serde_json::json!({ "unsubscribe": actually_removed });
                            let _ = write.send(Message::Text(msg.to_string().into())).await;
                            info!("Removed subscriptions: {:?}", actually_removed);
                        }
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

/// Handle incoming text message from Yahoo WebSocket.
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

        // Broadcast to all receivers
        if broadcast_tx.receiver_count() > 0 {
            let _ = broadcast_tx.send(price_update);
        }
    }

    Ok(())
}
