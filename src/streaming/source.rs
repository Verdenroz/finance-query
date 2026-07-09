//! Pluggable streaming-source abstraction.
//!
//! Separates the provider-specific transport + wire protocol (connect,
//! subscribe, decode) from the generic machinery — reconnection, the
//! subscription set, and the public [`PriceStream`](super::PriceStream) API.
//! Yahoo is the reference implementation in [`super::yahoo`]; additional
//! backends (e.g. Polygon) implement the same trait.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{RwLock, broadcast, mpsc};
use tracing::{error, info};

use super::client::StreamResult;
use super::pricing::PriceUpdate;

/// Commands sent to a running streaming session.
pub(crate) enum StreamCommand {
    /// Add symbols to the subscription.
    Subscribe(Vec<String>),
    /// Remove symbols from the subscription.
    Unsubscribe(Vec<String>),
    /// Close the session and stop reconnecting.
    Close,
}

/// A real-time price source backing a [`PriceStream`](super::PriceStream).
///
/// Implementations own the transport and wire protocol and push decoded
/// [`PriceUpdate`]s onto `broadcast_tx`. Reconnection and the public stream
/// API are provided generically by [`run_stream_loop`].
#[async_trait::async_trait]
pub(crate) trait StreamSource: Send + Sync + 'static {
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
        broadcast_tx: &broadcast::Sender<PriceUpdate>,
        command_rx: &mut mpsc::Receiver<StreamCommand>,
    ) -> StreamResult<()>;
}

/// Drive a [`StreamSource`] with automatic reconnection until it shuts down.
pub(crate) async fn run_stream_loop(
    source: Arc<dyn StreamSource>,
    initial_symbols: Vec<String>,
    broadcast_tx: broadcast::Sender<PriceUpdate>,
    mut command_rx: mpsc::Receiver<StreamCommand>,
    retry_delay: Duration,
) -> StreamResult<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::client::PriceStream;
    use crate::streaming::pricing::PricingData;
    use futures::StreamExt;

    /// A network-free source that emits one synthetic update per subscribed
    /// symbol, then stays alive until a Close command.
    struct MockSource;

    #[async_trait::async_trait]
    impl StreamSource for MockSource {
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
}
