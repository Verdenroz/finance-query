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

/// Maximum number of entries before we trigger an eviction sweep.
///
/// Eviction only runs when the map exceeds this size, amortizing the O(n)
/// retain cost across many inserts instead of running on every single write.
pub(crate) const EVICTION_THRESHOLD: usize = 64;

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
/// [`EVICTION_THRESHOLD`]. No-op when caching is off.
pub(crate) fn cache_insert<K: Eq + Hash, V>(
    map: &mut HashMap<K, CacheEntry<V>>,
    key: K,
    value: V,
    mode: CacheMode,
) {
    if !mode.enabled() {
        return;
    }
    if map.len() >= EVICTION_THRESHOLD {
        evict(map, mode);
    }
    map.insert(key, CacheEntry::new(value));
}

fn evict<K: Eq + Hash, V>(map: &mut HashMap<K, CacheEntry<V>>, mode: CacheMode) {
    map.retain(|_, entry| entry.is_fresh(mode));
    if map.len() < EVICTION_THRESHOLD {
        return;
    }
    // Lifetime entries never go stale, so fall back to dropping the older half
    // by fetch time — halving amortizes the O(n) sort across many inserts.
    let mut times: Vec<Instant> = map.values().map(|e| e.fetched_at).collect();
    times.sort_unstable();
    let cutoff = times[times.len() / 2];
    map.retain(|_, entry| entry.fetched_at >= cutoff);
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
        cache_insert(&mut map, 1, 1, CacheMode::Off);
        assert!(map.is_empty());
    }

    #[test]
    fn lifetime_mode_evicts_oldest_past_threshold() {
        let mut map: HashMap<u32, CacheEntry<u32>> = HashMap::new();
        for i in 0..200u32 {
            cache_insert(&mut map, i, i, CacheMode::Lifetime);
        }
        assert!(map.len() <= EVICTION_THRESHOLD);
        assert!(map.contains_key(&199), "newest entry must survive eviction");
    }
}
