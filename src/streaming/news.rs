//! Continuous RSS/Atom polling exposed as a `Stream`, for a source that's
//! inherently pull-only (RSS/Atom has no server push): a background task
//! polls the configured sources on an interval and broadcasts newly-seen
//! entries (deduplicated by URL) to every subscriber.

use std::collections::{HashSet, VecDeque};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::stream::Stream;
use tokio::sync::{broadcast, mpsc};
use tracing::warn;

use super::subscription::Subscription;
use crate::feeds::{FeedEntry, FeedSource, fetch_all};

/// Default interval between polls of all subscribed sources.
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(300);

/// Channel capacity for news entries (much lower churn than price ticks).
const CHANNEL_CAPACITY: usize = 256;

/// Cap on remembered entry URLs used for new-vs-seen dedup, bounding memory
/// for long-running subscriptions.
const SEEN_CAP: usize = 2000;

enum FeedCommand {
    AddSources(Vec<FeedSource>),
    RemoveSources(Vec<String>),
    Close,
}

/// A continuous subscription to one or more RSS/Atom sources.
///
/// Polls the configured sources on an interval (5 minutes by default,
/// configurable via [`NewsStreamBuilder::poll_interval`]) and yields entries
/// as they're first seen: an initial batch on subscribe, then only new items
/// on each subsequent poll. Backed by a broadcast channel, so
/// [`resubscribe`](Self::resubscribe) supports multiple independent
/// consumers of the same subscription.
///
/// # Example
///
/// ```no_run
/// use finance_query::streaming::NewsStream;
/// use finance_query::feeds::FeedSource;
/// use futures::StreamExt;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut stream =
///     NewsStream::subscribe([FeedSource::Bloomberg, FeedSource::MarketWatch]).await;
///
/// while let Some(entry) = stream.next().await {
///     println!("[{}] {}", entry.source, entry.title);
/// }
/// # Ok(())
/// # }
/// ```
pub struct NewsStream {
    inner: Subscription<FeedEntry, FeedCommand>,
}

impl NewsStream {
    /// Subscribe to a continuous stream of new entries from the given
    /// sources, polling every 5 minutes. Use [`NewsStreamBuilder`] to
    /// customize the poll interval.
    pub async fn subscribe(sources: impl IntoIterator<Item = FeedSource>) -> Self {
        NewsStreamBuilder::new().sources(sources).build().await
    }

    async fn start(sources: Vec<FeedSource>, poll_interval: Duration) -> Self {
        let inner = Subscription::start(CHANNEL_CAPACITY, 32, move |broadcast_tx, command_rx| {
            run_feed_loop(sources, poll_interval, broadcast_tx, command_rx)
        });
        NewsStream { inner }
    }

    /// Create a new receiver for this stream.
    ///
    /// Useful when you need multiple consumers of the same news subscription.
    pub fn resubscribe(&self) -> Self {
        NewsStream {
            inner: self.inner.resubscribe(),
        }
    }

    /// Add more sources to the subscription.
    pub async fn add_sources(&self, sources: impl IntoIterator<Item = FeedSource>) {
        self.inner
            .send(FeedCommand::AddSources(sources.into_iter().collect()))
            .await;
    }

    /// Remove sources from the subscription (matched by [`FeedSource::url`]).
    pub async fn remove_sources(&self, sources: impl IntoIterator<Item = FeedSource>) {
        let urls = sources.into_iter().map(|s| s.url()).collect();
        self.inner.send(FeedCommand::RemoveSources(urls)).await;
    }

    /// Stop polling and close the stream.
    pub async fn close(&self) {
        self.inner.send(FeedCommand::Close).await;
    }
}

impl Stream for NewsStream {
    type Item = FeedEntry;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// Builder for creating a [`NewsStream`] with custom configuration.
pub struct NewsStreamBuilder {
    sources: Vec<FeedSource>,
    poll_interval: Duration,
}

impl NewsStreamBuilder {
    /// Create a new builder with no sources and the default 5-minute poll interval.
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            poll_interval: DEFAULT_POLL_INTERVAL,
        }
    }

    /// Add sources to subscribe to.
    pub fn sources(mut self, sources: impl IntoIterator<Item = FeedSource>) -> Self {
        self.sources.extend(sources);
        self
    }

    /// Set the interval between polls of all subscribed sources (default: 5 minutes).
    pub fn poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Start the news stream.
    pub async fn build(self) -> NewsStream {
        NewsStream::start(self.sources, self.poll_interval).await
    }
}

impl Default for NewsStreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

async fn run_feed_loop(
    initial_sources: Vec<FeedSource>,
    poll_interval: Duration,
    broadcast_tx: broadcast::Sender<FeedEntry>,
    mut command_rx: mpsc::Receiver<FeedCommand>,
) {
    let mut sources: Vec<(String, FeedSource)> =
        initial_sources.into_iter().map(|s| (s.url(), s)).collect();
    let mut seen: HashSet<String> = HashSet::new();
    let mut seen_order: VecDeque<String> = VecDeque::new();

    let mut ticker = tokio::time::interval(poll_interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if sources.is_empty() {
                    continue;
                }
                let batch = sources.iter().map(|(_, s)| s.clone());
                match fetch_all(batch).await {
                    Ok(entries) => {
                        for entry in entries {
                            if seen.insert(entry.url.clone()) {
                                seen_order.push_back(entry.url.clone());
                                if seen_order.len() > SEEN_CAP
                                    && let Some(old) = seen_order.pop_front()
                                {
                                    seen.remove(&old);
                                }
                                let _ = broadcast_tx.send(entry);
                            }
                        }
                    }
                    Err(e) => warn!("news stream poll failed: {e}"),
                }
            }
            cmd = command_rx.recv() => {
                match cmd {
                    Some(FeedCommand::AddSources(new_sources)) => {
                        for s in new_sources {
                            let url = s.url();
                            if let Some(existing) = sources.iter_mut().find(|(u, _)| *u == url) {
                                existing.1 = s;
                            } else {
                                sources.push((url, s));
                            }
                        }
                    }
                    Some(FeedCommand::RemoveSources(urls)) => {
                        sources.retain(|(u, _)| !urls.contains(u));
                    }
                    Some(FeedCommand::Close) | None => break,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn add_and_remove_sources_are_accepted() {
        // No network: sources list is empty so the poll tick is a no-op;
        // this only exercises the command channel plumbing.
        let stream = NewsStream::subscribe([]).await;
        stream.add_sources([FeedSource::Bloomberg]).await;
        stream.remove_sources([FeedSource::Bloomberg]).await;
        stream.close().await;
    }

    #[tokio::test]
    async fn resubscribe_gives_an_independent_receiver() {
        let stream = NewsStream::subscribe([]).await;
        let other = stream.resubscribe();
        drop(other);
        stream.close().await;
    }

    #[tokio::test]
    async fn close_ends_the_stream() {
        let mut stream = NewsStreamBuilder::new()
            .poll_interval(Duration::from_millis(20))
            .build()
            .await;
        stream.close().await;
        let timeout = tokio::time::timeout(Duration::from_secs(2), stream.next()).await;
        assert!(matches!(timeout, Ok(None)));
    }
}
