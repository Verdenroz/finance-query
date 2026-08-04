//! Polygon.io-backed streaming sources.
//!
//! Polygon runs one WebSocket cluster per asset class, so each class gets a
//! purpose-built source instead of riding a single generic feed. Everything
//! here reuses the existing [`PolygonStream`] adapter — the only Polygon
//! WebSocket client in the crate.

mod book;
mod options;
mod price;
mod trades;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use futures::StreamExt;
use tokio::sync::{RwLock, broadcast, mpsc};
use tracing::{debug, info, warn};

use crate::adapters::polygon::websocket::{ClusterDTO, PolygonMessage, PolygonStream};

use super::client::{StreamError, StreamResult};
use super::source::{StreamCommand, apply_command};

pub(crate) use book::PolygonBookSource;
pub(crate) use options::PolygonOptionsSource;
pub(crate) use price::PolygonPriceSource;
pub(crate) use trades::PolygonTradeSource;

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

    /// Channel carrying individual trade prints, where the class has one.
    pub(crate) fn trade_channel(self) -> Option<&'static str> {
        match self {
            Self::Stocks | Self::Options | Self::Futures => Some("T"),
            Self::Crypto => Some("XT"),
            // Forex and indices publish no per-trade prints.
            Self::Forex | Self::Indices => None,
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
    ///
    /// A bare options underlying (no digits) becomes an `O:AAPL*` wildcard so
    /// the whole chain is followed; a full OCC symbol stays exact.
    pub(crate) fn wire_symbol(self, symbol: &str) -> String {
        let symbol = symbol.trim().to_uppercase();
        match self {
            Self::Indices if !symbol.starts_with("I:") => format!("I:{symbol}"),
            Self::Options => {
                let body = symbol.strip_prefix("O:").unwrap_or(&symbol);
                if body.chars().any(|c| c.is_ascii_digit()) {
                    format!("O:{body}")
                } else {
                    format!("O:{}*", body.trim_end_matches('*'))
                }
            }
            _ => symbol,
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

/// Decodes wire events into `T`, and drops any per-symbol state on unsubscribe.
///
/// Stateful because sources merge several event types into one snapshot per
/// symbol; that state must not outlive the subscription that created it.
pub(crate) trait SessionHandler<T>: Send {
    /// Decode one wire event.
    fn on_event(&mut self, msg: PolygonMessage) -> Vec<T>;

    /// Forget state for symbols that just left the subscription set.
    fn on_unsubscribe(&mut self, _removed: &[String]) {}
}

/// Adapts a stateless decode function to [`SessionHandler`].
pub(crate) struct Decode<F>(pub(crate) F);

impl<T, F> SessionHandler<T> for Decode<F>
where
    F: FnMut(PolygonMessage) -> Vec<T> + Send,
{
    fn on_event(&mut self, msg: PolygonMessage) -> Vec<T> {
        (self.0)(msg)
    }
}

/// `true` when a wire symbol — possibly an `O:AAPL*` wildcard — covers `key`.
fn covers(pattern: &str, key: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => key.starts_with(prefix),
        None => key == pattern,
    }
}

/// Drop per-symbol state for symbols that just left the subscription set.
///
/// Keys are wire symbols, so user input is normalised the same way the
/// subscription was; an options wildcard prunes the whole chain it created.
pub(crate) fn prune_symbols<V>(
    state: &mut HashMap<String, V>,
    class: AssetClass,
    removed: &[String],
) {
    let patterns: Vec<String> = removed.iter().map(|s| class.wire_symbol(s)).collect();
    state.retain(|key, _| !patterns.iter().any(|p| covers(p, key)));
}

/// Run one connected Polygon session, decoding wire events via `handler`.
pub(crate) async fn run_polygon_session<T, H>(
    class: AssetClass,
    prefixes: &[&str],
    subscriptions: &Arc<RwLock<HashSet<String>>>,
    broadcast_tx: &broadcast::Sender<T>,
    command_rx: &mut mpsc::Receiver<StreamCommand>,
    mut handler: H,
) -> StreamResult<()>
where
    T: Clone + Send + 'static,
    H: SessionHandler<T>,
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
                        for item in handler.on_event(msg) {
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
                let subscribing = matches!(cmd, StreamCommand::Subscribe(_));
                if !subscribing {
                    handler.on_unsubscribe(&changed);
                }
                let channels = channels_for(class, prefixes, changed);
                let result = if subscribing {
                    sender.subscribe_channels(&channels).await
                } else {
                    sender.unsubscribe_channels(&channels).await
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
        assert_eq!(AssetClass::Options.wire_symbol("aapl"), "O:AAPL*");
        assert_eq!(AssetClass::Options.wire_symbol("O:AAPL*"), "O:AAPL*");
    }

    #[test]
    fn pruning_drops_exact_and_wildcard_matches() {
        let mut state: HashMap<String, u8> = HashMap::from([
            ("O:AAPL250117C00150000".to_string(), 1),
            ("O:AAPL250117P00150000".to_string(), 2),
            ("O:SPY250117C00500000".to_string(), 3),
        ]);

        // A bare underlying was subscribed as the `O:AAPL*` wildcard, so it
        // must take the whole chain with it.
        prune_symbols(&mut state, AssetClass::Options, &["AAPL".to_string()]);
        assert_eq!(state.len(), 1);
        assert!(state.contains_key("O:SPY250117C00500000"));

        let mut equities: HashMap<String, u8> =
            HashMap::from([("AAPL".to_string(), 1), ("NVDA".to_string(), 2)]);
        prune_symbols(&mut equities, AssetClass::Stocks, &["aapl".to_string()]);
        assert_eq!(equities.keys().collect::<Vec<_>>(), vec!["NVDA"]);
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
