use super::indicators::IndicatorCategory;
use super::state::{
    App, ConditionPanel, ConditionTarget, ConfigField, OPTIMIZER_FIELD_END,
    OPTIMIZER_FIELD_IN_SAMPLE, OPTIMIZER_FIELD_MAX, OPTIMIZER_FIELD_OOS, OPTIMIZER_FIELD_START,
    OPTIMIZER_FIELD_STEP, Screen,
};
use super::types::ComparisonType;
use super::types::{OptimizeConfig, OptimizerParamDef, all_optimize_metrics};
use super::user_presets;
use crossterm::event::{KeyCode, KeyModifiers};

/// Increment/decrement step for arrow-key adjustment of condition target values.
const TARGET_VALUE_STEP: f64 = 0.1;
/// Minimum allowed value for an optimizer parameter step (prevents zero/negative steps).
const MIN_OPTIMIZER_STEP: f64 = 0.1;

/// Main input handler that dispatches to screen-specific handlers
pub fn handle_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    // Global quit
    if matches!(key, KeyCode::Char('c')) && modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }

    // Editing mode — route to the screen's own handler so each screen
    // can parse its edit buffer correctly (e.g. optimizer fields vs config fields).
    if app.editing {
        if app.screen == Screen::OptimizerSetup {
            handle_optimizer_input(app, key);
        } else {
            match key {
                KeyCode::Enter => app.finish_editing(),
                KeyCode::Esc => app.cancel_editing(),
                KeyCode::Char(c) => app.edit_buffer.push(c),
                KeyCode::Backspace => {
                    app.edit_buffer.pop();
                }
                _ => {}
            }
        }
        return;
    }

    match app.screen {
        Screen::Welcome => handle_welcome_input(app, key),
        Screen::PresetSelect => handle_preset_input(app, key),
        Screen::ConfigEditor => handle_config_input(app, key),
        Screen::StrategyBuilder => handle_strategy_input(app, key),
        Screen::IndicatorBrowser => handle_indicator_browser_input(app, key),
        Screen::IndicatorConfig => handle_indicator_config_input(app, key),
        Screen::ComparisonConfig => handle_comparison_input(app, key),
        Screen::TargetConfig => handle_target_input(app, key),
        Screen::Confirmation => handle_confirmation_input(app, key),
        Screen::OptimizerSetup => handle_optimizer_input(app, key),
        Screen::SavePreset => handle_save_preset_input(app, key),
    }
}

fn handle_welcome_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('1') | KeyCode::Char('n') => {
            app.push_screen(Screen::ConfigEditor);
        }
        KeyCode::Char('2') | KeyCode::Char('p') => {
            app.push_screen(Screen::PresetSelect);
        }
        KeyCode::Char('3') | KeyCode::Char('s') => {
            app.push_screen(Screen::StrategyBuilder);
        }
        _ => {}
    }
}

fn handle_preset_input(app: &mut App, key: KeyCode) {
    let len = app.total_preset_count();
    if len == 0 {
        if matches!(key, KeyCode::Char('q') | KeyCode::Esc) {
            app.pop_screen();
        }
        return;
    }
    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.pop_screen(),
        KeyCode::Down | KeyCode::Char('j') => {
            app.preset_idx = (app.preset_idx + 1) % len;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.preset_idx = (app.preset_idx + len - 1) % len;
        }
        KeyCode::Enter => {
            app.load_preset(app.preset_idx);
            app.screen = Screen::ConfigEditor;
            app.prev_screens.clear();
        }
        KeyCode::Char('d') if app.is_user_preset(app.preset_idx) => {
            let user_idx = app.preset_idx - app.presets.len();
            if let Some(preset) = app.user_presets.get(user_idx) {
                let name = preset.name.clone();
                if let Err(e) = user_presets::delete_user_preset(&name) {
                    tracing::warn!("Failed to delete preset '{name}': {e}");
                }
                app.reload_user_presets();
                // Keep cursor in bounds after deletion
                let new_len = app.total_preset_count();
                if new_len > 0 && app.preset_idx >= new_len {
                    app.preset_idx = new_len - 1;
                }
            }
        }
        _ => {}
    }
}

