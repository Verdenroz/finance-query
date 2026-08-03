//! finance-query-server library: shared types and modules used by both the
//! HTTP server binary and the MCP server.

pub mod cache;
pub mod graphql;
pub mod lang;
pub mod metrics;
pub mod rate_limit;
pub mod services;

use finance_query::FinanceError;
use finance_query::feeds::FeedSource;
use finance_query::streaming::{NewsStream, PriceStream, PriceUpdate};
use futures_util::{Stream, StreamExt};
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

/// Fan-out channel depth, matching the library's own price-stream capacity.
const FANOUT_CAPACITY: usize = 1024;

#[derive(Clone)]
pub struct AppState {
    pub cache: cache::Cache,
    pub stream_hub: StreamHub,
    pub feed_hub: FeedHub,
}

/// One price tick plus its wire JSON, shared by every connected client.
///
/// Serializing per client rebuilt identical JSON once per client per tick; the
/// first consumer that needs the wire form now builds it for all of them, and
/// consumers that never need it (GraphQL) pay nothing.
pub struct SharedTick {
    update: PriceUpdate,
    json: OnceLock<Arc<str>>,
}

impl SharedTick {
    fn new(update: PriceUpdate) -> Self {
        Self {
            update,
            json: OnceLock::new(),
        }
    }

    /// The decoded tick.
    pub fn update(&self) -> &PriceUpdate {
        &self.update
    }

    /// The tick's JSON encoding, built once and shared.
    pub fn json(&self) -> Arc<str> {
        self.json
            .get_or_init(|| {
                serde_json::to_string(&self.update)
                    .unwrap_or_default()
                    .into()
            })
            .clone()
    }
}

/// A per-client receiver over the hub's shared tick fan-out.
pub struct TickStream {
    inner: BroadcastStream<Arc<SharedTick>>,
}

impl Stream for TickStream {
    type Item = Arc<SharedTick>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            match Pin::new(&mut this.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(tick))) => return Poll::Ready(Some(tick)),
                // A lag means this client missed ticks, not that the feed ended.
                Poll::Ready(Some(Err(_))) => continue,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Process-wide hub that maintains a single upstream Yahoo Finance stream.
///
/// Multiple downstream WebSocket clients can subscribe/unsubscribe to symbols.
/// Symbol subscriptions are ref-counted so each symbol is only subscribed once upstream.
#[derive(Clone, Default)]
pub struct StreamHub {
    inner: Arc<tokio::sync::Mutex<StreamHubInner>>,
}

#[derive(Default)]
struct StreamHubInner {
    upstream: Option<PriceStream>,
    /// Re-broadcast of `upstream` carrying shareable ticks.
    fanout: Option<broadcast::Sender<Arc<SharedTick>>>,
    pump: Option<tokio::task::JoinHandle<()>>,
    symbol_ref_counts: HashMap<String, usize>,
}

/// Read the upstream stream once and hand every client the same tick.
async fn pump_ticks(mut upstream: PriceStream, fanout: broadcast::Sender<Arc<SharedTick>>) {
    while let Some(update) = upstream.next().await {
        let _ = fanout.send(Arc::new(SharedTick::new(update)));
    }
}

impl StreamHub {
    /// Create a new empty hub.
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn resubscribe(&self) -> Option<TickStream> {
        let inner = self.inner.lock().await;
        inner.fanout.as_ref().map(|fanout| TickStream {
            inner: BroadcastStream::new(fanout.subscribe()),
        })
    }

    pub async fn subscribe_symbols(&self, symbols: &[String]) -> Result<(), FinanceError> {
        let unique: HashSet<String> = symbols
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        if unique.is_empty() {
            return Ok(());
        }

        let mut inner = self.inner.lock().await;

        // Track which symbols are newly needed upstream.
        let mut newly_needed: Vec<String> = Vec::new();
        for symbol in &unique {
            let count = inner.symbol_ref_counts.entry(symbol.clone()).or_insert(0);
            if *count == 0 {
                newly_needed.push(symbol.clone());
            }
            *count += 1;
        }
        metrics::STREAM_SUBSCRIPTIONS_ACTIVE.set(inner.symbol_ref_counts.len() as f64);

        // Create upstream stream if this is the first active subscription.
        if inner.upstream.is_none() {
            let stream = PriceStream::subscribe(unique.iter().map(|s| s.as_str())).await?;
            let (fanout, _) = broadcast::channel(FANOUT_CAPACITY);
            inner.pump = Some(tokio::spawn(pump_ticks(
                stream.resubscribe(),
                fanout.clone(),
            )));
            inner.fanout = Some(fanout);
            inner.upstream = Some(stream);
            return Ok(());
        }

        // Add newly needed symbols to upstream.
        if !newly_needed.is_empty()
            && let Some(upstream) = inner.upstream.as_ref()
        {
            upstream
                .add_symbols(newly_needed.iter().map(|s| s.as_str()))
                .await;
        }

        Ok(())
    }

