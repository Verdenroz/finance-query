//! Polygon/Massive WebSocket streaming for real-time market data.
//!
//! Provides real-time trades, quotes, and aggregate bars for stocks, options, forex,
//! crypto, futures, and indices. Internal adapter; the public API is
//! [`streaming`](crate::streaming), which wraps it in provider-neutral types.
//!
//! # Example
//!
//! ```no_run
//! use finance_query::streaming::PriceStream;
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut stream = PriceStream::subscribe(["AAPL", "NVDA"]).await?;
//! while let Some(update) = stream.next().await {
//!     println!("{:?}", update);
//! }
//! # Ok(())
//! # }
//! ```

use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

use crate::adapters::common::keyed::redact_key;
use crate::error::{FinanceError, Result};

const POLYGON_WEBSOCKET_BASE: &str = "wss://socket.massive.com";
const AUTH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// WebSocket cluster (asset class).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClusterDTO {
    /// Real-time stock data.
    Stocks,
    /// Real-time options data.
    Options,
    /// Real-time forex data.
    Forex,
    /// Real-time crypto data.
    Crypto,
    /// Real-time futures data.
    Futures,
    /// Real-time index data.
    Indices,
}

impl ClusterDTO {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Stocks => "stocks",
            Self::Options => "options",
            Self::Forex => "forex",
            Self::Crypto => "crypto",
            Self::Futures => "futures",
            Self::Indices => "indices",
        }
    }
}

/// A real-time trade message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StreamTrade {
    /// Event type (e.g., `"T"`).
    pub ev: Option<String>,
    /// Symbol. Absent on the crypto cluster, which sends `pair` instead.
    pub sym: Option<String>,
    /// Currency pair (crypto cluster, e.g. `"BTC-USD"`).
    pub pair: Option<String>,
    /// Price.
    pub p: Option<f64>,
    /// Size.
    pub s: Option<f64>,
    /// Exchange ID.
    pub x: Option<i32>,
    /// Conditions.
    pub c: Option<Vec<i32>>,
    /// Timestamp (milliseconds).
    pub t: Option<i64>,
    /// Trade ID (crypto cluster).
    pub i: Option<String>,
}

impl StreamTrade {
    /// Symbol under whichever key this cluster uses.
    pub fn symbol(&self) -> Option<&str> {
        self.sym.as_deref().or(self.pair.as_deref())
    }
}

/// A real-time quote message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StreamQuote {
    /// Event type (e.g., `"Q"`).
    pub ev: Option<String>,
    /// Symbol. Absent on the crypto cluster, which sends `pair` instead.
    pub sym: Option<String>,
    /// Currency pair (crypto cluster, e.g. `"BTC-USD"`).
    pub pair: Option<String>,
    /// Bid price.
    pub bp: Option<f64>,
    /// Bid size.
    pub bs: Option<f64>,
    /// Ask price.
    pub ap: Option<f64>,
    /// Ask size.
    #[serde(rename = "as")]
    pub ask_size: Option<f64>,
    /// Bid exchange.
    pub bx: Option<i32>,
    /// Ask exchange.
    pub ax: Option<i32>,
    /// Conditions.
    pub c: Option<Vec<i32>>,
    /// Timestamp (milliseconds).
    pub t: Option<i64>,
}

impl StreamQuote {
    /// Symbol under whichever key this cluster uses.
    pub fn symbol(&self) -> Option<&str> {
        self.sym.as_deref().or(self.pair.as_deref())
    }
}

/// A real-time aggregate bar message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StreamAggregate {
    /// Event type (e.g., `"A"` per-second, `"AM"` per-minute).
    pub ev: Option<String>,
    /// Symbol. Absent on the crypto/forex clusters, which send `pair` instead.
    pub sym: Option<String>,
    /// Currency pair (crypto/forex clusters, e.g. `"BTC-USD"`, `"USD/EUR"`).
    pub pair: Option<String>,
    /// Open.
    pub o: Option<f64>,
    /// High.
    pub h: Option<f64>,
    /// Low.
    pub l: Option<f64>,
    /// Close.
    pub c: Option<f64>,
    /// Volume.
    pub v: Option<f64>,
    /// VWAP.
    pub vw: Option<f64>,
    /// Start timestamp.
    pub s: Option<i64>,
    /// End timestamp.
    pub e: Option<i64>,
    /// Number of trades.
    pub z: Option<u64>,
}