fn handle_config_input(app: &mut App, key: KeyCode) {
    let len = ConfigField::all().len();
    match key {
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.prev_screens.is_empty() {
                app.should_quit = true;
            } else {
                app.pop_screen();
            }
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
            app.config_field_idx = (app.config_field_idx + 1) % len;
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
            app.config_field_idx = (app.config_field_idx + len - 1) % len;
        }
        KeyCode::Enter | KeyCode::Char('e') => {
            app.start_editing();
        }
        KeyCode::Char('s') => {
            app.push_screen(Screen::StrategyBuilder);
        }
        KeyCode::Char('p') => {
            app.push_screen(Screen::PresetSelect);
        }
        KeyCode::Char('r') => {
            if app.can_run() {
                app.push_screen(Screen::Confirmation);
            } else {
                app.edit_error = Some("Need symbol and at least one entry/exit condition".into());
            }
        }
        _ => {}
    }
}

fn handle_strategy_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.prev_screens.is_empty() {
                app.screen = Screen::ConfigEditor;
            } else {
                app.pop_screen();
            }
        }
        // Switch between entry/exit panels
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            app.active_condition_panel = match app.active_condition_panel {
                ConditionPanel::Entry => ConditionPanel::Exit,
                ConditionPanel::Exit => ConditionPanel::Entry,
            };
        }
        // Navigate within the active condition list
        KeyCode::Up | KeyCode::Char('k') => {
            let len = match app.active_condition_panel {
                ConditionPanel::Entry => app.config.strategy.entry_conditions.conditions.len(),
                ConditionPanel::Exit => app.config.strategy.exit_conditions.conditions.len(),
            };
            if len > 0 {
                let idx = match app.active_condition_panel {
                    ConditionPanel::Entry => &mut app.entry_condition_idx,
                    ConditionPanel::Exit => &mut app.exit_condition_idx,
                };
                *idx = (*idx + len - 1) % len;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let len = match app.active_condition_panel {
                ConditionPanel::Entry => app.config.strategy.entry_conditions.conditions.len(),
                ConditionPanel::Exit => app.config.strategy.exit_conditions.conditions.len(),
            };
            if len > 0 {
                let idx = match app.active_condition_panel {
                    ConditionPanel::Entry => &mut app.entry_condition_idx,
                    ConditionPanel::Exit => &mut app.exit_condition_idx,
                };
                *idx = (*idx + 1) % len;
            }
        }
        // Add new condition based on active panel
        KeyCode::Enter => {
            match app.active_condition_panel {
                ConditionPanel::Entry => {
                    app.condition_target = ConditionTarget::Entry;
                }
                ConditionPanel::Exit => {
                    app.condition_target = ConditionTarget::Exit;
                }
            }
            app.category_idx = 0;
            app.indicator_idx = 0;
            app.push_screen(Screen::IndicatorBrowser);
        }
        // Add new condition to entry
        KeyCode::Char('1') | KeyCode::Char('e') => {
            app.condition_target = ConditionTarget::Entry;
            app.category_idx = 0;
            app.indicator_idx = 0;
            app.push_screen(Screen::IndicatorBrowser);
        }
        // Add new condition to exit
        KeyCode::Char('2') | KeyCode::Char('x') => {
            app.condition_target = ConditionTarget::Exit;
            app.category_idx = 0;
            app.indicator_idx = 0;
            app.push_screen(Screen::IndicatorBrowser);
        }
        KeyCode::Char('3') => {
            if app.config.allow_short {
                app.condition_target = ConditionTarget::ShortEntry;
                app.category_idx = 0;
                app.indicator_idx = 0;
                app.push_screen(Screen::IndicatorBrowser);
            }
        }
        KeyCode::Char('4') => {
            if app.config.allow_short {
                app.condition_target = ConditionTarget::ShortExit;
                app.category_idx = 0;
                app.indicator_idx = 0;
                app.push_screen(Screen::IndicatorBrowser);
            }
        }
        // Toggle AND/OR for the selected condition
        KeyCode::Char('c') => match app.active_condition_panel {
            ConditionPanel::Entry => {
                let idx = app.entry_condition_idx;
                app.config.strategy.entry_conditions.toggle_op_at(idx);
            }
            ConditionPanel::Exit => {
                let idx = app.exit_condition_idx;
                app.config.strategy.exit_conditions.toggle_op_at(idx);
            }
        },
        // Delete the selected condition
        KeyCode::Char('d') | KeyCode::Delete | KeyCode::Backspace => {
            match app.active_condition_panel {
                ConditionPanel::Entry => {
                    let len = app.config.strategy.entry_conditions.conditions.len();
                    if len > 0 {
                        let idx = app.entry_condition_idx;
                        app.config.strategy.entry_conditions.remove_at(idx);
                        // Adjust index if needed
                        if app.entry_condition_idx
                            >= app.config.strategy.entry_conditions.conditions.len()
                            && app.entry_condition_idx > 0
                        {
                            app.entry_condition_idx -= 1;
                        }
                    }
                }
                ConditionPanel::Exit => {
                    let len = app.config.strategy.exit_conditions.conditions.len();
                    if len > 0 {
                        let idx = app.exit_condition_idx;
                        app.config.strategy.exit_conditions.remove_at(idx);
                        // Adjust index if needed
                        if app.exit_condition_idx
                            >= app.config.strategy.exit_conditions.conditions.len()
                            && app.exit_condition_idx > 0
                        {
                            app.exit_condition_idx -= 1;
                        }
                    }
                }
            }
        }
        KeyCode::Char('r') => {
            if app.can_run() {
                app.push_screen(Screen::Confirmation);
            }
        }
        KeyCode::Char('b') => {
            app.screen = Screen::ConfigEditor;
        }
        _ => {}
    }
}

