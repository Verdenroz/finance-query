//! Polygon-backed options-chain source.
//!
//! The real-time options cluster carries quotes and trades only, so greeks,
//! implied volatility and open interest are folded in from a periodic chain
//! snapshot on the REST side.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{RwLock, broadcast, mpsc};
use tracing::warn;

use crate::adapters::polygon::options::snapshots::options_chain_snapshot;
use crate::adapters::polygon::websocket::PolygonMessage;
use crate::streaming::client::StreamResult;
use crate::streaming::options::{
    ContractParts, Greeks, OptionContractUpdate, parse_contract_symbol,
};
use crate::streaming::source::{StreamCommand, StreamSource};

use super::{AssetClass, run_polygon_session};

/// Live options-chain source backed by Polygon's options cluster.
pub(crate) struct PolygonOptionsSource {
    greeks_refresh: Option<Duration>,
}

impl PolygonOptionsSource {
    pub(crate) fn new(greeks_refresh: Option<Duration>) -> Self {
        Self { greeks_refresh }
    }
}

#[async_trait::async_trait]
impl StreamSource<OptionContractUpdate> for PolygonOptionsSource {
    fn id(&self) -> &'static str {
        "polygon-options"
    }

    async fn run_session(
        &self,
        subscriptions: &Arc<RwLock<HashSet<String>>>,
        broadcast_tx: &broadcast::Sender<OptionContractUpdate>,
        command_rx: &mut mpsc::Receiver<StreamCommand>,
    ) -> StreamResult<()> {
        let refresh = self.greeks_refresh.map(|interval| {
            tokio::spawn(refresh_snapshots(
                interval,
                Arc::clone(subscriptions),
                broadcast_tx.clone(),
            ))
        });

        let mut merger = ContractMerger::default();
        let result = run_polygon_session(
            AssetClass::Options,
            AssetClass::Options.price_channels(),
            subscriptions,
            broadcast_tx,
            command_rx,
            move |msg| merger.apply(msg).into_iter().collect(),
        )
        .await;

        if let Some(handle) = refresh {
            handle.abort();
        }
        result
    }
}

/// Keeps the last known state per contract so a quote-only event still
/// publishes the contract's most recent trade (and vice versa).
#[derive(Default)]
pub(crate) struct ContractMerger {
    contracts: HashMap<String, OptionContractUpdate>,
}

impl ContractMerger {
    fn entry(&mut self, symbol: &str) -> Option<&mut OptionContractUpdate> {
        if !self.contracts.contains_key(symbol) {
            let parts = parse_contract_symbol(symbol)?;
            self.contracts
                .insert(symbol.to_string(), new_contract(symbol, &parts));
        }
        self.contracts.get_mut(symbol)
    }

    /// Merge one wire event into the contract's snapshot.
    pub(crate) fn apply(&mut self, msg: PolygonMessage) -> Option<OptionContractUpdate> {
        match msg {
            PolygonMessage::Trade(trade) => {
                let symbol = trade.symbol()?.to_string();
                let contract = self.entry(&symbol)?;
                contract.last_price = trade.p.or(contract.last_price);
                contract.last_size = trade.s.or(contract.last_size);
                if let Some(t) = trade.t {
                    contract.time = t;
                }
                Some(contract.clone())
            }
            PolygonMessage::Quote(quote) => {
                let symbol = quote.symbol()?.to_string();
                let contract = self.entry(&symbol)?;
                contract.bid = quote.bp.or(contract.bid);
                contract.ask = quote.ap.or(contract.ask);
                contract.bid_size = quote.bs.or(contract.bid_size);
                contract.ask_size = quote.ask_size.or(contract.ask_size);
                if let Some(t) = quote.t {
                    contract.time = t;
                }
                Some(contract.clone())
            }
            _ => None,
        }
    }
}

fn new_contract(symbol: &str, parts: &ContractParts) -> OptionContractUpdate {
    OptionContractUpdate {
        contract_symbol: symbol.to_string(),
        underlying: parts.underlying.clone(),
        expiration: Some(parts.expiration),
        strike: Some(parts.strike),
        option_type: Some(parts.option_type),
        ..Default::default()
    }
}

/// Underlyings to snapshot, derived from the live subscription set.
fn underlyings(symbols: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for symbol in symbols {
        let trimmed = symbol.trim_start_matches("O:").trim_end_matches('*');
        let root = parse_contract_symbol(&symbol)
            .map(|p| p.underlying)
            .unwrap_or_else(|| trimmed.to_uppercase());
        if !root.is_empty() && !seen.contains(&root) {
            seen.push(root);
        }
    }
    seen
}

/// Periodically fold greeks/IV/open interest from the REST chain snapshot
/// into the same broadcast channel the WebSocket feeds.
async fn refresh_snapshots(
    interval: Duration,
    subscriptions: Arc<RwLock<HashSet<String>>>,
    broadcast_tx: broadcast::Sender<OptionContractUpdate>,
) {
    let mut ticker = tokio::time::interval(interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        ticker.tick().await;
        // Snapshot the set first — never hold the lock across the HTTP call.
        let subscribed: Vec<String> = subscriptions.read().await.iter().cloned().collect();

        for underlying in underlyings(subscribed) {
            match options_chain_snapshot(&underlying, &[("limit", "250")]).await {
                Ok(page) => {
                    for update in page
                        .results
                        .unwrap_or_default()
                        .iter()
                        .filter_map(snapshot_to_update)
                    {
                        let _ = broadcast_tx.send(update);
                    }
                }
                Err(e) => warn!("options snapshot refresh failed for {underlying}: {e}"),
            }
        }
    }
}

