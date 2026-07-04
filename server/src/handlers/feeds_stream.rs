//! WebSocket /v2/feeds/stream — continuous RSS/Atom feed entries.
//!
//! RSS/Atom itself has no push transport (see `finance_query::streaming::NewsStream`),
//! so "streaming" here means: the server polls the configured sources on an
//! interval and pushes newly-seen entries (deduplicated by URL) to every
//! connected client, same as the WebSocket does for real-time prices in
//! `stream.rs` — just with a poll-based upstream instead of a push-based one.
//!
//! # Protocol
//!
//! **Subscribe to sources:**
//! ```json
//! {"subscribe": ["bloomberg", "marketwatch"], "formType": "10-K"}
//! ```
//!
//! **Unsubscribe from sources:**
//! ```json
//! {"unsubscribe": ["bloomberg"]}
//! ```
//!
//! **Receive feed entries:**
//! ```json
//! {
//!   "title": "...",
//!   "url": "https://...",
//!   "published": "2026-07-03T12:00:00Z",
//!   "summary": "...",
//!   "source": "Bloomberg"
//! }
//! ```
//!
//! Source slugs are the same ones accepted by `GET /v2/feeds?sources=...`.

use axum::{
    extract::{
        Extension, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use finance_query::feeds::FeedSource;
use finance_query_server::{AppState, metrics};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{info, warn};

/// Stream command from client
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FeedStreamCommand {
    subscribe: Option<Vec<String>>,
    unsubscribe: Option<Vec<String>>,
    form_type: Option<String>,
}

/// RAII guard to decrement WebSocket connection count on drop
struct ConnectionGuard;

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        metrics::WEBSOCKET_CONNECTIONS.dec();
    }
}

/// WebSocket /v2/feeds/stream
pub(crate) async fn ws_feeds_stream_handler(
    Extension(state): Extension<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    metrics::WEBSOCKET_CONNECTIONS.inc();
    ws.on_upgrade(move |socket| handle_feeds_stream_socket(state, socket))
}

/// Parse requested source slugs into `FeedSource`s.
fn parse_slugs(slugs: &[String], form_type: Option<&str>) -> Result<Vec<FeedSource>, String> {
    finance_query_server::services::feeds::parse_sources(Some(slugs), form_type)
}

fn error_message(msg: &str) -> Message {
    Message::Text(serde_json::json!({"error": msg}).to_string().into())
}

async fn handle_feeds_stream_socket(state: AppState, mut socket: WebSocket) {
    let _guard = ConnectionGuard;
    info!("New feeds streaming WebSocket connection");

    let (out_tx, mut out_rx) = tokio::sync::mpsc::channel::<Message>(32);

    // Wait for initial subscription message.
    let (initial_sources, initial_names) = match wait_for_subscription(&mut socket).await {
        Some(v) => v,
        None => {
            warn!("Feeds WebSocket closed before subscription");
            return;
        }
    };

    info!("Starting feeds stream for sources: {:?}", initial_names);

    state.feed_hub.subscribe_sources(&initial_sources).await;

    let mut hub_stream = match state.feed_hub.resubscribe().await {
        Some(s) => s,
        None => {
            let _ = socket
                .send(Message::Text(
                    serde_json::json!({"error": "feed stream unavailable"})
                        .to_string()
                        .into(),
                ))
                .await;
            return;
        }
    };

    let subscribed_names = Arc::new(tokio::sync::RwLock::new(initial_names));
    let subscribed_sources = Arc::new(tokio::sync::Mutex::new(initial_sources));

    let (mut sender, mut receiver) = socket.split();

    // Forward filtered feed entries + outbound messages to the client.
    let names_for_send = Arc::clone(&subscribed_names);
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = out_rx.recv() => {
                    match msg {
                        Some(msg) => {
                            if sender.send(msg).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }

                maybe_entry = hub_stream.next() => {
                    match maybe_entry {
                        Some(entry) => {
                            let should_send = {
                                let names = names_for_send.read().await;
                                names.contains(&entry.source)
                            };

                            if should_send {
                                let json = serde_json::to_string(&entry).unwrap_or_default();
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

    // Handle incoming subscribe/unsubscribe messages.
    let names_for_recv = Arc::clone(&subscribed_names);
    let sources_for_recv = Arc::clone(&subscribed_sources);
    let feed_hub = state.feed_hub.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(cmd) = serde_json::from_str::<FeedStreamCommand>(&text) {
                        metrics::WEBSOCKET_MESSAGES_RECEIVED.inc();
                        info!("Received feeds stream command: {:?}", cmd);

                        if let Some(slugs) = cmd.subscribe {
                            match parse_slugs(&slugs, cmd.form_type.as_deref()) {
                                Ok(sources) => {
                                    feed_hub.subscribe_sources(&sources).await;
                                    let mut names = names_for_recv.write().await;
                                    let mut all = sources_for_recv.lock().await;
                                    for source in sources {
                                        names.insert(source.name());
                                        all.push(source);
                                    }
                                }
                                Err(msg) => {
                                    let _ = out_tx.send(error_message(&msg)).await;
                                }
                            }
                        }

                        if let Some(slugs) = cmd.unsubscribe {
                            match parse_slugs(&slugs, cmd.form_type.as_deref()) {
                                Ok(sources) => {
                                    let removed_names: HashSet<String> =
                                        sources.iter().map(FeedSource::name).collect();
                                    {
                                        let mut names = names_for_recv.write().await;
                                        names.retain(|n| !removed_names.contains(n));
                                    }
                                    {
                                        let mut all = sources_for_recv.lock().await;
                                        all.retain(|s| !removed_names.contains(&s.name()));
                                    }
                                    feed_hub.unsubscribe_sources(&sources).await;
                                }
                                Err(msg) => {
                                    let _ = out_tx.send(error_message(&msg)).await;
                                }
                            }
                        }
                    }
                }
                Message::Close(_) => {
                    info!("Feeds WebSocket closed by client");
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => info!("Feeds send task completed"),
        _ = &mut recv_task => info!("Feeds receive task completed"),
    }

    send_task.abort();
    recv_task.abort();

    // Release this client's active subscriptions from the global hub.
    let sources_to_release = subscribed_sources.lock().await.clone();
    state
        .feed_hub
        .unsubscribe_sources(&sources_to_release)
        .await;

    info!("Feeds WebSocket stream connection closed");
}

/// Wait for the initial subscription message, reporting parse errors directly
/// on the socket and continuing to wait rather than closing on a bad message.
async fn wait_for_subscription(
    socket: &mut WebSocket,
) -> Option<(Vec<FeedSource>, HashSet<String>)> {
    while let Some(Ok(msg)) = socket.next().await {
        let Message::Text(text) = msg else { continue };
        let Ok(cmd) = serde_json::from_str::<FeedStreamCommand>(&text) else {
            continue;
        };
        let Some(slugs) = cmd.subscribe else { continue };
        let sources = match parse_slugs(&slugs, cmd.form_type.as_deref()) {
            Ok(sources) => sources,
            Err(msg) => {
                let _ = socket.send(error_message(&msg)).await;
                continue;
            }
        };
        let names: HashSet<String> = sources.iter().map(FeedSource::name).collect();
        return Some((sources, names));
    }
    None
}