fn handle_indicator_browser_input(app: &mut App, key: KeyCode) {
    let cat_len = IndicatorCategory::all().len();
    let ind_len = app.indicators_in_category().len();

    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.pop_screen(),
        KeyCode::Left | KeyCode::Char('h') => {
            app.category_idx = (app.category_idx + cat_len - 1) % cat_len;
            app.indicator_idx = 0;
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.category_idx = (app.category_idx + 1) % cat_len;
            app.indicator_idx = 0;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if ind_len > 0 {
                app.indicator_idx = (app.indicator_idx + 1) % ind_len;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if ind_len > 0 {
                app.indicator_idx = (app.indicator_idx + ind_len - 1) % ind_len;
            }
        }
        KeyCode::Enter => {
            app.select_indicator();
        }
        _ => {}
    }
}

fn handle_indicator_config_input(app: &mut App, key: KeyCode) {
    let param_len = app.param_values.len();
    if param_len == 0 {
        // No parameters, go directly to comparison
        if matches!(key, KeyCode::Enter | KeyCode::Char('n')) {
            app.finish_indicator_config();
        } else if matches!(key, KeyCode::Esc | KeyCode::Char('q')) {
            app.pop_screen();
        }
        return;
    }

    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.pop_screen(),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
            app.param_idx = (app.param_idx + 1) % param_len;
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
            app.param_idx = (app.param_idx + param_len - 1) % param_len;
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if let Some(ref ind) = app.building_indicator {
                let param = &ind.indicator.params[app.param_idx];
                let step = param.step;
                app.param_values[app.param_idx] =
                    (app.param_values[app.param_idx] - step).max(param.min);
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if let Some(ref ind) = app.building_indicator {
                let param = &ind.indicator.params[app.param_idx];
                let step = param.step;
                app.param_values[app.param_idx] =
                    (app.param_values[app.param_idx] + step).min(param.max);
            }
        }
        KeyCode::Enter | KeyCode::Char('n') => {
            app.finish_indicator_config();
        }
        _ => {}
    }
}

fn handle_comparison_input(app: &mut App, key: KeyCode) {
    let len = ComparisonType::all().len();
    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.pop_screen(),
        KeyCode::Down | KeyCode::Char('j') => {
            app.comparison_idx = (app.comparison_idx + 1) % len;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.comparison_idx = (app.comparison_idx + len - 1) % len;
        }
        KeyCode::Enter => {
            app.select_comparison();
        }
        _ => {}
    }
}

