use super::state::{App, FocusPane, InputMode, SectorsViewMode, Tab};
use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyModifiers};

pub async fn handle_key_event(app: &mut App, key: event::KeyEvent) -> Result<()> {
    match app.input_mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                app.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.should_quit = true;
            }

            KeyCode::Esc => {
                if app.active_tab == Tab::Sectors
                    && app.sectors_view_mode == SectorsViewMode::Industries
                {
                    app.sectors_view_mode = SectorsViewMode::Overview;
                    app.sectors_selected_industry = 0;
                } else if app.focus_pane == FocusPane::Right {
                    app.focus_pane = FocusPane::Left;
                    // When unfocusing from News right pane, go back to general news
                    if app.active_tab == Tab::News && app.news_symbol.is_some() {
                        let _ = app.fetch_general_news().await;
                    }
                }
            }

            KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                app.active_tab = app.active_tab.prev();
                app.focus_pane = FocusPane::Left;
                if app.active_tab == Tab::Screeners && app.screener_data.is_empty() {
                    let _ = app.fetch_screeners().await;
                } else if app.active_tab == Tab::Sectors && app.sectors_data.is_empty() {
                    // Just set loading flag - runtime will handle the actual fetch
                    app.is_loading_sectors = true;
                } else if app.active_tab == Tab::News && app.news_items.is_empty() {
                    // Fetch general news when entering News tab
                    let _ = app.fetch_general_news().await;
                } else {
                    let _ = app.refresh_details().await;
                }
            }
            KeyCode::Tab => {
                app.active_tab = app.active_tab.next();
                app.focus_pane = FocusPane::Left;
                if app.active_tab == Tab::Screeners && app.screener_data.is_empty() {
                    let _ = app.fetch_screeners().await;
                } else if app.active_tab == Tab::Sectors && app.sectors_data.is_empty() {
                    // Just set loading flag - runtime will handle the actual fetch
                    app.is_loading_sectors = true;
                } else if app.active_tab == Tab::News && app.news_items.is_empty() {
                    // Fetch general news when entering News tab
                    let _ = app.fetch_general_news().await;
                } else {
                    let _ = app.refresh_details().await;
                }
            }
            KeyCode::BackTab => {
                app.active_tab = app.active_tab.prev();
                app.focus_pane = FocusPane::Left;
                if app.active_tab == Tab::Screeners && app.screener_data.is_empty() {
                    let _ = app.fetch_screeners().await;
                } else if app.active_tab == Tab::Sectors && app.sectors_data.is_empty() {
                    // Just set loading flag - runtime will handle the actual fetch
                    app.is_loading_sectors = true;
                } else if app.active_tab == Tab::News && app.news_items.is_empty() {
                    // Fetch general news when entering News tab
                    let _ = app.fetch_general_news().await;
                } else {
                    let _ = app.refresh_details().await;
                }
            }

            KeyCode::Char('h') if app.focus_pane == FocusPane::Right => {
                app.focus_pane = FocusPane::Left;
            }
            KeyCode::Char('l') | KeyCode::Right if app.focus_pane == FocusPane::Left => {
                if app.active_tab == Tab::Charts
                    || app.active_tab == Tab::News
                    || app.active_tab == Tab::Lookup
                    || app.active_tab == Tab::Screeners
                {
                    app.focus_pane = FocusPane::Right;
                    if app.active_tab == Tab::Screeners && app.screener_data.is_empty() {
                        let _ = app.fetch_screeners().await;
                    }
                }
            }

            KeyCode::Char('j') | KeyCode::Down => {
                if app.active_tab == Tab::Sectors {
                    // Navigate sectors
                    match app.sectors_view_mode {
                        SectorsViewMode::Overview => {
                            let len = app.sectors_data.len();
                            if len > 0 {
                                app.sectors_selected_idx = (app.sectors_selected_idx + 1) % len;
                            }
                        }
                        SectorsViewMode::Industries => {
                            if let Some((_, sector)) = app.selected_sector() {
                                let len = sector.industries.len();
                                if len > 0 {
                                    app.sectors_selected_industry =
                                        (app.sectors_selected_industry + 1) % len;
                                }
                            }
                        }
                    }
                } else if app.active_tab == Tab::Alerts {
                    // Navigate alerts list
                    let len = app.alerts.len();
                    if len > 0 {
                        app.selected_alert_idx = (app.selected_alert_idx + 1) % len;
                    }
                } else if app.focus_pane == FocusPane::Left {
                    if app.active_tab == Tab::Portfolio {
                        let len = app.portfolio.positions.len();
                        if len > 0 {
                            app.selected_portfolio_idx = (app.selected_portfolio_idx + 1) % len;
                        }
                    } else {
                        app.move_selection(1);
                        let _ = app.refresh_details().await;
                    }
                } else if app.focus_pane == FocusPane::Right && app.active_tab == Tab::News {
                    let len = app.news_items.len();
                    if len > 0 {
                        app.selected_news_idx = (app.selected_news_idx + 1) % len;
                    }
                } else if app.focus_pane == FocusPane::Right && app.active_tab == Tab::Lookup {
                    let len = app.search_results.len();
                    if len > 0 {
                        app.selected_search_idx = (app.selected_search_idx + 1) % len;
                    }
                } else if app.focus_pane == FocusPane::Right && app.active_tab == Tab::Screeners {
                    let len = app.screener_data.len();
                    if len > 0 {
                        app.selected_screener_idx = (app.selected_screener_idx + 1) % len;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if app.active_tab == Tab::Sectors {
                    // Navigate sectors
                    match app.sectors_view_mode {
                        SectorsViewMode::Overview => {
                            let len = app.sectors_data.len();
                            if len > 0 {
                                app.sectors_selected_idx =
                                    (app.sectors_selected_idx + len - 1) % len;
                            }
                        }
                        SectorsViewMode::Industries => {
                            if let Some((_, sector)) = app.selected_sector() {
                                let len = sector.industries.len();
                                if len > 0 {
                                    app.sectors_selected_industry =
                                        (app.sectors_selected_industry + len - 1) % len;
                                }
                            }
                        }
                    }
                } else if app.active_tab == Tab::Alerts {
                    // Navigate alerts list
                    let len = app.alerts.len();
                    if len > 0 {
                        app.selected_alert_idx = (app.selected_alert_idx + len - 1) % len;
                    }
                } else if app.focus_pane == FocusPane::Left {
                    if app.active_tab == Tab::Portfolio {
                        let len = app.portfolio.positions.len();
                        if len > 0 {
                            app.selected_portfolio_idx =
                                (app.selected_portfolio_idx + len - 1) % len;
                        }
                    } else {
                        app.move_selection(-1);
                        let _ = app.refresh_details().await;
                    }
                } else if app.focus_pane == FocusPane::Right && app.active_tab == Tab::News {
                    let len = app.news_items.len();
                    if len > 0 {
                        app.selected_news_idx = (app.selected_news_idx + len - 1) % len;
                    }
                } else if app.focus_pane == FocusPane::Right && app.active_tab == Tab::Lookup {
                    let len = app.search_results.len();
                    if len > 0 {
                        app.selected_search_idx = (app.selected_search_idx + len - 1) % len;
                    }
                } else if app.focus_pane == FocusPane::Right && app.active_tab == Tab::Screeners {
                    let len = app.screener_data.len();
                    if len > 0 {
                        app.selected_screener_idx = (app.selected_screener_idx + len - 1) % len;
                    }
                }
            }

            KeyCode::Char('g')
                if key.modifiers.contains(KeyModifiers::NONE)
                    && app.focus_pane == FocusPane::Left =>
            {
                app.selected_index = 0;
                let _ = app.refresh_details().await;
            }
            KeyCode::Char('G')
                if key.modifiers.contains(KeyModifiers::SHIFT)
                    && app.focus_pane == FocusPane::Left =>
            {
                app.selected_index = app.current_watchlist.symbols.len().saturating_sub(1);
                let _ = app.refresh_details().await;
            }

            KeyCode::Left
                if app.focus_pane == FocusPane::Right && app.active_tab == Tab::Charts =>
            {
                const NUM_RANGES: usize = 8;
                app.selected_chart_range_idx =
                    (app.selected_chart_range_idx + NUM_RANGES - 1) % NUM_RANGES;
                let _ = app.refresh_details_with_range().await;
            }
            KeyCode::Right
                if app.focus_pane == FocusPane::Right && app.active_tab == Tab::Charts =>
            {
                const NUM_RANGES: usize = 8;
                app.selected_chart_range_idx = (app.selected_chart_range_idx + 1) % NUM_RANGES;
                let _ = app.refresh_details_with_range().await;
            }

            KeyCode::Left
                if app.focus_pane == FocusPane::Right && app.active_tab == Tab::Screeners =>
            {
                app.screener_category = app.screener_category.prev();
                let _ = app.fetch_screeners().await;
            }
            KeyCode::Right
                if app.focus_pane == FocusPane::Right && app.active_tab == Tab::Screeners =>
            {
                app.screener_category = app.screener_category.next();
                let _ = app.fetch_screeners().await;
            }

            KeyCode::Enter if app.focus_pane == FocusPane::Right && app.active_tab == Tab::News => {
                if let Some(article) = app.news_items.get(app.selected_news_idx) {
                    let _ = open_url(&article.link);
                }
            }
            // Fetch symbol-specific news with Enter on left pane
            KeyCode::Enter if app.focus_pane == FocusPane::Left && app.active_tab == Tab::News => {
                if let Some(symbol) = app.selected_symbol().cloned() {
                    let _ = app.fetch_symbol_news(&symbol).await;
                    app.focus_pane = FocusPane::Right;
                }
            }
            // Return to general market news with 'g'
            KeyCode::Char('g') if app.active_tab == Tab::News && app.news_symbol.is_some() => {
                let _ = app.fetch_general_news().await;
            }
            KeyCode::Enter
                if app.focus_pane == FocusPane::Right && app.active_tab == Tab::Lookup =>
            {
                let _ = app.add_selected_search_result().await;
            }
            KeyCode::Enter
                if app.focus_pane == FocusPane::Right && app.active_tab == Tab::Screeners =>
            {
                let _ = app.add_screener_to_watchlist().await;
            }
            KeyCode::Enter if app.active_tab == Tab::Sectors => {
                // Toggle between sectors and industries view
                app.toggle_sectors_view();
            }

            KeyCode::Char(c)
                if app.active_tab == Tab::Lookup
                    && !key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                if matches!(
                    c,
                    'q' | 'Q' | 'a' | 'd' | 'r' | 'h' | 'l' | 'g' | 'G' | 'j' | 'k'
                ) {
                } else {
                    app.search_query.push(c);
                    let _ = app.perform_search().await;
                    app.focus_pane = FocusPane::Right;
                }
            }
            KeyCode::Backspace if app.active_tab == Tab::Lookup => {
                app.search_query.pop();
                let _ = app.perform_search().await;
            }

            KeyCode::Char('a') => {
                if app.active_tab == Tab::Portfolio {
                    app.input_mode = InputMode::AddPosition;
                    app.add_form_symbol.clear();
                    app.add_form_shares.clear();
                    app.add_form_cost.clear();
                    app.add_form_field = 0;
                    app.status_message =
                        "Add Position: Tab to switch fields, Enter to confirm".to_string();
                } else if app.active_tab == Tab::Alerts {
                    // Add alert from Alerts tab
                    app.alert_form_symbol.clear();
                    app.alert_form_type_idx = 0;
                    app.alert_form_threshold.clear();
                    app.alert_form_field = 0;
                    app.input_mode = InputMode::AddAlert;
                    app.status_message =
                        "Create Alert: Tab to switch fields, ←/→ to change type".to_string();
                } else {
                    app.input_mode = InputMode::AddSymbol;
                    app.input_buffer.clear();
                    app.status_message = "Enter symbol to add:".to_string();
                }
            }
            // 'd' to delete alert when on Alerts tab
            KeyCode::Char('d') if app.active_tab == Tab::Alerts && !app.alerts.is_empty() => {
                if app.selected_alert_idx < app.alerts.len() {
                    let alert = &app.alerts[app.selected_alert_idx];
                    let alert_id = alert.id;
                    let symbol = alert.symbol.clone();
                    if app.alert_store.delete_alert(alert_id).is_ok() {
                        if let Ok(updated_alerts) = app.alert_store.get_alerts() {
                            app.alerts = updated_alerts;
                        }
                        if app.selected_alert_idx >= app.alerts.len() && app.selected_alert_idx > 0
                        {
                            app.selected_alert_idx -= 1;
                        }
                        app.status_message = format!("Deleted alert for {}", symbol);
                    }
                }
            }
            // 'e' to toggle enable/disable alert
            KeyCode::Char('e') if app.active_tab == Tab::Alerts && !app.alerts.is_empty() => {
                if app.selected_alert_idx < app.alerts.len() {
                    let alert = &app.alerts[app.selected_alert_idx];
                    let alert_id = alert.id;
                    let new_state = !alert.enabled;
                    if app.alert_store.set_enabled(alert_id, new_state).is_ok() {
                        if let Ok(updated_alerts) = app.alert_store.get_alerts() {
                            app.alerts = updated_alerts;
                        }
                        let status = if new_state { "enabled" } else { "disabled" };
                        app.status_message = format!("Alert {}", status);
                    }
                }
            }
            // 'd' or Delete to remove symbol/position (not on Alerts tab)
            KeyCode::Char('d') | KeyCode::Delete
                if app.active_tab != Tab::Alerts && app.focus_pane == FocusPane::Left =>
            {
                if app.active_tab == Tab::Portfolio {
                    app.delete_portfolio_position();
                } else if app.active_tab == Tab::Watchlist {
                    let _ = app.remove_selected_symbol().await;
                }
            }
            // 'A' (shift+a) to create alert for current symbol (from other tabs, not Alerts tab)
            KeyCode::Char('A')
                if key.modifiers.contains(KeyModifiers::SHIFT) && app.active_tab != Tab::Alerts =>
            {
                // Pre-fill symbol from current context
                let symbol = match app.active_tab {
                    Tab::Watchlist => app.selected_symbol().cloned(),
                    Tab::Lookup if !app.search_results.is_empty() => {
                        Some(app.search_results[app.selected_search_idx].symbol.clone())
                    }
                    Tab::Screeners if !app.screener_data.is_empty() => {
                        Some(app.screener_data[app.selected_screener_idx].symbol.clone())
                    }
                    Tab::Portfolio if !app.portfolio.positions.is_empty() => Some(
                        app.portfolio.positions[app.selected_portfolio_idx]
                            .symbol
                            .clone(),
                    ),
                    _ => None,
                };

                app.alert_form_symbol = symbol.unwrap_or_default();
                app.alert_form_type_idx = 0;
                app.alert_form_threshold.clear();
                app.alert_form_field = if app.alert_form_symbol.is_empty() {
                    0
                } else {
                    1
                };
                app.input_mode = InputMode::AddAlert;
                app.status_message = "Create Alert: Tab to switch, ←/→ to change type".to_string();
            }
            KeyCode::Char('r') => {
                if app.active_tab == Tab::Sectors {
                    let _ = app.fetch_sectors_data().await;
                } else {
                    let _ = app.refresh_details().await;
                    app.status_message = "Refreshed".to_string();
                }
            }

            _ => {}
        },

        InputMode::AddSymbol => match key.code {
            KeyCode::Enter => {
                let _ = app.add_symbol().await;
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                app.input_buffer.clear();
                app.status_message = "Cancelled".to_string();
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }
            _ => {}
        },

        InputMode::AddAlert => match key.code {
            KeyCode::Tab => {
                // Cycle through fields: symbol -> type -> threshold
                app.alert_form_field = (app.alert_form_field + 1) % 3;
            }
            KeyCode::BackTab => {
                // Cycle backwards
                app.alert_form_field = (app.alert_form_field + 2) % 3;
            }
            KeyCode::Enter => {
                // Validate and create alert
                use crate::alerts::AlertType;

                let symbol = app.alert_form_symbol.trim().to_uppercase();
                let threshold: Result<f64, _> = app.alert_form_threshold.trim().parse();
                let all_types = AlertType::all();
                let alert_type = all_types[app.alert_form_type_idx];

                if symbol.is_empty() {
                    app.status_message = "Symbol required".to_string();
                } else if let Ok(val) = threshold {
                    if val <= 0.0 {
                        app.status_message = "Value must be positive".to_string();
                    } else {
                        match app.alert_store.create_alert(&symbol, alert_type, val, None) {
                            Ok(_) => {
                                // Reload alerts
                                if let Ok(updated_alerts) = app.alert_store.get_alerts() {
                                    app.alerts = updated_alerts;
                                }
                                app.status_message =
                                    format!("Alert: {} {}", symbol, alert_type.short_display());
                                app.input_mode = InputMode::Normal;
                                // Reset form
                                app.alert_form_symbol.clear();
                                app.alert_form_threshold.clear();
                                app.alert_form_type_idx = 0;
                                app.alert_form_field = 0;
                            }
                            Err(e) => {
                                app.status_message = format!("Error: {}", e);
                            }
                        }
                    }
                } else {
                    app.status_message = "Invalid value".to_string();
                }
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                app.status_message = "Cancelled".to_string();
                // Reset form
                app.alert_form_symbol.clear();
                app.alert_form_threshold.clear();
                app.alert_form_type_idx = 0;
                app.alert_form_field = 0;
            }
            KeyCode::Left if app.alert_form_field == 1 => {
                // Cycle alert type backwards
                use crate::alerts::AlertType;
                let len = AlertType::all().len();
                app.alert_form_type_idx = (app.alert_form_type_idx + len - 1) % len;
            }
            KeyCode::Right if app.alert_form_field == 1 => {
                // Cycle alert type forwards
                use crate::alerts::AlertType;
                let len = AlertType::all().len();
                app.alert_form_type_idx = (app.alert_form_type_idx + 1) % len;
            }
            KeyCode::Backspace => match app.alert_form_field {
                0 => {
                    app.alert_form_symbol.pop();
                }
                2 => {
                    app.alert_form_threshold.pop();
                }
                _ => {}
            },
            KeyCode::Char(c) => {
                match app.alert_form_field {
                    0 => app.alert_form_symbol.push(c),
                    1 => {
                        // Use left/right arrows to cycle types
                    }
                    2 => {
                        // Only allow numbers, decimal point, and minus for threshold
                        if c.is_ascii_digit() || c == '.' || c == '-' {
                            app.alert_form_threshold.push(c);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        },

        InputMode::AddPosition => match key.code {
            KeyCode::Tab | KeyCode::Down => {
                app.add_form_field = (app.add_form_field + 1) % 3;
            }
            KeyCode::BackTab | KeyCode::Up => {
                app.add_form_field = if app.add_form_field == 0 {
                    2
                } else {
                    app.add_form_field - 1
                };
            }
            KeyCode::Enter => {
                if app.add_form_field == 2 {
                    app.add_portfolio_position();
                } else {
                    app.add_form_field = (app.add_form_field + 1) % 3;
                }
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                app.add_form_symbol.clear();
                app.add_form_shares.clear();
                app.add_form_cost.clear();
                app.add_form_field = 0;
                app.status_message = "Cancelled".to_string();
            }
            KeyCode::Char(c) => match app.add_form_field {
                0 => app.add_form_symbol.push(c.to_ascii_uppercase()),
                1 | 2 => {
                    if c.is_ascii_digit() || c == '.' {
                        match app.add_form_field {
                            1 => app.add_form_shares.push(c),
                            2 => app.add_form_cost.push(c),
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            KeyCode::Backspace => match app.add_form_field {
                0 => {
                    app.add_form_symbol.pop();
                }
                1 => {
                    app.add_form_shares.pop();
                }
                2 => {
                    app.add_form_cost.pop();
                }
                _ => {}
            },
            _ => {}
        },
    }

    Ok(())
}

fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(url).spawn()?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(url).spawn()?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(&["/C", "start", url])
        .spawn()?;

    Ok(())
}
