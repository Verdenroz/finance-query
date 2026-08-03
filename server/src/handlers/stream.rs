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
//! **Subscribe to threshold alerts instead of every tick:**
//! ```json
//! {"alerts": [{"symbol": "AAPL", "condition": "crossesAbove", "value": 200.0}]}
//! ```
//! Conditions: `crossesAbove`, `crossesBelow`, `priceAbove`, `priceBelow`,
//! `percentChangeAbove`, `percentChangeBelow`, `volumeAbove`. Add
//! `"repeat": true` to re-fire whenever the condition becomes true again.
//! While a rule set is active the socket emits `AlertEvent` objects rather
//! than raw ticks; send `{"alerts": []}` to go back to raw ticks.
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
use finance_query::streaming::{AlertCondition, AlertEvaluator, AlertRule};
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
    alerts: Option<Vec<WsAlertRule>>,
}

/// One alert rule as sent over the wire.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WsAlertRule {
    symbol: String,
    condition: String,
    value: f64,
    #[serde(default)]
    repeat: bool,
}

impl WsAlertRule {
    fn parse(&self) -> Result<AlertRule, String> {
        let condition = match self.condition.as_str() {
            "crossesAbove" => AlertCondition::CrossesAbove(self.value),
            "crossesBelow" => AlertCondition::CrossesBelow(self.value),
            "priceAbove" => AlertCondition::PriceAbove(self.value),
            "priceBelow" => AlertCondition::PriceBelow(self.value),
            "percentChangeAbove" => AlertCondition::PercentChangeAbove(self.value),
            "percentChangeBelow" => AlertCondition::PercentChangeBelow(self.value),
            "volumeAbove" => AlertCondition::VolumeAbove(self.value as i64),
            other => return Err(format!("unknown alert condition: {other}")),
        };
        let rule = AlertRule::new(self.symbol.clone(), condition);
        Ok(if self.repeat { rule.repeating() } else { rule })
    }
}

