//! Shared exponential-backoff-with-jitter delay computation.
//!
//! Used by both provider dispatch retries ([`crate::providers::config::ProvidersBuilder::retry`])
//! and streaming reconnection (`src/streaming/source.rs`) so the two features
//! share one tested implementation instead of two hand-rolled copies.

use std::time::Duration;

/// Deterministic exponential delay for a zero-indexed attempt number, before
/// jitter is applied.
///
/// `attempt` `0` is the delay before the *first* retry/reconnect. Grows by
/// `multiplier` per attempt (clamped to at least `1.0` so a misconfigured
/// multiplier can't shrink the delay), capped at `max`.
pub(crate) fn exponential_delay(
    attempt: u32,
    base: Duration,
    multiplier: f64,
    max: Duration,
) -> Duration {
    let base_secs = base.as_secs_f64();
    // A zero base stays zero: `0.0 * inf` is NaN at high attempt counts, and
    // `f64::min` returns the *other* operand for NaN, which would silently
    // promote a no-delay config to the cap.
    if base_secs <= 0.0 {
        return Duration::ZERO;
    }
    let factor = multiplier.max(1.0).powi(attempt as i32);
    let secs = (base_secs * factor).min(max.as_secs_f64());
    Duration::from_secs_f64(secs.max(0.0))
}

/// The four knobs every backoff sequence here needs, so `RetryPolicy` and
/// `ReconnectConfig` share one implementation instead of two copies of the
/// same nested `with_jitter(exponential_delay(..))` call.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BackoffParams {
    pub base: Duration,
    pub max: Duration,
    pub multiplier: f64,
    pub jitter: f64,
}

impl BackoffParams {
    /// Jittered delay before the retry/reconnect numbered `attempt` (0-indexed).
    pub(crate) fn delay_for(&self, attempt: u32, seed: &mut u64) -> Duration {
        with_jitter(
            exponential_delay(attempt, self.base, self.multiplier, self.max),
            self.jitter,
            seed,
        )
    }
}

/// Apply a `+/- jitter` fraction (clamped to `0.0..=1.0`) of randomness to a
/// delay, so many concurrently-retrying callers don't retry in lockstep.
///
/// Uses a tiny internal xorshift64 PRNG seeded by the caller rather than
/// pulling in a `rand` dependency for this one call site — the jitter only
/// needs to be "random enough to desynchronize retries," not
/// cryptographically strong.
pub(crate) fn with_jitter(delay: Duration, jitter: f64, seed: &mut u64) -> Duration {
    let jitter = jitter.clamp(0.0, 1.0);
    if jitter == 0.0 {
        return delay;
    }
    let r = next_unit_f64(seed);
    // r in [0,1) -> factor in [1-jitter, 1+jitter]
    let factor = 1.0 + (r * 2.0 - 1.0) * jitter;
    Duration::from_secs_f64((delay.as_secs_f64() * factor).max(0.0))
}

/// Advance the xorshift64 state and return a value in `[0.0, 1.0)`.
fn next_unit_f64(state: &mut u64) -> f64 {
    if *state == 0 {
        // xorshift is stuck at 0 forever; recover with a fixed nonzero seed.
        *state = 0x9E3779B97F4A7C15;
    }
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    (*state >> 11) as f64 / (1u64 << 53) as f64
}

/// A seed for [`with_jitter`] derived from the current time, so independent
/// reconnect loops/retry sequences don't share the same jitter sequence.
pub(crate) fn seed_from_time() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x2545_F491_4F6C_DD1D);
    // XOR with a fixed odd constant so a `nanos` of 0 (unlikely, but possible
    // in a mocked clock) still yields a nonzero, well-mixed seed.
    nanos ^ 0x9E3779B97F4A7C15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exponential_delay_doubles_by_default_and_caps() {
        let base = Duration::from_secs(1);
        let max = Duration::from_secs(10);
        assert_eq!(exponential_delay(0, base, 2.0, max), Duration::from_secs(1));
        assert_eq!(exponential_delay(1, base, 2.0, max), Duration::from_secs(2));
        assert_eq!(exponential_delay(2, base, 2.0, max), Duration::from_secs(4));
        assert_eq!(exponential_delay(3, base, 2.0, max), Duration::from_secs(8));
        // Would be 16s uncapped; must clamp to max.
        assert_eq!(exponential_delay(4, base, 2.0, max), max);
        assert_eq!(exponential_delay(10, base, 2.0, max), max);
    }

    #[test]
    fn exponential_delay_rejects_shrinking_multiplier() {
        let base = Duration::from_secs(2);
        let max = Duration::from_secs(60);
        // A multiplier < 1.0 is clamped to 1.0 - the delay never shrinks.
        assert_eq!(exponential_delay(5, base, 0.1, max), base);
    }

    #[test]
    fn a_zero_base_delay_stays_zero_at_every_attempt() {
        // `0.0 * inf` is NaN, and `f64::min` would return the cap instead.
        let max = Duration::from_secs(60);
        for attempt in [0, 1, 10, 100, 1000] {
            assert_eq!(
                exponential_delay(attempt, Duration::ZERO, 2.0, max),
                Duration::ZERO,
                "attempt {attempt}"
            );
        }
    }

    #[test]
    fn with_jitter_zero_is_identity() {
        let delay = Duration::from_secs(4);
        let mut seed = 12345u64;
        assert_eq!(with_jitter(delay, 0.0, &mut seed), delay);
    }

    #[test]
    fn with_jitter_stays_within_bounds() {
        let delay = Duration::from_secs(10);
        let mut seed = seed_from_time();
        for _ in 0..1000 {
            let jittered = with_jitter(delay, 0.2, &mut seed);
            assert!(jittered.as_secs_f64() >= 8.0 - 1e-9);
            assert!(jittered.as_secs_f64() <= 12.0 + 1e-9);
        }
    }

    #[test]
    fn with_jitter_full_range_never_negative() {
        let delay = Duration::from_millis(1);
        let mut seed = 1u64;
        for _ in 0..1000 {
            let jittered = with_jitter(delay, 1.0, &mut seed);
            assert!(jittered.as_secs_f64() >= 0.0);
            assert!(jittered.as_secs_f64() <= 0.002 + 1e-9);
        }
    }

    #[test]
    fn seed_from_time_is_nonzero() {
        assert_ne!(seed_from_time(), 0);
    }

    #[test]
    fn jitter_sequence_is_not_constant() {
        // Sanity check that the PRNG actually advances rather than returning
        // the same jittered value every call.
        let delay = Duration::from_secs(10);
        let mut seed = 42u64;
        let a = with_jitter(delay, 0.3, &mut seed);
        let b = with_jitter(delay, 0.3, &mut seed);
        assert_ne!(a, b);
    }
}
