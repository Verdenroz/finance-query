//! Generic broadcast-channel-backed `Stream` subscription primitive.
//!
//! A background task owns a data source and pushes items onto a `broadcast`
//! channel; a command channel lets the public handle send live control
//! messages into that task. [`Subscription::resubscribe`] hands out
//! additional broadcast receivers so multiple consumers can share one
//! background task.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::stream::Stream;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::BroadcastStream;
use tracing::warn;

struct SubscriptionHandle<T, C> {
    command_tx: mpsc::Sender<C>,
    // A spare *receiver*, not a sender: minting new receivers via
    // `resubscribe()` needs some existing subscription to branch off of, but
    // a sender held here would keep the channel open even after the
    // background task drops its own sender — the stream would never end.
    resubscribe_source: broadcast::Receiver<T>,
}

/// A `Stream<Item = T>` backed by a broadcast channel, plus a command
/// channel of type `C` for driving the background task that produces items.
pub(crate) struct Subscription<T, C> {
    inner: BroadcastStream<T>,
    handle: Arc<SubscriptionHandle<T, C>>,
}

impl<T, C> Subscription<T, C>
where
    T: Clone + Send + 'static,
    C: Send + 'static,
{
    /// Start a new subscription: spawns `run` as a background task wired to
    /// fresh broadcast/command channels.
    ///
    /// `run` is handed the broadcast sender (to publish items) and the
    /// command receiver (to react to live control messages); it owns the
    /// data source for as long as the task runs.
    pub(crate) fn start<F, Fut>(broadcast_capacity: usize, command_capacity: usize, run: F) -> Self
    where
        F: FnOnce(broadcast::Sender<T>, mpsc::Receiver<C>) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let (broadcast_tx, broadcast_rx) = broadcast::channel(broadcast_capacity);
        let (command_tx, command_rx) = mpsc::channel(command_capacity);
        let resubscribe_source = broadcast_tx.subscribe();

        tokio::spawn(run(broadcast_tx, command_rx));

        let handle = Arc::new(SubscriptionHandle {
            command_tx,
            resubscribe_source,
        });

        Subscription {
            inner: BroadcastStream::new(broadcast_rx),
            handle,
        }
    }

    /// Create a new receiver sharing this subscription's background task.
    pub(crate) fn resubscribe(&self) -> Self {
        Subscription {
            inner: BroadcastStream::new(self.handle.resubscribe_source.resubscribe()),
            handle: Arc::clone(&self.handle),
        }
    }

    /// Send a command to the background task (best-effort — dropped if the
    /// task has already ended).
    pub(crate) async fn send(&self, command: C) {
        let _ = self.handle.command_tx.send(command).await;
    }
}

impl<T, C> Stream for Subscription<T, C>
where
    T: Clone + Send + 'static,
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(item))) => Poll::Ready(Some(item)),
            Poll::Ready(Some(Err(e))) => {
                warn!("Broadcast subscription lagged: {:?}", e);
                // A lag means the receiver missed items, not that the stream
                // ended — retry immediately so the caller sees the next item.
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
