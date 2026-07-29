//! Volume Weighted Moving Average (VWMA) indicator.

use super::{IndicatorError, Result};

/// Calculate Volume Weighted Moving Average (VWMA).
///
/// Prices weighted by volume over the given period.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `volumes` - Volume data
/// * `period` - Number of periods
///
/// # Formula
///
/// VWMA = Sum(Price * Volume) / Sum(Volume)
///
/// # Example
///
/// ```
/// use finance_query::indicators::vwma;
///
/// let prices = vec![10.0, 10.0, 10.0];
/// let volumes = vec![100.0, 100.0, 100.0];
/// let result = vwma(&prices, &volumes, 2).unwrap();
/// ```
pub fn vwma(data: &[f64], volumes: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    if data.len() != volumes.len() {
        return Err(IndicatorError::InvalidPeriod(
            "Data and volumes must have same length".to_string(),
        ));
    }
    if data.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: period,
            got: data.len(),
        });
    }

    let mut result = vec![None; data.len()];
    let mut pv_sum = 0.0;
    let mut volume_sum = 0.0;

    // A rolling sum whose window spans wildly different magnitudes can lose the
    // small terms entirely, leaving a denominator that is zero or near-zero once
    // the large term is subtracted back out. Rebuilding the window restores the
    // value a full rescan would have produced.
    let rebuild = |i: usize| -> (f64, f64) {
        let start = i + 1 - period;
        data[start..=i]
            .iter()
            .zip(volumes[start..=i].iter())
            .fold((0.0, 0.0), |(pv, v), (&p, &q)| (pv + p * q, v + q))
    };

    // Rolling add-new/subtract-old sums instead of rescanning the window each bar;
    // this reassociates the float sums, so results drift from a full rescan by ~1e-15 relative.
    for i in 0..data.len() {
        pv_sum += data[i] * volumes[i];
        volume_sum += volumes[i];
        if i >= period {
            let drop = i - period;
            pv_sum -= data[drop] * volumes[drop];
            volume_sum -= volumes[drop];
        }
        if i + 1 >= period {
            // Volumes and price*volume terms are non-negative, so a non-positive
            // running sum can only come from cancellation; the same is true of a
            // sum that has shrunk below the bar's own contribution.
            let term = data[i] * volumes[i];
            if volume_sum <= 0.0 || volume_sum < volumes[i] || pv_sum < 0.0 || pv_sum < term {
                (pv_sum, volume_sum) = rebuild(i);
            }
        }
        if i + 1 >= period && volume_sum != 0.0 {
            result[i] = Some(pv_sum / volume_sum);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vwma() {
        let prices = vec![10.0, 12.0, 14.0, 16.0];
        let volumes = vec![100.0, 200.0, 100.0, 200.0];
        let result = vwma(&prices, &volumes, 2).unwrap();

        assert_eq!(result.len(), 4);
        assert!(result[0].is_none());

        // i=1: (10*100 + 12*200) / (100+200) = (1000 + 2400) / 300 = 3400/300 = 11.333
        assert!(result[1].is_some());
        let val = result[1].unwrap();
        assert!((val - 11.3333).abs() < 0.001);
    }

    /// Reference implementation: the pre-optimization full-window rescan.
    fn vwma_reference(data: &[f64], volumes: &[f64], period: usize) -> Vec<Option<f64>> {
        let mut out = vec![None; data.len()];
        for i in (period - 1)..data.len() {
            let s = i + 1 - period;
            let mut pv = 0.0;
            let mut v = 0.0;
            for (&p, &q) in data[s..=i].iter().zip(volumes[s..=i].iter()) {
                pv += p * q;
                v += q;
            }
            if v != 0.0 {
                out[i] = Some(pv / v);
            }
        }
        out
    }

    #[test]
    fn rolling_survives_catastrophic_cancellation() {
        // One enormous volume swamps the small ones; subtracting it back out
        // leaves a denominator of zero unless the window is rebuilt.
        let data = [10.0, 11.0, 12.0, 13.0, 14.0];
        let volumes = [1e17, 1.0, 1.0, 1.0, 1.0];
        let got = vwma(&data, &volumes, 3).unwrap();
        let want = vwma_reference(&data, &volumes, 3);
        assert_eq!(got, want, "rolling diverged from a full rescan");
        assert_eq!(got[3], Some(12.0));
        assert_eq!(got[4], Some(13.0));
    }

    #[test]
    fn rolling_survives_numerator_cancellation() {
        // The volumes are ordinary here, so only the numerator cancels: a price
        // outlier swamps the later price*volume terms and subtracting it back
        // out leaves a numerator near zero unless the window is rebuilt.
        let data = [1e20, 10.0, 11.0, 12.0, 13.0];
        let volumes = [1.0; 5];
        let got = vwma(&data, &volumes, 3).unwrap();
        let want = vwma_reference(&data, &volumes, 3);
        assert_eq!(got, want, "rolling diverged from a full rescan");
        assert_eq!(got[3], Some(11.0));
        assert_eq!(got[4], Some(12.0));
    }

    #[test]
    fn rolling_matches_reference_within_bound() {
        let n = 5_000;
        let data: Vec<f64> = (0..n)
            .map(|i| 100.0 + (i as f64 * 0.7).sin() * 20.0)
            .collect();
        let volumes: Vec<f64> = (0..n).map(|i| 1_000.0 + (i % 997) as f64).collect();

        for period in [2usize, 20, 200] {
            let got = vwma(&data, &volumes, period).unwrap();
            let want = vwma_reference(&data, &volumes, period);
            assert_eq!(got.len(), want.len());
            for (i, (g, w)) in got.iter().zip(want.iter()).enumerate() {
                match (g, w) {
                    (Some(a), Some(b)) => {
                        let rel = (a - b).abs() / b.abs().max(1e-12);
                        assert!(
                            rel < 1e-11,
                            "period={period} i={i}: {a} vs {b} (rel {rel:e})"
                        );
                    }
                    (None, None) => {}
                    _ => panic!("period={period} i={i}: None/Some mismatch"),
                }
            }
        }
    }
}
