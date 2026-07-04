//! GraphQL `SubscriptionRoot` — real-time price and feed streams.

use std::collections::HashSet;
use std::pin::Pin;
use std::task;

use async_graphql::{Context, Error, Result, Subscription};
use finance_query::feeds::FeedSource;
use futures_util::{Stream, StreamExt};

use super::types::feeds::GqlFeedEntry;
use super::types::streaming::GqlPriceUpdate;
use crate::{AppState, FeedHub, StreamHub};

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    /// Subscribe to real-time price updates for the given ticker symbols.
    ///
    /// The stream yields a `GqlPriceUpdate` whenever the upstream Yahoo Finance
    /// WebSocket delivers a matching price tick. Symbols are ref-counted across
    /// all active subscriptions — upstream subscribes only once per symbol.
    async fn price_stream(
        &self,
        ctx: &Context<'_>,
        symbols: Vec<String>,
    ) -> Result<impl Stream<Item = GqlPriceUpdate>> {
        let state = ctx.data::<AppState>()?;
        let hub = state.stream_hub.clone();

        hub.subscribe_symbols(&symbols)
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        let hub_stream = hub
            .resubscribe()
            .await
            .ok_or_else(|| Error::new("Price stream unavailable"))?;

        let subscribed: HashSet<String> = symbols.iter().cloned().collect();
        let guard = SubscriptionGuard {
            hub: hub.clone(),
            symbols,
        };

        let filtered = hub_stream
            .filter(move |u| std::future::ready(subscribed.contains(&u.id)))
            .map(GqlPriceUpdate::from);

        Ok(GuardedStream {
            _guard: guard,
            inner: Box::pin(filtered),
        })
    }

    /// Subscribe to a continuous stream of new RSS/Atom feed entries from the
    /// given sources.
    ///
    /// RSS/Atom has no push transport of its own, so the stream is backed by
    /// a shared poll loop (`finance_query::streaming::NewsStream`) — this
    /// field yields entries as they're first seen (deduplicated by URL), same
    /// contract as `GET /v2/feeds/stream`. Sources are ref-counted across all
    /// active subscriptions — upstream polls each source only once.
    async fn feed_stream(
        &self,
        ctx: &Context<'_>,
        sources: Option<Vec<String>>,
        form_type: Option<String>,
    ) -> Result<impl Stream<Item = GqlFeedEntry>> {
        let state = ctx.data::<AppState>()?;
        let hub = state.feed_hub.clone();

        let parsed =
            crate::services::feeds::parse_sources(sources.as_deref(), form_type.as_deref())
                .map_err(Error::new)?;

        hub.subscribe_sources(&parsed).await;

        let hub_stream = hub
            .resubscribe()
            .await
            .ok_or_else(|| Error::new("Feed stream unavailable"))?;

        let subscribed: HashSet<String> = parsed.iter().map(FeedSource::name).collect();
        let guard = FeedSubscriptionGuard {
            hub: hub.clone(),
            sources: parsed,
        };

        let filtered = hub_stream
            .filter(move |entry| std::future::ready(subscribed.contains(&entry.source)))
            .map(GqlFeedEntry::from);

        Ok(GuardedFeedStream {
            _guard: guard,
            inner: Box::pin(filtered),
        })
    }
}

/// RAII guard that unsubscribes symbols when the GraphQL subscription is dropped.
struct SubscriptionGuard {
    hub: StreamHub,
    symbols: Vec<String>,
}

impl Drop for SubscriptionGuard {
    fn drop(&mut self) {
        let hub = self.hub.clone();
        let symbols = std::mem::take(&mut self.symbols);
        tokio::spawn(async move {
            hub.unsubscribe_symbols(&symbols).await;
        });
    }
}

/// Wraps a filtered `PriceStream` and holds a `SubscriptionGuard` so symbols
/// are unsubscribed when the stream is dropped.
struct GuardedStream {
    _guard: SubscriptionGuard,
    inner: Pin<Box<dyn Stream<Item = GqlPriceUpdate> + Send>>,
}

// SAFETY: `Pin<Box<T>>` is always `Unpin` (the box pointer can be moved),
// and `SubscriptionGuard` contains no pinned fields.
impl Unpin for GuardedStream {}

impl Stream for GuardedStream {
    type Item = GqlPriceUpdate;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

/// RAII guard that unsubscribes sources when the GraphQL subscription is dropped.
struct FeedSubscriptionGuard {
    hub: FeedHub,
    sources: Vec<FeedSource>,
}

impl Drop for FeedSubscriptionGuard {
    fn drop(&mut self) {
        let hub = self.hub.clone();
        let sources = std::mem::take(&mut self.sources);
        tokio::spawn(async move {
            hub.unsubscribe_sources(&sources).await;
        });
    }
}

/// Wraps a filtered `NewsStream` and holds a `FeedSubscriptionGuard` so
/// sources are unsubscribed when the stream is dropped.
struct GuardedFeedStream {
    _guard: FeedSubscriptionGuard,
    inner: Pin<Box<dyn Stream<Item = GqlFeedEntry> + Send>>,
}

// SAFETY: `Pin<Box<T>>` is always `Unpin` (the box pointer can be moved),
// and `FeedSubscriptionGuard` contains no pinned fields.
impl Unpin for GuardedFeedStream {}

impl Stream for GuardedFeedStream {
    type Item = GqlFeedEntry;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}
