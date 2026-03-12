//! GraphQL `SubscriptionRoot` — real-time price stream.

use std::collections::HashSet;
use std::pin::Pin;
use std::task;

use async_graphql::{Context, Error, Result, Subscription};
use futures_util::{Stream, StreamExt};

use super::types::streaming::GqlPriceUpdate;
use crate::{AppState, StreamHub};

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
