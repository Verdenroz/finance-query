//! Lightweight in-memory provider health tracking ([`ProviderHealth`]),
//! exposed via [`crate::Providers::health`].

use super::Provider;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

/// How many recent call outcomes each provider's health snapshot considers.
const WINDOW: usize = 20;

/// Snapshot of one provider's recent health and (where derivable) remaining
/// rate-limit budget.
///
/// Purely observational — computed from the last `WINDOW` dispatch
/// outcomes recorded in-process by [`super::ProviderSet`]; it is not a
/// circuit breaker and does not itself change dispatch behavior (routing and
/// [`RetryPolicy`](super::retry::RetryPolicy) are unaffected by it).
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ProviderHealth {
    /// The provider this snapshot describes.
    pub provider: Provider,
    /// `true` when at least half of the recent recorded outcomes succeeded,
    /// or when no calls have been recorded yet (optimistic default).
    pub is_healthy: bool,
    /// Successes among the last `WINDOW` recorded outcomes.
    pub recent_successes: u32,
    /// Failures among the last `WINDOW` recorded outcomes.
    pub recent_failures: u32,
    /// The most recent failure's message. Cleared by any success, so it is
    /// set only when the latest recorded outcome was a failure.
    pub last_error: Option<String>,
    /// Best-effort estimate of remaining rate-limit budget (tokens in the
    /// adapter's own token bucket), when the provider exposes one via
    /// [`super::ProviderAdapter::rate_limit_remaining`]. `None` for providers
    /// with no local rate limiter to peek (e.g. Yahoo) or that haven't been
    /// initialized.
    pub requests_remaining_estimate: Option<f64>,
}

#[derive(Default)]
struct ProviderHealthState {
    outcomes: VecDeque<bool>,
    last_error: Option<String>,
}

/// Per-provider ring buffer of recent success/failure outcomes.
///
/// A `std::sync::Mutex` per tracker (not per provider) is fine here: critical
/// sections are a handful of `VecDeque` pushes, never held across an `.await`.
pub(crate) struct HealthTracker {
    state: Mutex<HashMap<Provider, ProviderHealthState>>,
}

impl HealthTracker {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }

    /// Record one dispatch outcome for `provider`.
    pub(crate) fn record(&self, provider: Provider, success: bool, error: Option<String>) {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let entry = guard.entry(provider).or_default();
        entry.outcomes.push_back(success);
        if entry.outcomes.len() > WINDOW {
            entry.outcomes.pop_front();
        }
        if success {
            entry.last_error = None;
        } else if let Some(e) = error {
            entry.last_error = Some(e);
        }
    }

    /// Snapshot the current health for `provider` (optimistic default with no
    /// recorded calls yet).
    pub(crate) fn snapshot(&self, provider: Provider) -> ProviderHealth {
        let guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let state = guard.get(&provider);
        let total = state.map_or(0, |s| s.outcomes.len());
        let successes = state.map_or(0, |s| s.outcomes.iter().filter(|o| **o).count());
        ProviderHealth {
            provider,
            // No calls recorded yet reads as healthy (0 >= 0), the optimistic default.
            is_healthy: successes * 2 >= total,
            recent_successes: successes as u32,
            recent_failures: (total - successes) as u32,
            last_error: state.and_then(|s| s.last_error.clone()),
            requests_remaining_estimate: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_provider_with_no_calls_is_healthy_by_default() {
        let tracker = HealthTracker::new();
        let health = tracker.snapshot(Provider::Yahoo);
        assert!(health.is_healthy);
        assert_eq!(health.recent_successes, 0);
        assert_eq!(health.recent_failures, 0);
        assert!(health.last_error.is_none());
    }

    #[test]
    fn all_successes_are_healthy() {
        let tracker = HealthTracker::new();
        for _ in 0..5 {
            tracker.record(Provider::Yahoo, true, None);
        }
        let health = tracker.snapshot(Provider::Yahoo);
        assert!(health.is_healthy);
        assert_eq!(health.recent_successes, 5);
        assert_eq!(health.recent_failures, 0);
    }

    #[test]
    fn a_majority_of_failures_is_unhealthy() {
        let tracker = HealthTracker::new();
        tracker.record(Provider::Yahoo, true, None);
        tracker.record(Provider::Yahoo, false, Some("boom".to_string()));
        tracker.record(Provider::Yahoo, false, Some("boom again".to_string()));
        let health = tracker.snapshot(Provider::Yahoo);
        assert!(!health.is_healthy);
        assert_eq!(health.recent_successes, 1);
        assert_eq!(health.recent_failures, 2);
        assert_eq!(health.last_error.as_deref(), Some("boom again"));
    }

    #[test]
    fn a_success_clears_the_last_error() {
        let tracker = HealthTracker::new();
        tracker.record(Provider::Yahoo, false, Some("boom".to_string()));
        tracker.record(Provider::Yahoo, true, None);
        let health = tracker.snapshot(Provider::Yahoo);
        assert!(health.last_error.is_none());
    }

    #[test]
    fn window_evicts_the_oldest_outcome() {
        let tracker = HealthTracker::new();
        // Fill the window with failures, then enough successes to push every
        // failure out of the window.
        for _ in 0..WINDOW {
            tracker.record(Provider::Yahoo, false, Some("boom".to_string()));
        }
        assert!(!tracker.snapshot(Provider::Yahoo).is_healthy);
        for _ in 0..WINDOW {
            tracker.record(Provider::Yahoo, true, None);
        }
        let health = tracker.snapshot(Provider::Yahoo);
        assert!(health.is_healthy);
        assert_eq!(health.recent_successes, WINDOW as u32);
        assert_eq!(health.recent_failures, 0);
    }

    #[test]
    fn providers_are_tracked_independently() {
        let tracker = HealthTracker::new();
        tracker.record(Provider::Yahoo, true, None);
        tracker.record(Provider::Edgar, false, Some("boom".to_string()));
        tracker.record(Provider::Edgar, false, Some("boom".to_string()));
        assert!(tracker.snapshot(Provider::Yahoo).is_healthy);
        assert!(!tracker.snapshot(Provider::Edgar).is_healthy);
    }
}
