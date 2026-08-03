//! Economic-release streaming.
//!
//! Macro data has no push transport — series are revised on a publication
//! calendar — so this is a poll loop that emits a purpose-built
//! [`EconomicRelease`] only when a series' latest observation actually
//! changes, rather than pushing an unrelated price-tick shape.

use std::collections::HashMap;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};
use tracing::warn;

use super::subscription::Subscription;

/// Default interval between polls of all subscribed series.
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(900);

/// Channel capacity — macro releases are rare compared to price ticks.
const CHANNEL_CAPACITY: usize = 128;

/// A newly published (or revised) observation for an economic series.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EconomicRelease {
    /// Series identifier (e.g. `"FEDFUNDS"`, `"CPIAUCSL"`).
    pub series_id: String,
    /// Observation date as `YYYY-MM-DD`.
    pub date: String,
    /// Newly published value, or `None` when the source reports a gap.
    pub value: Option<f64>,
    /// Value this release replaced: the prior observation, or the prior value
    /// for the same date when the release is a revision.
    pub previous_value: Option<f64>,
    /// `true` when the same observation date was re-published with a new value.
    pub revision: bool,
    /// Unix timestamp (seconds) at which this release was observed.
    pub observed_at: i64,
}

/// Commands accepted by a running economic poll loop.
enum EconomicCommand {
    AddSeries(Vec<String>),
    RemoveSeries(Vec<String>),
    Close,
}

/// Fetches the latest observation for a series.
///
/// A trait rather than a direct FRED call so the poll loop can be exercised
/// without a socket.
#[async_trait::async_trait]
pub(crate) trait ReleaseSource: Send + Sync + 'static {
    async fn latest(&self, series_id: &str) -> Option<(String, Option<f64>)>;
}

/// A continuous subscription to economic-series releases.
///
/// Polls each subscribed series on an interval (15 minutes by default) and
/// yields an [`EconomicRelease`] only when the latest observation is new or
/// revised. Requires the `fred` feature and
/// [`fred::init`](crate::fred::init).
///
/// # Example
///
/// ```no_run
/// use finance_query::streaming::EconomicStream;
/// use futures::StreamExt;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut stream = EconomicStream::subscribe(["FEDFUNDS", "CPIAUCSL"]).await;
///
/// while let Some(release) = stream.next().await {
///     println!("{} = {:?} ({})", release.series_id, release.value, release.date);
/// }
/// # Ok(())
/// # }
/// ```
pub struct EconomicStream {
    inner: Subscription<EconomicRelease, EconomicCommand>,
}

impl EconomicStream {
    /// Subscribe to the given series, polling every 15 minutes.
    pub async fn subscribe<S, I>(series: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        EconomicStreamBuilder::new().series(series).build().await
    }

    pub(crate) fn start(
        source: std::sync::Arc<dyn ReleaseSource>,
        series: Vec<String>,
        poll_interval: Duration,
    ) -> Self {
        let inner = Subscription::start(CHANNEL_CAPACITY, 32, move |broadcast_tx, command_rx| {
            run_economic_loop(source, series, poll_interval, broadcast_tx, command_rx)
        });
        EconomicStream { inner }
    }

    /// Create an independent receiver sharing this subscription's poll loop.
    pub fn resubscribe(&self) -> Self {
        EconomicStream {
            inner: self.inner.resubscribe(),
        }
    }

    /// Add series to the subscription.
    pub async fn add_series<S, I>(&self, series: I)
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        let series = series.into_iter().map(Into::into).collect();
        self.inner.send(EconomicCommand::AddSeries(series)).await;
    }

    /// Remove series from the subscription.
    pub async fn remove_series<S, I>(&self, series: I)
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        let series = series.into_iter().map(Into::into).collect();
        self.inner.send(EconomicCommand::RemoveSeries(series)).await;
    }

    /// Stop polling and close the stream.
    pub async fn close(&self) {
        self.inner.send(EconomicCommand::Close).await;
    }
}

impl Stream for EconomicStream {
    type Item = EconomicRelease;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// Builder for an [`EconomicStream`] with a custom poll interval.
pub struct EconomicStreamBuilder {
    series: Vec<String>,
    poll_interval: Duration,
}

impl EconomicStreamBuilder {
    /// Create a builder with no series and the default 15-minute interval.
    pub fn new() -> Self {
        Self {
            series: Vec::new(),
            poll_interval: DEFAULT_POLL_INTERVAL,
        }
    }

    /// Add series identifiers to poll.
    pub fn series<S, I>(mut self, series: I) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        self.series.extend(series.into_iter().map(Into::into));
        self
    }

    /// Set the interval between polls (default: 15 minutes).
    pub fn poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Start the stream.
    pub async fn build(self) -> EconomicStream {
        EconomicStream::start(
            std::sync::Arc::new(DefaultReleaseSource),
            self.series,
            self.poll_interval,
        )
    }
}

impl Default for EconomicStreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// FRED-backed release source (no-op without the `fred` feature).
struct DefaultReleaseSource;

#[async_trait::async_trait]
impl ReleaseSource for DefaultReleaseSource {
    #[cfg(feature = "fred")]
    async fn latest(&self, series_id: &str) -> Option<(String, Option<f64>)> {
        match crate::adapters::fred::series(series_id).await {
            Ok(series) => series
                .observations
                .last()
                .map(|o| (o.date.clone(), o.value)),
            Err(e) => {
                warn!("economic stream poll failed for {series_id}: {e}");
                None
            }
        }
    }

