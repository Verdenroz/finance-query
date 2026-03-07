//! Higher-timeframe candle resampling.
//!
//! Aggregates base-timeframe candles into a higher-timeframe (HTF) series
//! using standard OHLCV rules:
//! - Open  = first constituent bar's open
//! - High  = max of constituent highs
//! - Low   = min of constituent lows
//! - Close = last constituent bar's close
//! - Volume = sum of constituent volumes
//! - Timestamp = last constituent bar's timestamp (marks bar completion)

use crate::constants::Interval;
use crate::models::chart::Candle;

/// Resample `candles` from their base timeframe to `interval`.
///
/// `utc_offset_secs` shifts each candle's timestamp into the exchange's local
/// time before computing calendar bucket boundaries (weekly Monday start,
/// month boundary, etc.). Pass `0` for UTC-aligned bucketing (default for US
/// markets). Use [`Region::utc_offset_secs`] to obtain the correct value for
/// non-US exchanges.
///
/// # Notes
///
/// - Calendar-aligned intervals (`OneWeek`, `OneMonth`, `ThreeMonths`) respect
///   `utc_offset_secs`. Weekly bars start on the local Monday.
/// - Sub-daily intervals use fixed-second buckets relative to local midnight.
///
/// [`Region::utc_offset_secs`]: crate::constants::Region::utc_offset_secs
pub fn resample(candles: &[Candle], interval: Interval, utc_offset_secs: i64) -> Vec<Candle> {
    if candles.is_empty() {
        return vec![];
    }

    let mut result = Vec::new();
    let mut group_start = 0;
    let mut current_bucket = bucket_id(&candles[0], interval, utc_offset_secs);

    for i in 1..candles.len() {
        let b = bucket_id(&candles[i], interval, utc_offset_secs);
        if b != current_bucket {
            result.push(aggregate(&candles[group_start..i]));
            group_start = i;
            current_bucket = b;
        }
    }
    result.push(aggregate(&candles[group_start..]));
    result
}

/// Map each base-timeframe index to the most recently *completed* HTF bar index.
///
/// A "completed" HTF bar is one whose timestamp (the last constituent bar's
/// timestamp) is less than or equal to the current base bar's timestamp.
/// Using `<=` rather than `<` ensures that on the final bar of an HTF period
/// (e.g. a Friday close for a weekly bar), the engine can immediately see the
/// now-finalized HTF candle. Using `<` would introduce an artificial one-bar
/// delay: on Friday, `htf.timestamp == base.timestamp`, so `<` fails and the
/// engine falls back to the prior week's data even though the weekly bar is
/// already complete.
///
/// `htf_candles` must have been produced by [`resample`] with the same
/// `utc_offset_secs` used for the base series so that bucket boundaries are
/// consistent.
///
/// Returns `None` for bars where no HTF bar has completed yet (e.g. during
/// the first HTF period).
pub fn base_to_htf_index(base_candles: &[Candle], htf_candles: &[Candle]) -> Vec<Option<usize>> {
    let mut result = Vec::with_capacity(base_candles.len());
    let mut last_completed: Option<usize> = None;
    let mut htf_idx = 0;

    for base in base_candles {
        // Advance past any HTF bars whose period has fully closed by this bar.
        // `<=` includes the bar where htf.timestamp == base.timestamp, i.e. the
        // last constituent bar of the HTF period — that bar IS completed at this
        // point, so it should be visible.
        while htf_idx < htf_candles.len() && htf_candles[htf_idx].timestamp <= base.timestamp {
            last_completed = Some(htf_idx);
            htf_idx += 1;
        }
        result.push(last_completed);
    }
    result
}

fn aggregate(group: &[Candle]) -> Candle {
    let first = &group[0];
    let last = &group[group.len() - 1];
    Candle {
        timestamp: last.timestamp,
        open: first.open,
        high: group
            .iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, f64::max),
        low: group.iter().map(|c| c.low).fold(f64::INFINITY, f64::min),
        close: last.close,
        volume: group.iter().map(|c| c.volume).sum(),
        adj_close: last.adj_close,
    }
}

