use super::indicators::IndicatorCategory;
use super::state::{App, ConditionPanel, ConditionTarget, ConfigField, Screen};
use super::types::ComparisonType;
use crossterm::event::{KeyCode, KeyModifiers};

/// Main input handler that dispatches to screen-specific handlers
pub fn handle_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    // Global quit
    if matches!(key, KeyCode::Char('c')) && modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }

    // Editing mode
    if app.editing {
        match key {
            KeyCode::Enter => app.finish_editing(),
            KeyCode::Esc => app.cancel_editing(),
            KeyCode::Char(c) => app.edit_buffer.push(c),
            KeyCode::Backspace => {
                app.edit_buffer.pop();
            }
            _ => {}
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
    let len = app.presets.len();
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
                app.target_value -= 0.1;
            } else {
                app.target_value2 -= 0.1;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.editing_target_value {
                app.target_value += 0.1;
            } else {
                app.target_value2 += 0.1;
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
        KeyCode::Char('i') => {
            // Toggle between value and indicator target
            app.target_is_indicator = !app.target_is_indicator;
        }
        _ => {}
    }
}

fn handle_confirmation_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('y') | KeyCode::Enter => {
            app.confirmed = true;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.pop_screen();
        }
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}
