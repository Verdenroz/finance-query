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

/// Exponential-backoff-with-jitter configuration for [`run_stream_loop`]'s
/// reconnect delay.
///
/// `base_delay` is the same knob every builder's `.retry(Duration)` method has
/// always exposed — the delay before the first reconnect attempt. Later
/// attempts grow by `multiplier` each time (capped at `max_delay`), with
/// jitter applied so many concurrently-reconnecting streams don't retry in
/// lockstep. `max_attempts` optionally stops the loop from reconnecting
/// forever; `None` (the default) preserves the original "retry forever"
/// behavior.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ReconnectConfig {
    base_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
    jitter: f64,
    max_attempts: Option<u32>,
    healthy_after: Duration,
}

impl ReconnectConfig {
    pub(crate) fn new(base_delay: Duration) -> Self {
        Self {
            base_delay,
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            jitter: 0.2,
            max_attempts: None,
            healthy_after: Duration::from_secs(60),
        }
    }

    /// Cap the number of consecutive reconnect attempts before the loop gives
    /// up and ends. `None` (the default) retries forever.
    pub(crate) fn max_attempts(mut self, max: Option<u32>) -> Self {
        self.max_attempts = max;
        self
    }

    /// How long a session must stay up before its failure resets the attempt
    /// counter. Test hook — production uses the 60s default.
    #[cfg(test)]
    pub(crate) fn healthy_after(mut self, healthy_after: Duration) -> Self {
        self.healthy_after = healthy_after;
        self
    }

    /// Delay before the reconnect attempt numbered `attempt` (0-indexed).
    fn delay_for(&self, attempt: u32, seed: &mut u64) -> Duration {
        crate::backoff::BackoffParams {
            base: self.base_delay,
            max: self.max_delay,
            multiplier: self.multiplier,
            jitter: self.jitter,
        }
        .delay_for(attempt, seed)
    }
}

