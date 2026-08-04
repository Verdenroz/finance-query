//! GraphQL `SubscriptionRoot` — real-time price and feed streams.

use std::collections::HashSet;
use std::pin::Pin;
use std::task;

use async_graphql::{Context, Error, Result, Subscription};
use finance_query::feeds::FeedSource;
use futures_util::{Stream, StreamExt};

use finance_query::streaming::{AlertEvaluator, AlertRule};

use super::types::feeds::GqlFeedEntry;
use super::types::streaming::{GqlAlertEvent, GqlAlertRuleInput, GqlPriceUpdate};
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
            .filter(move |t| std::future::ready(subscribed.contains(&t.update().id)))
            .map(|t| GqlPriceUpdate::from(t.update().clone()));

        Ok(Guarded {
            _guard: guard,
            inner: Box::pin(filtered),
        })
    }

    /// Subscribe to threshold-triggered price alerts.
    ///
    /// Same upstream as `priceStream`, but only ticks that satisfy one of the
    /// supplied rules are sent — the predicate is evaluated here rather than
    /// pushing every tick to the client. Symbols are derived from the rules
    /// and ref-counted through the same hub.
    async fn price_alerts(
        &self,
        ctx: &Context<'_>,
        rules: Vec<GqlAlertRuleInput>,
    ) -> Result<impl Stream<Item = GqlAlertEvent>> {
        if rules.is_empty() {
            return Err(Error::new("at least one alert rule is required"));
        }

        let state = ctx.data::<AppState>()?;
        let hub = state.stream_hub.clone();

        let rules: Vec<AlertRule> = rules.into_iter().map(AlertRule::from).collect();
        let mut evaluator = AlertEvaluator::new(rules);
        let symbols = evaluator.symbols();

        hub.subscribe_symbols(&symbols)
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        let hub_stream = hub
            .resubscribe()
            .await
            .ok_or_else(|| Error::new("Price stream unavailable"))?;

        // Pre-filter on the rules' symbols: the hub carries every symbol any
        // client anywhere subscribed to.
        let watched: HashSet<String> = symbols.iter().cloned().collect();
        let guard = SubscriptionGuard {
            hub: hub.clone(),
            symbols,
        };

        let alerts = hub_stream
            .filter(move |t| std::future::ready(watched.contains(&t.update().id)))
            .flat_map(move |tick| {
                futures_util::stream::iter(
                    evaluator
                        .evaluate(tick.update())
                        .into_iter()
                        .map(GqlAlertEvent::from)
                        .collect::<Vec<_>>(),
                )
            })
            .boxed();

        Ok(Guarded {
            _guard: guard,
            inner: alerts,
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

        Ok(Guarded {
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

/// A stream that keeps an RAII guard alive for as long as it is polled, so
/// dropping the subscription releases whatever the guard holds.
struct Guarded<T, G> {
    _guard: G,
    inner: Pin<Box<dyn Stream<Item = T> + Send>>,
}

// SAFETY: `Pin<Box<T>>` is always `Unpin` (the box pointer can be moved), and
// the guards stored here contain no pinned fields.
impl<T, G> Unpin for Guarded<T, G> {}

impl<T, G> Stream for Guarded<T, G> {
    type Item = T;

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
