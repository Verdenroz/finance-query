//! Polygon.io-backed streaming sources.
//!
//! Polygon runs one WebSocket cluster per asset class, so each class gets a
//! purpose-built source instead of riding a single generic feed. Everything
//! here reuses the existing [`PolygonStream`] adapter — the only Polygon
//! WebSocket client in the crate.

mod price;

use std::collections::HashSet;
use std::sync::Arc;

use futures::StreamExt;
use tokio::sync::{RwLock, broadcast, mpsc};
use tracing::{debug, info, warn};

use crate::adapters::polygon::websocket::{ClusterDTO, PolygonMessage, PolygonStream};

use super::client::{StreamError, StreamResult};
use super::source::{StreamCommand, apply_command};

pub(crate) use price::PolygonPriceSource;

/// Asset class of a Polygon real-time cluster.
///
/// Each variant is a separate upstream connection with its own channel
/// vocabulary and symbol format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum AssetClass {
    /// US equities and ETFs (`AAPL`).
    #[default]
    Stocks,
    /// Options contracts in OCC format (`O:AAPL250117C00150000`).
    Options,
    /// Currency pairs (`EUR/USD`).
    Forex,
    /// Crypto pairs (`BTC-USD`).
    Crypto,
    /// Futures contracts (`ESZ4`).
    Futures,
    /// Index values (`I:SPX`).
    Indices,
}

impl AssetClass {
    pub(crate) fn cluster(self) -> ClusterDTO {
        match self {
            Self::Stocks => ClusterDTO::Stocks,
            Self::Options => ClusterDTO::Options,
            Self::Forex => ClusterDTO::Forex,
            Self::Crypto => ClusterDTO::Crypto,
            Self::Futures => ClusterDTO::Futures,
            Self::Indices => ClusterDTO::Indices,
        }
    }

    /// Channels carrying price-forming events for this class.
    pub(crate) fn price_channels(self) -> &'static [&'static str] {
        match self {
            Self::Stocks | Self::Options | Self::Futures => &["T", "Q"],
            Self::Forex => &["C", "CA"],
            Self::Crypto => &["XT", "XQ"],
            Self::Indices => &["V"],
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Stocks => "polygon-stocks",
            Self::Options => "polygon-options",
            Self::Forex => "polygon-forex",
            Self::Crypto => "polygon-crypto",
            Self::Futures => "polygon-futures",
            Self::Indices => "polygon-indices",
        }
    }

    /// Put a user-supplied symbol into the cluster's wire format.
    pub(crate) fn wire_symbol(self, symbol: &str) -> String {
        let symbol = symbol.trim();
        match self {
            Self::Indices if !symbol.starts_with("I:") => format!("I:{}", symbol.to_uppercase()),
            Self::Options if !symbol.starts_with("O:") => format!("O:{}", symbol.to_uppercase()),
            _ => symbol.to_uppercase(),
        }
    }
}

/// Expand `symbols` into `prefix.symbol` channel names.
pub(crate) fn channels_for(
    class: AssetClass,
    prefixes: &[&str],
    symbols: impl IntoIterator<Item = String>,
) -> Vec<String> {
    let symbols: Vec<String> = symbols
        .into_iter()
        .map(|s| class.wire_symbol(&s))
        .filter(|s| !s.is_empty())
        .collect();
    prefixes
        .iter()
        .flat_map(|p| symbols.iter().map(move |s| format!("{p}.{s}")))
        .collect()
}

/// Run one connected Polygon session, mapping wire events to `T` via `map`.
///
/// `map` is `FnMut` so sources can carry per-session state (e.g. merging a
/// quote and a trade into one snapshot) without a second broadcast hop.
pub(crate) async fn run_polygon_session<T, M>(
    class: AssetClass,
    prefixes: &[&str],
    subscriptions: &Arc<RwLock<HashSet<String>>>,
    broadcast_tx: &broadcast::Sender<T>,
    command_rx: &mut mpsc::Receiver<StreamCommand>,
    mut map: M,
) -> StreamResult<()>
where
    T: Clone + Send + 'static,
    M: FnMut(PolygonMessage) -> Vec<T> + Send,
{
    let initial: Vec<String> = subscriptions.read().await.iter().cloned().collect();
    let channels = channels_for(class, prefixes, initial);
    let channel_refs: Vec<&str> = channels.iter().map(String::as_str).collect();

    let mut stream = PolygonStream::from_singleton()
        .map_err(|e| StreamError::ConnectionFailed(e.to_string()))?
        .cluster(class.cluster())
        .subscribe(&channel_refs)
        .build()
        .await
        .map_err(|e| StreamError::ConnectionFailed(e.to_string()))?;

    let sender = stream.sender();
    info!("Connected to Polygon {} cluster", class.label());

    loop {
        tokio::select! {
            Some(msg) = stream.next() => {
                match msg {
                    PolygonMessage::Status(status) => debug!("polygon status: {status}"),
                    PolygonMessage::Unknown(raw) => warn!("unparsed polygon frame: {raw}"),
                    msg => {
                        for item in map(msg) {
                            let _ = broadcast_tx.send(item);
                        }
                    }
                }
            }

            Some(cmd) = command_rx.recv() => {
                let Some(changed) = apply_command(&cmd, subscriptions).await else {
                    return Ok(());
                };
                if changed.is_empty() {
                    continue;
                }
                let channels = channels_for(class, prefixes, changed);
                let result = match cmd {
                    StreamCommand::Subscribe(_) => sender.subscribe_channels(&channels).await,
                    _ => sender.unsubscribe_channels(&channels).await,
                };
                if let Err(e) = result {
                    return Err(StreamError::WebSocketError(e.to_string()));
                }
            }

            else => break,
        }
    }

    // Upstream ended without a Close command — reconnect.
    Err(StreamError::WebSocketError(format!(
        "{} connection closed",
        class.label()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_symbol_applies_cluster_prefixes() {
        assert_eq!(AssetClass::Indices.wire_symbol("spx"), "I:SPX");
        assert_eq!(AssetClass::Indices.wire_symbol("I:SPX"), "I:SPX");
        assert_eq!(AssetClass::Stocks.wire_symbol("aapl"), "AAPL");
        assert_eq!(AssetClass::Crypto.wire_symbol("btc-usd"), "BTC-USD");
        assert_eq!(
            AssetClass::Options.wire_symbol("AAPL250117C00150000"),
            "O:AAPL250117C00150000"
        );
    }

    #[test]
    fn channels_expand_across_prefixes_and_symbols() {
        let channels = channels_for(
            AssetClass::Crypto,
            &["XT", "XQ"],
            ["btc-usd".to_string(), "eth-usd".to_string()],
        );
        assert_eq!(
            channels,
            vec!["XT.BTC-USD", "XT.ETH-USD", "XQ.BTC-USD", "XQ.ETH-USD"]
        );
    }
}