fn handle_target_input(app: &mut App, key: KeyCode) {
    // Check if we're in input mode (typing a number)
    if app.target_input_mode {
        match key {
            KeyCode::Enter => {
                // Parse and apply the value
                if let Ok(val) = app.target_edit_buffer.parse::<f64>() {
                    if app.editing_target_value {
                        app.target_value = val;
                    } else {
                        app.target_value2 = val;
                    }
                }
                app.target_input_mode = false;
                app.target_edit_buffer.clear();
            }
            KeyCode::Esc => {
                // Cancel editing
                app.target_input_mode = false;
                app.target_edit_buffer.clear();
            }
            KeyCode::Backspace => {
                app.target_edit_buffer.pop();
                // If buffer becomes empty, exit input mode
                if app.target_edit_buffer.is_empty() {
                    app.target_input_mode = false;
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' || c == '-' => {
                // Only allow one decimal point and minus at start
                if c == '.' && app.target_edit_buffer.contains('.') {
                    return;
                }
                if c == '-' && !app.target_edit_buffer.is_empty() {
                    return;
                }
                app.target_edit_buffer.push(c);
            }
            _ => {}
        }
        return;
    }

    // Normal mode
    let needs_two_values = app
        .building_comparison
        .map(|c| c.needs_range())
        .unwrap_or(false);

    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.pop_screen(),
        // Switch between primary and secondary value
        KeyCode::Tab => {
            if needs_two_values {
                app.editing_target_value = !app.editing_target_value;
            }
        }
        // Arrow keys for quick adjustment
        KeyCode::Left | KeyCode::Char('h') => {
            if app.editing_target_value {
                app.target_value -= 1.0;
            } else {
                app.target_value2 -= 1.0;
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if app.editing_target_value {
                app.target_value += 1.0;
            } else {
                app.target_value2 += 1.0;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.editing_target_value {
                app.target_value -= TARGET_VALUE_STEP;
            } else {
                app.target_value2 -= TARGET_VALUE_STEP;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.editing_target_value {
                app.target_value += TARGET_VALUE_STEP;
            } else {
                app.target_value2 += TARGET_VALUE_STEP;
            }
        }
        // Start typing a number
        KeyCode::Char(c) if c.is_ascii_digit() || c == '.' || c == '-' => {
            app.target_input_mode = true;
            app.target_edit_buffer.clear();
            app.target_edit_buffer.push(c);
        }
        KeyCode::Enter | KeyCode::Char('n') => {
            app.finish_condition();
        }
        _ => {}
    }
}

fn handle_confirmation_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('y') | KeyCode::Enter => {
            app.run_with_optimizer = false;
            app.confirmed = true;
        }
        KeyCode::Char('s') => {
            app.save_preset_buffer = app.config.strategy.name.clone();
            app.save_preset_error = None;
            app.push_screen(Screen::SavePreset);
        }
        KeyCode::Char('o') => {
            // Launch optimizer setup — auto-extract params from current strategy
            app.optimizer_params = OptimizerParamDef::from_strategy(&app.config.strategy);
            app.optimizer_param_idx = 0;
            app.optimizer_field_idx = 0;
            app.push_screen(Screen::OptimizerSetup);
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.pop_screen();
        }
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}

fn handle_save_preset_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Enter => {
            let name = app.save_preset_buffer.trim().to_string();
            if name.is_empty() {
                app.save_preset_error = Some("Preset name cannot be empty".into());
                return;
            }
            let description = format!("Saved from {}", app.config.strategy.name);
            match user_presets::save_user_preset(name, description, &app.config) {
                Ok(()) => {
                    app.reload_user_presets();
                    app.save_preset_buffer.clear();
                    app.save_preset_error = None;
                    app.pop_screen();
                }
                Err(e) => {
                    app.save_preset_error = Some(e.to_string());
                }
            }
        }
        KeyCode::Esc => {
            app.save_preset_buffer.clear();
            app.save_preset_error = None;
            app.pop_screen();
        }
        KeyCode::Char(c) => {
            app.save_preset_buffer.push(c);
        }
        KeyCode::Backspace => {
            app.save_preset_buffer.pop();
        }
        _ => {}
    }
}

