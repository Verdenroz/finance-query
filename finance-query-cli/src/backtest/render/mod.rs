mod conditions;
mod confirm;
mod setup;
mod strategy;

use super::state::{App, Screen};

/// Main UI rendering dispatch function
pub fn ui(f: &mut ratatui::Frame, app: &App) {
    match app.screen {
        Screen::Welcome => setup::render_welcome(f, app),
        Screen::PresetSelect => setup::render_preset_select(f, app),
        Screen::ConfigEditor => setup::render_config_editor(f, app),
        Screen::StrategyBuilder => strategy::render_strategy_builder(f, app),
        Screen::IndicatorBrowser => strategy::render_indicator_browser(f, app),
        Screen::IndicatorConfig => strategy::render_indicator_config(f, app),
        Screen::ComparisonConfig => conditions::render_comparison_config(f, app),
        Screen::TargetConfig => conditions::render_target_config(f, app),
        Screen::Confirmation => confirm::render_confirmation(f, app),
        Screen::OptimizerSetup => confirm::render_optimizer_setup(f, app),
        Screen::SavePreset => confirm::render_save_preset(f, app),
    }
}
