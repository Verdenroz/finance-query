//! Bit-identity checks of the mfi, cmo, dema and tema loops against reference copies.

use super::{IndicatorError, Result};

fn ema_raw(data: &[f64], period: usize) -> Vec<f64> {
    if period == 0 || data.is_empty() || data.len() < period {
        return Vec::new();
    }
    let multiplier = 2.0 / (period as f64 + 1.0);
    let initial: f64 = data[..period].iter().sum::<f64>() / period as f64;
    let mut result = Vec::with_capacity(data.len() - period + 1);
    result.push(initial);
    let mut prev = initial;
    for &price in &data[period..] {
        let val = (price - prev) * multiplier + prev;
        result.push(val);
        prev = val;
    }
    result
}

fn mfi(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    volumes: &[f64],
    period: usize,
) -> Result<Vec<Option<f64>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    let len = highs.len();
    if lows.len() != len || closes.len() != len || volumes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }
    if len < period + 1 {
        return Err(IndicatorError::InsufficientData {
            need: period + 1,
            got: len,
        });
    }

    let mut typical_prices = Vec::with_capacity(len);
    for i in 0..len {
        typical_prices.push((highs[i] + lows[i] + closes[i]) / 3.0);
    }

    let mut result = vec![None; len];

    let mut raw_money_flow = Vec::with_capacity(len);
    raw_money_flow.push(0.0);

    for i in 1..len {
        raw_money_flow.push(typical_prices[i] * volumes[i]);
    }

    let mut positive_flow = 0.0;
    let mut negative_flow = 0.0;

    for i in 1..=period {
        if typical_prices[i] > typical_prices[i - 1] {
            positive_flow += raw_money_flow[i];
        } else if typical_prices[i] < typical_prices[i - 1] {
            negative_flow += raw_money_flow[i];
        }
    }

    if negative_flow == 0.0 {
        result[period] = Some(100.0);
    } else {
        let money_ratio = positive_flow / negative_flow;
        result[period] = Some(100.0 - (100.0 / (1.0 + money_ratio)));
    }

    for i in (period + 1)..len {
        let old_idx = i - period;
        if typical_prices[old_idx] > typical_prices[old_idx - 1] {
            positive_flow -= raw_money_flow[old_idx];
        } else if typical_prices[old_idx] < typical_prices[old_idx - 1] {
            negative_flow -= raw_money_flow[old_idx];
        }

        if typical_prices[i] > typical_prices[i - 1] {
            positive_flow += raw_money_flow[i];
        } else if typical_prices[i] < typical_prices[i - 1] {
            negative_flow += raw_money_flow[i];
        }

        if negative_flow == 0.0 {
            result[i] = Some(100.0);
        } else {
            let money_ratio = positive_flow / negative_flow;
            result[i] = Some(100.0 - (100.0 / (1.0 + money_ratio)));
        }
    }

    Ok(result)
}

fn cmo(data: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    if data.len() < period + 1 {
        return Err(IndicatorError::InsufficientData {
            need: period + 1,
            got: data.len(),
        });
    }

    let mut result = vec![None; data.len()];

    let mut changes = Vec::with_capacity(data.len());
    changes.push(0.0);
    for i in 1..data.len() {
        changes.push(data[i] - data[i - 1]);
    }

    let mut gains_sum = 0.0;
    let mut losses_sum = 0.0;

    for &change in changes.iter().skip(1).take(period) {
        if change > 0.0 {
            gains_sum += change;
        } else {
            losses_sum += -change;
        }
    }

    let total = gains_sum + losses_sum;
    if total != 0.0 {
        result[period] = Some(((gains_sum - losses_sum) / total) * 100.0);
    } else {
        result[period] = Some(0.0);
    }

    for i in (period + 1)..data.len() {
        let old_change = changes[i - period];
        if old_change > 0.0 {
            gains_sum -= old_change;
        } else {
            losses_sum -= -old_change;
        }

        let new_change = changes[i];
        if new_change > 0.0 {
            gains_sum += new_change;
        } else {
            losses_sum += -new_change;
        }

        let total = gains_sum + losses_sum;
        if total != 0.0 {
            result[i] = Some(((gains_sum - losses_sum) / total) * 100.0);
        } else {
            result[i] = Some(0.0);
        }
    }

    Ok(result)
}

fn dema(data: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    if data.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: period,
            got: data.len(),
        });
    }

    let ema1 = ema_raw(data, period);
    if ema1.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: 2 * period - 1,
            got: data.len(),
        });
    }
    let ema2 = ema_raw(&ema1, period);

    let mut result = vec![None; data.len()];
    let off = period - 1;

    for (k2, &e2) in ema2.iter().enumerate() {
        let orig_idx = k2 + 2 * off;
        result[orig_idx] = Some(2.0 * ema1[k2 + off] - e2);
    }

    Ok(result)
}

fn tema(data: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    if data.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: period,
            got: data.len(),
        });
    }

    let ema1 = ema_raw(data, period);
    if ema1.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: 2 * period - 1,
            got: data.len(),
        });
    }
    let ema2 = ema_raw(&ema1, period);
    if ema2.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: 3 * period - 2,
            got: data.len(),
        });
    }
    let ema3 = ema_raw(&ema2, period);

    let mut result = vec![None; data.len()];
    let off = period - 1;

    for (k3, &e3) in ema3.iter().enumerate() {
        let orig_idx = k3 + 3 * off;
        let e1 = ema1[k3 + 2 * off];
        let e2 = ema2[k3 + off];
        result[orig_idx] = Some(3.0 * e1 - 3.0 * e2 + e3);
    }

    Ok(result)
}

