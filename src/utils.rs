//! Shared utility functions for ticker and tickers modules.

use crate::constants::TimeRange;
use crate::models::chart::{CapitalGain, Dividend, Split};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Maximum number of entries before we trigger a stale-entry eviction sweep.
///
/// Eviction only runs when the map exceeds this size, amortizing the O(n)
/// retain cost across many inserts instead of running on every single write.
pub(crate) const EVICTION_THRESHOLD: usize = 64;

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

    /// Returns `true` if the entry has not exceeded the given TTL.
    #[inline]
    pub(crate) fn is_fresh(&self, ttl: Duration) -> bool {
        self.fetched_at.elapsed() < ttl
    }

    /// Returns `true` if the entry exists and has not exceeded the given TTL.
    ///
    /// Returns `false` if the entry is `None` or the TTL is `None` (caching disabled).
    /// This consolidates the common pattern of checking both the TTL and entry existence.
    #[inline]
    pub(crate) fn is_fresh_with_ttl(entry: Option<&CacheEntry<T>>, ttl: Option<Duration>) -> bool {
        match (ttl, entry) {
            (Some(ttl), Some(e)) => e.is_fresh(ttl),
            _ => false,
        }
    }
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
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

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