fn handle_optimizer_input(app: &mut App, key: KeyCode) {
    let n_params = app.optimizer_params.len();
    let n_metrics = all_optimize_metrics().len();

    // Editing mode: routed here from handle_input when screen == OptimizerSetup
    if app.editing {
        match key {
            KeyCode::Enter => {
                let val: Option<f64> = app.edit_buffer.trim().parse().ok();
                if let Some(v) = val {
                    let param = app.optimizer_params.get_mut(app.optimizer_param_idx);
                    match app.optimizer_field_idx {
                        OPTIMIZER_FIELD_START => {
                            if let Some(p) = param {
                                if v >= p.end {
                                    app.edit_error = Some(format!(
                                        "Start ({v}) must be less than end ({})",
                                        p.end
                                    ));
                                } else {
                                    p.start = v;
                                    app.edit_error = None;
                                }
                            }
                        }
                        OPTIMIZER_FIELD_END => {
                            if let Some(p) = param {
                                if v <= p.start {
                                    app.edit_error = Some(format!(
                                        "End ({v}) must be greater than start ({})",
                                        p.start
                                    ));
                                } else {
                                    p.end = v;
                                    app.edit_error = None;
                                }
                            }
                        }
                        OPTIMIZER_FIELD_STEP => {
                            if let Some(p) = param {
                                p.step = v.max(MIN_OPTIMIZER_STEP);
                            }
                        }
                        OPTIMIZER_FIELD_IN_SAMPLE => app.optimizer_in_sample = v as usize,
                        OPTIMIZER_FIELD_OOS => app.optimizer_oos = v as usize,
                        _ => {}
                    }
                }
                app.editing = false;
                app.edit_buffer.clear();
                app.edit_error = None;
            }
            KeyCode::Esc => {
                app.editing = false;
                app.edit_buffer.clear();
                app.edit_error = None;
            }
            KeyCode::Char(c) => app.edit_buffer.push(c),
            KeyCode::Backspace => {
                app.edit_buffer.pop();
            }
            _ => {}
        }
        return;
    }

    match key {
        // Navigate params
        KeyCode::Up | KeyCode::Char('k') => {
            if n_params > 0 {
                app.optimizer_param_idx = app.optimizer_param_idx.saturating_sub(1);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if n_params > 0 {
                app.optimizer_param_idx =
                    (app.optimizer_param_idx + 1).min(n_params.saturating_sub(1));
            }
        }
        // Switch which field is being edited (start/end/step/in_sample/oos)
        KeyCode::Left | KeyCode::Char('h') => {
            app.optimizer_field_idx = app.optimizer_field_idx.saturating_sub(1);
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.optimizer_field_idx = (app.optimizer_field_idx + 1).min(OPTIMIZER_FIELD_MAX);
        }
        // Edit current field
        KeyCode::Enter => {
            let param = app.optimizer_params.get(app.optimizer_param_idx);
            let buf = match app.optimizer_field_idx {
                OPTIMIZER_FIELD_START => param.map(|p| p.start.to_string()),
                OPTIMIZER_FIELD_END => param.map(|p| p.end.to_string()),
                OPTIMIZER_FIELD_STEP => param.map(|p| p.step.to_string()),
                OPTIMIZER_FIELD_IN_SAMPLE => Some(app.optimizer_in_sample.to_string()),
                OPTIMIZER_FIELD_OOS => Some(app.optimizer_oos.to_string()),
                _ => None,
            };
            if let Some(val) = buf {
                app.edit_buffer = val;
                app.editing = true;
                app.edit_error = None;
            }
        }
        // Toggle enabled
        KeyCode::Char(' ') => {
            if let Some(param) = app.optimizer_params.get_mut(app.optimizer_param_idx) {
                param.enabled = !param.enabled;
            }
        }
        // Cycle optimize metric
        KeyCode::Char('m') => {
            app.optimizer_metric_idx = (app.optimizer_metric_idx + 1) % n_metrics;
        }
        // Toggle walk-forward
        KeyCode::Char('w') => {
            app.optimizer_walk_forward = !app.optimizer_walk_forward;
        }
        // Run with optimizer
        KeyCode::Char('r') => {
            let metrics = all_optimize_metrics();
            let idx = app
                .optimizer_metric_idx
                .min(metrics.len().saturating_sub(1));
            let metric = metrics[idx];
            app.config.optimizer = Some(OptimizeConfig {
                params: app.optimizer_params.clone(),
                metric,
                walk_forward: app.optimizer_walk_forward,
                in_sample_bars: app.optimizer_in_sample,
                out_of_sample_bars: app.optimizer_oos,
            });
            app.run_with_optimizer = true;
            app.confirmed = true;
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.pop_screen();
        }
        _ => {}
    }
}
