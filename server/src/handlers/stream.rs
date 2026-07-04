//! WebSocket /v2/stream — real-time price streaming.
//!
//! # Protocol
//!
//! **Subscribe to symbols:**
//! ```json
//! {"subscribe": ["AAPL", "NVDA", "TSLA"]}
//! ```
//!
//! **Unsubscribe from symbols:**
//! ```json
//! {"unsubscribe": ["AAPL"]}
//! ```
//!
//! **Receive price updates:**
//! ```json
//! {
//!   "id": "AAPL",
//!   "price": 178.52,
//!   "change": 2.34,
//!   "changePercent": 1.33,
//!   "time": 1703123456000,
//!   "exchange": "NMS",
//!   "marketHours": 2
//! }
//! ```

use axum::{
    extract::{
        Extension, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use finance_query_server::{AppState, metrics};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Stream command from client
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamCommand {
    subscribe: Option<Vec<String>>,
    unsubscribe: Option<Vec<String>>,
}

/// RAII guard to decrement WebSocket connection count on drop
struct ConnectionGuard;

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        metrics::WEBSOCKET_CONNECTIONS.dec();
    }
}

/// WebSocket /v2/stream
pub(crate) async fn ws_stream_handler(
    Extension(state): Extension<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Track new WebSocket connection
    metrics::WEBSOCKET_CONNECTIONS.inc();
    ws.on_upgrade(move |socket| handle_stream_socket(state, socket))
}

/// Handle the WebSocket connection for streaming
async fn handle_stream_socket(state: AppState, mut socket: WebSocket) {
    let _guard = ConnectionGuard; // Ensures connection count is decremented on exit
    info!("New streaming WebSocket connection");

    // Wait for initial subscription message
    let symbols = match wait_for_subscription(&mut socket).await {
        Some(symbols) => {
            metrics::WEBSOCKET_MESSAGES_RECEIVED.inc();
            symbols
        }
        None => {
            warn!("WebSocket closed before subscription");
            return;
        }
    };

    info!("Starting stream for symbols: {:?}", symbols);
    metrics::WEBSOCKET_SYMBOLS_SUBSCRIBED.set(symbols.len() as f64);

    // Ref-counted subscribe (shared upstream stream).
    if let Err(e) = state.stream_hub.subscribe_symbols(&symbols).await {
        error!("Failed to create shared price stream: {}", e);
        let _ = socket
            .send(Message::Text(
                serde_json::json!({"error": e.to_string()})
                    .to_string()
                    .into(),
            ))
            .await;
        return;
    }

    let mut hub_stream = match state.stream_hub.resubscribe().await {
        Some(s) => s,
        None => {
            let _ = socket
                .send(Message::Text(
                    serde_json::json!({"error": "stream unavailable"})
                        .to_string()
                        .into(),
                ))
                .await;
            return;
        }
    };

    let subscriptions = Arc::new(tokio::sync::RwLock::new(
        symbols.iter().cloned().collect::<HashSet<String>>(),
    ));

    let (out_tx, mut out_rx) = tokio::sync::mpsc::channel::<Message>(32);

    // Split socket for concurrent read/write
    let (mut sender, mut receiver) = socket.split();

    // Spawn task to forward filtered price updates + outbound messages to client
    let subscriptions_for_send = Arc::clone(&subscriptions);
    let mut send_task = tokio::spawn(async move {
        use futures_util::stream::StreamExt;
        loop {
            tokio::select! {
                msg = out_rx.recv() => {
                    match msg {
                        Some(msg) => {
                            if sender.send(msg).await.is_err() {
                                break;
                            }
                        }
                        None => {
                            // Control channel closed.
                            break;
                        }
                    }
                }

                maybe_price = hub_stream.next() => {
                    match maybe_price {
                        Some(price) => {
                            let should_send = {
                                let subs = subscriptions_for_send.read().await;
                                subs.contains(&price.id)
                            };

                            if should_send {
                                let json = serde_json::to_string(&price).unwrap_or_default();
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                                metrics::WEBSOCKET_MESSAGES_SENT.inc();
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    // Handle incoming messages (subscribe/unsubscribe)
    let subscriptions_for_recv = Arc::clone(&subscriptions);
    let stream_hub = state.stream_hub.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(cmd) = serde_json::from_str::<StreamCommand>(&text) {
                        metrics::WEBSOCKET_MESSAGES_RECEIVED.inc();
                        info!("Received stream command: {:?}", cmd);

                        if let Some(symbols) = cmd.subscribe {
                            let mut newly_added: Vec<String> = Vec::new();
                            {
                                let mut subs = subscriptions_for_recv.write().await;
                                for s in symbols {
                                    if subs.insert(s.clone()) {
                                        newly_added.push(s);
                                    }
                                }
                            }

                            if !newly_added.is_empty() {
                                if let Err(e) = stream_hub.subscribe_symbols(&newly_added).await {
                                    error!("Failed to subscribe symbols: {}", e);
                                    {
                                        let mut subs = subscriptions_for_recv.write().await;
                                        for s in &newly_added {
                                            subs.remove(s);
                                        }
                                    }
                                    let _ = out_tx
                                        .send(Message::Text(
                                            serde_json::json!({"error": e.to_string()})
                                                .to_string()
                                                .into(),
                                        ))
                                        .await;
                                } else {
                                    // Update symbol count on successful subscription
                                    let count = {
                                        let subs = subscriptions_for_recv.read().await;
                                        subs.len()
                                    };
                                    metrics::WEBSOCKET_SYMBOLS_SUBSCRIBED.set(count as f64);
                                }
                            }
                        }

                        if let Some(symbols) = cmd.unsubscribe {
                            let mut removed: Vec<String> = Vec::new();
                            {
                                let mut subs = subscriptions_for_recv.write().await;
                                for s in symbols {
                                    if subs.remove(&s) {
                                        removed.push(s);
                                    }
                                }
                            }

                            if !removed.is_empty() {
                                stream_hub.unsubscribe_symbols(&removed).await;
                                // Update symbol count after unsubscription
                                let count = {
                                    let subs = subscriptions_for_recv.read().await;
                                    subs.len()
                                };
                                metrics::WEBSOCKET_SYMBOLS_SUBSCRIBED.set(count as f64);
                            }
                        }
                    }
                }
                Message::Close(_) => {
                    info!("WebSocket closed by client");
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to complete, then ensure per-client resources are torn down.
    tokio::select! {
        _ = &mut send_task => info!("Send task completed"),
        _ = &mut recv_task => info!("Receive task completed"),
    }

    // Ensure tasks stop promptly.
    send_task.abort();
    recv_task.abort();

    // Release this client's active subscriptions from the global hub.
    let symbols_to_release: Vec<String> = {
        let subs = subscriptions.read().await;
        subs.iter().cloned().collect()
    };
    state
        .stream_hub
        .unsubscribe_symbols(&symbols_to_release)
        .await;

    info!("WebSocket stream connection closed");
}

/// Wait for initial subscription message
async fn wait_for_subscription(socket: &mut WebSocket) -> Option<Vec<String>> {
    while let Some(Ok(msg)) = socket.next().await {
        if let Message::Text(text) = msg
            && let Ok(cmd) = serde_json::from_str::<StreamCommand>(&text)
            && let Some(symbols) = cmd.subscribe
        {
            return Some(symbols);
        }
    }
    None
}
