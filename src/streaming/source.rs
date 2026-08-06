//! Pluggable streaming-source abstraction.
//!
//! Separates the provider-specific transport + wire protocol (connect,
//! subscribe, decode) from the generic machinery — reconnection, the
//! subscription set, and the public stream handles. The trait is generic over
//! the item type so price ticks, trade prints, order books and options
//! contracts all reuse one reconnect loop.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{RwLock, broadcast, mpsc};
use tracing::{error, info};

use super::client::StreamResult;

/// Commands sent to a running streaming session.
pub(crate) enum StreamCommand {
    /// Add symbols to the subscription.
    Subscribe(Vec<String>),
    /// Remove symbols from the subscription.
    Unsubscribe(Vec<String>),
    /// Close the session and stop reconnecting.
    Close,
}

/// A real-time source backing one of the public stream handles.
///
/// Implementations own the transport and wire protocol and push decoded items
/// of type `T` onto `broadcast_tx`. Reconnection and the public stream API are
/// provided generically by [`run_stream_loop`].
#[async_trait::async_trait]
pub(crate) trait StreamSource<T>: Send + Sync + 'static
where
    T: Clone + Send + 'static,
{
    /// Short identifier for logging (e.g. `"yahoo"`).
    fn id(&self) -> &'static str;

    /// Run one connected session until it ends.
    ///
    /// Returns `Ok(())` for a graceful shutdown (a [`StreamCommand::Close`] or a
    /// server close frame) — the loop stops. Returns `Err(..)` for a recoverable
    /// disconnect — the loop reconnects after a backoff. The session should
    /// honor `command_rx` for live (un)subscribe and reflect changes into the
    /// shared `subscriptions` set.
    async fn run_session(
        &self,
        subscriptions: &Arc<RwLock<HashSet<String>>>,
        broadcast_tx: &broadcast::Sender<T>,
        command_rx: &mut mpsc::Receiver<StreamCommand>,
    ) -> StreamResult<()>;
}

/// Drive a [`StreamSource`] with automatic reconnection until it shuts down.
///
/// `retry_delay` is supplied by the caller rather than hard-coded so a smarter
/// policy can be swapped in without touching sources.
pub(crate) async fn run_stream_loop<T>(
    source: Arc<dyn StreamSource<T>>,
    initial_symbols: Vec<String>,
    broadcast_tx: broadcast::Sender<T>,
    mut command_rx: mpsc::Receiver<StreamCommand>,
    retry_delay: Duration,
) -> StreamResult<()>
where
    T: Clone + Send + 'static,
{
    let subscriptions = Arc::new(RwLock::new(HashSet::<String>::from_iter(initial_symbols)));

    loop {
        match source
            .run_session(&subscriptions, &broadcast_tx, &mut command_rx)
            .await
        {
            Ok(()) => {
                info!("{} stream closed gracefully", source.id());
                break;
            }
            Err(e) => {
                error!(
                    "{} stream error: {}, reconnecting in {:.1}s...",
                    source.id(),
                    e,
                    retry_delay.as_secs_f32()
                );
                tokio::time::sleep(retry_delay).await;
            }
        }
    }

    Ok(())
}

/// Apply a [`StreamCommand`] to the shared subscription set.
///
/// Returns the symbols that actually changed state, so sources only send
/// wire-level (un)subscribes for real deltas. `Close` returns `None`.
pub(crate) async fn apply_command(
    command: &StreamCommand,
    subscriptions: &Arc<RwLock<HashSet<String>>>,
) -> Option<Vec<String>> {
    match command {
        StreamCommand::Subscribe(symbols) => {
            let mut subs = subscriptions.write().await;
            Some(
                symbols
                    .iter()
                    .filter(|s| subs.insert((*s).clone()))
                    .cloned()
                    .collect(),
            )
        }
        StreamCommand::Unsubscribe(symbols) => {
            let mut subs = subscriptions.write().await;
            Some(
                symbols
                    .iter()
                    .filter(|s| subs.remove(*s))
                    .cloned()
                    .collect(),
            )
        }
        StreamCommand::Close => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::client::PriceStream;
    use crate::streaming::pricing::{PriceUpdate, PricingData};
    use futures::StreamExt;

    /// A network-free source that emits one synthetic update per subscribed
    /// symbol, then stays alive until a Close command.
    struct MockSource;

    #[async_trait::async_trait]
    impl StreamSource<PriceUpdate> for MockSource {
        fn id(&self) -> &'static str {
            "mock"
        }

        async fn run_session(
            &self,
            subscriptions: &Arc<RwLock<HashSet<String>>>,
            broadcast_tx: &broadcast::Sender<PriceUpdate>,
            command_rx: &mut mpsc::Receiver<StreamCommand>,
        ) -> StreamResult<()> {
            let subs: Vec<String> = subscriptions.read().await.iter().cloned().collect();
            for sym in subs {
                let data = PricingData {
                    id: sym,
                    price: 42.0,
                    ..Default::default()
                };
                let _ = broadcast_tx.send(data.into());
            }
            while let Some(cmd) = command_rx.recv().await {
                if matches!(cmd, StreamCommand::Close) {
                    return Ok(());
                }
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn pluggable_source_delivers_updates() {
        let mut stream = PriceStream::subscribe_with_source(
            Arc::new(MockSource),
            ["AAPL"],
            Duration::from_millis(50),
        )
        .await
        .unwrap();

        let update = tokio::time::timeout(Duration::from_secs(2), stream.next())
            .await
            .expect("timed out waiting for update")
            .expect("stream ended without an update");

        assert_eq!(update.id, "AAPL");
        assert_eq!(update.price, 42.0);
        stream.close().await;
    }

    #[tokio::test]
    async fn apply_command_reports_only_real_deltas() {
        let subs = Arc::new(RwLock::new(HashSet::from(["AAPL".to_string()])));

        let added = apply_command(
            &StreamCommand::Subscribe(vec!["AAPL".into(), "NVDA".into()]),
            &subs,
        )
        .await;
        assert_eq!(added, Some(vec!["NVDA".to_string()]));

        let removed = apply_command(
            &StreamCommand::Unsubscribe(vec!["NVDA".into(), "TSLA".into()]),
            &subs,
        )
        .await;
        assert_eq!(removed, Some(vec!["NVDA".to_string()]));

        assert!(apply_command(&StreamCommand::Close, &subs).await.is_none());
    }
}
