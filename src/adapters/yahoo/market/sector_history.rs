//! Sector-performance history computed from SPDR sector-ETF daily closes.
//!
//! Yahoo's sector overview pages (`sectors.rs`) carry only today's change,
//! no lookback. This fans out to each of the 11 GICS sectors' SPDR ETF
//! daily chart and computes day-over-day close deltas locally instead — a
//! proxy for the sector's own aggregate move, not FMP's own methodology,
//! and without an `exchange` breakdown.

use crate::adapters::yahoo::client::YahooClient;
use crate::constants::sectors::Sector;
use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use crate::models::chart::Candle;
use crate::models::market::performance::{SectorPerformance, SectorPerformanceHistory};
use std::collections::BTreeMap;

fn range_for_limit(limit: u32) -> TimeRange {
    match limit {
        0..=15 => TimeRange::OneMonth,
        16..=55 => TimeRange::ThreeMonths,
        56..=110 => TimeRange::SixMonths,
        111..=230 => TimeRange::OneYear,
        231..=460 => TimeRange::TwoYears,
        _ => TimeRange::Max,
    }
}

fn timestamp_to_date(timestamp: i64) -> Option<String> {
    chrono::DateTime::from_timestamp(timestamp, 0).map(|dt| dt.format("%Y-%m-%d").to_string())
}

fn daily_changes(candles: &[Candle]) -> Vec<(String, f64)> {
    candles
        .windows(2)
        .filter_map(|w| {
            let date = timestamp_to_date(w[1].timestamp)?;
            let change_percent = (w[1].close - w[0].close) / w[0].close * 100.0;
            Some((date, change_percent))
        })
        .collect()
}

fn pivot_to_history(
    per_sector: Vec<(Sector, Vec<(String, f64)>)>,
    limit: u32,
) -> Vec<SectorPerformanceHistory> {
    let mut by_date: BTreeMap<String, Vec<SectorPerformance>> = BTreeMap::new();
    for (sector, changes) in per_sector {
        for (date, change_percent) in changes {
            by_date.entry(date).or_default().push(SectorPerformance {
                sector: sector.display_name().to_string(),
                exchange: None,
                change_percent: Some(change_percent),
            });
        }
    }

    by_date
        .into_iter()
        .rev()
        .take(limit as usize)
        .map(|(date, sectors)| SectorPerformanceHistory {
            date: Some(date),
            sectors,
        })
        .collect()
}

pub(crate) async fn fetch_sector_performance_history(
    client: &YahooClient,
    limit: u32,
) -> Result<Vec<SectorPerformanceHistory>> {
    let range = range_for_limit(limit);
    let fetches = Sector::all().iter().map(|&sector| async move {
        let ticker = sector.spdr_etf();
        match crate::adapters::yahoo::chart::fetch_chart(client, ticker, Interval::OneDay, range)
            .await
        {
            Ok(chart) => Some((sector, daily_changes(&chart.candles))),
            Err(err) => {
                tracing::warn!("failed to fetch {ticker} sector-ETF chart: {err}");
                None
            }
        }
    });

    let per_sector: Vec<(Sector, Vec<(String, f64)>)> = futures::future::join_all(fetches)
        .await
        .into_iter()
        .flatten()
        .collect();

    Ok(pivot_to_history(per_sector, limit))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(timestamp: i64, close: f64) -> Candle {
        Candle {
            timestamp,
            open: close,
            high: close,
            low: close,
            close,
            volume: 0,
            adj_close: None,
            provider_id: None,
        }
    }

    #[test]
    fn daily_changes_computes_close_over_close_percent() {
        let candles = vec![candle(1_700_000_000, 100.0), candle(1_700_086_400, 105.0)];
        let changes = daily_changes(&candles);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].1, 5.0);
    }

    #[test]
    fn pivot_groups_by_date_and_caps_at_limit() {
        let per_sector = vec![
            (
                Sector::Technology,
                vec![
                    ("2026-01-02".to_string(), 1.0),
                    ("2026-01-05".to_string(), 2.0),
                ],
            ),
            (
                Sector::Energy,
                vec![
                    ("2026-01-02".to_string(), -0.5),
                    ("2026-01-05".to_string(), 0.3),
                ],
            ),
        ];

        let history = pivot_to_history(per_sector, 1);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].date.as_deref(), Some("2026-01-05"));
        assert_eq!(history[0].sectors.len(), 2);
    }

    #[test]
    fn range_widens_with_limit() {
        assert_eq!(range_for_limit(5), TimeRange::OneMonth);
        assert_eq!(range_for_limit(300), TimeRange::TwoYears);
        assert_eq!(range_for_limit(10_000), TimeRange::Max);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn fetches_ten_days_of_history_from_live_yahoo() {
        use crate::adapters::yahoo::client::ClientConfig;

        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let history = fetch_sector_performance_history(&client, 10).await.unwrap();
        assert_eq!(history.len(), 10);
        assert_eq!(history[0].sectors.len(), Sector::all().len());
    }
}
