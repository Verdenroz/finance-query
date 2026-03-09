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

enum CondNavDir {
    Prev,
    Next,
}

fn active_condition_len(app: &App) -> usize {
    match app.active_condition_panel {
        ConditionPanel::Entry => app.config.strategy.entry_conditions.conditions.len(),
        ConditionPanel::Exit => app.config.strategy.exit_conditions.conditions.len(),
        ConditionPanel::Regime => app.config.strategy.regime_conditions.conditions.len(),
        ConditionPanel::ScaleIn => app.config.strategy.scale_in_conditions.conditions.len(),
        ConditionPanel::ScaleOut => app.config.strategy.scale_out_conditions.conditions.len(),
    }
}

fn active_condition_idx(app: &mut App) -> &mut usize {
    match app.active_condition_panel {
        ConditionPanel::Entry => &mut app.entry_condition_idx,
        ConditionPanel::Exit => &mut app.exit_condition_idx,
        ConditionPanel::Regime => &mut app.regime_condition_idx,
        ConditionPanel::ScaleIn => &mut app.scale_in_condition_idx,
        ConditionPanel::ScaleOut => &mut app.scale_out_condition_idx,
    }
}

fn navigate_condition(app: &mut App, dir: CondNavDir) {
    let len = active_condition_len(app);
    if len > 0 {
        let idx = active_condition_idx(app);
        *idx = match dir {
            CondNavDir::Prev => (*idx + len - 1) % len,
            CondNavDir::Next => (*idx + 1) % len,
        };
    }
}

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
        } else if app.screen == Screen::StrategyBuilder {
            // Scale fraction editing is handled inside handle_strategy_input.
            handle_strategy_input(app, key);
        } else if app.screen == Screen::EnsembleCompose {
            // Weight input editing is handled inside handle_ensemble_compose_input.
            handle_ensemble_compose_input(app, key);
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
        Screen::EnsembleCompose => handle_ensemble_compose_input(app, key),
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
        KeyCode::Char('4') | KeyCode::Char('c') => {
            open_ensemble_compose(app, None);
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
        KeyCode::Char('c') => {
            open_ensemble_compose(app, Some(app.preset_idx));
        }
        _ => {}
    }
}

fn open_ensemble_compose(app: &mut App, initial_idx: Option<usize>) {
    let total = app.total_preset_count();

    app.ensemble_selected.clear();
    app.ensemble_weights.clear();
    app.editing = false;
    app.edit_buffer.clear();

    if let Some(ensemble) = &app.config.ensemble {
        app.ensemble_mode = ensemble.mode;
        // Track which member positions have been consumed so that two presets
        // with the same name do not both claim the same ensemble member.
        let mut remaining: Vec<Option<(String, f64)>> = ensemble
            .members
            .iter()
            .map(|m| Some((m.name.clone(), m.weight)))
            .collect();
        for idx in 0..total {
            if let Some((name, _, _)) = app.preset_entry(idx)
                && let Some(slot) = remaining
                    .iter_mut()
                    .find(|s| s.as_ref().map(|(n, _)| n == &name).unwrap_or(false))
            {
                let (_, weight) = slot.take().unwrap();
                app.ensemble_selected.push(idx);
                app.ensemble_weights.insert(idx, weight);
            }
        }
    } else {
        app.ensemble_mode = Default::default();
    }

    if total == 0 {
        app.ensemble_cursor_idx = 0;
    } else if let Some(idx) = initial_idx {
        let clamped = idx.min(total - 1);
        app.ensemble_cursor_idx = clamped;
        if !app.ensemble_selected.contains(&clamped) {
            app.ensemble_selected.push(clamped);
        }
        app.ensemble_weights.entry(clamped).or_insert(1.0);
    } else if let Some(first) = app.ensemble_selected.first().copied() {
        app.ensemble_cursor_idx = first;
    } else {
        app.ensemble_cursor_idx = app.ensemble_cursor_idx.min(total - 1);
    }

    app.ensemble_selected.sort_unstable();
    app.ensemble_selected.dedup();
    for idx in &app.ensemble_selected {
        app.ensemble_weights.entry(*idx).or_insert(1.0);
    }

    app.edit_error = None;
    app.push_screen(Screen::EnsembleCompose);
}

