use super::helpers::{
    available_htf_intervals_for_base, indicator_supports_htf, target_supports_htf,
};
use super::{App, ConditionTarget, Screen};
use crate::backtest::types::{
    BuiltCondition, BuiltIndicator, CompareTarget, ComparisonType, ConditionGroup, LogicalOp,
};
use finance_query::Interval;

impl App {
    pub fn select_indicator(&mut self) {
        // Clone indicator to avoid borrowing issues
        let ind = self.current_indicator().cloned();
        if let Some(ind) = ind {
            // Initialize parameter values with defaults
            let param_values: Vec<f64> = ind.params.iter().map(|p| p.default).collect();
            self.param_values = param_values.clone();
            self.param_idx = 0;
            self.building_htf_interval = None;
            self.building_indicator = Some(BuiltIndicator {
                indicator: ind,
                param_values,
                output: None,
            });
            self.push_screen(Screen::IndicatorConfig);
        }
    }

    pub fn finish_indicator_config(&mut self) {
        if let Some(ref mut ind) = self.building_indicator {
            ind.param_values = self.param_values.clone();
        }
        self.comparison_idx = 0;
        self.push_screen(Screen::ComparisonConfig);
    }

    pub fn select_comparison(&mut self) {
        self.building_comparison = Some(ComparisonType::all()[self.comparison_idx]);

        // Set default target value based on indicator's typical range
        if let Some(ref ind) = self.building_indicator {
            if let Some((low, high)) = ind.indicator.typical_range {
                self.target_value = (low + high) / 2.0;
                self.target_value2 = high;
            } else {
                self.target_value = 0.0;
                self.target_value2 = 0.0;
            }
        }

        self.push_screen(Screen::TargetConfig);
    }

    pub fn finish_condition(&mut self) {
        if let (Some(ind), Some(comp)) = (
            self.building_indicator.take(),
            self.building_comparison.take(),
        ) {
            let target = if comp.needs_range() {
                CompareTarget::Range(self.target_value, self.target_value2)
            } else {
                CompareTarget::Value(self.target_value)
            };

            // HTF scope is only valid when all compared values come from
            // precomputed indicators (price-action values are base-TF only).
            let htf_interval = self
                .building_htf_interval
                .filter(|_| indicator_supports_htf(&ind) && target_supports_htf(&target));

            let condition = BuiltCondition {
                indicator: ind,
                comparison: comp,
                target,
                htf_interval,
                next_op: LogicalOp::And, // Default to AND for new conditions
            };

            // Add to appropriate condition group
            match self.condition_target {
                ConditionTarget::Entry => {
                    self.config
                        .strategy
                        .entry_conditions
                        .conditions
                        .push(condition);
                }
                ConditionTarget::Exit => {
                    self.config
                        .strategy
                        .exit_conditions
                        .conditions
                        .push(condition);
                }
                ConditionTarget::ShortEntry => {
                    if self.config.strategy.short_entry_conditions.is_none() {
                        self.config.strategy.short_entry_conditions = Some(ConditionGroup::new());
                    }
                    self.config
                        .strategy
                        .short_entry_conditions
                        .as_mut()
                        .unwrap()
                        .conditions
                        .push(condition);
                }
                ConditionTarget::ShortExit => {
                    if self.config.strategy.short_exit_conditions.is_none() {
                        self.config.strategy.short_exit_conditions = Some(ConditionGroup::new());
                    }
                    self.config
                        .strategy
                        .short_exit_conditions
                        .as_mut()
                        .unwrap()
                        .conditions
                        .push(condition);
                }
                ConditionTarget::Regime => {
                    self.config
                        .strategy
                        .regime_conditions
                        .conditions
                        .push(condition);
                }
                ConditionTarget::ScaleIn => {
                    self.config
                        .strategy
                        .scale_in_conditions
                        .conditions
                        .push(condition);
                }
                ConditionTarget::ScaleOut => {
                    self.config
                        .strategy
                        .scale_out_conditions
                        .conditions
                        .push(condition);
                }
            }

            // Return to strategy builder
            self.building_htf_interval = None;
            self.prev_screens.clear();
            self.screen = Screen::StrategyBuilder;
        }
    }

    pub fn available_htf_intervals(&self) -> Vec<Option<Interval>> {
        available_htf_intervals_for_base(self.config.interval)
    }

    pub fn cycle_building_htf_interval(&mut self) {
        if self
            .building_indicator
            .as_ref()
            .is_some_and(|ind| !indicator_supports_htf(ind))
        {
            self.building_htf_interval = None;
            return;
        }

        let options = self.available_htf_intervals();
        let current = self.building_htf_interval;
        let idx = options.iter().position(|opt| *opt == current).unwrap_or(0);
        self.building_htf_interval = options[(idx + 1) % options.len()];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::indicators::IndicatorDef;

    #[test]
    fn cycle_htf_interval_wraps_to_none() {
        let mut app = App::new(None);
        app.config.interval = Interval::OneMonth;

        app.cycle_building_htf_interval();
        assert_eq!(app.building_htf_interval, Some(Interval::ThreeMonths));

        app.cycle_building_htf_interval();
        assert_eq!(app.building_htf_interval, None);
    }

    #[test]
    fn price_action_indicator_cannot_cycle_htf_scope() {
        let mut app = App::new(None);
        app.config.interval = Interval::OneDay;
        app.building_indicator = Some(BuiltIndicator {
            indicator: IndicatorDef::find("close"),
            param_values: vec![],
            output: None,
        });

        app.cycle_building_htf_interval();

        assert_eq!(app.building_htf_interval, None);
    }

    #[test]
    fn finish_condition_drops_htf_for_price_action_indicator() {
        let mut app = App::new(None);
        app.condition_target = ConditionTarget::Entry;
        app.building_indicator = Some(BuiltIndicator {
            indicator: IndicatorDef::find("close"),
            param_values: vec![],
            output: None,
        });
        app.building_comparison = Some(ComparisonType::Above);
        app.target_value = 100.0;
        app.building_htf_interval = Some(Interval::OneWeek);

        app.finish_condition();

        let cond = app
            .config
            .strategy
            .entry_conditions
            .conditions
            .first()
            .expect("condition should be created");
        assert_eq!(cond.htf_interval, None);
    }
}