    pub async fn unsubscribe_symbols(&self, symbols: &[String]) {
        let unique: HashSet<String> = symbols
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        if unique.is_empty() {
            return;
        }

        let mut inner = self.inner.lock().await;

        let mut newly_unneeded: Vec<String> = Vec::new();
        for symbol in &unique {
            if let Some(count) = inner.symbol_ref_counts.get_mut(symbol)
                && *count > 0
            {
                *count -= 1;
                if *count == 0 {
                    newly_unneeded.push(symbol.clone());
                }
            }
        }

        for symbol in &newly_unneeded {
            inner.symbol_ref_counts.remove(symbol);
        }
        metrics::STREAM_SUBSCRIPTIONS_ACTIVE.set(inner.symbol_ref_counts.len() as f64);

        if let Some(upstream) = inner.upstream.as_ref()
            && !newly_unneeded.is_empty()
        {
            upstream
                .remove_symbols(newly_unneeded.iter().map(|s| s.as_str()))
                .await;
        }

        // If nothing is subscribed anywhere, close upstream to stop background tasks.
        if inner.symbol_ref_counts.is_empty() {
            if let Some(pump) = inner.pump.take() {
                pump.abort();
            }
            inner.fanout = None;
            if let Some(upstream) = inner.upstream.take() {
                upstream.close().await;
            }
        }
    }
}

/// Process-wide hub that maintains a single upstream `NewsStream`, polling
/// RSS/Atom sources on an interval (RSS/Atom has no push transport of its
/// own — see `finance_query::streaming::NewsStream`).
///
/// Multiple downstream WebSocket/GraphQL clients can subscribe/unsubscribe to
/// sources. Sources are ref-counted by URL (mirroring `StreamHub`'s symbol
/// ref-counting) so each source is only polled once upstream regardless of
/// how many clients want it.
#[derive(Clone, Default)]
pub struct FeedHub {
    inner: Arc<tokio::sync::Mutex<FeedHubInner>>,
}

#[derive(Default)]
struct FeedHubInner {
    upstream: Option<NewsStream>,
    source_ref_counts: HashMap<String, usize>,
}

impl FeedHub {
    /// Create a new empty hub.
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn resubscribe(&self) -> Option<NewsStream> {
        let inner = self.inner.lock().await;
        inner.upstream.as_ref().map(|s| s.resubscribe())
    }

    pub async fn subscribe_sources(&self, sources: &[FeedSource]) {
        if sources.is_empty() {
            return;
        }

        let mut inner = self.inner.lock().await;

        // Track which sources are newly needed upstream.
        let mut newly_needed: Vec<FeedSource> = Vec::new();
        for source in sources {
            let count = inner.source_ref_counts.entry(source.url()).or_insert(0);
            if *count == 0 {
                newly_needed.push(source.clone());
            }
            *count += 1;
        }
        metrics::FEED_SUBSCRIPTIONS_ACTIVE.set(inner.source_ref_counts.len() as f64);

        // Create upstream stream if this is the first active subscription.
        if inner.upstream.is_none() {
            let stream = NewsStream::subscribe(sources.iter().cloned()).await;
            inner.upstream = Some(stream);
            return;
        }

        // Add newly needed sources to upstream.
        if !newly_needed.is_empty()
            && let Some(upstream) = inner.upstream.as_ref()
        {
            upstream.add_sources(newly_needed).await;
        }
    }

    pub async fn unsubscribe_sources(&self, sources: &[FeedSource]) {
        if sources.is_empty() {
            return;
        }

        let mut inner = self.inner.lock().await;

        let mut newly_unneeded: Vec<FeedSource> = Vec::new();
        for source in sources {
            let url = source.url();
            if let Some(count) = inner.source_ref_counts.get_mut(&url)
                && *count > 0
            {
                *count -= 1;
                if *count == 0 {
                    newly_unneeded.push(source.clone());
                }
            }
        }

        for source in &newly_unneeded {
            inner.source_ref_counts.remove(&source.url());
        }
        metrics::FEED_SUBSCRIPTIONS_ACTIVE.set(inner.source_ref_counts.len() as f64);

        if let Some(upstream) = inner.upstream.as_ref()
            && !newly_unneeded.is_empty()
        {
            upstream.remove_sources(newly_unneeded).await;
        }

        // If nothing is subscribed anywhere, close upstream to stop the poll loop.
        if inner.source_ref_counts.is_empty()
            && let Some(upstream) = inner.upstream.take()
        {
            upstream.close().await;
        }
    }
}