fn bucket_id(candle: &Candle, interval: Interval, utc_offset_secs: i64) -> i64 {
    // Shift the raw UTC timestamp into the exchange's local time before computing
    // calendar boundaries. For sub-daily intervals this aligns session buckets;
    // for weekly/monthly it ensures Monday/month-start is local, not UTC.
    let ts = candle.timestamp + utc_offset_secs;
    match interval {
        // Use Euclidean division so that negative timestamps (pre-1970 data)
        // are bucketed correctly. Truncation-toward-zero would map e.g.
        // Dec 31 1969 (-1 s) and Jan 1 1970 (0 s) to the same bucket 0.
        Interval::OneDay => ts.div_euclid(86_400),
        Interval::OneWeek => {
            // Days-since-epoch (Euclidean) of the local Monday that starts this ISO week.
            // Unix epoch (1970-01-01) was a Thursday; adding 3 shifts so Mon = 0.
            let days = ts.div_euclid(86_400);
            let weekday = (days + 3).rem_euclid(7); // 0 = Mon … 6 = Sun
            days - weekday
        }
        Interval::OneMonth => {
            let (y, m, _) = ymd(ts);
            y * 100 + m
        }
        Interval::ThreeMonths => {
            let (y, m, _) = ymd(ts);
            y * 10 + (m - 1) / 3 + 1
        }
        _ => ts.div_euclid(interval_seconds(interval)),
    }
}

const fn interval_seconds(interval: Interval) -> i64 {
    match interval {
        Interval::OneMinute => 60,
        Interval::FiveMinutes => 300,
        Interval::FifteenMinutes => 900,
        Interval::ThirtyMinutes => 1_800,
        Interval::OneHour => 3_600,
        Interval::OneDay => 86_400,
        Interval::OneWeek => 604_800,
        Interval::OneMonth => 2_592_000,
        Interval::ThreeMonths => 7_776_000,
    }
}

