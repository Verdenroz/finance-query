//! Polygon-backed [`PriceUpdate`] sources, one per asset class.
//!
//! Trades, quotes and aggregates arrive as separate events; the merger keeps
//! the last snapshot per symbol so subscribers always see a complete tick
//! rather than a partial one.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::{RwLock, broadcast, mpsc};

use crate::adapters::polygon::websocket::PolygonMessage;
use crate::streaming::client::StreamResult;
use crate::streaming::pricing::{MarketHoursType, PriceUpdate, QuoteType};
use crate::streaming::source::{StreamCommand, StreamSource};

use super::{AssetClass, run_polygon_session};

/// Real-time price source backed by one Polygon cluster.
pub(crate) struct PolygonPriceSource {
    class: AssetClass,
}

impl PolygonPriceSource {
    pub(crate) fn new(class: AssetClass) -> Self {
        Self { class }
    }
}

#[async_trait::async_trait]
impl StreamSource<PriceUpdate> for PolygonPriceSource {
    fn id(&self) -> &'static str {
        self.class.label()
    }

    async fn run_session(
        &self,
        subscriptions: &Arc<RwLock<HashSet<String>>>,
        broadcast_tx: &broadcast::Sender<PriceUpdate>,
        command_rx: &mut mpsc::Receiver<StreamCommand>,
    ) -> StreamResult<()> {
        let mut merger = PriceMerger::new(self.class);
        run_polygon_session(
            self.class,
            self.class.price_channels(),
            subscriptions,
            broadcast_tx,
            command_rx,
            move |msg| merger.apply(msg).into_iter().collect(),
        )
        .await
    }
}

/// Folds per-event Polygon messages into one snapshot per symbol.
pub(crate) struct PriceMerger {
    quote_type: QuoteType,
    snapshots: HashMap<String, PriceUpdate>,
}

impl PriceMerger {
    pub(crate) fn new(class: AssetClass) -> Self {
        Self {
            quote_type: match class {
                AssetClass::Stocks => QuoteType::Equity,
                AssetClass::Options => QuoteType::Option,
                AssetClass::Forex => QuoteType::Currency,
                AssetClass::Crypto => QuoteType::Cryptocurrency,
                AssetClass::Futures => QuoteType::Future,
                AssetClass::Indices => QuoteType::Index,
            },
            snapshots: HashMap::new(),
        }
    }

    fn entry(&mut self, symbol: &str) -> &mut PriceUpdate {
        let quote_type = self.quote_type;
        self.snapshots.entry(symbol.to_string()).or_insert_with(|| {
            let mut update = PriceUpdate {
                id: symbol.to_string(),
                quote_type,
                market_hours: MarketHoursType::RegularMarket,
                ..Default::default()
            };
            update.currency = default_currency(symbol).to_string();
            update
        })
    }

    /// Merge one event, returning the updated snapshot when it carried a price.
    pub(crate) fn apply(&mut self, msg: PolygonMessage) -> Option<PriceUpdate> {
        match msg {
            PolygonMessage::Trade(trade) => {
                let symbol = trade.symbol()?.to_string();
                let snapshot = self.entry(&symbol);
                if let Some(p) = trade.p {
                    snapshot.price = p as f32;
                }
                if let Some(s) = trade.s {
                    snapshot.last_size = s as i64;
                }
                if let Some(x) = trade.x {
                    snapshot.exchange = x.to_string();
                }
                if let Some(t) = trade.t {
                    snapshot.time = t;
                }
                Some(snapshot.clone())
            }
            PolygonMessage::Quote(quote) => {
                let symbol = quote.symbol()?.to_string();
                let snapshot = self.entry(&symbol);
                if let Some(bp) = quote.bp {
                    snapshot.bid = bp as f32;
                }
                if let Some(ap) = quote.ap {
                    snapshot.ask = ap as f32;
                }
                if let Some(bs) = quote.bs {
                    snapshot.bid_size = bs as i64;
                }
                if let Some(a_s) = quote.ask_size {
                    snapshot.ask_size = a_s as i64;
                }
                if let Some(t) = quote.t {
                    snapshot.time = t;
                }
                // Quote-only symbols would otherwise never report a price.
                if snapshot.price == 0.0 && snapshot.bid > 0.0 && snapshot.ask > 0.0 {
                    snapshot.price = (snapshot.bid + snapshot.ask) / 2.0;
                }
                Some(snapshot.clone())
            }
            PolygonMessage::ForexQuote(quote) => {
                let symbol = quote.p.clone()?;
                let snapshot = self.entry(&symbol);
                if let Some(b) = quote.b {
                    snapshot.bid = b as f32;
                }
                if let Some(a) = quote.a {
                    snapshot.ask = a as f32;
                }
                if let Some(t) = quote.t {
                    snapshot.time = t;
                }
                if snapshot.bid > 0.0 && snapshot.ask > 0.0 {
                    snapshot.price = (snapshot.bid + snapshot.ask) / 2.0;
                }
                Some(snapshot.clone())
            }
            PolygonMessage::Aggregate(agg) => {
                let symbol = agg.symbol()?.to_string();
                let snapshot = self.entry(&symbol);
                if let Some(c) = agg.c {
                    snapshot.price = c as f32;
                }
                if let Some(o) = agg.o {
                    snapshot.open_price = o as f32;
                }
                if let Some(h) = agg.h {
                    snapshot.day_high = snapshot.day_high.max(h as f32);
                }
                if let Some(l) = agg.l {
                    snapshot.day_low = if snapshot.day_low == 0.0 {
                        l as f32
                    } else {
                        snapshot.day_low.min(l as f32)
                    };
                }
                if let Some(v) = agg.v {
                    snapshot.day_volume = v as i64;
                }
                if let Some(e) = agg.e {
                    snapshot.time = e;
                }
                recompute_change(snapshot);
                Some(snapshot.clone())
            }
            PolygonMessage::IndexValue(index) => {
                let symbol = index.ticker.clone()?;
                let snapshot = self.entry(&symbol);
                if let Some(val) = index.val {
                    snapshot.price = val as f32;
                }
                if let Some(t) = index.t {
                    snapshot.time = t;
                }
                Some(snapshot.clone())
            }
            _ => None,
        }
    }
}

