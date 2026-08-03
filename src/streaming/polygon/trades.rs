//! Polygon-backed tick-by-tick trade source.

use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::{RwLock, broadcast, mpsc};

use crate::adapters::polygon::websocket::PolygonMessage;
use crate::streaming::client::{StreamError, StreamResult};
use crate::streaming::source::{StreamCommand, StreamSource};
use crate::streaming::trades::TradeTick;

use super::{AssetClass, Decode, run_polygon_session};

/// Trade-print source for one Polygon cluster.
pub(crate) struct PolygonTradeSource {
    class: AssetClass,
    channel: &'static str,
}

impl PolygonTradeSource {
    /// Fails fast for asset classes that publish no per-trade prints.
    pub(crate) fn new(class: AssetClass) -> StreamResult<Self> {
        let channel = class.trade_channel().ok_or_else(|| {
            StreamError::ConnectionFailed(format!("{} publishes no trade prints", class.label()))
        })?;
        Ok(Self { class, channel })
    }
}

#[async_trait::async_trait]
impl StreamSource<TradeTick> for PolygonTradeSource {
    fn id(&self) -> &'static str {
        self.class.label()
    }

    async fn run_session(
        &self,
        subscriptions: &Arc<RwLock<HashSet<String>>>,
        broadcast_tx: &broadcast::Sender<TradeTick>,
        command_rx: &mut mpsc::Receiver<StreamCommand>,
    ) -> StreamResult<()> {
        run_polygon_session(
            self.class,
            &[self.channel],
            subscriptions,
            broadcast_tx,
            command_rx,
            Decode(|msg| to_tick(msg).into_iter().collect()),
        )
        .await
    }
}

/// Map a wire trade event onto the public tick shape.
pub(crate) fn to_tick(msg: PolygonMessage) -> Option<TradeTick> {
    let PolygonMessage::Trade(trade) = msg else {
        return None;
    };
    let symbol = trade.symbol()?.to_string();
    Some(TradeTick {
        symbol,
        price: trade.p?,
        size: trade.s.unwrap_or_default(),
        exchange: trade.x,
        conditions: trade.c.unwrap_or_default(),
        trade_id: trade.i,
        time: trade.t.unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::polygon::websocket::parse_messages;

    #[test]
    fn every_print_in_a_frame_becomes_a_tick() {
        let frame = r#"[{"ev":"T","sym":"AAPL","p":186.19,"s":100,"x":4,"c":[12,37],"t":1},
                        {"ev":"T","sym":"AAPL","p":186.20,"s":50,"x":4,"t":2},
                        {"ev":"T","sym":"AAPL","p":186.21,"s":25,"x":11,"t":3}]"#;
        let ticks: Vec<TradeTick> = parse_messages(frame)
            .into_iter()
            .filter_map(to_tick)
            .collect();

        assert_eq!(ticks.len(), 3, "no print may be coalesced away");
        assert_eq!(ticks[0].conditions, vec![12, 37]);
        assert_eq!(ticks[1].size, 50.0);
        assert_eq!(ticks[2].exchange, Some(11));
        assert!((ticks[0].notional() - 18619.0).abs() < 1e-6);
    }

    #[test]
    fn crypto_prints_carry_pair_and_trade_id() {
        let frame = r#"[{"ev":"XT","pair":"BTC-USD","p":65000.5,"s":0.25,"i":"t-1","t":9}]"#;
        let tick = parse_messages(frame)
            .into_iter()
            .find_map(to_tick)
            .expect("crypto print dropped");
        assert_eq!(tick.symbol, "BTC-USD");
        assert_eq!(tick.trade_id.as_deref(), Some("t-1"));
    }

    #[test]
    fn non_trade_events_are_ignored() {
        let frame = r#"[{"ev":"Q","sym":"AAPL","bp":1.0,"ap":1.1,"t":1}]"#;
        assert!(
            parse_messages(frame)
                .into_iter()
                .find_map(to_tick)
                .is_none()
        );
    }

    #[test]
    fn classes_without_a_trade_feed_fail_construction() {
        assert!(PolygonTradeSource::new(AssetClass::Forex).is_err());
        assert!(PolygonTradeSource::new(AssetClass::Stocks).is_ok());
    }
}
