//! Opt-in retry policy for provider dispatch ([`RetryPolicy`]).

use std::time::Duration;

/// Retry policy for [`super::ProviderSet`] dispatch, opt-in via
/// [`crate::Providers::builder`]`().`[`retry`](super::config::ProvidersBuilder::retry)`(..)`.
///
/// When configured, a candidate provider that returns
/// [`FinanceError::RateLimited`](crate::error::FinanceError::RateLimited) is
/// retried in place — honoring the error's `retry_after` hint verbatim when
/// present, or this policy's own exponential-backoff-plus-jitter delay
/// otherwise — up to `max_attempts` times before dispatch falls through to
/// the next routed provider (or fails, if it was the last).
///
/// Other error kinds are never retried by this policy: a `RateLimited` is the
/// one error a policy-level retry can reliably fix by waiting; anything else
/// (auth failures, 5xx, not-found, ...) is left to the existing
/// sequential/parallel provider fallback.
///
/// **Default is no retry** — a [`Providers`](crate::Providers) built without
/// calling `.retry(..)` behaves exactly as before this policy existed.
///
/// # Example
///
/// ```no_run
/// use finance_query::{Providers, RetryPolicy};
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let providers = Providers::builder()
///     .retry(RetryPolicy::new(3).base_delay(Duration::from_millis(500)))
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct RetryPolicy {
    /// Maximum number of attempts per provider candidate, including the
    /// first try. Values below `1` are treated as `1` (no retry).
    pub max_attempts: u32,
    /// Delay before the first retry when the error carries no `retry_after`
    /// hint. Default: 500ms.
    pub base_delay: Duration,
    /// Multiplier applied to the delay after each retry (clamped to at least
    /// `1.0`). Default: `2.0`.
    pub multiplier: f64,
    /// Jitter fraction (`0.0..=1.0`) applied to the computed delay so many
    /// concurrent callers don't retry in lockstep. Default: `0.2`.
    pub jitter: f64,
    /// Upper bound on the computed delay. Never applied to an explicit
    /// `retry_after` hint — that value is honored verbatim. Default: 30s.
    pub max_delay: Duration,
}

impl RetryPolicy {
    /// A policy allowing up to `max_attempts` tries per candidate provider,
    /// with sane defaults for the rest (500ms base delay, 2x multiplier, 20%
    /// jitter, 30s cap).
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            base_delay: Duration::from_millis(500),
            multiplier: 2.0,
            jitter: 0.2,
            max_delay: Duration::from_secs(30),
        }
    }

    /// Override the base delay (see [`RetryPolicy::base_delay`] field docs).
    pub fn base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Override the backoff multiplier (see [`RetryPolicy::multiplier`] field docs).
    pub fn multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Override the jitter fraction (see [`RetryPolicy::jitter`] field docs).
    pub fn jitter(mut self, jitter: f64) -> Self {
        self.jitter = jitter.clamp(0.0, 1.0);
        self
    }

    /// Override the max delay cap (see [`RetryPolicy::max_delay`] field docs).
    pub fn max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    /// Delay before the retry numbered `attempt` (0-indexed: `0` is the delay
    /// before the first retry), honoring an explicit `retry_after` hint
    /// verbatim when present.
    pub(crate) fn delay_for(
        &self,
        attempt: u32,
        retry_after: Option<Duration>,
        seed: &mut u64,
    ) -> Duration {
        match retry_after {
            Some(explicit) => explicit,
            None => crate::backoff::with_jitter(
                crate::backoff::exponential_delay(
                    attempt,
                    self.base_delay,
                    self.multiplier,
                    self.max_delay,
                ),
                self.jitter,
                seed,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_zero_attempts_to_one() {
        assert_eq!(RetryPolicy::new(0).max_attempts, 1);
    }

    #[test]
    fn defaults_are_sane() {
        let policy = RetryPolicy::new(3);
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.base_delay, Duration::from_millis(500));
        assert_eq!(policy.multiplier, 2.0);
        assert_eq!(policy.jitter, 0.2);
        assert_eq!(policy.max_delay, Duration::from_secs(30));
    }

    #[test]
    fn jitter_is_clamped() {
        assert_eq!(RetryPolicy::new(1).jitter(5.0).jitter, 1.0);
        assert_eq!(RetryPolicy::new(1).jitter(-5.0).jitter, 0.0);
    }

    #[test]
    fn explicit_retry_after_is_honored_verbatim_ignoring_max_delay() {
        let policy = RetryPolicy::new(3).max_delay(Duration::from_secs(1));
        let mut seed = 42;
        let delay = policy.delay_for(5, Some(Duration::from_secs(999)), &mut seed);
        assert_eq!(delay, Duration::from_secs(999));
    }

    #[test]
    fn falls_back_to_exponential_backoff_when_no_retry_after() {
        let policy = RetryPolicy::new(5)
            .base_delay(Duration::from_secs(1))
            .multiplier(2.0)
            .jitter(0.0)
            .max_delay(Duration::from_secs(10));
        let mut seed = 1;
        assert_eq!(policy.delay_for(0, None, &mut seed), Duration::from_secs(1));
        assert_eq!(policy.delay_for(1, None, &mut seed), Duration::from_secs(2));
        assert_eq!(policy.delay_for(2, None, &mut seed), Duration::from_secs(4));
        // Capped.
        assert_eq!(
            policy.delay_for(10, None, &mut seed),
            Duration::from_secs(10)
        );
    }
}
