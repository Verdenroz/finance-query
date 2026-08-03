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
use finance_query::FinanceError;
use finance_query::streaming::{AlertConditionKind, AlertEvaluator, AlertEvent, AlertRule};
use finance_query_server::{AppState, SharedTick, StreamHub, TickStream, metrics};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::HashSet;
use std::fmt::Display;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Outbound control-message channel depth.
const OUTBOUND_CAPACITY: usize = 32;

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
        let kind: AlertConditionKind = self
            .condition
            .parse()
            .map_err(|e: FinanceError| e.to_string())?;
        let rule = AlertRule::new(self.symbol.clone(), kind.with_value(self.value));
        Ok(if self.repeat { rule.repeating() } else { rule })
    }
}

/// Turn wire rules into library rules, reporting the first bad condition.
fn parse_rules(rules: &[WsAlertRule]) -> Result<Vec<AlertRule>, String> {
    rules.iter().map(WsAlertRule::parse).collect()
}

/// One `{"error": ...}` frame.
fn error_msg(e: impl Display) -> Message {
    Message::Text(
        serde_json::json!({ "error": e.to_string() })
            .to_string()
            .into(),
    )
}

/// Per-connection state shared by this client's send and receive tasks.
///
/// The locks are `std`, not `tokio`: no guard is ever held across an `await`,
/// so an async lock would add waker bookkeeping to the per-tick delivery path
/// for nothing.
struct ClientState {
    subscriptions: RwLock<HashSet<String>>,
    evaluator: Mutex<Option<AlertEvaluator>>,
    /// Mirrors `evaluator.is_some()` so delivery skips the lock entirely for
    /// the common case of a client with no rules.
    has_rules: AtomicBool,
}

impl ClientState {
    fn new(symbols: &[String], rules: Option<Vec<AlertRule>>) -> Self {
        Self {
            subscriptions: RwLock::new(symbols.iter().cloned().collect()),
            has_rules: AtomicBool::new(rules.is_some()),
            evaluator: Mutex::new(rules.map(AlertEvaluator::new)),
        }
    }

    fn wants(&self, symbol: &str) -> bool {
        self.subscriptions
            .read()
            .expect("subscription lock poisoned")
            .contains(symbol)
    }

    fn len(&self) -> usize {
        self.subscriptions
            .read()
            .expect("subscription lock poisoned")
            .len()
    }

    fn snapshot(&self) -> Vec<String> {
        self.subscriptions
            .read()
            .expect("subscription lock poisoned")
            .iter()
            .cloned()
            .collect()
    }

    /// Add `symbols`, returning only those not already subscribed.
    fn add(&self, symbols: Vec<String>) -> Vec<String> {
        let mut subs = self
            .subscriptions
            .write()
            .expect("subscription lock poisoned");
        symbols
            .into_iter()
            .filter(|s| subs.insert(s.clone()))
            .collect()
    }

    /// Remove `symbols`, returning only those that were actually subscribed.
    fn remove(&self, symbols: Vec<String>) -> Vec<String> {
        let mut subs = self
            .subscriptions
            .write()
            .expect("subscription lock poisoned");
        symbols.into_iter().filter(|s| subs.remove(s)).collect()
    }

    fn extend(&self, symbols: Vec<String>) {
        self.subscriptions
            .write()
            .expect("subscription lock poisoned")
            .extend(symbols);
    }

    /// Install a rule set; an empty one reverts the socket to raw ticks.
    fn set_rules(&self, rules: Vec<AlertRule>) {
        let mut guard = self.evaluator.lock().expect("evaluator lock poisoned");
        *guard = (!rules.is_empty()).then(|| AlertEvaluator::new(rules));
        // Flagged under the lock, so a reader that sees `true` finds the rules.
        self.has_rules.store(guard.is_some(), Ordering::Release);
    }

    /// Alerts this tick fired, or `None` when the client wants raw ticks.
    fn evaluate(&self, tick: &SharedTick) -> Option<Vec<AlertEvent>> {
        if !self.has_rules.load(Ordering::Acquire) {
            return None;
        }
        self.evaluator
            .lock()
            .expect("evaluator lock poisoned")
            .as_mut()
            .map(|e| e.evaluate(tick.update()))
    }
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

    let Some((symbols, initial_rules)) = wait_for_subscription(&mut socket).await else {
        warn!("WebSocket closed before subscription");
        return;
    };
    metrics::WEBSOCKET_MESSAGES_RECEIVED.inc();
    info!("Starting stream for symbols: {:?}", symbols);
    metrics::WEBSOCKET_SYMBOLS_SUBSCRIBED.set(symbols.len() as f64);

    let Some(hub_stream) = open_hub_stream(&state, &symbols, &mut socket).await else {
        return;
    };

    let client = Arc::new(ClientState::new(&symbols, initial_rules));
    let (out_tx, out_rx) = mpsc::channel::<Message>(OUTBOUND_CAPACITY);
    let (sender, receiver) = socket.split();

    let mut send_task = tokio::spawn(run_send_task(
        sender,
        hub_stream,
        out_rx,
        Arc::clone(&client),
    ));
    let mut recv_task = tokio::spawn(run_recv_task(
        receiver,
        Arc::clone(&client),
        state.stream_hub.clone(),
        out_tx,
    ));

    // Wait for either task to complete, then ensure per-client resources are torn down.
    tokio::select! {
        _ = &mut send_task => info!("Send task completed"),
        _ = &mut recv_task => info!("Receive task completed"),
    }
    send_task.abort();
    recv_task.abort();

