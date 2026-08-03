//! Polygon-backed level-2 depth source (crypto `XL2`).

use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::{RwLock, broadcast, mpsc};

use crate::adapters::polygon::websocket::PolygonMessage;
use crate::streaming::book::{BookLevel, OrderBookUpdate};
use crate::streaming::client::StreamResult;
use crate::streaming::source::{StreamCommand, StreamSource};

use super::{AssetClass, Decode, run_polygon_session};

/// Depth-of-book channel — only the crypto cluster publishes level 2.
const BOOK_CHANNEL: &str = "XL2";

/// Order-book source backed by Polygon's crypto level-2 feed.
pub(crate) struct PolygonBookSource;

#[async_trait::async_trait]
impl StreamSource<OrderBookUpdate> for PolygonBookSource {
    fn id(&self) -> &'static str {
        "polygon-book"
    }

    async fn run_session(
        &self,
        subscriptions: &Arc<RwLock<HashSet<String>>>,
        broadcast_tx: &broadcast::Sender<OrderBookUpdate>,
        command_rx: &mut mpsc::Receiver<StreamCommand>,
    ) -> StreamResult<()> {
        run_polygon_session(
            AssetClass::Crypto,
            &[BOOK_CHANNEL],
            subscriptions,
            broadcast_tx,
            command_rx,
            Decode(|msg| to_book(msg).into_iter().collect()),
        )
        .await
    }
}

/// Map a wire level-2 event onto the public book shape.
///
/// Sides are sorted defensively (bids high→low, asks low→high) so the
/// top-of-book helpers hold regardless of upstream ordering.
pub(crate) fn to_book(msg: PolygonMessage) -> Option<OrderBookUpdate> {
    let PolygonMessage::Level2(book) = msg else {
        return None;
    };

    let mut bids = levels(book.b.as_deref());
    let mut asks = levels(book.a.as_deref());
    bids.sort_by(|a, b| b.price.total_cmp(&a.price));
    asks.sort_by(|a, b| a.price.total_cmp(&b.price));

    Some(OrderBookUpdate {
        symbol: book.pair.clone()?,
        bids,
        asks,
        exchange: book.x,
        time: book.t.unwrap_or_default(),
    })
}

fn levels(side: Option<&[[f64; 2]]>) -> Vec<BookLevel> {
    side.unwrap_or_default()
        .iter()
        .map(|[price, size]| BookLevel {
            price: *price,
            size: *size,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::polygon::websocket::parse_messages;

    const FRAME: &str = r#"[{"ev":"XL2","pair":"BTC-USD","x":1,"t":1705363200000,
        "b":[[99.0,3.0],[100.0,2.0]],
        "a":[[102.0,4.0],[101.0,1.0]]}]"#;

    #[test]
    fn level2_frames_become_sorted_books() {
        let book = parse_messages(FRAME)
            .into_iter()
            .find_map(to_book)
            .expect("book dropped");

        assert_eq!(book.symbol, "BTC-USD");
        assert_eq!(book.exchange, Some(1));
        assert_eq!(book.bids.len(), 2);
        assert_eq!(book.best_bid().unwrap().price, 100.0);
        assert_eq!(book.best_ask().unwrap().price, 101.0);
        assert!((book.spread().unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn a_one_sided_book_still_parses() {
        let frame = r#"[{"ev":"XL2","pair":"ETH-USD","b":[[10.0,1.0]],"t":5}]"#;
        let book = parse_messages(frame)
            .into_iter()
            .find_map(to_book)
            .expect("book dropped");
        assert!(book.asks.is_empty());
        assert!(book.spread().is_none());
    }

    #[test]
    fn non_book_events_are_ignored() {
        let frame = r#"[{"ev":"XT","pair":"BTC-USD","p":1.0,"t":1}]"#;
        assert!(
            parse_messages(frame)
                .into_iter()
                .find_map(to_book)
                .is_none()
        );
    }
}