impl StreamAggregate {
    /// Symbol under whichever key this cluster uses.
    pub fn symbol(&self) -> Option<&str> {
        self.sym.as_deref().or(self.pair.as_deref())
    }
}

/// A real-time forex quote message (`forex` cluster, event `"C"`).
///
/// The forex cluster uses single-letter keys that collide with the stock
/// quote shape (`a`/`b` rather than `ap`/`bp`), so it needs its own type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StreamForexQuote {
    /// Event type (`"C"`).
    pub ev: Option<String>,
    /// Currency pair (e.g., `"USD/EUR"`).
    pub p: Option<String>,
    /// Ask price.
    pub a: Option<f64>,
    /// Bid price.
    pub b: Option<f64>,
    /// Exchange ID.
    pub x: Option<i32>,
    /// Timestamp (milliseconds).
    pub t: Option<i64>,
}

/// One side of an order book: `[price, size]` pairs.
pub type BookSide = Vec<[f64; 2]>;

/// A real-time level-2 order book snapshot (`crypto` cluster, event `"XL2"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StreamLevel2 {
    /// Event type (`"XL2"`).
    pub ev: Option<String>,
    /// Currency pair (e.g., `"BTC-USD"`).
    pub pair: Option<String>,
    /// Bid levels, `[price, size]`, best first.
    pub b: Option<BookSide>,
    /// Ask levels, `[price, size]`, best first.
    pub a: Option<BookSide>,
    /// Exchange ID.
    pub x: Option<i32>,
    /// Timestamp (milliseconds).
    pub t: Option<i64>,
}

/// A real-time index value message (`indices` cluster).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StreamIndexValue {
    /// Event type (e.g., `"V"`).
    pub ev: Option<String>,
    /// Index ticker (e.g., `"I:SPX"`). Sent as `T`, not `sym`, on this cluster.
    #[serde(rename = "T")]
    pub ticker: Option<String>,
    /// Index value.
    pub val: Option<f64>,
    /// Timestamp (milliseconds).
    pub t: Option<i64>,
}

/// A parsed WebSocket message from Polygon.
#[derive(Debug, Clone)]
pub enum PolygonMessage {
    /// Trade event.
    Trade(StreamTrade),
    /// Quote event.
    Quote(StreamQuote),
    /// Aggregate bar (per-second or per-minute).
    Aggregate(StreamAggregate),
    /// Forex quote (`forex` cluster).
    ForexQuote(StreamForexQuote),
    /// Level-2 order book (`crypto` cluster).
    Level2(StreamLevel2),
    /// Index value (`indices` cluster).
    IndexValue(StreamIndexValue),
    /// Status/control message (auth, subscription confirmations).
    Status(serde_json::Value),
    /// Unknown/unparsed message.
    Unknown(String),
}

/// Builder for a Polygon WebSocket stream.
pub struct PolygonStreamBuilder {
    api_key: String,
    cluster: ClusterDTO,
    subscriptions: Vec<String>,
}

impl PolygonStreamBuilder {
    /// Set the cluster (asset class) to connect to.
    pub fn cluster(mut self, cluster: ClusterDTO) -> Self {
        self.cluster = cluster;
        self
    }

    /// Add subscription channels.
    ///
    /// Channel prefixes:
    /// - `T.*` — Trades (e.g., `"T.AAPL"`)
    /// - `Q.*` — Quotes (e.g., `"Q.AAPL"`)
    /// - `A.*` — Per-second aggregates (e.g., `"A.AAPL"`)
    /// - `AM.*` — Per-minute aggregates (e.g., `"AM.AAPL"`)
    /// - `V.*` — Index values, `indices` cluster only (e.g., `"V.I:SPX"`)
    pub fn subscribe(mut self, channels: &[&str]) -> Self {
        self.subscriptions
            .extend(channels.iter().map(|s| s.to_string()));
        self
    }