/// Drive a [`StreamSource`] with automatic reconnection until it shuts down.
///
/// `reconnect` is supplied by the caller rather than hard-coded so builders
/// can tune the base delay and attempt cap without touching sources. A session
/// that stays connected for `healthy_after` is treated as healthy — its
/// disconnect resets the backoff, so a single brief blip doesn't inherit a
/// long delay accumulated from an earlier outage. That threshold is
/// deliberately independent of `base_delay`: keying it to the (short) base
/// delay would let a source that accepts the connection and then fails a few
/// seconds in — auth rejection, subscription error, idle kill — reset the
/// counter every cycle and reconnect forever despite `max_attempts`.
pub(crate) async fn run_stream_loop<T>(
    source: Arc<dyn StreamSource<T>>,
    initial_symbols: Vec<String>,
    broadcast_tx: broadcast::Sender<T>,
    mut command_rx: mpsc::Receiver<StreamCommand>,
    reconnect: ReconnectConfig,
) -> StreamResult<()>
where
    T: Clone + Send + 'static,
{
    let subscriptions = Arc::new(RwLock::new(HashSet::<String>::from_iter(initial_symbols)));
    let mut attempt: u32 = 0;
    let mut seed = crate::backoff::seed_from_time();

    loop {
        let session_start = tokio::time::Instant::now();
        match source
            .run_session(&subscriptions, &broadcast_tx, &mut command_rx)
            .await
        {
            Ok(()) => {
                info!("{} stream closed gracefully", source.id());
                break;
            }
            Err(e) => {
                if session_start.elapsed() >= reconnect.healthy_after {
                    attempt = 0;
                }
                if let Some(max) = reconnect.max_attempts
                    && attempt >= max
                {
                    error!(
                        "{} stream error: {}, giving up after {} reconnect attempts",
                        source.id(),
                        e,
                        max
                    );
                    break;
                }
                let delay = reconnect.delay_for(attempt, &mut seed);
                error!(
                    "{} stream error: {}, reconnecting in {:.1}s (attempt {})...",
                    source.id(),
                    e,
                    delay.as_secs_f32(),
                    attempt + 1
                );
                tokio::time::sleep(delay).await;
                attempt += 1;
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
            ReconnectConfig::new(Duration::from_millis(50)),
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

    /// A source whose session always fails immediately — for exercising the
    /// reconnect loop's `max_attempts` cap without waiting on real timeouts.
    struct AlwaysFailSource;

    #[async_trait::async_trait]
    impl StreamSource<PriceUpdate> for AlwaysFailSource {
        fn id(&self) -> &'static str {
            "always-fail"
        }

        async fn run_session(
            &self,
            _subscriptions: &Arc<RwLock<HashSet<String>>>,
            _broadcast_tx: &broadcast::Sender<PriceUpdate>,
            _command_rx: &mut mpsc::Receiver<StreamCommand>,
        ) -> StreamResult<()> {
            Err(super::super::client::StreamError::ConnectionFailed(
                "boom".to_string(),
            ))
        }
    }

    /// A source whose session survives briefly, then fails — the shape that
    /// used to defeat `max_attempts` by resetting the counter every cycle.
    struct BrieflyUpThenFailSource;

    #[async_trait::async_trait]
    impl StreamSource<PriceUpdate> for BrieflyUpThenFailSource {
        fn id(&self) -> &'static str {
            "briefly-up"
        }

        async fn run_session(
            &self,
            _subscriptions: &Arc<RwLock<HashSet<String>>>,
            _broadcast_tx: &broadcast::Sender<PriceUpdate>,
            _command_rx: &mut mpsc::Receiver<StreamCommand>,
        ) -> StreamResult<()> {
            tokio::time::sleep(Duration::from_millis(20)).await;
            Err(super::super::client::StreamError::ConnectionFailed(
                "dropped".to_string(),
            ))
        }
    }

    #[tokio::test]
    async fn a_session_shorter_than_healthy_after_still_counts_toward_max_attempts() {
        // Sessions last ~20ms — longer than base_delay but well short of
        // healthy_after, so the attempt counter must keep climbing.
        let reconnect = ReconnectConfig::new(Duration::from_millis(1))
            .max_attempts(Some(2))
            .healthy_after(Duration::from_secs(60));
        let mut stream = PriceStream::subscribe_with_source(
            Arc::new(BrieflyUpThenFailSource),
            Vec::<String>::new(),
            reconnect,
        )
        .await
        .unwrap();

        let ended = tokio::time::timeout(Duration::from_secs(5), stream.next()).await;
        assert!(
            matches!(ended, Ok(None)),
            "stream should have given up, not reconnected forever"
        );
    }

    #[tokio::test]
    async fn reconnect_loop_gives_up_after_max_attempts() {
        let reconnect = ReconnectConfig::new(Duration::from_millis(1)).max_attempts(Some(2));
        let mut stream = PriceStream::subscribe_with_source(
            Arc::new(AlwaysFailSource),
            Vec::<String>::new(),
            reconnect,
        )
        .await
        .unwrap();

        // The loop must stop retrying and end the stream rather than
        // reconnecting forever.
        let ended = tokio::time::timeout(Duration::from_secs(5), stream.next()).await;
        assert!(matches!(ended, Ok(None)), "stream should have ended");
    }

    #[test]
    fn reconnect_config_delay_grows_and_caps() {
        let reconnect = ReconnectConfig::new(Duration::from_secs(1));
        let mut seed = 1u64;
        // Default max_delay is 60s with 20% jitter: no sample may exceed 72s.
        for attempt in 0..12 {
            let delay = reconnect.delay_for(attempt, &mut seed);
            assert!(
                delay <= Duration::from_secs(72),
                "attempt {attempt}: {delay:?}"
            );
        }
        // Deep into the attempt sequence the delay must be pinned near the
        // cap (>= 60s * 0.8), not still climbing toward it from the 1s base.
        let late = reconnect.delay_for(20, &mut seed);
        assert!(
            late >= Duration::from_secs(48),
            "expected the delay capped near max_delay, got {late:?}"
        );
    }
}
