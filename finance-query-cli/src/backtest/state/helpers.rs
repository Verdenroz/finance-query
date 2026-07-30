use super::CostProfile;
use crate::backtest::indicators::IndicatorCategory;
use crate::backtest::types::{BacktestConfiguration, BuiltIndicator, CompareTarget};
use crate::error::Result;
use finance_query::{Interval, TimeRange};

// Helper functions for parsing and formatting

pub fn interval_to_string(interval: Interval) -> String {
    interval.to_string()
}

pub fn range_to_string(range: TimeRange) -> String {
    range.to_string()
}

pub fn parse_bool(s: &str) -> Result<bool> {
    match s.to_lowercase().as_str() {
        "true" | "yes" | "y" | "1" => Ok(true),
        "false" | "no" | "n" | "0" => Ok(false),
        _ => Err(crate::error::CliError::InvalidArgument(format!(
            "Invalid boolean: {}",
            s
        ))),
    }
}

fn interval_rank(interval: Interval) -> usize {
    match interval {
        Interval::OneMinute => 0,
        Interval::FiveMinutes => 1,
        Interval::FifteenMinutes => 2,
        Interval::ThirtyMinutes => 3,
        Interval::OneHour => 4,
        Interval::OneDay => 5,
        Interval::OneWeek => 6,
        Interval::OneMonth => 7,
        Interval::ThreeMonths => 8,
    }
}

pub(super) fn available_htf_intervals_for_base(base_interval: Interval) -> Vec<Option<Interval>> {
    let all = [
        Interval::OneMinute,
        Interval::FiveMinutes,
        Interval::FifteenMinutes,
        Interval::ThirtyMinutes,
        Interval::OneHour,
        Interval::OneDay,
        Interval::OneWeek,
        Interval::OneMonth,
        Interval::ThreeMonths,
    ];

    let base_rank = interval_rank(base_interval);
    let mut options = vec![None];
    options.extend(
        all.into_iter()
            .filter(|interval| interval_rank(*interval) > base_rank)
            .map(Some),
    );
    options
}

pub(super) fn indicator_supports_htf(indicator: &BuiltIndicator) -> bool {
    !matches!(indicator.indicator.category, IndicatorCategory::PriceAction)
}

pub(super) fn target_supports_htf(target: &CompareTarget) -> bool {
    match target {
        CompareTarget::Indicator(other) => indicator_supports_htf(other),
        CompareTarget::Value(_) | CompareTarget::Range(_, _) => true,
    }
}

const COST_EPS: f64 = 1e-9;

// Cost profiles are intentionally conservative defaults that are easy to reason
// about in the TUI and can be tweaked manually afterward.
const PROFILE_ZERO_COST: (f64, f64, f64, f64, f64) = (0.0, 0.0, 0.0, 0.0, 0.0);
const PROFILE_IBKR_US: (f64, f64, f64, f64, f64) = (0.0005, 0.35, 0.0002, 0.0001, 0.0);
const PROFILE_COMMISSION_FREE: (f64, f64, f64, f64, f64) = (0.0, 0.0, 0.0005, 0.0004, 0.0);
const PROFILE_UK_STAMP_DUTY: (f64, f64, f64, f64, f64) = (0.0005, 0.0, 0.0005, 0.0002, 0.005);

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= COST_EPS
}

fn matches_cost_tuple(config: &BacktestConfiguration, tuple: (f64, f64, f64, f64, f64)) -> bool {
    approx_eq(config.commission, tuple.0)
        && approx_eq(config.commission_flat, tuple.1)
        && approx_eq(config.slippage, tuple.2)
        && approx_eq(config.spread_pct, tuple.3)
        && approx_eq(config.transaction_tax_pct, tuple.4)
}

pub(super) fn detect_cost_profile(config: &BacktestConfiguration) -> CostProfile {
    if matches_cost_tuple(config, PROFILE_ZERO_COST) {
        CostProfile::ZeroCost
    } else if matches_cost_tuple(config, PROFILE_IBKR_US) {
        CostProfile::InteractiveBrokersUs
    } else if matches_cost_tuple(config, PROFILE_COMMISSION_FREE) {
        CostProfile::CommissionFreeRetail
    } else if matches_cost_tuple(config, PROFILE_UK_STAMP_DUTY) {
        CostProfile::UkStampDuty
    } else {
        CostProfile::Custom
    }
}

pub(super) fn apply_cost_profile(config: &mut BacktestConfiguration, profile: CostProfile) {
    let (commission, commission_flat, slippage, spread_pct, transaction_tax_pct) = match profile {
        CostProfile::Custom => {
            return;
        }
        CostProfile::ZeroCost => PROFILE_ZERO_COST,
        CostProfile::InteractiveBrokersUs => PROFILE_IBKR_US,
        CostProfile::CommissionFreeRetail => PROFILE_COMMISSION_FREE,
        CostProfile::UkStampDuty => PROFILE_UK_STAMP_DUTY,
    };

    config.commission = commission;
    config.commission_flat = commission_flat;
    config.slippage = slippage;
    config.spread_pct = spread_pct;
    config.transaction_tax_pct = transaction_tax_pct;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn htf_options_only_include_higher_intervals() {
        let options = available_htf_intervals_for_base(Interval::OneHour);
        assert_eq!(options[0], None);
        assert_eq!(
            options,
            vec![
                None,
                Some(Interval::OneDay),
                Some(Interval::OneWeek),
                Some(Interval::OneMonth),
                Some(Interval::ThreeMonths),
            ]
        );
    }

    #[test]
    fn detect_zero_cost_profile() {
        let mut cfg = BacktestConfiguration::default();
        apply_cost_profile(&mut cfg, CostProfile::ZeroCost);
        assert_eq!(detect_cost_profile(&cfg), CostProfile::ZeroCost);
    }

    #[test]
    fn apply_uk_stamp_duty_profile_sets_expected_fields() {
        let mut cfg = BacktestConfiguration::default();
        apply_cost_profile(&mut cfg, CostProfile::UkStampDuty);

        assert!(approx_eq(cfg.commission, PROFILE_UK_STAMP_DUTY.0));
        assert!(approx_eq(cfg.commission_flat, PROFILE_UK_STAMP_DUTY.1));
        assert!(approx_eq(cfg.slippage, PROFILE_UK_STAMP_DUTY.2));
        assert!(approx_eq(cfg.spread_pct, PROFILE_UK_STAMP_DUTY.3));
        assert!(approx_eq(cfg.transaction_tax_pct, PROFILE_UK_STAMP_DUTY.4));
    }
}