fn handle_ensemble_compose_input(app: &mut App, key: KeyCode) {
    if app.editing {
        match key {
            KeyCode::Enter => {
                let parsed = app.edit_buffer.trim().parse::<f64>();
                match parsed {
                    Ok(weight) => match app.set_ensemble_weight(app.ensemble_cursor_idx, weight) {
                        Ok(()) => {
                            app.editing = false;
                            app.edit_buffer.clear();
                            app.edit_error = None;
                        }
                        Err(e) => {
                            app.edit_error = Some(e.to_string());
                        }
                    },
                    Err(_) => {
                        app.edit_error = Some("Invalid number".into());
                    }
                }
            }
            KeyCode::Esc => {
                app.editing = false;
                app.edit_buffer.clear();
                app.edit_error = None;
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                app.edit_buffer.push(c);
            }
            KeyCode::Backspace => {
                app.edit_buffer.pop();
            }
            _ => {}
        }
        return;
    }

    let total = app.total_preset_count();

    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.pop_screen(),
        KeyCode::Up | KeyCode::Char('k') => {
            if total > 0 {
                app.ensemble_cursor_idx = (app.ensemble_cursor_idx + total - 1) % total;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if total > 0 {
                app.ensemble_cursor_idx = (app.ensemble_cursor_idx + 1) % total;
            }
        }
        KeyCode::Char(' ') => {
            if total > 0 {
                app.toggle_ensemble_selection(app.ensemble_cursor_idx);
                app.edit_error = None;
            }
        }
        KeyCode::Char('a') => {
            if total > 0 {
                if app.ensemble_selected.len() == total {
                    app.ensemble_selected.clear();
                    app.ensemble_weights.clear();
                } else {
                    app.ensemble_selected = (0..total).collect();
                    for idx in 0..total {
                        app.ensemble_weights.entry(idx).or_insert(1.0);
                    }
                }
                app.edit_error = None;
            }
        }
        KeyCode::Char('c') => {
            app.ensemble_selected.clear();
            app.ensemble_weights.clear();
            app.edit_error = None;
        }
        KeyCode::Char('m') => {
            app.ensemble_mode = app.ensemble_mode.cycle();
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if total > 0 && app.ensemble_selected.contains(&app.ensemble_cursor_idx) {
                app.adjust_ensemble_weight(app.ensemble_cursor_idx, -0.1);
                app.edit_error = None;
            } else {
                app.edit_error =
                    Some("Select the preset first (Space) before editing weight".into());
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if total > 0 && app.ensemble_selected.contains(&app.ensemble_cursor_idx) {
                app.adjust_ensemble_weight(app.ensemble_cursor_idx, 0.1);
                app.edit_error = None;
            } else {
                app.edit_error =
                    Some("Select the preset first (Space) before editing weight".into());
            }
        }
        KeyCode::Char('w') => {
            if total > 0 && app.ensemble_selected.contains(&app.ensemble_cursor_idx) {
                app.editing = true;
                app.edit_buffer =
                    format!("{:.2}", app.ensemble_weight_for(app.ensemble_cursor_idx));
                app.edit_error = None;
            } else {
                app.edit_error =
                    Some("Select the preset first (Space) before editing weight".into());
            }
        }
        KeyCode::Enter => match app.apply_selected_ensemble() {
            Ok(()) => {
                app.screen = Screen::ConfigEditor;
                app.prev_screens.clear();
            }
            Err(e) => {
                app.edit_error = Some(e.to_string());
            }
        },
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
        KeyCode::Char('c') => {
            open_ensemble_compose(app, None);
        }
        KeyCode::Char('r') => {
            if app.can_run() {
                app.push_screen(Screen::Confirmation);
            } else {
                app.edit_error = Some(
                    "Need a symbol and either entry/exit conditions or a 2+ member ensemble".into(),
                );
            }
        }
        _ => {}
    }
}

fn handle_strategy_input(app: &mut App, key: KeyCode) {
    // When editing a scale fraction value, intercept all keys here.
    if app.editing {
        match key {
            KeyCode::Enter => {
                if let Ok(v) = app.edit_buffer.trim().parse::<f64>() {
                    if v > 0.0 && v <= 100.0 {
                        let frac = v / 100.0;
                        match app.active_condition_panel {
                            ConditionPanel::ScaleIn => {
                                app.config.strategy.scale_in_fraction = frac;
                            }
                            ConditionPanel::ScaleOut => {
                                app.config.strategy.scale_out_fraction = frac;
                            }
                            _ => {}
                        }
                        app.editing = false;
                        app.edit_buffer.clear();
                        app.edit_error = None;
                    } else {
                        app.edit_error = Some("Enter a value between 1 and 100 (%)".into());
                    }
                } else {
                    app.edit_error = Some("Invalid number".into());
                }
            }
            KeyCode::Esc => {
                app.editing = false;
                app.edit_buffer.clear();
                app.edit_error = None;
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                app.edit_buffer.push(c);
            }
            KeyCode::Backspace => {
                app.edit_buffer.pop();
            }
            _ => {}
        }
        return;
    }

    match key {
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.prev_screens.is_empty() {
                app.screen = Screen::ConfigEditor;
            } else {
                app.pop_screen();
            }
        }
        // Cycle through all 5 panels
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            app.active_condition_panel = match app.active_condition_panel {
                ConditionPanel::Entry => ConditionPanel::Exit,
                ConditionPanel::Exit => ConditionPanel::Regime,
                ConditionPanel::Regime => ConditionPanel::ScaleIn,
                ConditionPanel::ScaleIn => ConditionPanel::ScaleOut,
                ConditionPanel::ScaleOut => ConditionPanel::Entry,
            };
        }
        // Navigate within the active condition list
        KeyCode::Up | KeyCode::Char('k') => {
            navigate_condition(app, CondNavDir::Prev);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            navigate_condition(app, CondNavDir::Next);
        }
        // Add new condition based on active panel
        KeyCode::Enter => {
            match app.active_condition_panel {
                ConditionPanel::Entry => app.condition_target = ConditionTarget::Entry,
                ConditionPanel::Exit => app.condition_target = ConditionTarget::Exit,
                ConditionPanel::Regime => app.condition_target = ConditionTarget::Regime,
                ConditionPanel::ScaleIn => app.condition_target = ConditionTarget::ScaleIn,
                ConditionPanel::ScaleOut => app.condition_target = ConditionTarget::ScaleOut,
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
        // Add to regime filter
        KeyCode::Char('5') | KeyCode::Char('g') => {
            app.condition_target = ConditionTarget::Regime;
            app.category_idx = 0;
            app.indicator_idx = 0;
            app.push_screen(Screen::IndicatorBrowser);
        }
        // Add to scale-in
        KeyCode::Char('6') => {
            app.condition_target = ConditionTarget::ScaleIn;
            app.category_idx = 0;
            app.indicator_idx = 0;
            app.push_screen(Screen::IndicatorBrowser);
        }
        // Add to scale-out
        KeyCode::Char('7') => {
            app.condition_target = ConditionTarget::ScaleOut;
            app.category_idx = 0;
            app.indicator_idx = 0;
            app.push_screen(Screen::IndicatorBrowser);
        }
        // Edit fraction for scale-in/out panels
        KeyCode::Char('f') => match app.active_condition_panel {
            ConditionPanel::ScaleIn => {
                app.edit_buffer = format!("{:.0}", app.config.strategy.scale_in_fraction * 100.0);
                app.editing = true;
                app.edit_error = None;
            }
            ConditionPanel::ScaleOut => {
                app.edit_buffer = format!("{:.0}", app.config.strategy.scale_out_fraction * 100.0);
                app.editing = true;
                app.edit_error = None;
            }
            _ => {}
        },
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
            ConditionPanel::Regime => {
                let idx = app.regime_condition_idx;
                app.config.strategy.regime_conditions.toggle_op_at(idx);
            }
            ConditionPanel::ScaleIn => {
                let idx = app.scale_in_condition_idx;
                app.config.strategy.scale_in_conditions.toggle_op_at(idx);
            }
            ConditionPanel::ScaleOut => {
                let idx = app.scale_out_condition_idx;
                app.config.strategy.scale_out_conditions.toggle_op_at(idx);
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
                        if app.exit_condition_idx
                            >= app.config.strategy.exit_conditions.conditions.len()
                            && app.exit_condition_idx > 0
                        {
                            app.exit_condition_idx -= 1;
                        }
                    }
                }
                ConditionPanel::Regime => {
                    let len = app.config.strategy.regime_conditions.conditions.len();
                    if len > 0 {
                        let idx = app.regime_condition_idx;
                        app.config.strategy.regime_conditions.remove_at(idx);
                        if app.regime_condition_idx
                            >= app.config.strategy.regime_conditions.conditions.len()
                            && app.regime_condition_idx > 0
                        {
                            app.regime_condition_idx -= 1;
                        }
                    }
                }
                ConditionPanel::ScaleIn => {
                    let len = app.config.strategy.scale_in_conditions.conditions.len();
                    if len > 0 {
                        let idx = app.scale_in_condition_idx;
                        app.config.strategy.scale_in_conditions.remove_at(idx);
                        if app.scale_in_condition_idx
                            >= app.config.strategy.scale_in_conditions.conditions.len()
                            && app.scale_in_condition_idx > 0
                        {
                            app.scale_in_condition_idx -= 1;
                        }
                    }
                }
                ConditionPanel::ScaleOut => {
                    let len = app.config.strategy.scale_out_conditions.conditions.len();
                    if len > 0 {
                        let idx = app.scale_out_condition_idx;
                        app.config.strategy.scale_out_conditions.remove_at(idx);
                        if app.scale_out_condition_idx
                            >= app.config.strategy.scale_out_conditions.conditions.len()
                            && app.scale_out_condition_idx > 0
                        {
                            app.scale_out_condition_idx -= 1;
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
        match key {
            KeyCode::Char('t') => app.cycle_building_htf_interval(),
            KeyCode::Enter | KeyCode::Char('n') => app.finish_indicator_config(),
            KeyCode::Esc | KeyCode::Char('q') => app.pop_screen(),
            _ => {}
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
        KeyCode::Char('t') => {
            app.cycle_building_htf_interval();
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
        // Toggle search method (Grid ↔ Bayesian)
        KeyCode::Char('b') => {
            app.optimizer_search_method = app.optimizer_search_method.toggle();
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
                search_method: app.optimizer_search_method,
                bayesian_trials: 100,
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

#[cfg(test)]
mod tests {
    use crossterm::event::KeyModifiers;

    use super::*;

    #[test]
    fn ensemble_weight_typing_sets_exact_value() {
        let mut app = App::new(None);
        open_ensemble_compose(&mut app, Some(0));

        handle_input(&mut app, KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(app.editing);

        app.edit_buffer = "2.75".to_string();
        handle_input(&mut app, KeyCode::Enter, KeyModifiers::NONE);

        assert!(!app.editing);
        assert!((app.ensemble_weight_for(0) - 2.75).abs() < 1e-9);
    }

    #[test]
    fn ensemble_weight_typing_requires_selected_member() {
        let mut app = App::new(None);
        open_ensemble_compose(&mut app, None);
        app.ensemble_selected.clear();
        app.ensemble_weights.clear();
        app.ensemble_cursor_idx = 0;

        handle_input(&mut app, KeyCode::Char('w'), KeyModifiers::NONE);

        assert!(!app.editing);
        assert!(
            app.edit_error
                .as_deref()
                .is_some_and(|msg| msg.contains("Select the preset first"))
        );
    }

    #[test]
    fn ensemble_weight_typing_rejects_out_of_range() {
        let mut app = App::new(None);
        open_ensemble_compose(&mut app, Some(0));

        handle_input(&mut app, KeyCode::Char('w'), KeyModifiers::NONE);
        app.edit_buffer = "11".to_string();
        handle_input(&mut app, KeyCode::Enter, KeyModifiers::NONE);

        assert!(app.editing);
        assert!(
            app.edit_error
                .as_deref()
                .is_some_and(|msg| msg.contains("between 0.0 and 10.0"))
        );
    }
}
