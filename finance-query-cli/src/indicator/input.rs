use super::state::{App, ChartField, IndicatorCategory, Screen};
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
        Screen::IndicatorSelect => handle_indicator_select_input(app, key),
        Screen::ParamConfig => handle_param_config_input(app, key),
        Screen::ChartConfig => handle_chart_config_input(app, key),
        Screen::Confirmation => handle_confirmation_input(app, key),
    }
}

fn handle_indicator_select_input(app: &mut App, key: KeyCode) {
    let num_categories = IndicatorCategory::all().len();
    let num_indicators = app.indicators_in_category().len();

    match key {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        // Category navigation (left/right or Tab)
        KeyCode::Left | KeyCode::Char('h') => {
            app.category_idx = (app.category_idx + num_categories - 1) % num_categories;
            app.indicator_idx = 0; // Reset indicator selection
        }
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => {
            app.category_idx = (app.category_idx + 1) % num_categories;
            app.indicator_idx = 0;
        }
        // Indicator navigation (up/down)
        KeyCode::Up | KeyCode::Char('k') => {
            if num_indicators > 0 {
                app.indicator_idx = (app.indicator_idx + num_indicators - 1) % num_indicators;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if num_indicators > 0 {
                app.indicator_idx = (app.indicator_idx + 1) % num_indicators;
            }
        }
        // Select indicator
        KeyCode::Enter => {
            app.select_indicator();
        }
        // Quick jump to categories by number
        KeyCode::Char('1') => {
            app.category_idx = 0;
            app.indicator_idx = 0;
        }
        KeyCode::Char('2') => {
            if num_categories > 1 {
                app.category_idx = 1;
                app.indicator_idx = 0;
            }
        }
        KeyCode::Char('3') => {
            if num_categories > 2 {
                app.category_idx = 2;
                app.indicator_idx = 0;
            }
        }
        KeyCode::Char('4') => {
            if num_categories > 3 {
                app.category_idx = 3;
                app.indicator_idx = 0;
            }
        }
        KeyCode::Char('5') => {
            if num_categories > 4 {
                app.category_idx = 4;
                app.indicator_idx = 0;
            }
        }
        _ => {}
    }
}

fn handle_param_config_input(app: &mut App, key: KeyCode) {
    let num_params = app
        .selected_indicator
        .as_ref()
        .map(|i| i.params.len())
        .unwrap_or(0);

    match key {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.pop_screen();
        }
        // Parameter navigation
        KeyCode::Up | KeyCode::Char('k') => {
            if num_params > 0 {
                app.param_idx = (app.param_idx + num_params - 1) % num_params;
            }
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
            if num_params > 0 {
                app.param_idx = (app.param_idx + 1) % num_params;
            }
        }
        // Adjust parameter value
        KeyCode::Left | KeyCode::Char('h') => {
            if let Some(ref ind) = app.selected_indicator
                && let Some(param) = ind.params.get(app.param_idx)
                && let Some(value) = app.param_values.get_mut(app.param_idx)
            {
                *value = (*value - param.step).max(param.min);
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if let Some(ref ind) = app.selected_indicator
                && let Some(param) = ind.params.get(app.param_idx)
                && let Some(value) = app.param_values.get_mut(app.param_idx)
            {
                *value = (*value + param.step).min(param.max);
            }
        }
        // Big adjustments with shift
        KeyCode::Char('H') => {
            if let Some(ref ind) = app.selected_indicator
                && let Some(param) = ind.params.get(app.param_idx)
                && let Some(value) = app.param_values.get_mut(app.param_idx)
            {
                *value = (*value - param.step * 10.0).max(param.min);
            }
        }
        KeyCode::Char('L') => {
            if let Some(ref ind) = app.selected_indicator
                && let Some(param) = ind.params.get(app.param_idx)
                && let Some(value) = app.param_values.get_mut(app.param_idx)
            {
                *value = (*value + param.step * 10.0).min(param.max);
            }
        }
        // Reset to default
        KeyCode::Char('d') => {
            if let Some(ref ind) = app.selected_indicator
                && let Some(param) = ind.params.get(app.param_idx)
                && let Some(value) = app.param_values.get_mut(app.param_idx)
            {
                *value = param.default;
            }
        }
        // Continue to chart config
        KeyCode::Enter => {
            app.push_screen(Screen::ChartConfig);
        }
        _ => {}
    }
}

fn handle_chart_config_input(app: &mut App, key: KeyCode) {
    let num_fields = ChartField::all().len();

    match key {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.pop_screen();
        }
        // Field navigation
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
            app.chart_field_idx = (app.chart_field_idx + num_fields - 1) % num_fields;
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
            app.chart_field_idx = (app.chart_field_idx + 1) % num_fields;
        }
        // Edit field
        KeyCode::Enter | KeyCode::Char('e') => {
            app.start_editing();
        }
        // Run if ready
        KeyCode::Char('r') => {
            if app.can_run() {
                app.push_screen(Screen::Confirmation);
            } else {
                app.edit_error = Some("Enter a symbol to continue".to_string());
            }
        }
        // Quick interval selection
        KeyCode::Char('1') => app.interval = finance_query::Interval::OneMinute,
        KeyCode::Char('5') => app.interval = finance_query::Interval::FiveMinutes,
        _ => {}
    }
}

fn handle_confirmation_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('n') => {
            app.pop_screen();
        }
        KeyCode::Enter | KeyCode::Char('y') => {
            app.confirmed = true;
        }
        _ => {}
    }
}