/// Intraday change against the session open — Polygon's real-time feed carries
/// no previous close, so that field stays zero.
fn recompute_change(snapshot: &mut PriceUpdate) {
    if snapshot.open_price > 0.0 && snapshot.price > 0.0 {
        snapshot.change = snapshot.price - snapshot.open_price;
        snapshot.change_percent = snapshot.change / snapshot.open_price * 100.0;
    }
}

/// Quote currency for pair-shaped symbols (`BTC-USD`, `EUR/USD`).
fn default_currency(symbol: &str) -> &str {
    symbol
        .rsplit(['-', '/'])
        .next()
        .filter(|q| q.len() == 3 && *q != symbol)
        .unwrap_or("USD")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::polygon::websocket::parse_messages;

    fn events(frame: &str) -> Vec<PolygonMessage> {
        parse_messages(frame)
    }

    #[test]
    fn trade_then_quote_merge_into_one_snapshot() {
        let mut merger = PriceMerger::new(AssetClass::Stocks);
        let frame = r#"[{"ev":"T","sym":"AAPL","p":186.19,"s":100,"x":4,"t":1705363200000},
                        {"ev":"Q","sym":"AAPL","bp":186.18,"bs":2,"ap":186.25,"as":3,"t":1705363200001}]"#;

        let mut last = None;
        for msg in events(frame) {
            last = merger.apply(msg);
        }

        let snapshot = last.expect("no snapshot emitted");
        assert_eq!(snapshot.id, "AAPL");
        assert!((snapshot.price - 186.19).abs() < 0.01, "trade price kept");
        assert!((snapshot.bid - 186.18).abs() < 0.01);
        assert!((snapshot.ask - 186.25).abs() < 0.01);
        assert_eq!(snapshot.quote_type, QuoteType::Equity);
        assert_eq!(snapshot.time, 1705363200001);
    }

    #[test]
    fn crypto_events_key_off_pair() {
        let mut merger = PriceMerger::new(AssetClass::Crypto);
        let frame = r#"[{"ev":"XT","pair":"BTC-USD","p":65000.5,"s":0.25,"t":1705363200000}]"#;
        let snapshot = merger
            .apply(events(frame).remove(0))
            .expect("crypto trade dropped");
        assert_eq!(snapshot.id, "BTC-USD");
        assert_eq!(snapshot.quote_type, QuoteType::Cryptocurrency);
        assert_eq!(snapshot.currency, "USD");
    }

    #[test]
    fn forex_quote_produces_a_midpoint_price() {
        let mut merger = PriceMerger::new(AssetClass::Forex);
        let frame = r#"[{"ev":"C","p":"EUR/USD","a":1.0902,"b":1.0898,"x":48,"t":1705363200000}]"#;
        let snapshot = merger
            .apply(events(frame).remove(0))
            .expect("forex quote dropped");
        assert_eq!(snapshot.id, "EUR/USD");
        assert!((snapshot.price - 1.09).abs() < 0.001);
        assert_eq!(snapshot.quote_type, QuoteType::Currency);
    }

    #[test]
    fn aggregate_fills_ohlcv_and_intraday_change() {
        let mut merger = PriceMerger::new(AssetClass::Stocks);
        let frame = r#"[{"ev":"AM","sym":"AAPL","o":100.0,"h":110.0,"l":95.0,"c":105.0,"v":1500,"s":1,"e":2}]"#;
        let snapshot = merger
            .apply(events(frame).remove(0))
            .expect("aggregate dropped");
        assert_eq!(snapshot.day_volume, 1500);
        assert!((snapshot.day_high - 110.0).abs() < 0.01);
        assert!((snapshot.day_low - 95.0).abs() < 0.01);
        assert!((snapshot.change - 5.0).abs() < 0.01);
        assert!((snapshot.change_percent - 5.0).abs() < 0.01);
    }

    #[test]
    fn index_values_map_to_price() {
        let mut merger = PriceMerger::new(AssetClass::Indices);
        let frame = r#"[{"ev":"V","val":3988.5,"T":"I:SPX","t":1678220098130}]"#;
        let snapshot = merger
            .apply(events(frame).remove(0))
            .expect("index value dropped");
        assert_eq!(snapshot.id, "I:SPX");
        assert!((snapshot.price - 3988.5).abs() < 0.01);
        assert_eq!(snapshot.quote_type, QuoteType::Index);
    }

    #[test]
    fn status_frames_produce_no_snapshot() {
        let mut merger = PriceMerger::new(AssetClass::Stocks);
        let frame = r#"[{"ev":"status","status":"auth_success"}]"#;
        assert!(merger.apply(events(frame).remove(0)).is_none());
    }
}