    // Release this client's active subscriptions from the global hub.
    state
        .stream_hub
        .unsubscribe_symbols(&client.snapshot())
        .await;
    info!("WebSocket stream connection closed");
}

/// Ref-counted subscribe plus a receiver on the shared hub, reporting failure
/// to the client rather than closing silently.
async fn open_hub_stream(
    state: &AppState,
    symbols: &[String],
    socket: &mut WebSocket,
) -> Option<TickStream> {
    if let Err(e) = state.stream_hub.subscribe_symbols(symbols).await {
        error!("Failed to create shared price stream: {}", e);
        let _ = socket.send(error_msg(e)).await;
        return None;
    }
    match state.stream_hub.resubscribe().await {
        Some(stream) => Some(stream),
        None => {
            let _ = socket.send(error_msg("stream unavailable")).await;
            None
        }
    }
}

/// Forward filtered price updates and outbound control messages to the client.
async fn run_send_task(
    mut sender: SplitSink<WebSocket, Message>,
    mut hub_stream: TickStream,
    mut out_rx: mpsc::Receiver<Message>,
    client: Arc<ClientState>,
) {
    loop {
        tokio::select! {
            msg = out_rx.recv() => {
                // Control channel closed.
                let Some(msg) = msg else { break };
                if sender.send(msg).await.is_err() {
                    break;
                }
            }

            maybe_tick = hub_stream.next() => {
                let Some(tick) = maybe_tick else { break };
                if !client.wants(&tick.update().id) {
                    continue;
                }
                if !deliver(&mut sender, &client, &tick).await {
                    break;
                }
            }
        }
    }
}

/// Send one tick (or the alerts it fired); `false` once the socket is gone.
///
/// Raw ticks reuse the hub's shared JSON — alert payloads are per-client by
/// nature, but rare enough that serializing them here costs nothing.
async fn deliver(
    sender: &mut SplitSink<WebSocket, Message>,
    client: &ClientState,
    tick: &SharedTick,
) -> bool {
    let payloads: Vec<Arc<str>> = match client.evaluate(tick) {
        Some(events) => events
            .iter()
            .map(|e| Arc::from(serde_json::to_string(e).unwrap_or_default()))
            .collect(),
        None => vec![tick.json()],
    };

    for payload in payloads {
        if sender
            .send(Message::Text(payload.as_ref().into()))
            .await
            .is_err()
        {
            return false;
        }
        metrics::WEBSOCKET_MESSAGES_SENT.inc();
    }
    true
}

/// Handle incoming subscribe/unsubscribe/alerts commands.
async fn run_recv_task(
    mut receiver: SplitStream<WebSocket>,
    client: Arc<ClientState>,
    hub: StreamHub,
    out_tx: mpsc::Sender<Message>,
) {
    while let Some(Ok(msg)) = receiver.next().await {
        if matches!(msg, Message::Close(_)) {
            info!("WebSocket closed by client");
            break;
        }
        let Message::Text(text) = msg else {
            continue;
        };
        let Ok(cmd) = serde_json::from_str::<StreamCommand>(&text) else {
            continue;
        };
        metrics::WEBSOCKET_MESSAGES_RECEIVED.inc();
        info!("Received stream command: {:?}", cmd);

        if let Some(symbols) = cmd.subscribe {
            apply_subscribe(symbols, &client, &hub, &out_tx).await;
        }
        if let Some(rules) = cmd.alerts {
            apply_alerts(&rules, &client, &hub, &out_tx).await;
        }
        if let Some(symbols) = cmd.unsubscribe {
            apply_unsubscribe(symbols, &client, &hub).await;
        }
    }
}

async fn apply_subscribe(
    symbols: Vec<String>,
    client: &ClientState,
    hub: &StreamHub,
    out_tx: &mpsc::Sender<Message>,
) {
    let added = client.add(symbols);
    if added.is_empty() {
        return;
    }
    if let Err(e) = hub.subscribe_symbols(&added).await {
        error!("Failed to subscribe symbols: {}", e);
        client.remove(added);
        let _ = out_tx.send(error_msg(e)).await;
        return;
    }
    metrics::WEBSOCKET_SYMBOLS_SUBSCRIBED.set(client.len() as f64);
}

async fn apply_alerts(
    rules: &[WsAlertRule],
    client: &ClientState,
    hub: &StreamHub,
    out_tx: &mpsc::Sender<Message>,
) {
    let parsed = match parse_rules(rules) {
        Ok(parsed) => parsed,
        Err(message) => {
            let _ = out_tx.send(error_msg(message)).await;
            return;
        }
    };

    // Alert symbols need their own upstream subscription — a client may alert
    // on a symbol it never asked for ticks on.
    let extra: Vec<String> = parsed
        .iter()
        .map(|r| r.symbol.clone())
        .filter(|s| !s.is_empty())
        .collect();
    if !extra.is_empty() {
        match hub.subscribe_symbols(&extra).await {
            Ok(()) => client.extend(extra),
            Err(e) => error!("Failed to subscribe alert symbols: {}", e),
        }
    }
    client.set_rules(parsed);
}

async fn apply_unsubscribe(symbols: Vec<String>, client: &ClientState, hub: &StreamHub) {
    let removed = client.remove(symbols);
    if removed.is_empty() {
        return;
    }
    hub.unsubscribe_symbols(&removed).await;
    metrics::WEBSOCKET_SYMBOLS_SUBSCRIBED.set(client.len() as f64);
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
                let _ = socket.send(error_msg(message)).await;
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
    use finance_query::streaming::AlertCondition;

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
