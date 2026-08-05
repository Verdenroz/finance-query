//! Shared plumbing behind every public stream handle.
//!
//! Wraps the generic broadcast [`Subscription`] plus the reconnect loop into
//! one reusable, item-generic handle so `PriceStream`, `TradeStream`,
//! `DepthStream`, `OptionsChainStream` and `EconomicStream` are thin newtypes
//! over the same machinery rather than five copies of it. The
//! [`stream_handle!`] and [`stream_builder!`] macros emit the delegating
//! methods, `Stream` impl and builder boilerplate those newtypes would
//! otherwise repeat verbatim.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
#[cfg(feature = "polygon")]
use std::time::Duration;

use futures::stream::Stream;
use tokio::sync::{broadcast, mpsc};

use super::source::{ReconnectConfig, StreamCommand, StreamSource, run_stream_loop};
use super::subscription::Subscription;

/// Command-channel depth: control messages are rare compared to data.
const COMMAND_CAPACITY: usize = 32;

/// Delay between reconnection attempts, shared by every socket-backed handle.
#[cfg(feature = "polygon")]
pub(crate) const RECONNECT_BACKOFF: Duration = Duration::from_secs(3);

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
        reconnect: ReconnectConfig,
        capacity: usize,
    ) -> Self {
        Self::spawn(capacity, move |broadcast_tx, command_rx| async move {
            let _ = run_stream_loop(source, symbols, broadcast_tx, command_rx, reconnect).await;
        })
    }

    /// Spawn an arbitrary background loop behind the same handle API.
    ///
    /// For producers with no socket to reconnect (a poll loop, say) that still
    /// want subscribe/unsubscribe/close and shared receivers.
    pub(crate) fn spawn<F, Fut>(capacity: usize, run: F) -> Self
    where
        F: FnOnce(broadcast::Sender<T>, mpsc::Receiver<StreamCommand>) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        SourceStream {
            inner: Subscription::start(capacity, COMMAND_CAPACITY, run),
        }
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

/// Emit a public stream handle over [`SourceStream`].
///
/// The handles differ only in item type and in the noun their add/remove
/// methods use, so everything else — the newtype, `resubscribe`, `close` and
/// the `Stream` impl — is generated here instead of copied per file.
#[cfg(any(feature = "polygon", feature = "fred"))]
macro_rules! stream_handle {
    (
        $(#[$attr:meta])*
        $name:ident($item:ty);
        add: $add:ident = $add_doc:literal,
        remove: $remove:ident = $remove_doc:literal,
    ) => {
        $(#[$attr])*
        pub struct $name {
            inner: $crate::streaming::handle::SourceStream<$item>,
        }

        impl $name {
            /// Create an independent receiver sharing this subscription's
            /// background task.
            pub fn resubscribe(&self) -> Self {
                Self {
                    inner: self.inner.resubscribe(),
                }
            }

            #[doc = $add_doc]
            pub async fn $add<S, I>(&self, symbols: I)
            where
                S: Into<String>,
                I: IntoIterator<Item = S>,
            {
                self.inner.add(symbols).await;
            }

            #[doc = $remove_doc]
            pub async fn $remove<S, I>(&self, symbols: I)
            where
                S: Into<String>,
                I: IntoIterator<Item = S>,
            {
                self.inner.remove(symbols).await;
            }

            /// Close the stream and stop its background task.
            pub async fn close(&self) {
                self.inner.close().await;
            }
        }

        impl ::futures::stream::Stream for $name {
            type Item = $item;

            fn poll_next(
                mut self: ::std::pin::Pin<&mut Self>,
                cx: &mut ::std::task::Context<'_>,
            ) -> ::std::task::Poll<Option<Self::Item>> {
                ::futures::stream::Stream::poll_next(
                    ::std::pin::Pin::new(&mut self.inner),
                    cx,
                )
            }
        }
    };
}

/// Emit the symbol-accumulating setter, `retry`, `max_reconnect_attempts` and
/// `Default` for a builder whose fields are `$field: Vec<String>`,
/// `retry_delay: Duration` and `max_reconnect_attempts: Option<u32>`.
#[cfg(feature = "polygon")]
macro_rules! stream_builder {
    ($builder:ident, $field:ident = $doc:literal) => {
        impl $builder {
            #[doc = $doc]
            pub fn $field<S, I>(mut self, symbols: I) -> Self
            where
                S: Into<String>,
                I: IntoIterator<Item = S>,
            {
                self.$field.extend(symbols.into_iter().map(Into::into));
                self
            }

            /// Set the base delay before the first reconnection attempt
            /// (default: 3s). Later attempts grow exponentially from this,
            /// capped and jittered — see [`Self::max_reconnect_attempts`] to
            /// also cap how many attempts are made.
            pub fn retry(mut self, delay: ::std::time::Duration) -> Self {
                self.retry_delay = delay;
                self
            }

            /// Cap the number of consecutive reconnect attempts before the
            /// stream gives up and ends (default: unlimited, i.e. retry
            /// forever like before issue #276).
            pub fn max_reconnect_attempts(mut self, max: u32) -> Self {
                self.max_reconnect_attempts = Some(max);
                self
            }
        }

        impl Default for $builder {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

#[cfg(feature = "polygon")]
pub(crate) use stream_builder;
#[cfg(any(feature = "polygon", feature = "fred"))]
pub(crate) use stream_handle;
