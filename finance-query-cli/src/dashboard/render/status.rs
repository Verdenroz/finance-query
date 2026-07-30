use crate::dashboard::state::{App, FocusPane, InputMode, Tab};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

pub(super) fn render_status(f: &mut Frame, app: &App, area: Rect) {
    use crate::alerts::AlertType;

    let available_width = area.width.saturating_sub(2) as usize; // Account for borders

    let status_text = match app.input_mode {
        InputMode::Normal => {
            let update_time = app
                .last_update
                .map(|t| t.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "N/A".to_string());

            let focus_hint = match app.active_tab {
                Tab::Alerts => {
                    // Alerts tab uses full width, no left/right panes
                    "j/k:nav a:add d:del e:toggle c:check"
                }
                Tab::Portfolio => {
                    // Portfolio tab: left pane is portfolio list, not watchlist
                    if app.focus_pane == FocusPane::Left {
                        "j/k:nav a:add d:delete"
                    } else {
                        "Esc:back"
                    }
                }
                _ => {
                    // Other tabs have watchlist on left, details on right
                    if app.focus_pane == FocusPane::Left {
                        match app.active_tab {
                            Tab::Watchlist => "j/k:nav a:add d:del 1/2/3:scroll 0:reset",
                            _ => "j/k:nav a:add d:delete l/→:focus",
                        }
                    } else {
                        match app.active_tab {
                            Tab::Charts => "←/→:range Esc/h:back",
                            Tab::News => "j/k:scroll Enter:open Esc/h:back",
                            Tab::Watchlist => "Esc/h:back",
                            Tab::Lookup => "j/k:nav Enter:add Esc/h:back",
                            Tab::Screeners => "←/→:cat j/k:nav Enter:add Esc/h:back",
                            _ => "Esc:back",
                        }
                    }
                }
            };

            // Truncate status message if too long
            let max_msg_len = available_width.saturating_sub(60);
            let msg = if app.status_message.len() > max_msg_len && max_msg_len > 3 {
                format!("{}...", &app.status_message[..max_msg_len - 3])
            } else {
                app.status_message.clone()
            };

            let symbol_count = match app.active_tab {
                Tab::Alerts => format!("{} alerts", app.alerts.len()),
                Tab::Portfolio => format!("{} positions", app.portfolio.positions.len()),
                _ => format!("{} symbols", app.current_watchlist.symbols.len()),
            };

            format!(
                " {} | {} | Last: {} | {}",
                msg, symbol_count, update_time, focus_hint
            )
        }
        InputMode::AddSymbol => {
            format!(
                " Add symbol: {} | Enter:confirm Esc:cancel",
                app.input_buffer
            )
        }
        InputMode::AddAlert => {
            let all_types = AlertType::all();
            let alert_type = all_types[app.alert_form_type_idx];

            let symbol_field = if app.alert_form_field == 0 {
                format!("[{}▌]", app.alert_form_symbol)
            } else if app.alert_form_symbol.is_empty() {
                "[SYM]".to_string()
            } else {
                app.alert_form_symbol.clone()
            };
            let type_field = if app.alert_form_field == 1 {
                format!("<{}>", alert_type.short_display())
            } else {
                alert_type.short_display().to_string()
            };
            let threshold_field = if app.alert_form_field == 2 {
                format!("[{}▌]", app.alert_form_threshold)
            } else if app.alert_form_threshold.is_empty() {
                "[VAL]".to_string()
            } else {
                app.alert_form_threshold.clone()
            };
            format!(
                " {} {} {} | Tab:next ←/→:type Enter:create Esc:cancel",
                symbol_field, type_field, threshold_field
            )
        }
        InputMode::AddPosition => " Adding position... | Esc:cancel".to_string(),
    };

    // Final truncation to fit width (character-aware for UTF-8)
    let final_text = if status_text.chars().count() > available_width {
        let truncated: String = status_text
            .chars()
            .take(available_width.saturating_sub(1))
            .collect();
        format!("{}…", truncated)
    } else {
        status_text
    };

    let paragraph = Paragraph::new(final_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, area);
}
