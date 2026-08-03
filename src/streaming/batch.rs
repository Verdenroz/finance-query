//! Batched delivery for any streaming subscription.
//!
//! A consumer watching N symbols normally receives N separate messages per
//! tick. [`Batched`] coalesces whatever arrived inside a time window into one
//! `Vec`, cutting per-message overhead for wide watchlists. It is a plain
//! `Stream` adapter, so it composes with every handle in this module instead
//! of being a per-stream delivery mode.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::stream::Stream;
use tokio::time::{Instant, Sleep, sleep_until};

/// Default cap on items per batch when none is given.
const DEFAULT_MAX_BATCH: usize = 512;

/// A stream that yields `Vec<T>` batches collected over a time window.
///
/// A batch is emitted when the window since the batch's first item elapses, or
/// as soon as `max_items` is reached — whichever comes first. Empty batches are
/// never emitted, and any partial batch is flushed when the source ends.
pub struct Batched<S>
where
    S: Stream,
{
    inner: S,
    window: Duration,
    max_items: usize,
    buffer: Vec<S::Item>,
    deadline: Option<Pin<Box<Sleep>>>,
    exhausted: bool,
}

impl<S> Batched<S>
where
    S: Stream + Unpin,
{
    /// Batch `inner` over `window`, emitting early once `max_items` accumulate.
    pub fn new(inner: S, window: Duration, max_items: usize) -> Self {
        Self {
            inner,
            window,
            max_items: max_items.max(1),
            buffer: Vec::new(),
            deadline: None,
            exhausted: false,
        }
    }

    fn take_batch(&mut self) -> Vec<S::Item> {
        self.deadline = None;
        std::mem::take(&mut self.buffer)
    }
}

impl<S> Stream for Batched<S>
where
    S: Stream + Unpin,
    S::Item: Unpin,
{
    type Item = Vec<S::Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            if this.exhausted {
                return if this.buffer.is_empty() {
                    Poll::Ready(None)
                } else {
                    Poll::Ready(Some(this.take_batch()))
                };
            }

            match Pin::new(&mut this.inner).poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    this.buffer.push(item);
                    if this.deadline.is_none() {
                        this.deadline = Some(Box::pin(sleep_until(Instant::now() + this.window)));
                    }
                    if this.buffer.len() >= this.max_items {
                        return Poll::Ready(Some(this.take_batch()));
                    }
                }
                Poll::Ready(None) => this.exhausted = true,
                Poll::Pending => break,
            }
        }

        // Source is idle: emit once the current batch's window expires.
        if let Some(deadline) = this.deadline.as_mut()
            && deadline.as_mut().poll(cx).is_ready()
        {
            return Poll::Ready(Some(this.take_batch()));
        }

        Poll::Pending
    }
}

/// Adds [`batched`](StreamBatchExt::batched) to every `Stream`.
///
/// # Example
///
/// ```no_run
/// use finance_query::streaming::{PriceStream, StreamBatchExt};
/// use futures::StreamExt;
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut batches = PriceStream::subscribe(["AAPL", "NVDA", "TSLA"])
///     .await?
///     .batched(Duration::from_millis(250));
///
/// while let Some(batch) = batches.next().await {
///     println!("{} updates in this window", batch.len());
/// }
/// # Ok(())
/// # }
/// ```
pub trait StreamBatchExt: Stream + Sized + Unpin {
    /// Coalesce items arriving within `window` into one `Vec` (max 512 items).
    fn batched(self, window: Duration) -> Batched<Self> {
        Batched::new(self, window, DEFAULT_MAX_BATCH)
    }

    /// Same as [`batched`](Self::batched) with an explicit per-batch cap.
    fn batched_with_capacity(self, window: Duration, max_items: usize) -> Batched<Self> {
        Batched::new(self, window, max_items)
    }
}

impl<S> StreamBatchExt for S where S: Stream + Sized + Unpin {}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use tokio::sync::mpsc;
    use tokio_stream::wrappers::ReceiverStream;

    #[tokio::test]
    async fn window_coalesces_items_into_one_batch() {
        let (tx, rx) = mpsc::channel(16);
        let mut batched = ReceiverStream::new(rx).batched(Duration::from_millis(60));

        for i in 0..5 {
            tx.send(i).await.unwrap();
        }

        let batch = tokio::time::timeout(Duration::from_secs(2), batched.next())
            .await
            .expect("timed out")
            .expect("stream ended");
        assert_eq!(batch, vec![0, 1, 2, 3, 4]);
        drop(tx);
    }

    #[tokio::test]
    async fn max_items_flushes_before_the_window_elapses() {
        let (tx, rx) = mpsc::channel(16);
        let mut batched = ReceiverStream::new(rx).batched_with_capacity(Duration::from_secs(30), 2);

        for i in 0..4 {
            tx.send(i).await.unwrap();
        }

        // Would block for 30s if the size cap were not honored.
        let batch = tokio::time::timeout(Duration::from_secs(2), batched.next())
            .await
            .expect("timed out")
            .expect("stream ended");
        assert_eq!(batch, vec![0, 1]);
        drop(tx);
    }

    #[tokio::test]
    async fn partial_batch_is_flushed_when_the_source_ends() {
        let (tx, rx) = mpsc::channel(16);
        let mut batched = ReceiverStream::new(rx).batched(Duration::from_secs(30));
        tx.send(7).await.unwrap();
        drop(tx);

        let batch = tokio::time::timeout(Duration::from_secs(2), batched.next())
            .await
            .expect("timed out")
            .expect("stream ended");
        assert_eq!(batch, vec![7]);

        let end = tokio::time::timeout(Duration::from_secs(2), batched.next())
            .await
            .expect("timed out");
        assert!(end.is_none());
    }

    #[tokio::test]
    async fn empty_batches_are_never_emitted() {
        let (tx, rx) = mpsc::channel::<u8>(1);
        let mut batched = ReceiverStream::new(rx).batched(Duration::from_millis(20));

        let idle = tokio::time::timeout(Duration::from_millis(150), batched.next()).await;
        assert!(idle.is_err(), "idle source must not emit empty batches");
        drop(tx);
    }
}