    /// Connect and return a `PolygonStream`.
    pub async fn build(self) -> Result<PolygonStream> {
        let url = format!("{POLYGON_WEBSOCKET_BASE}/{}", self.cluster.as_str());

        let (ws_stream, _) = tokio_tungstenite::connect_async(&url)
            .await
            .map_err(|e| FinanceError::ApiError(format!("Polygon WebSocket connect error: {e}")))?;

        let (write, mut read) = futures::StreamExt::split(ws_stream);
        let write = std::sync::Arc::new(tokio::sync::Mutex::new(write));

        // Auth
        {
            use futures::SinkExt;
            let auth_msg = serde_json::json!({
                "action": "auth",
                "params": self.api_key
            });
            write
                .lock()
                .await
                .send(Message::Text(auth_msg.to_string().into()))
                .await
                .map_err(|e| {
                    FinanceError::ApiError(format!("Polygon WebSocket auth error: {e}"))
                })?;
        }

        wait_for_authentication(&mut read, &self.api_key).await?;

        // Subscribe
        if !self.subscriptions.is_empty() {
            use futures::SinkExt;
            let sub_msg = serde_json::json!({
                "action": "subscribe",
                "params": self.subscriptions.join(",")
            });
            write
                .lock()
                .await
                .send(Message::Text(sub_msg.to_string().into()))
                .await
                .map_err(|e| {
                    FinanceError::ApiError(format!("Polygon WebSocket subscribe error: {e}"))
                })?;
        }

        Ok(PolygonStream {
            read: Box::pin(read),
            write,
            pending: std::collections::VecDeque::new(),
        })
    }
}

/// A real-time Polygon WebSocket stream.
///
/// Implements `futures::Stream<Item = PolygonMessage>`.
pub struct PolygonStream {
    read: Pin<
        Box<
            dyn Stream<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
                + Send,
        >,
    >,
    write: SharedSink,
    // One frame carries many events; hold the tail so none are dropped.
    pending: std::collections::VecDeque<PolygonMessage>,
}

type SharedSink = std::sync::Arc<
    tokio::sync::Mutex<
        futures::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
    >,
>;

/// Send half of a [`PolygonStream`], detachable so a session can change its
/// subscriptions while the read half is borrowed by a `select!` arm.
#[derive(Clone)]
pub struct PolygonSender {
    write: SharedSink,
}

impl PolygonSender {
    /// Subscribe to additional channels on the live connection.
    pub async fn subscribe_channels(&self, channels: &[String]) -> Result<()> {
        self.send_action("subscribe", channels).await
    }

    /// Unsubscribe from channels on the live connection.
    pub async fn unsubscribe_channels(&self, channels: &[String]) -> Result<()> {
        self.send_action("unsubscribe", channels).await
    }

    async fn send_action(&self, action: &str, channels: &[String]) -> Result<()> {
        use futures::SinkExt;

        if channels.is_empty() {
            return Ok(());
        }
        let msg = serde_json::json!({ "action": action, "params": channels.join(",") });
        self.write
            .lock()
            .await
            .send(Message::Text(msg.to_string().into()))
            .await
            .map_err(|e| FinanceError::ApiError(format!("Polygon WebSocket {action} error: {e}")))
    }
}

impl PolygonStream {
    /// Create a stream builder using an explicit API key.
    pub fn builder(api_key: impl Into<String>) -> Result<PolygonStreamBuilder> {
        let api_key = api_key.into();
        if api_key.trim().is_empty() {
            return Err(FinanceError::InvalidParameter {
                param: "polygon".to_string(),
                reason: "API key must not be empty".to_string(),
            });
        }
        Ok(PolygonStreamBuilder {
            api_key,
            cluster: ClusterDTO::Stocks,
            subscriptions: Vec::new(),
        })
    }

    /// Create a new builder for a Polygon WebSocket stream.
    ///
    /// Requires [`crate::polygon::init`] to have been called first.
    pub fn from_singleton() -> Result<PolygonStreamBuilder> {
        Self::builder(super::api_key()?)
    }

    /// Detach the send half so subscriptions can change mid-session.
    pub fn sender(&self) -> PolygonSender {
        PolygonSender {
            write: std::sync::Arc::clone(&self.write),
        }
    }
}

