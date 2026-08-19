use super::App;
use crate::backtest::types::BacktestConfiguration;
use crate::backtest::user_presets;

impl App {
    pub fn total_preset_count(&self) -> usize {
        self.presets.len() + self.user_presets.len()
    }

    pub fn is_user_preset(&self, idx: usize) -> bool {
        idx >= self.presets.len()
    }

    pub fn load_preset(&mut self, idx: usize) {
        let symbol = if self.config.symbol.is_empty() {
            None
        } else {
            Some(self.config.symbol.clone())
        };

        let Some((_, _, mut preset_config)) = self.preset_entry(idx) else {
            return;
        };

        if let Some(sym) = symbol {
            preset_config.symbol = sym;
        }
        self.config = preset_config;
    }

    /// Returns (name, description, config) for a preset index in the combined
    /// built-in + user list.
    pub fn preset_entry(&self, idx: usize) -> Option<(String, String, BacktestConfiguration)> {
        if idx < self.presets.len() {
            let preset = &self.presets[idx];
            Some((
                preset.name.to_string(),
                preset.description.to_string(),
                (preset.config)(),
            ))
        } else {
            let user_idx = idx - self.presets.len();
            self.user_presets.get(user_idx).map(|preset| {
                (
                    preset.name.clone(),
                    preset.description.clone(),
                    preset.config.clone(),
                )
            })
        }
    }

    pub fn reload_user_presets(&mut self) {
        self.user_presets = user_presets::load_user_presets();
    }
}
