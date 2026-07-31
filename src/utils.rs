//! Shared utility functions for ticker and tickers modules.

use crate::constants::TimeRange;
use crate::models::chart::{CapitalGain, Dividend, Split};
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
// tokio's Instant tracks the runtime clock, so TTL expiry is testable under
// `tokio::time::pause`. Outside tests it is std::time::Instant.
use tokio::time::Instant;

/// Returns the current time as Unix timestamp in seconds.
#[inline]
pub(crate) fn now_unix_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Default number of entries before we trigger an eviction sweep.
///
/// Eviction only runs when the map exceeds this size, amortizing the O(n)
/// retain cost across many inserts instead of running on every single write.
///
/// This is a floor, not a ceiling: a handle whose working set is inherently
/// larger — a [`Tickers`] basket keys one entry per symbol — raises it via
/// [`eviction_threshold_for`], or the cache could never hold a full working set
/// and would never produce a hit.
///
/// [`Tickers`]: crate::Tickers
pub(crate) const EVICTION_THRESHOLD: usize = 64;

/// Eviction threshold for a cache whose working set is `working_set` entries.
///
/// Sized so a full working set survives a sweep: eviction keeps the newest half,
/// so the threshold must be at least twice the working set.
#[inline]
pub(crate) fn eviction_threshold_for(working_set: usize) -> usize {
    EVICTION_THRESHOLD.max(working_set.saturating_mul(2))
}

/// How long a cached response stays usable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum CacheMode {
    /// Cache until the owning handle is dropped.
    #[default]
    Lifetime,
    /// Cache for a bounded duration.
    Ttl(Duration),
    /// Never cache.
    Off,
}

impl CacheMode {
    #[inline]
    pub(crate) fn enabled(self) -> bool {
        !matches!(self, Self::Off)
    }
}

/// Wrapper that tracks when a cached value was fetched.
///
/// Stores the value directly. Callers must clone on read, which is appropriate
/// for our access patterns where cached data is typically consumed immediately.
pub(crate) struct CacheEntry<T> {
    /// The cached value.
    pub(crate) value: T,
    /// Timestamp when this entry was created.
    fetched_at: Instant,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry with the current timestamp.
    #[inline]
    pub(crate) fn new(value: T) -> Self {
        Self {
            value,
            fetched_at: Instant::now(),
        }
    }

    /// Returns `true` if the entry is still usable under `mode`.
    #[inline]
    pub(crate) fn is_fresh(&self, mode: CacheMode) -> bool {
        match mode {
            CacheMode::Lifetime => true,
            CacheMode::Ttl(ttl) => self.fetched_at.elapsed() < ttl,
            CacheMode::Off => false,
        }
    }

    /// Returns `true` if the entry exists and is still usable under `mode`.
    #[inline]
    pub(crate) fn is_fresh_entry(entry: Option<&CacheEntry<T>>, mode: CacheMode) -> bool {
        matches!(entry, Some(e) if e.is_fresh(mode))
    }
}

/// Insert into a map cache, evicting first if the map has grown past
/// `threshold`. No-op when caching is off.
///
/// `threshold` comes from [`eviction_threshold_for`] so a handle with a large
/// working set — a `Tickers` basket of more than 32 symbols — can still hold
/// every key it needs at once.
pub(crate) fn cache_insert<K: Eq + Hash, V>(
    map: &mut HashMap<K, CacheEntry<V>>,
    key: K,
    value: V,
    mode: CacheMode,
    threshold: usize,
) {
    if !mode.enabled() {
        return;
    }
    if map.len() >= threshold {
        evict(map, mode, threshold);
    }
    map.insert(key, CacheEntry::new(value));
}

fn evict<K: Eq + Hash, V>(map: &mut HashMap<K, CacheEntry<V>>, mode: CacheMode, threshold: usize) {
    map.retain(|_, entry| entry.is_fresh(mode));
    if map.len() < threshold {
        return;
    }
    // Lifetime entries never go stale, so fall back to keeping the newest half by
    // fetch time. The survivor budget is a count, not just a timestamp cutoff:
    // a coarse or paused clock gives every entry the same `Instant`, and a pure
    // cutoff comparison would then retain all of them and never evict.
    let keep = (threshold / 2).max(1);
    let mut times: Vec<Instant> = map.values().map(|e| e.fetched_at).collect();
    times.sort_unstable();
    let cutoff = times[times.len() - keep];
    let mut kept = 0usize;
    map.retain(|_, entry| {
        let survives = entry.fetched_at >= cutoff && kept < keep;
        kept += usize::from(survives);
        survives
    });
}

/// Trait for types with a timestamp field
pub(crate) trait HasTimestamp {
    /// Returns the Unix timestamp
    fn timestamp(&self) -> i64;
}