    #[cfg(not(feature = "fred"))]
    async fn latest(&self, series_id: &str) -> Option<(String, Option<f64>)> {
        warn!("economic stream needs the `fred` feature; ignoring {series_id}");
        None
    }
}

/// Last observation seen per series, used to detect new vs. revised releases.
#[derive(Clone)]
struct LastSeen {
    date: String,
    value: Option<f64>,
}

async fn run_economic_loop(
    source: std::sync::Arc<dyn ReleaseSource>,
    initial_series: Vec<String>,
    poll_interval: Duration,
    broadcast_tx: broadcast::Sender<EconomicRelease>,
    mut command_rx: mpsc::Receiver<EconomicCommand>,
) {
    let mut series: Vec<String> = initial_series;
    let mut seen: HashMap<String, LastSeen> = HashMap::new();

    let mut ticker = tokio::time::interval(poll_interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                for id in &series {
                    let Some((date, value)) = source.latest(id).await else {
                        continue;
                    };
                    let release = classify(id, &date, value, seen.get(id));
                    seen.insert(id.clone(), LastSeen { date, value });
                    if let Some(release) = release {
                        let _ = broadcast_tx.send(release);
                    }
                }
            }
            cmd = command_rx.recv() => {
                match cmd {
                    Some(EconomicCommand::AddSeries(added)) => {
                        for id in added {
                            if !series.contains(&id) {
                                series.push(id);
                            }
                        }
                    }
                    Some(EconomicCommand::RemoveSeries(removed)) => {
                        series.retain(|id| !removed.contains(id));
                        for id in removed {
                            seen.remove(&id);
                        }
                    }
                    Some(EconomicCommand::Close) | None => break,
                }
            }
        }
    }
}

/// Decide whether an observation is worth emitting.
///
/// The first poll of a series only records a baseline — emitting there would
/// report every subscribe as a fresh release.
fn classify(
    series_id: &str,
    date: &str,
    value: Option<f64>,
    previous: Option<&LastSeen>,
) -> Option<EconomicRelease> {
    let previous = previous?;
    let revision = previous.date == date;
    if revision && previous.value == value {
        return None;
    }
    Some(EconomicRelease {
        series_id: series_id.to_string(),
        date: date.to_string(),
        value,
        previous_value: previous.value,
        revision,
        observed_at: chrono::Utc::now().timestamp(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Canned source: returns a scripted observation per poll, no network.
    struct ScriptedSource {
        observations: Vec<(String, Option<f64>)>,
        calls: AtomicUsize,
    }

    #[async_trait::async_trait]
    impl ReleaseSource for ScriptedSource {
        async fn latest(&self, _series_id: &str) -> Option<(String, Option<f64>)> {
            let idx = self.calls.fetch_add(1, Ordering::SeqCst);
            self.observations.get(idx).cloned()
        }
    }

    #[test]
    fn first_observation_only_sets_a_baseline() {
        assert!(classify("FEDFUNDS", "2026-01-01", Some(5.0), None).is_none());
    }

    #[test]
    fn unchanged_observation_is_not_a_release() {
        let last = LastSeen {
            date: "2026-01-01".into(),
            value: Some(5.0),
        };
        assert!(classify("FEDFUNDS", "2026-01-01", Some(5.0), Some(&last)).is_none());
    }

    #[test]
    fn same_date_with_a_new_value_is_a_revision() {
        let last = LastSeen {
            date: "2026-01-01".into(),
            value: Some(5.0),
        };
        let release = classify("FEDFUNDS", "2026-01-01", Some(5.25), Some(&last)).unwrap();
        assert!(release.revision);
        assert_eq!(release.previous_value, Some(5.0));
        assert_eq!(release.value, Some(5.25));
    }

    #[test]
    fn a_new_date_is_a_fresh_release() {
        let last = LastSeen {
            date: "2026-01-01".into(),
            value: Some(5.0),
        };
        let release = classify("FEDFUNDS", "2026-02-01", Some(5.5), Some(&last)).unwrap();
        assert!(!release.revision);
        assert_eq!(release.date, "2026-02-01");
    }

    #[tokio::test]
    async fn poll_loop_emits_only_changed_observations() {
        let source = Arc::new(ScriptedSource {
            observations: vec![
                ("2026-01-01".into(), Some(5.0)),
                ("2026-01-01".into(), Some(5.0)),
                ("2026-02-01".into(), Some(5.5)),
            ],
            calls: AtomicUsize::new(0),
        });

        let mut stream = EconomicStream::start(
            source,
            vec!["FEDFUNDS".to_string()],
            Duration::from_millis(10),
        );

        let release = tokio::time::timeout(Duration::from_secs(5), stream.next())
            .await
            .expect("timed out")
            .expect("stream ended");
        assert_eq!(release.date, "2026-02-01");
        assert_eq!(release.previous_value, Some(5.0));
        stream.close().await;
    }

    #[tokio::test]
    async fn close_ends_the_stream() {
        let source = Arc::new(ScriptedSource {
            observations: Vec::new(),
            calls: AtomicUsize::new(0),
        });
        let mut stream = EconomicStream::start(source, Vec::new(), Duration::from_millis(10));
        stream.close().await;
        let ended = tokio::time::timeout(Duration::from_secs(2), stream.next()).await;
        assert!(matches!(ended, Ok(None)));
    }
}
