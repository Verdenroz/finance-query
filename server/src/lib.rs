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
use finance_query::streaming::{NewsStream, PriceStream};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub cache: cache::Cache,
    pub stream_hub: StreamHub,
    pub feed_hub: FeedHub,
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
    symbol_ref_counts: HashMap<String, usize>,
}

impl StreamHub {
    /// Create a new empty hub.
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn resubscribe(&self) -> Option<PriceStream> {
        let inner = self.inner.lock().await;
        inner.upstream.as_ref().map(|s| s.resubscribe())
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

        // Create upstream stream if this is the first active subscription.
        if inner.upstream.is_none() {
            let refs: Vec<&str> = unique.iter().map(|s| s.as_str()).collect();
            let stream = PriceStream::subscribe(&refs).await?;
            inner.upstream = Some(stream);
            return Ok(());
        }

        // Add newly needed symbols to upstream.
        if !newly_needed.is_empty()
            && let Some(upstream) = inner.upstream.as_ref()
        {
            let refs: Vec<&str> = newly_needed.iter().map(|s| s.as_str()).collect();
            upstream.add_symbols(&refs).await;
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

        if let Some(upstream) = inner.upstream.as_ref()
            && !newly_unneeded.is_empty()
        {
            let refs: Vec<&str> = newly_unneeded.iter().map(|s| s.as_str()).collect();
            upstream.remove_symbols(&refs).await;
        }

        // If nothing is subscribed anywhere, close upstream to stop background tasks.
        if inner.symbol_ref_counts.is_empty()
            && let Some(upstream) = inner.upstream.take()
        {
            upstream.close().await;
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

        // Create upstream stream if this is the first active subscription.
        if inner.upstream.is_none() {
            let stream = NewsStream::subscribe(sources).await;
            inner.upstream = Some(stream);
            return;
        }

        // Add newly needed sources to upstream.
        if !newly_needed.is_empty()
            && let Some(upstream) = inner.upstream.as_ref()
        {
            upstream.add_sources(&newly_needed).await;
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

        if let Some(upstream) = inner.upstream.as_ref()
            && !newly_unneeded.is_empty()
        {
            upstream.remove_sources(&newly_unneeded).await;
        }

        // If nothing is subscribed anywhere, close upstream to stop the poll loop.
        if inner.source_ref_counts.is_empty()
            && let Some(upstream) = inner.upstream.take()
        {
            upstream.close().await;
        }
    }
}
