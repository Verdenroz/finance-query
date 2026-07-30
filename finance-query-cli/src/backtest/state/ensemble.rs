use super::App;
use crate::backtest::types::{EnsembleConfig, EnsembleMemberConfig};
use crate::error::Result;

impl App {
    pub fn toggle_ensemble_selection(&mut self, idx: usize) {
        if let Some(pos) = self.ensemble_selected.iter().position(|v| *v == idx) {
            self.ensemble_selected.remove(pos);
            self.ensemble_weights.remove(&idx);
        } else {
            self.ensemble_selected.push(idx);
            self.ensemble_weights.entry(idx).or_insert(1.0);
            self.ensemble_selected.sort_unstable();
            self.ensemble_selected.dedup();
        }
    }

    pub fn ensemble_weight_for(&self, idx: usize) -> f64 {
        self.ensemble_weights.get(&idx).copied().unwrap_or(1.0)
    }

    pub fn adjust_ensemble_weight(&mut self, idx: usize, delta: f64) {
        if !self.ensemble_selected.contains(&idx) {
            return;
        }
        let current = self.ensemble_weight_for(idx);
        let next = (current + delta).clamp(0.0, 10.0);
        self.ensemble_weights.insert(idx, next);
    }

    pub fn set_ensemble_weight(&mut self, idx: usize, weight: f64) -> Result<()> {
        if !self.ensemble_selected.contains(&idx) {
            return Err(crate::error::CliError::InvalidArgument(
                "Select the preset first (Space) before editing weight".into(),
            ));
        }
        if !weight.is_finite() || !(0.0..=10.0).contains(&weight) {
            return Err(crate::error::CliError::InvalidArgument(
                "Weight must be between 0.0 and 10.0".into(),
            ));
        }
        self.ensemble_weights.insert(idx, weight);
        Ok(())
    }

    /// Build ensemble config from selected presets and apply it as active config.
    /// Existing execution and risk settings (capital, commissions, slippage, etc.) are preserved;
    /// only the ensemble/strategy composition fields are updated.
    pub fn apply_selected_ensemble(&mut self) -> Result<()> {
        let mut selected = self.ensemble_selected.clone();
        selected.sort_unstable();
        selected.dedup();

        let had_optimizer = self.config.optimizer.is_some();

        if selected.len() < 2 {
            return Err(crate::error::CliError::InvalidArgument(
                "Select at least 2 presets to compose an ensemble".into(),
            ));
        }

        let mut members = Vec::with_capacity(selected.len());
        for idx in selected {
            let (name, _, cfg) = self.preset_entry(idx).ok_or_else(|| {
                crate::error::CliError::InvalidArgument("Invalid ensemble preset selection".into())
            })?;
            if cfg.strategy.entry_conditions.conditions.is_empty()
                || cfg.strategy.exit_conditions.conditions.is_empty()
            {
                return Err(crate::error::CliError::InvalidArgument(format!(
                    "Preset '{}' must have both entry and exit conditions",
                    name
                )));
            }
            let weight = self.ensemble_weight_for(idx);
            members.push(EnsembleMemberConfig {
                name,
                strategy: cfg.strategy,
                weight,
            });
        }

        // Only update ensemble/strategy fields — preserve the user's existing capital,
        // commission, slippage, risk-free rate, and all other execution/risk settings.
        self.config.ensemble = Some(EnsembleConfig {
            mode: self.ensemble_mode,
            members,
        });
        self.config.strategy.name = format!("Ensemble ({})", self.ensemble_mode.name());

        // Optimizer currently tunes one StrategyConfig parameter space.
        // Ensemble optimization can be added later with explicit semantics.
        self.config.optimizer = None;
        if had_optimizer {
            self.edit_error = Some(
                "Optimizer settings were cleared because ensembles are not optimizer-compatible yet"
                    .into(),
            );
        } else {
            self.edit_error = None;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::types::{BacktestConfiguration, OptimizeConfig};
    use crate::backtest::user_presets::UserStrategyPreset;

    #[test]
    fn apply_selected_ensemble_rejects_member_without_conditions() {
        let mut app = App::new(Some("AAPL".to_string()));
        app.user_presets.push(UserStrategyPreset {
            name: "Broken".to_string(),
            description: "No conditions".to_string(),
            config: BacktestConfiguration::default(),
        });

        let invalid_idx = app.total_preset_count() - 1;
        app.ensemble_selected = vec![0, invalid_idx];

        let err = app
            .apply_selected_ensemble()
            .expect_err("should reject invalid member preset");
        assert!(err.to_string().contains("Broken"));
    }

    #[test]
    fn apply_selected_ensemble_clears_optimizer_with_warning() {
        let mut app = App::new(Some("AAPL".to_string()));
        app.config.optimizer = Some(OptimizeConfig::default());
        app.ensemble_selected = vec![0, 1];

        app.apply_selected_ensemble()
            .expect("ensemble composition should succeed");

        assert!(app.config.optimizer.is_none());
        let warning = app
            .edit_error
            .clone()
            .expect("optimizer warning should be surfaced");
        assert!(warning.contains("Optimizer settings were cleared"));
    }

    #[test]
    fn ensemble_toggle_sets_and_clears_weight_defaults() {
        let mut app = App::new(None);
        app.toggle_ensemble_selection(1);
        assert!(app.ensemble_selected.contains(&1));
        assert!((app.ensemble_weight_for(1) - 1.0).abs() < 1e-9);

        app.toggle_ensemble_selection(1);
        assert!(!app.ensemble_selected.contains(&1));
        assert!(!app.ensemble_weights.contains_key(&1));
    }

    #[test]
    fn apply_selected_ensemble_uses_custom_member_weights() {
        let mut app = App::new(Some("AAPL".to_string()));
        app.ensemble_selected = vec![0, 1];
        app.ensemble_weights.insert(0, 1.25);
        app.ensemble_weights.insert(1, 0.75);

        app.apply_selected_ensemble()
            .expect("ensemble composition should succeed");

        let ensemble = app
            .config
            .ensemble
            .as_ref()
            .expect("ensemble should be set");
        assert_eq!(ensemble.members.len(), 2);
        assert!((ensemble.members[0].weight - 1.25).abs() < 1e-9);
        assert!((ensemble.members[1].weight - 0.75).abs() < 1e-9);
    }

    #[test]
    fn set_ensemble_weight_validates_selection_and_range() {
        let mut app = App::new(None);

        let err = app
            .set_ensemble_weight(0, 1.5)
            .expect_err("selection guard should fail");
        assert!(err.to_string().contains("Select the preset first"));

        app.ensemble_selected.push(0);
        app.set_ensemble_weight(0, 2.5)
            .expect("valid weight should be accepted");
        assert!((app.ensemble_weight_for(0) - 2.5).abs() < 1e-9);

        let err = app
            .set_ensemble_weight(0, 10.5)
            .expect_err("range guard should fail");
        assert!(err.to_string().contains("between 0.0 and 10.0"));
    }
}