/// Map a REST chain snapshot entry onto the live update shape.
pub(crate) fn snapshot_to_update(
    snapshot: &crate::adapters::polygon::options::snapshots::OptionsSnapshotDTO,
) -> Option<OptionContractUpdate> {
    let details = snapshot.details.as_ref()?;
    let symbol = details.ticker.clone()?;
    let parts = parse_contract_symbol(&symbol)?;

    let quote = snapshot.last_quote.as_ref();
    let trade = snapshot.last_trade.as_ref();
    let greeks = snapshot.greeks.as_ref().map(|g| Greeks {
        delta: g.delta,
        gamma: g.gamma,
        theta: g.theta,
        vega: g.vega,
    });

    Some(OptionContractUpdate {
        contract_symbol: symbol,
        underlying: parts.underlying,
        expiration: Some(parts.expiration),
        strike: details.strike_price.or(Some(parts.strike)),
        option_type: Some(parts.option_type),
        bid: quote.and_then(|q| q.bid),
        bid_size: quote.and_then(|q| q.bid_size),
        ask: quote.and_then(|q| q.ask),
        ask_size: quote.and_then(|q| q.ask_size),
        last_price: trade.and_then(|t| t.price),
        last_size: trade.and_then(|t| t.size),
        volume: snapshot
            .day
            .as_ref()
            .and_then(|d| d.volume)
            .map(|v| v as i64),
        open_interest: snapshot.open_interest.map(|oi| oi as i64),
        implied_volatility: snapshot.implied_volatility,
        greeks: greeks.filter(|g| !g.is_empty()),
        time: quote
            .and_then(|q| q.last_updated)
            .or_else(|| trade.and_then(|t| t.sip_timestamp))
            .unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::polygon::options::snapshots::OptionsSnapshotDTO;
    use crate::adapters::polygon::websocket::parse_messages;
    use crate::streaming::pricing::OptionType;

    fn event(frame: &str) -> PolygonMessage {
        parse_messages(frame).remove(0)
    }

    #[test]
    fn quote_and_trade_merge_into_one_contract() {
        let mut merger = ContractMerger::default();
        merger.apply(event(
            r#"[{"ev":"Q","sym":"O:AAPL250117C00150000","bp":3.2,"bs":10,"ap":3.4,"as":12,"t":1}]"#,
        ));
        let update = merger
            .apply(event(
                r#"[{"ev":"T","sym":"O:AAPL250117C00150000","p":3.3,"s":5,"t":2}]"#,
            ))
            .expect("trade dropped");

        assert_eq!(update.underlying, "AAPL");
        assert_eq!(update.option_type, Some(OptionType::Call));
        assert_eq!(update.strike, Some(150.0));
        assert_eq!(update.bid, Some(3.2));
        assert_eq!(update.ask, Some(3.4));
        assert_eq!(update.last_price, Some(3.3));
        assert_eq!(update.time, 2);
    }

    #[test]
    fn unparsable_contract_symbols_are_skipped() {
        let mut merger = ContractMerger::default();
        assert!(
            merger
                .apply(event(
                    r#"[{"ev":"T","sym":"NOT-A-CONTRACT","p":1.0,"t":1}]"#
                ))
                .is_none()
        );
    }

    #[test]
    fn snapshot_supplies_greeks_and_open_interest() {
        let dto: OptionsSnapshotDTO = serde_json::from_value(serde_json::json!({
            "details": {"ticker": "O:AAPL250117C00150000", "strike_price": 150.0,
                        "contract_type": "call", "expiration_date": "2025-01-17"},
            "greeks": {"delta": 0.55, "gamma": 0.02, "theta": -0.03, "vega": 0.11},
            "implied_volatility": 0.28,
            "open_interest": 4200,
            "day": {"v": 1500.0},
            "last_quote": {"bid": 3.2, "ask": 3.4, "last_updated": 99}
        }))
        .expect("fixture should deserialize");

        let update = snapshot_to_update(&dto).expect("snapshot dropped");
        assert_eq!(update.open_interest, Some(4200));
        assert_eq!(update.implied_volatility, Some(0.28));
        assert_eq!(update.greeks.unwrap().delta, Some(0.55));
        assert_eq!(update.volume, Some(1500));
        assert_eq!(update.time, 99);
    }

    #[test]
    fn snapshot_without_greeks_leaves_the_field_unset() {
        let dto: OptionsSnapshotDTO = serde_json::from_value(serde_json::json!({
            "details": {"ticker": "O:AAPL250117C00150000"}
        }))
        .expect("fixture should deserialize");
        assert!(snapshot_to_update(&dto).unwrap().greeks.is_none());
    }

    #[test]
    fn underlyings_dedupe_across_wildcards_and_contracts() {
        let roots = underlyings(vec![
            "O:AAPL*".to_string(),
            "AAPL".to_string(),
            "O:AAPL250117C00150000".to_string(),
            "O:SPY*".to_string(),
        ]);
        assert_eq!(roots, vec!["AAPL", "SPY"]);
    }
}
