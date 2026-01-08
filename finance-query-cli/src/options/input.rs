use crate::error::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::state::OptionsApp;

pub async fn handle_key_event(app: &mut OptionsApp, key: KeyEvent) -> Result<()> {
    // Help overlay takes priority
    if app.show_help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Enter => {
                app.show_help = false;
            }
            _ => {}
        }
        return Ok(());
    }

    match key.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }

        // Help
        KeyCode::Char('?') | KeyCode::Char('h') => {
            app.show_help = true;
        }

        // Navigation - up/down
        KeyCode::Up | KeyCode::Char('k') => {
            app.move_selection_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.move_selection_down();
        }

        // Navigation - page up/down
        KeyCode::PageUp => {
            for _ in 0..10 {
                app.move_selection_up();
            }
        }
        KeyCode::PageDown => {
            for _ in 0..10 {
                app.move_selection_down();
            }
        }

        // Home/End
        KeyCode::Home => {
            app.calls_table_state.select(Some(0));
            app.puts_table_state.select(Some(0));
        }
        KeyCode::End => {
            let calls_len = app.filtered_calls().len();
            let puts_len = app.filtered_puts().len();
            if calls_len > 0 {
                app.calls_table_state.select(Some(calls_len - 1));
            }
            if puts_len > 0 {
                app.puts_table_state.select(Some(puts_len - 1));
            }
        }

        // Switch focus between calls/puts in straddle view
        KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
            app.toggle_focus();
        }

        // Cycle view mode
        KeyCode::Char('v') => {
            app.view_mode = app.view_mode.cycle();
        }

        // Change expiration - next
        KeyCode::Char(']') | KeyCode::Char('n') => {
            app.next_expiration();
            app.load_chain_for_expiration().await?;
        }

        // Change expiration - previous
        KeyCode::Char('[') | KeyCode::Char('p') => {
            app.prev_expiration();
            app.load_chain_for_expiration().await?;
        }

        // Sort options
        KeyCode::Char('s') => {
            app.sort_field = app.sort_field.cycle();
            app.sort_contracts();
        }

        // Toggle sort order
        KeyCode::Char('o') => {
            app.sort_ascending = !app.sort_ascending;
            app.sort_contracts();
        }

        // Toggle ITM filter
        KeyCode::Char('i') => {
            app.filter_itm_only = !app.filter_itm_only;
        }

        // Toggle volume filter
        KeyCode::Char('V') => {
            app.filter_min_volume = match app.filter_min_volume {
                None => Some(100),
                Some(100) => Some(1000),
                Some(1000) => Some(10000),
                _ => None,
            };
        }

        // Refresh data
        KeyCode::Char('r') => {
            app.load_chain_for_expiration().await?;
        }

        _ => {}
    }

    Ok(())
}