/// Turn wire rules into library rules, reporting the first bad condition.
fn parse_rules(rules: &[WsAlertRule]) -> Result<Vec<AlertRule>, String> {
    rules.iter().map(WsAlertRule::parse).collect()
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

    // Wait for the initial subscribe/alerts message
    let (symbols, initial_rules) = match wait_for_subscription(&mut socket).await {
        Some(start) => {
            metrics::WEBSOCKET_MESSAGES_RECEIVED.inc();
            start
        }
        None => {
            warn!("WebSocket closed before subscription");
            return;
        }
    };

    let evaluator = Arc::new(tokio::sync::Mutex::new(
        initial_rules.map(AlertEvaluator::new),
    ));

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
    let evaluator_for_send = Arc::clone(&evaluator);
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
                            if !should_send {
                                continue;
                            }

                            // Evaluate under the lock, send outside it.
                            let alerts = {
                                let mut guard = evaluator_for_send.lock().await;
                                guard.as_mut().map(|e| e.evaluate(&price))
                            };

                            let payloads: Vec<String> = match alerts {
                                Some(events) => events
                                    .iter()
                                    .map(|e| serde_json::to_string(e).unwrap_or_default())
                                    .collect(),
                                None => vec![serde_json::to_string(&price).unwrap_or_default()],
                            };

                            for payload in payloads {
                                if sender.send(Message::Text(payload.into())).await.is_err() {
                                    return;
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
    let evaluator_for_recv = Arc::clone(&evaluator);
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

                        if let Some(rules) = cmd.alerts {
                            match parse_rules(&rules) {
                                Ok(parsed) => {
                                    let extra: Vec<String> = parsed
                                        .iter()
                                        .map(|r| r.symbol.clone())
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                    if !extra.is_empty()
                                        && let Err(e) = stream_hub.subscribe_symbols(&extra).await
                                    {
                                        error!("Failed to subscribe alert symbols: {}", e);
                                    } else {
                                        let mut subs = subscriptions_for_recv.write().await;
                                        subs.extend(extra);
                                    }
                                    // An empty rule set reverts to raw ticks.
                                    let mut guard = evaluator_for_recv.lock().await;
                                    *guard =
                                        (!parsed.is_empty()).then(|| AlertEvaluator::new(parsed));
                                }
                                Err(message) => {
                                    let _ = out_tx
                                        .send(Message::Text(
                                            serde_json::json!({"error": message})
                                                .to_string()
                                                .into(),
                                        ))
                                        .await;
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

/// Wait for the first message carrying symbols and/or alert rules.
///
/// Returns the symbols to subscribe (the union of `subscribe` and the rules'
/// own symbols, so a client need not list alert symbols twice) plus the
/// parsed rules, if any.
async fn wait_for_subscription(
    socket: &mut WebSocket,
) -> Option<(Vec<String>, Option<Vec<AlertRule>>)> {
    while let Some(Ok(msg)) = socket.next().await {
        let Message::Text(text) = msg else {
            continue;
        };
        let Ok(cmd) = serde_json::from_str::<StreamCommand>(&text) else {
            continue;
        };

        let rules = match cmd.alerts.as_deref().map(parse_rules) {
            Some(Ok(parsed)) if !parsed.is_empty() => Some(parsed),
            Some(Err(message)) => {
                let _ = socket
                    .send(Message::Text(
                        serde_json::json!({"error": message}).to_string().into(),
                    ))
                    .await;
                continue;
            }
            _ => None,
        };

        let mut symbols = cmd.subscribe.unwrap_or_default();
        if let Some(rules) = rules.as_ref() {
            for rule in rules {
                if !symbols.contains(&rule.symbol) {
                    symbols.push(rule.symbol.clone());
                }
            }
        }

        if !symbols.is_empty() {
            return Some((symbols, rules));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command(json: &str) -> StreamCommand {
        serde_json::from_str(json).expect("command should deserialize")
    }

    #[test]
    fn alert_commands_parse_into_library_rules() {
        let cmd = command(
            r#"{"alerts":[{"symbol":"AAPL","condition":"crossesAbove","value":200.0},
                          {"symbol":"NVDA","condition":"volumeAbove","value":1000,"repeat":true}]}"#,
        );
        let rules = parse_rules(&cmd.alerts.unwrap()).expect("rules should parse");

        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].condition, AlertCondition::CrossesAbove(200.0));
        assert!(!rules[0].repeat);
        assert_eq!(rules[1].condition, AlertCondition::VolumeAbove(1000));
        assert!(rules[1].repeat);
    }

    #[test]
    fn an_unknown_condition_is_reported_not_ignored() {
        let cmd = command(r#"{"alerts":[{"symbol":"AAPL","condition":"wat","value":1.0}]}"#);
        let err = parse_rules(&cmd.alerts.unwrap()).expect_err("should reject");
        assert!(err.contains("wat"), "error should name the bad condition");
    }

    #[test]
    fn plain_subscribe_commands_carry_no_alerts() {
        let cmd = command(r#"{"subscribe":["AAPL"]}"#);
        assert!(cmd.alerts.is_none());
        assert_eq!(cmd.subscribe.unwrap(), vec!["AAPL"]);
    }

    #[test]
    fn alert_rules_drive_an_evaluator_end_to_end() {
        let cmd =
            command(r#"{"alerts":[{"symbol":"AAPL","condition":"crossesAbove","value":150}]}"#);
        let rules = parse_rules(&cmd.alerts.unwrap()).unwrap();
        let mut evaluator = AlertEvaluator::new(rules);

        let below = finance_query::streaming::PriceUpdate {
            id: "AAPL".into(),
            price: 149.0,
            ..Default::default()
        };
        let above = finance_query::streaming::PriceUpdate {
            id: "AAPL".into(),
            price: 151.0,
            ..Default::default()
        };

        assert!(evaluator.evaluate(&below).is_empty());
        assert_eq!(evaluator.evaluate(&above).len(), 1);
    }
}
