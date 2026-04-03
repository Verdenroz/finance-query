//! Polygon.io WebSocket streaming for real-time market data.
//!
//! Provides real-time trades, quotes, and aggregate bars for stocks, options, forex,
//! crypto, futures, and indices.
//!
//! # Example
//!
//! ```no_run
//! use finance_query::adapters::polygon::websocket::*;
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut stream = PolygonStream::builder("YOUR_API_KEY")
//!     .cluster(Cluster::Stocks)
//!     .subscribe(&["T.AAPL", "Q.AAPL", "AM.AAPL"])
//!     .build()
//!     .await?;
//!
//! while let Some(msg) = stream.next().await {
//!     println!("{:?}", msg);
//! }
//! # Ok(())
//! # }
//! ```

use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

use crate::error::{FinanceError, Result};

/// WebSocket cluster (asset class).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cluster {
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

impl Cluster {
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
    /// Symbol.
    pub sym: Option<String>,
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
}

/// A real-time quote message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StreamQuote {
    /// Event type (e.g., `"Q"`).
    pub ev: Option<String>,
    /// Symbol.
    pub sym: Option<String>,
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

/// A real-time aggregate bar message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StreamAggregate {
    /// Event type (e.g., `"A"` per-second, `"AM"` per-minute).
    pub ev: Option<String>,
    /// Symbol.
    pub sym: Option<String>,
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

/// A parsed WebSocket message from Polygon.
#[derive(Debug, Clone)]
pub enum PolygonMessage {
    /// Trade event.
    Trade(StreamTrade),
    /// Quote event.
    Quote(StreamQuote),
    /// Aggregate bar (per-second or per-minute).
    Aggregate(StreamAggregate),
    /// Status/control message (auth, subscription confirmations).
    Status(serde_json::Value),
    /// Unknown/unparsed message.
    Unknown(String),
}

/// Builder for a Polygon WebSocket stream.
pub struct PolygonStreamBuilder {
    api_key: String,
    cluster: Cluster,
    subscriptions: Vec<String>,
}

impl PolygonStreamBuilder {
    /// Set the cluster (asset class) to connect to.
    pub fn cluster(mut self, cluster: Cluster) -> Self {
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
    pub fn subscribe(mut self, channels: &[&str]) -> Self {
        self.subscriptions
            .extend(channels.iter().map(|s| s.to_string()));
        self
    }

    /// Connect and return a `PolygonStream`.
    pub async fn build(self) -> Result<PolygonStream> {
        let url = format!("wss://socket.polygon.io/{}", self.cluster.as_str());

        let (ws_stream, _) = tokio_tungstenite::connect_async(&url).await.map_err(|_e| {
            FinanceError::ExternalApiError {
                api: "Polygon WebSocket".to_string(),
                status: 0,
            }
        })?;

        let (write, read) = futures::StreamExt::split(ws_stream);
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
                .map_err(|_e| FinanceError::ExternalApiError {
                    api: "Polygon WebSocket".to_string(),
                    status: 0,
                })?;
        }

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
                .map_err(|_e| FinanceError::ExternalApiError {
                    api: "Polygon WebSocket".to_string(),
                    status: 0,
                })?;
        }

        Ok(PolygonStream {
            read: Box::pin(read),
            _write: write,
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
    _write: std::sync::Arc<
        tokio::sync::Mutex<
            futures::stream::SplitSink<
                tokio_tungstenite::WebSocketStream<
                    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
                >,
                Message,
            >,
        >,
    >,
}

impl PolygonStream {
    /// Create a new builder for a Polygon WebSocket stream.
    pub fn builder(api_key: impl Into<String>) -> PolygonStreamBuilder {
        PolygonStreamBuilder {
            api_key: api_key.into(),
            cluster: Cluster::Stocks,
            subscriptions: Vec::new(),
        }
    }
}

impl Stream for PolygonStream {
    type Item = PolygonMessage;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.read.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(Message::Text(text)))) => {
                    return Poll::Ready(Some(parse_message(&text)));
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

fn parse_message(text: &str) -> PolygonMessage {
    // Polygon sends arrays of events
    let events: Vec<serde_json::Value> = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return PolygonMessage::Unknown(text.to_string()),
    };

    // Return the first meaningful event
    for event in events {
        let ev = event.get("ev").and_then(|v| v.as_str()).unwrap_or("");
        match ev {
            "T" | "XT" => {
                if let Ok(trade) = serde_json::from_value(event) {
                    return PolygonMessage::Trade(trade);
                }
            }
            "Q" | "XQ" => {
                if let Ok(quote) = serde_json::from_value(event) {
                    return PolygonMessage::Quote(quote);
                }
            }
            "A" | "AM" | "XA" | "XAM" => {
                if let Ok(agg) = serde_json::from_value(event) {
                    return PolygonMessage::Aggregate(agg);
                }
            }
            "status" => {
                return PolygonMessage::Status(event);
            }
            _ => {}
        }
    }

    PolygonMessage::Unknown(text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_trade_message() {
        let msg =
            r#"[{"ev":"T","sym":"AAPL","p":186.19,"s":100,"x":4,"c":[12,37],"t":1705363200000}]"#;
        match parse_message(msg) {
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
        match parse_message(msg) {
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
        match parse_message(msg) {
            PolygonMessage::Aggregate(a) => {
                assert_eq!(a.sym.as_deref(), Some("AAPL"));
                assert!((a.c.unwrap() - 186.19).abs() < 0.01);
                assert_eq!(a.ev.as_deref(), Some("AM"));
            }
            other => panic!("Expected Aggregate, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_status_message() {
        let msg = r#"[{"ev":"status","status":"auth_success","message":"authenticated"}]"#;
        match parse_message(msg) {
            PolygonMessage::Status(v) => {
                assert_eq!(v.get("status").unwrap().as_str().unwrap(), "auth_success");
            }
            other => panic!("Expected Status, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_unknown_message() {
        let msg = "not json at all";
        assert!(matches!(parse_message(msg), PolygonMessage::Unknown(_)));
    }

    #[test]
    fn test_cluster_as_str() {
        assert_eq!(Cluster::Stocks.as_str(), "stocks");
        assert_eq!(Cluster::Options.as_str(), "options");
        assert_eq!(Cluster::Crypto.as_str(), "crypto");
        assert_eq!(Cluster::Futures.as_str(), "futures");
        assert_eq!(Cluster::Indices.as_str(), "indices");
    }
}