fn chaikin_oscillator(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    volumes: &[f64],
) -> Result<Vec<Option<f64>>> {
    let len = highs.len();
    if lows.len() != len || closes.len() != len || volumes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }
    if len < 10 {
        return Err(IndicatorError::InsufficientData { need: 10, got: len });
    }

    let mut ad_series = Vec::with_capacity(len);
    let mut ad_cumulative = 0.0;

    for i in 0..len {
        let high_low = highs[i] - lows[i];
        if high_low != 0.0 {
            let mf_multiplier = ((closes[i] - lows[i]) - (highs[i] - closes[i])) / high_low;
            ad_cumulative += mf_multiplier * volumes[i];
        }
        ad_series.push(ad_cumulative);
    }

    let ema3_raw = ema_raw(&ad_series, 3);
    let ema10_raw = ema_raw(&ad_series, 10);

    let mut result = vec![None; len];

    for (k, &e10) in ema10_raw.iter().enumerate() {
        result[k + 9] = Some(ema3_raw[k + 7] - e10);
    }

    Ok(result)
}

fn lcg(seed: u64, len: usize) -> Vec<f64> {
    let mut state = seed;
    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            100.0 + (state >> 40) as f64 / 65536.0
        })
        .collect()
}

fn series() -> Vec<Vec<f64>> {
    let mut walk = lcg(7, 64);
    for i in 1..walk.len() {
        walk[i] = walk[i - 1] + (walk[i] - 100.0) / 100.0 - 0.1;
    }
    let mut with_nan = lcg(11, 40);
    with_nan[9] = f64::NAN;
    let mut with_inf = lcg(13, 40);
    with_inf[20] = f64::INFINITY;
    vec![
        lcg(1, 64),
        lcg(2, 1000),
        walk,
        vec![42.0; 30],
        (0..30).map(f64::from).collect(),
        (0..30).map(|i| f64::from(30 - i)).collect(),
        vec![0.0; 12],
        with_nan,
        with_inf,
        lcg(3, 2),
        lcg(5, 1),
        Vec::new(),
    ]
}

fn periods(len: usize) -> Vec<usize> {
    let mut all = vec![0, 1, 2, 3, 5, 14, len.saturating_sub(1), len, len + 1];
    all.sort_unstable();
    all.dedup();
    all
}

fn same(a: Result<Vec<Option<f64>>>, b: Result<Vec<Option<f64>>>) {
    match (a, b) {
        (Ok(a), Ok(b)) => {
            let bits =
                |v: &[Option<f64>]| v.iter().map(|x| x.map(f64::to_bits)).collect::<Vec<_>>();
            assert_eq!(bits(&a), bits(&b));
        }
        (Err(a), Err(b)) => assert_eq!(format!("{a:?}"), format!("{b:?}")),
        (a, b) => panic!("result kinds differ: {a:?} vs {b:?}"),
    }
}

#[test]
fn cmo_matches_reference() {
    for data in series() {
        for period in periods(data.len()) {
            same(super::cmo(&data, period), cmo(&data, period));
        }
    }
}

#[test]
fn dema_matches_reference() {
    for data in series() {
        for period in periods(data.len()) {
            same(super::dema(&data, period), dema(&data, period));
        }
    }
}

#[test]
fn tema_matches_reference() {
    for data in series() {
        for period in periods(data.len()) {
            same(super::tema(&data, period), tema(&data, period));
        }
    }
}

#[test]
fn mfi_matches_reference() {
    for closes in series() {
        let highs: Vec<f64> = closes.iter().map(|c| c + 1.5).collect();
        let lows: Vec<f64> = closes.iter().map(|c| c - 1.0).collect();
        let volumes: Vec<f64> = lcg(9, closes.len()).iter().map(|v| v * 10.0).collect();
        let mut zero_volume = volumes.clone();
        zero_volume.iter_mut().step_by(3).for_each(|v| *v = 0.0);
        for period in periods(closes.len()) {
            for vols in [&volumes, &zero_volume] {
                same(
                    super::mfi(&highs, &lows, &closes, vols, period),
                    mfi(&highs, &lows, &closes, vols, period),
                );
            }
        }
    }
}

#[test]
fn mfi_rejects_mismatched_lengths_like_reference() {
    let a = [1.0, 2.0, 3.0];
    let b = [1.0, 2.0];
    same(super::mfi(&a, &b, &a, &a, 1), mfi(&a, &b, &a, &a, 1));
}

#[test]
fn chaikin_oscillator_matches_reference() {
    for closes in series() {
        let highs: Vec<f64> = closes.iter().map(|c| c + 1.5).collect();
        let lows: Vec<f64> = closes.iter().map(|c| c - 1.0).collect();
        let volumes: Vec<f64> = lcg(9, closes.len()).iter().map(|v| v * 10.0).collect();
        for (h, l) in [(&highs, &lows), (&closes, &closes)] {
            same(
                super::chaikin_oscillator(h, l, &closes, &volumes),
                chaikin_oscillator(h, l, &closes, &volumes),
            );
        }
    }
    let a = [1.0; 12];
    let b = [1.0; 11];
    same(
        super::chaikin_oscillator(&a, &b, &a, &a),
        chaikin_oscillator(&a, &b, &a, &a),
    );
}