async fn wait_for_authentication<S>(read: &mut S, api_key: &str) -> Result<()>
where
    S: Stream<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    tokio::time::timeout(AUTH_TIMEOUT, async {
        while let Some(frame) = futures::StreamExt::next(read).await {
            let frame = frame.map_err(|error| {
                FinanceError::ApiError(format!("Polygon WebSocket auth error: {error}"))
            })?;
            let Message::Text(text) = frame else {
                continue;
            };
            let events: Vec<serde_json::Value> = serde_json::from_str(&text).map_err(|error| {
                FinanceError::ResponseStructureError {
                    field: "polygon.websocket.auth".to_string(),
                    context: format!("Invalid authentication response: {error}"),
                }
            })?;
            for event in events {
                if event.get("ev").and_then(|value| value.as_str()) != Some("status") {
                    continue;
                }
                let status = event
                    .get("status")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default();
                let message = event
                    .get("message")
                    .and_then(|value| value.as_str())
                    .unwrap_or(status);
                if status == "auth_success" {
                    return Ok(());
                }
                if status == "auth_failed" || status == "not_authorized" {
                    return Err(FinanceError::AuthenticationFailed {
                        context: redact_key(message, api_key),
                    });
                }
            }
        }
        Err(FinanceError::ApiError(
            "Polygon WebSocket closed before authentication completed".to_string(),
        ))
    })
    .await
    .map_err(|_| FinanceError::AuthenticationFailed {
        context: "Polygon WebSocket authentication timed out".to_string(),
    })?
}

impl Stream for PolygonStream {
    type Item = PolygonMessage;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(msg) = self.pending.pop_front() {
                return Poll::Ready(Some(msg));
            }
            match self.read.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(Message::Text(text)))) => {
                    self.pending.extend(parse_messages(&text));
                }
                Poll::Ready(Some(Ok(Message::Close(_)))) | Poll::Ready(None) => {
                    return Poll::Ready(None);
                }
                Poll::Ready(Some(Ok(_))) => continue, // skip ping/pong/binary
                Poll::Ready(Some(Err(_))) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Parse every event in one Polygon frame.