impl HasTimestamp for Dividend {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

impl HasTimestamp for Split {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

impl HasTimestamp for CapitalGain {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

/// Calculate cutoff timestamp for a given time range
pub(crate) fn range_to_cutoff(range: TimeRange) -> i64 {
    let now = now_unix_secs();

    const DAY: i64 = 86400;

    match range {
        TimeRange::OneDay => now - DAY,
        TimeRange::FiveDays => now - 5 * DAY,
        TimeRange::OneMonth => now - 30 * DAY,
        TimeRange::ThreeMonths => now - 90 * DAY,
        TimeRange::SixMonths => now - 180 * DAY,
        TimeRange::OneYear => now - 365 * DAY,
        TimeRange::TwoYears => now - 2 * 365 * DAY,
        TimeRange::FiveYears => now - 5 * 365 * DAY,
        TimeRange::TenYears => now - 10 * 365 * DAY,
        TimeRange::YearToDate => {
            // Compute Jan 1 00:00:00 UTC of the current year from the Unix timestamp.
            // Algorithm: convert epoch seconds to days, walk the Gregorian calendar.
            let epoch_days = now / DAY;
            let mut year = 1970i32;
            let mut remaining = epoch_days;
            loop {
                let days_in_year = if is_leap_year(year) { 366 } else { 365 };
                if remaining < days_in_year {
                    break;
                }
                remaining -= days_in_year;
                year += 1;
            }
            // Jan 1 of `year` is (epoch_days - remaining) days from epoch
            (epoch_days - remaining) * DAY
        }
        TimeRange::Max => 0, // No cutoff
    }
}

/// Returns true if `year` is a Gregorian leap year.
const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Filter a list of timestamped items by time range
pub(crate) fn filter_by_range<T: HasTimestamp>(items: Vec<T>, range: TimeRange) -> Vec<T> {
    match range {
        TimeRange::Max => items,
        range => {
            let cutoff = range_to_cutoff(range);
            items
                .into_iter()
                .filter(|item| item.timestamp() >= cutoff)
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifetime_mode_is_always_fresh() {
        let entry = CacheEntry::new(1u32);
        assert!(entry.is_fresh(CacheMode::Lifetime));
    }

    #[test]
    fn off_mode_is_never_fresh() {
        let entry = CacheEntry::new(1u32);
        assert!(!entry.is_fresh(CacheMode::Off));
        assert!(!CacheEntry::is_fresh_entry(Some(&entry), CacheMode::Off));
    }

    #[test]
    fn ttl_mode_is_fresh_within_window() {
        let entry = CacheEntry::new(1u32);
        assert!(entry.is_fresh(CacheMode::Ttl(Duration::from_secs(60))));
        assert!(!entry.is_fresh(CacheMode::Ttl(Duration::ZERO)));
    }

    #[test]
    fn missing_entry_is_never_fresh() {
        assert!(!CacheEntry::<u32>::is_fresh_entry(
            None,
            CacheMode::Lifetime
        ));
    }

    #[test]
    fn off_mode_writes_nothing() {
        let mut map: HashMap<u32, CacheEntry<u32>> = HashMap::new();
        cache_insert(&mut map, 1, 1, CacheMode::Off, EVICTION_THRESHOLD);
        assert!(map.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn eviction_survives_a_frozen_clock() {
        // Under `tokio::time::pause` every entry shares one `Instant`. A pure
        // timestamp cutoff would retain all of them, so the cap must be enforced
        // by count as well.
        let mut map: HashMap<u32, CacheEntry<u32>> = HashMap::new();
        for i in 0..500u32 {
            cache_insert(&mut map, i, i, CacheMode::Lifetime, EVICTION_THRESHOLD);
        }
        assert!(
            map.len() <= EVICTION_THRESHOLD,
            "cache grew to {} with a frozen clock",
            map.len()
        );
        assert!(map.contains_key(&499), "newest entry must survive");
    }

    #[test]
    fn a_large_working_set_survives_eviction_intact() {
        // A `Tickers` basket keys one entry per symbol and only serves a hit when
        // every symbol is fresh. With the fixed 64-entry cap, a 100-symbol basket
        // was trimmed to 32 on every pass and never registered a single hit.
        let symbols = 100usize;
        let threshold = eviction_threshold_for(symbols);
        let mut map: HashMap<u32, CacheEntry<u32>> = HashMap::new();
        for i in 0..symbols as u32 {
            cache_insert(&mut map, i, i, CacheMode::Lifetime, threshold);
        }
        assert_eq!(map.len(), symbols, "the whole basket must stay cached");
        assert!(
            (0..symbols as u32).all(|i| map.contains_key(&i)),
            "every symbol must be present for all_cached to hit"
        );
    }

    #[test]
    fn a_large_working_set_is_still_bounded() {
        // Scaling the threshold must not turn the cache into an unbounded map:
        // rewriting the basket many times over still evicts.
        let threshold = eviction_threshold_for(100);
        let mut map: HashMap<u32, CacheEntry<u32>> = HashMap::new();
        for i in 0..5_000u32 {
            cache_insert(&mut map, i, i, CacheMode::Lifetime, threshold);
        }
        assert!(
            map.len() <= threshold,
            "cache grew to {} past a threshold of {threshold}",
            map.len()
        );
        assert!(map.contains_key(&4_999), "newest entry must survive");
    }

    #[test]
    fn lifetime_mode_evicts_oldest_past_threshold() {
        let mut map: HashMap<u32, CacheEntry<u32>> = HashMap::new();
        for i in 0..200u32 {
            cache_insert(&mut map, i, i, CacheMode::Lifetime, EVICTION_THRESHOLD);
        }
        assert!(map.len() <= EVICTION_THRESHOLD);
        assert!(map.contains_key(&199), "newest entry must survive eviction");
    }
}
