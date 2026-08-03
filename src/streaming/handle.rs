//! Shared plumbing behind every public stream handle.
//!
//! Wraps the generic broadcast [`Subscription`] plus the reconnect loop into
//! one reusable, item-generic handle so `PriceStream`, `TradeStream`,
//! `DepthStream` and `OptionsChainStream` are thin newtypes over the same
//! machinery rather than four copies of it.

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::stream::Stream;

use super::source::{StreamCommand, StreamSource, run_stream_loop};
use super::subscription::Subscription;

/// Command-channel depth: control messages are rare compared to data.
const COMMAND_CAPACITY: usize = 32;

/// A `Stream<Item = T>` fed by a [`StreamSource`] with automatic reconnection.
pub(crate) struct SourceStream<T>
where
    T: Clone + Send + 'static,
{
    inner: Subscription<T, StreamCommand>,
}

impl<T> SourceStream<T>
where
    T: Clone + Send + 'static,
{
    /// Spawn the source's reconnect loop and return a handle to its output.
    pub(crate) fn start(
        source: Arc<dyn StreamSource<T>>,
        symbols: Vec<String>,
        retry_delay: Duration,
        capacity: usize,
    ) -> Self {
        let inner = Subscription::start(
            capacity,
            COMMAND_CAPACITY,
            move |broadcast_tx, command_rx| async move {
                let _ =
                    run_stream_loop(source, symbols, broadcast_tx, command_rx, retry_delay).await;
            },
        );
        SourceStream { inner }
    }

    /// Create an independent receiver sharing the same background task.
    pub(crate) fn resubscribe(&self) -> Self {
        SourceStream {
            inner: self.inner.resubscribe(),
        }
    }

    /// Add symbols to the live subscription.
    pub(crate) async fn add<S, I>(&self, symbols: I)
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        let symbols: Vec<String> = symbols.into_iter().map(Into::into).collect();
        self.inner.send(StreamCommand::Subscribe(symbols)).await;
    }

    /// Remove symbols from the live subscription.
    pub(crate) async fn remove<S, I>(&self, symbols: I)
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        let symbols: Vec<String> = symbols.into_iter().map(Into::into).collect();
        self.inner.send(StreamCommand::Unsubscribe(symbols)).await;
    }

    /// Close the session and stop reconnecting.
    pub(crate) async fn close(&self) {
        self.inner.send(StreamCommand::Close).await;
    }
}

impl<T> Stream for SourceStream<T>
where
    T: Clone + Send + 'static,
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}