///
/// Frames carry an array of events; tick-by-tick consumers need all of them,
/// so nothing is collapsed to a single message here.
pub(crate) fn parse_messages(text: &str) -> Vec<PolygonMessage> {
    let events: Vec<serde_json::Value> = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return vec![PolygonMessage::Unknown(text.to_string())],
    };

    let mut out = Vec::with_capacity(events.len());
    for event in events {
        let ev = event.get("ev").and_then(|v| v.as_str()).unwrap_or("");
        let parsed = match ev {
            "T" | "XT" => serde_json::from_value(event)
                .ok()
                .map(PolygonMessage::Trade),
            "Q" | "XQ" => serde_json::from_value(event)
                .ok()
                .map(PolygonMessage::Quote),
            "A" | "AM" | "XA" | "XAM" | "CA" | "CAS" => serde_json::from_value(event)
                .ok()
                .map(PolygonMessage::Aggregate),
            "C" => serde_json::from_value(event)
                .ok()
                .map(PolygonMessage::ForexQuote),
            "XL2" => serde_json::from_value(event)
                .ok()
                .map(PolygonMessage::Level2),
            "V" => serde_json::from_value(event)
                .ok()
                .map(PolygonMessage::IndexValue),
            "status" => Some(PolygonMessage::Status(event)),
            _ => None,
        };
        if let Some(msg) = parsed {
            out.push(msg);
        }
    }

    if out.is_empty() {
        out.push(PolygonMessage::Unknown(text.to_string()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// First parsed event of a frame — most cases here send a single event.
    fn first(text: &str) -> PolygonMessage {
        parse_messages(text).into_iter().next().expect("no events")
    }

    #[test]
    fn test_parse_trade_message() {
        let msg =
            r#"[{"ev":"T","sym":"AAPL","p":186.19,"s":100,"x":4,"c":[12,37],"t":1705363200000}]"#;
        match first(msg) {
            PolygonMessage::Trade(t) => {
                assert_eq!(t.sym.as_deref(), Some("AAPL"));
                assert!((t.p.unwrap() - 186.19).abs() < 0.01);
                assert_eq!(t.s.unwrap() as u64, 100);
            }
            other => panic!("Expected Trade, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_quote_message() {
        let msg = r#"[{"ev":"Q","sym":"AAPL","bp":186.18,"bs":2,"ap":186.25,"as":3,"bx":19,"ax":11,"t":1705363200000}]"#;
        match first(msg) {
            PolygonMessage::Quote(q) => {
                assert_eq!(q.sym.as_deref(), Some("AAPL"));
                assert!((q.bp.unwrap() - 186.18).abs() < 0.01);
                assert!((q.ap.unwrap() - 186.25).abs() < 0.01);
            }
            other => panic!("Expected Quote, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_aggregate_message() {
        let msg = r#"[{"ev":"AM","sym":"AAPL","o":186.0,"h":186.25,"l":185.90,"c":186.19,"v":1500000,"vw":186.05,"s":1705363200000,"e":1705363260000,"z":823}]"#;
        match first(msg) {
            PolygonMessage::Aggregate(a) => {
                assert_eq!(a.sym.as_deref(), Some("AAPL"));
                assert!((a.c.unwrap() - 186.19).abs() < 0.01);
                assert_eq!(a.ev.as_deref(), Some("AM"));
            }
            other => panic!("Expected Aggregate, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_index_value_message() {
        let msg = r#"[{"ev":"V","val":3988.5,"T":"I:SPX","t":1678220098130}]"#;
        match first(msg) {
            PolygonMessage::IndexValue(v) => {
                assert_eq!(v.ev.as_deref(), Some("V"));
                assert_eq!(v.ticker.as_deref(), Some("I:SPX"));
                assert!((v.val.unwrap() - 3988.5).abs() < 0.01);
                assert_eq!(v.t, Some(1678220098130));
            }
            other => panic!("Expected IndexValue, got {:?}", other),
        }
    }

    #[test]
    fn test_index_value_not_dropped_as_unknown() {
        let msg = r#"[{"ev":"V","val":3988.5,"T":"I:SPX","t":1678220098130}]"#;
        assert!(!matches!(first(msg), PolygonMessage::Unknown(_)));
    }

    #[test]
    fn test_parse_status_message() {
        let msg = r#"[{"ev":"status","status":"auth_success","message":"authenticated"}]"#;
        match first(msg) {
            PolygonMessage::Status(v) => {
                assert_eq!(v.get("status").unwrap().as_str().unwrap(), "auth_success");
            }
            other => panic!("Expected Status, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_unknown_message() {
        let msg = "not json at all";
        assert!(matches!(first(msg), PolygonMessage::Unknown(_)));
    }

    #[test]
    fn test_cluster_as_str() {
        assert_eq!(ClusterDTO::Stocks.as_str(), "stocks");
        assert_eq!(ClusterDTO::Options.as_str(), "options");
        assert_eq!(ClusterDTO::Crypto.as_str(), "crypto");
        assert_eq!(ClusterDTO::Futures.as_str(), "futures");
        assert_eq!(ClusterDTO::Indices.as_str(), "indices");
    }

    #[test]
    fn explicit_builder_rejects_an_empty_key() {
        assert!(matches!(
            PolygonStream::builder("  "),
            Err(FinanceError::InvalidParameter { .. })
        ));
    }

    #[tokio::test]
    async fn authentication_waits_for_auth_success() {
        let frames = vec![
            Ok(Message::Text(
                r#"[{"ev":"status","status":"connected"}]"#.into(),
            )),
            Ok(Message::Text(
                r#"[{"ev":"status","status":"auth_success","message":"authenticated"}]"#.into(),
            )),
        ];
        let mut stream = futures::stream::iter(frames);
        wait_for_authentication(&mut stream, "test-key")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn authentication_failure_is_typed() {
        let frames = vec![Ok(Message::Text(
            r#"[{"ev":"status","status":"auth_failed","message":"invalid key"}]"#.into(),
        ))];
        let mut stream = futures::stream::iter(frames);
        assert!(matches!(
            wait_for_authentication(&mut stream, "test-key").await,
            Err(FinanceError::AuthenticationFailed { .. })
        ));
    }

    #[tokio::test]
    async fn authentication_failure_redacts_the_api_key() {
        const KEY: &str = "abc123";
        let frames = vec![Ok(Message::Text(
            format!(
                r#"[{{"ev":"status","status":"auth_failed","message":"key {KEY} is invalid"}}]"#
            )
            .into(),
        ))];
        let mut stream = futures::stream::iter(frames);
        let err = wait_for_authentication(&mut stream, KEY).await.unwrap_err();
        assert!(!format!("{err}").contains(KEY), "{err}");
    }
}