/// Gregorian calendar date from a Unix timestamp (seconds since epoch, UTC).
///
/// Uses the proleptic Gregorian calendar via Julian Day Number conversion.
/// Does not account for leap seconds.
fn ymd(ts: i64) -> (i64, i64, i64) {
    let days = ts.div_euclid(86_400);
    // Julian Day Number: Unix epoch (1970-01-01) = JDN 2_440_588
    let jdn = days + 2_440_588;
    let a = jdn + 32_044;
    let b = (4 * a + 3) / 146_097;
    let c = a - (146_097 * b) / 4;
    let d = (4 * c + 3) / 1_461;
    let e = c - (1_461 * d) / 4;
    let m = (5 * e + 2) / 153;
    let day = e - (153 * m + 2) / 5 + 1;
    let month = m + 3 - 12 * (m / 10);
    let year = 100 * b + d - 4_800 + m / 10;
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(ts: i64, o: f64, h: f64, l: f64, c: f64, v: i64) -> Candle {
        Candle {
            timestamp: ts,
            open: o,
            high: h,
            low: l,
            close: c,
            volume: v,
            adj_close: None,
        }
    }

    #[test]
    fn test_resample_empty() {
        assert!(resample(&[], Interval::OneWeek, 0).is_empty());
    }

    #[test]
    fn test_resample_weekly_ohlcv() {
        // 2024-01-08 (Mon) = 1_704_672_000
        let mon = 1_704_672_000_i64;
        let base: Vec<Candle> = (0..5)
            .map(|d| {
                candle(
                    mon + d * 86_400,
                    100.0 + d as f64,
                    110.0 + d as f64,
                    90.0 + d as f64,
                    105.0 + d as f64,
                    1_000 + d * 100,
                )
            })
            .collect();

        let weekly = resample(&base, Interval::OneWeek, 0);
        assert_eq!(weekly.len(), 1);

        let w = &weekly[0];
        assert_eq!(w.open, base[0].open);
        assert_eq!(w.close, base[4].close);
        assert!((w.high - 114.0).abs() < f64::EPSILON);
        assert!((w.low - 90.0).abs() < f64::EPSILON);
        assert_eq!(w.volume, base.iter().map(|c| c.volume).sum::<i64>());
        assert_eq!(w.timestamp, base[4].timestamp);
    }

    #[test]
    fn test_resample_two_weeks() {
        let mon_wk1 = 1_704_672_000_i64; // 2024-01-08
        let mon_wk2 = mon_wk1 + 7 * 86_400; // 2024-01-15
        let mut base: Vec<Candle> = (0..5)
            .map(|d| candle(mon_wk1 + d * 86_400, 100.0, 110.0, 90.0, 105.0, 1_000))
            .collect();
        base.extend(
            (0..5).map(|d| candle(mon_wk2 + d * 86_400, 200.0, 210.0, 190.0, 205.0, 2_000)),
        );

        let weekly = resample(&base, Interval::OneWeek, 0);
        assert_eq!(weekly.len(), 2);
        assert!((weekly[0].open - 100.0).abs() < f64::EPSILON);
        assert!((weekly[1].open - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_base_to_htf_no_completed_yet() {
        let mon = 1_704_672_000_i64;
        let base: Vec<Candle> = (0..5)
            .map(|d| candle(mon + d * 86_400, 100.0, 110.0, 90.0, 105.0, 1_000))
            .collect();
        let htf = resample(&base, Interval::OneWeek, 0);
        // htf[0].timestamp = Friday's timestamp.
        // Mon–Thu: htf[0].timestamp > their timestamps → None.
        // Fri: htf[0].timestamp == Friday.timestamp, so <= passes → Some(0).
        let mapping = base_to_htf_index(&base, &htf);
        for (i, val) in mapping.iter().enumerate().take(4) {
            assert_eq!(
                *val, None,
                "bar {i} (Mon-Thu) should have no completed HTF bar"
            );
        }
        assert_eq!(
            mapping[4],
            Some(0),
            "bar 4 (Fri) should see its own completed weekly bar"
        );
    }

    #[test]
    fn test_base_to_htf_with_completed() {
        let mon_wk1 = 1_704_672_000_i64;
        let mon_wk2 = mon_wk1 + 7 * 86_400;
        let mut base: Vec<Candle> = (0..5)
            .map(|d| candle(mon_wk1 + d * 86_400, 100.0, 110.0, 90.0, 105.0, 1_000))
            .collect();
        base.extend(
            (0..5).map(|d| candle(mon_wk2 + d * 86_400, 200.0, 210.0, 190.0, 205.0, 2_000)),
        );

        let htf = resample(&base, Interval::OneWeek, 0);
        assert_eq!(htf.len(), 2);

        let mapping = base_to_htf_index(&base, &htf);
        // Week 1: Mon–Thu have no completed HTF bar; Fri sees its own completed weekly bar.
        for (i, val) in mapping.iter().enumerate().take(4) {
            assert_eq!(
                *val, None,
                "bar {i} (Mon-Thu wk1) should have no completed HTF bar"
            );
        }
        assert_eq!(
            mapping[4],
            Some(0),
            "bar 4 (Fri wk1) should see wk1 bar as completed"
        );
        // Week 2: Mon–Thu see wk1 as the last completed bar; Fri sees its own wk2 bar completed.
        for (i, val) in mapping.iter().enumerate().take(9).skip(5) {
            assert_eq!(
                *val,
                Some(0),
                "bar {i} (Mon-Thu wk2) should see HTF bar 0 as completed"
            );
        }
        assert_eq!(
            mapping[9],
            Some(1),
            "bar 9 (Fri wk2) should see its own completed weekly bar"
        );
    }

    #[test]
    fn test_utc_offset_bucketing() {
        // UTC midnight on 2024-01-08 (Mon) = 1_704_672_000.
        // For a UTC+8 exchange (e.g. Tokyo/HK), that UTC midnight IS already
        // Monday 08:00 local time — still Monday, so offset makes no difference here.
        //
        // The key case: a bar whose UTC timestamp is Sunday 22:00 (= Monday 06:00 JST).
        // With offset=0  it falls in Sunday's bucket  → prior week.
        // With offset=+28800 (+8 h) it becomes Monday → current week.
        let sun_22_utc = 1_704_585_600_i64 + 22 * 3600; // Sun 2024-01-07 22:00 UTC
        let fri_utc = 1_704_585_600_i64 + 5 * 86_400; // Fri 2024-01-12 00:00 UTC (same "week" in JST)

        let c1 = candle(sun_22_utc, 100.0, 101.0, 99.0, 100.0, 1_000);
        let c2 = candle(fri_utc, 105.0, 106.0, 104.0, 105.0, 1_000);

        // Without offset: sun_22_utc is in the Sunday/prior week bucket → two separate weeks.
        let utc_result = resample(&[c1.clone(), c2.clone()], Interval::OneWeek, 0);
        assert_eq!(
            utc_result.len(),
            2,
            "UTC bucketing splits the Sunday bar into the prior week"
        );

        // With UTC+8: sun_22_utc + 28800 = Monday 06:00 JST → same week as Friday.
        let jst_result = resample(&[c1, c2], Interval::OneWeek, 28_800);
        assert_eq!(
            jst_result.len(),
            1,
            "JST bucketing groups Sunday-22h-UTC into Monday JST week"
        );
    }

    #[test]
    fn test_subdaily_utc_offset_bucketing() {
        // Verify that utc_offset_secs aligns intraday session boundaries.
        // Scenario: an exchange opens at 09:00 JST (= 00:00 UTC).
        // Two 1-hour bars bracketing local midnight:
        //   bar_a: 2024-01-08 23:00 UTC = 2024-01-09 08:00 JST  → still Monday JST
        //   bar_b: 2024-01-09 00:00 UTC = 2024-01-09 09:00 JST  → Monday JST session open
        // With UTC bucketing (offset=0) and OneDay, bar_a falls on 2024-01-08 and
        // bar_b falls on 2024-01-09 → two separate daily buckets.
        // With JST offset (+32400 = +9 h), bar_a + 32400 = 2024-01-09 08:00 JST and
        // bar_b + 32400 = 2024-01-09 09:00 JST → both on the same local date → one bucket.
        let bar_a_utc = 1_704_758_400_i64; // 2024-01-09 00:00 UTC — Mon midnight UTC
        let bar_b_utc = bar_a_utc + 3_600; // 2024-01-09 01:00 UTC

        let c_a = candle(bar_a_utc - 3_600, 100.0, 101.0, 99.0, 100.0, 500); // 2024-01-08 23:00 UTC
        let c_b = candle(bar_a_utc, 101.0, 102.0, 100.0, 101.0, 600); // 2024-01-09 00:00 UTC
        let c_c = candle(bar_b_utc, 102.0, 103.0, 101.0, 102.0, 700); // 2024-01-09 01:00 UTC

        // UTC bucketing: c_a is on Jan 8, c_b and c_c are on Jan 9 → 2 daily buckets.
        let utc_daily = resample(
            &[c_a.clone(), c_b.clone(), c_c.clone()],
            Interval::OneDay,
            0,
        );
        assert_eq!(
            utc_daily.len(),
            2,
            "UTC: Jan 8 23h and Jan 9 00h/01h are two calendar days"
        );

        // JST bucketing (+9h): c_a (23:00 UTC) + 9h = 08:00 JST Jan 9 → same day as c_b/c_c.
        let jst_daily = resample(&[c_a, c_b, c_c], Interval::OneDay, 32_400);
        assert_eq!(
            jst_daily.len(),
            1,
            "JST: all three bars fall on the same local calendar day"
        );
    }

    #[test]
    fn test_ymd() {
        // 2024-01-08 = 1_704_672_000 (confirmed via date math)
        let (y, m, d) = ymd(1_704_672_000);
        assert_eq!((y, m, d), (2024, 1, 8));

        // 2024-03-15
        let (y, m, d) = ymd(1_710_460_800);
        assert_eq!((y, m, d), (2024, 3, 15));
    }
}
