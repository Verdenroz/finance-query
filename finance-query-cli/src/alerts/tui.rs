use crate::alerts::{Alert, AlertStore, AlertType};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use finance_query::Tickers;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use std::io;
use tokio::time::Duration;

#[derive(PartialEq)]
enum Screen {
    List,
    Add,
    Help,
}

#[derive(PartialEq)]
enum AddField {
    Symbol,
    AlertType,
    Threshold,
}

struct App {
    store: AlertStore,
    alerts: Vec<Alert>,
    selected_idx: usize,
    screen: Screen,
    // Add form fields
    add_symbol: String,
    add_type_idx: usize,
    add_threshold: String,
    add_field: AddField,
    status_message: String,
    should_quit: bool,
}

impl App {
    fn new() -> Result<Self> {
        let store = AlertStore::new()?;
        let alerts = store.get_alerts()?;

        Ok(Self {
            store,
            alerts,
            selected_idx: 0,
            screen: Screen::List,
            add_symbol: String::new(),
            add_type_idx: 0,
            add_threshold: String::new(),
            add_field: AddField::Symbol,
            status_message: "Press 'a' to add alert, 'q' to quit".to_string(),
            should_quit: false,
        })
    }

    fn move_selection(&mut self, delta: isize) {
        if self.alerts.is_empty() {
            self.selected_idx = 0;
            return;
        }
        let len = self.alerts.len();
        let new_idx = (self.selected_idx as isize + delta).rem_euclid(len as isize);
        self.selected_idx = new_idx as usize;
    }

    fn delete_selected(&mut self) -> Result<()> {
        if self.alerts.is_empty() {
            return Ok(());
        }

        if self.selected_idx < self.alerts.len() {
            let alert = &self.alerts[self.selected_idx];
            let symbol = alert.symbol.clone();
            let id = alert.id;

            self.store.delete_alert(id)?;
            self.alerts = self.store.get_alerts()?;

            if self.selected_idx >= self.alerts.len() && self.selected_idx > 0 {
                self.selected_idx -= 1;
            }

            self.status_message = format!("Deleted alert for {}", symbol);
        }

        Ok(())
    }

    fn toggle_selected(&mut self) -> Result<()> {
        if self.alerts.is_empty() {
            return Ok(());
        }

        if self.selected_idx < self.alerts.len() {
            let alert = &self.alerts[self.selected_idx];
            let id = alert.id;
            let new_state = !alert.enabled;

            self.store.set_enabled(id, new_state)?;
            self.alerts = self.store.get_alerts()?;

            let status = if new_state { "enabled" } else { "disabled" };
            self.status_message = format!("Alert {}", status);
        }

        Ok(())
    }

    fn create_alert(&mut self) -> Result<()> {
        let symbol = self.add_symbol.trim().to_uppercase();
        let threshold: Result<f64, _> = self.add_threshold.trim().parse();

        if symbol.is_empty() {
            self.status_message = "Error: Symbol required".to_string();
            return Ok(());
        }

        match threshold {
            Ok(val) if val > 0.0 => {
                let alert_types = get_alert_types();
                let alert_type = alert_types[self.add_type_idx];

                self.store.create_alert(&symbol, alert_type, val, None)?;
                self.alerts = self.store.get_alerts()?;

                self.status_message =
                    format!("Created {} alert for {}", alert_type.display(), symbol);
                self.screen = Screen::List;
                self.add_symbol.clear();
                self.add_threshold.clear();
                self.add_type_idx = 0;
                self.add_field = AddField::Symbol;
            }
            _ => {
                self.status_message = "Error: Threshold must be a positive number".to_string();
            }
        }

        Ok(())
    }

    async fn check_alerts(&mut self) -> Result<()> {
        use super::send_alert_notification;

        let enabled = self
            .alerts
            .iter()
            .filter(|a| a.enabled)
            .cloned()
            .collect::<Vec<_>>();

        if enabled.is_empty() {
            self.status_message = "No enabled alerts to check".to_string();
            return Ok(());
        }

        let symbols: Vec<String> = enabled.iter().map(|a| a.symbol.clone()).collect();
        let unique_symbols: Vec<String> = symbols
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        self.status_message = "Checking alerts...".to_string();

        match Tickers::new(&unique_symbols).await {
            Ok(tickers) => match tickers.quotes().await {
                Ok(response) => {
                    let mut triggered_count = 0;
                    for alert in &enabled {
                        if let Some(quote) = response.quotes.get(&alert.symbol)
                            && alert.check(quote)
                        {
                            let _ = self.store.mark_triggered(alert.id);
                            triggered_count += 1;

                            // Send desktop notification
                            let current_value = alert.get_current_value(quote);
                            send_alert_notification(alert, current_value);
                        }
                    }

                    self.alerts = self.store.get_alerts()?;

                    if triggered_count > 0 {
                        self.status_message = format!("🔔 {} alert(s) triggered!", triggered_count);
                    } else {
                        self.status_message = "✓ No alerts triggered".to_string();
                    }
                }
                Err(e) => {
                    self.status_message = format!("Error fetching quotes: {}", e);
                }
            },
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }

        Ok(())
    }
}

fn get_alert_types() -> [AlertType; 12] {
    [
        AlertType::PriceAbove,
        AlertType::PriceBelow,
        AlertType::ChangeAbove,
        AlertType::ChangeBelow,
        AlertType::VolumeSpike,
        AlertType::Week52High,
        AlertType::Week52Low,
        AlertType::MarketCapAbove,
        AlertType::MarketCapBelow,
        AlertType::DividendYieldAbove,
        AlertType::PeRatioAbove,
        AlertType::PeRatioBelow,
    ]
}

pub async fn run_alerts_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let result = run_event_loop(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if app.should_quit {
            break;
        }

        // Event handling
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                while event::poll(Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        handle_key_event(app, key).await?;
                        break;
                    }
                }
            },
        }
    }

    Ok(())
}

async fn handle_key_event(app: &mut App, key: event::KeyEvent) -> Result<()> {
    match app.screen {
        Screen::List => handle_list_keys(app, key).await?,
        Screen::Add => handle_add_keys(app, key)?,
        Screen::Help => {
            if matches!(
                key.code,
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('h')
            ) {
                app.screen = Screen::List;
            }
        }
    }

    Ok(())
}

async fn handle_list_keys(app: &mut App, key: event::KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.should_quit = true;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_selection(1);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_selection(-1);
        }
        KeyCode::Char('g') => {
            app.selected_idx = 0;
        }
        KeyCode::Char('G') => {
            if !app.alerts.is_empty() {
                app.selected_idx = app.alerts.len() - 1;
            }
        }
        KeyCode::Char('a') => {
            app.screen = Screen::Add;
            app.status_message =
                "Fill in alert details. Tab to switch fields, Enter to create".to_string();
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            app.delete_selected()?;
        }
        KeyCode::Char('t') | KeyCode::Char(' ') => {
            app.toggle_selected()?;
        }
        KeyCode::Char('c') => {
            app.check_alerts().await?;
        }
        KeyCode::Char('r') => {
            app.alerts = app.store.get_alerts()?;
            app.status_message = "Refreshed alert list".to_string();
        }
        KeyCode::Char('h') | KeyCode::Char('?') => {
            app.screen = Screen::Help;
        }
        _ => {}
    }

    Ok(())
}

fn handle_add_keys(app: &mut App, key: event::KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.screen = Screen::List;
            app.add_symbol.clear();
            app.add_threshold.clear();
            app.add_type_idx = 0;
            app.add_field = AddField::Symbol;
            app.status_message = "Cancelled".to_string();
        }
        KeyCode::Tab | KeyCode::Down => {
            app.add_field = match app.add_field {
                AddField::Symbol => AddField::AlertType,
                AddField::AlertType => AddField::Threshold,
                AddField::Threshold => AddField::Symbol,
            };
        }
        KeyCode::BackTab | KeyCode::Up => {
            app.add_field = match app.add_field {
                AddField::Symbol => AddField::Threshold,
                AddField::AlertType => AddField::Symbol,
                AddField::Threshold => AddField::AlertType,
            };
        }
        KeyCode::Enter => {
            if app.add_field == AddField::Threshold && !app.add_threshold.is_empty() {
                app.create_alert()?;
            } else {
                // Move to next field
                app.add_field = match app.add_field {
                    AddField::Symbol => AddField::AlertType,
                    AddField::AlertType => AddField::Threshold,
                    AddField::Threshold => AddField::Symbol,
                };
            }
        }
        KeyCode::Left if app.add_field == AddField::AlertType => {
            let types = get_alert_types();
            app.add_type_idx = if app.add_type_idx == 0 {
                types.len() - 1
            } else {
                app.add_type_idx - 1
            };
        }
        KeyCode::Right if app.add_field == AddField::AlertType => {
            let types = get_alert_types();
            app.add_type_idx = (app.add_type_idx + 1) % types.len();
        }
        KeyCode::Char(c) => match app.add_field {
            AddField::Symbol => {
                app.add_symbol.push(c.to_ascii_uppercase());
            }
            AddField::Threshold => {
                if c.is_ascii_digit() || c == '.' {
                    app.add_threshold.push(c);
                }
            }
            _ => {}
        },
        KeyCode::Backspace => match app.add_field {
            AddField::Symbol => {
                app.add_symbol.pop();
            }
            AddField::Threshold => {
                app.add_threshold.pop();
            }
            _ => {}
        },
        _ => {}
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::List => render_list(f, app),
        Screen::Add => render_add(f, app),
        Screen::Help => render_help(f, app),
    }
}

fn render_list(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(f.area());

    // Alert list
    if app.alerts.is_empty() {
        let empty = Paragraph::new("No alerts configured.\n\nPress 'a' to add your first alert.")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Alerts"));
        f.render_widget(empty, chunks[0]);
    } else {
        let items: Vec<ListItem> = app
            .alerts
            .iter()
            .enumerate()
            .map(|(idx, alert)| {
                let (status_badge, color, icon) = if !alert.enabled {
                    ("[DISABLED]", Color::DarkGray, "⊗")
                } else if alert.last_triggered.is_some() {
                    ("[TRIGGERED]", Color::Red, "🔔")
                } else {
                    ("[ACTIVE]", Color::Green, "✓")
                };

                let text = format!(
                    "{} {:11} {} {} {}",
                    icon,
                    status_badge,
                    alert.symbol,
                    alert.alert_type.short_display(),
                    alert.alert_type.format_threshold(alert.threshold)
                );

                let style = if idx == app.selected_idx {
                    Style::default()
                        .fg(color)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(color)
                };

                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items).block(Block::default().borders(Borders::ALL).title(format!(
            "Alerts ({}/{})",
            app.alerts.len(),
            app.alerts.iter().filter(|a| a.enabled).count()
        )));

        f.render_widget(list, chunks[0]);
    }

    // Status bar
    let help_text = if app.alerts.is_empty() {
        "h:help"
    } else {
        "j/k:nav  d:delete  t:toggle  c:check  r:refresh  h:help"
    };

    let status = Paragraph::new(format!("{} | {}", app.status_message, help_text))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[1]);
}

fn render_add(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("Add New Alert")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Symbol field
    let symbol_style = if app.add_field == AddField::Symbol {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let symbol = Paragraph::new(app.add_symbol.as_str())
        .style(symbol_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Symbol (e.g., AAPL)"),
        );
    f.render_widget(symbol, chunks[1]);

    // Alert type selector
    let alert_types = get_alert_types();
    let current_type = alert_types[app.add_type_idx];

    let type_style = if app.add_field == AddField::AlertType {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let type_text = format!(
        "{}\n\n← {} | {} →",
        current_type.display(),
        if app.add_type_idx == 0 {
            alert_types[alert_types.len() - 1].display()
        } else {
            alert_types[app.add_type_idx - 1].display()
        },
        alert_types[(app.add_type_idx + 1) % alert_types.len()].display()
    );

    let type_selector = Paragraph::new(type_text).style(type_style).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Alert Type (use ←/→ arrows)"),
    );
    f.render_widget(type_selector, chunks[2]);

    // Threshold field
    let threshold_style = if app.add_field == AddField::Threshold {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let threshold = Paragraph::new(app.add_threshold.as_str())
        .style(threshold_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Threshold ({})", get_threshold_hint(current_type))),
        );
    f.render_widget(threshold, chunks[3]);

    // Help text
    let help_lines = vec![
        Line::from(vec![Span::styled(
            "Examples:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  • Price Above: 150 → Alert when price > $150".to_string()),
        Line::from("  • Change Above: 5 → Alert when change > +5%".to_string()),
        Line::from("  • Volume Spike: 2 → Alert when volume > 2x average".to_string()),
    ];

    let help = Paragraph::new(help_lines)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });
    f.render_widget(help, chunks[4]);

    // Status
    let status = Paragraph::new("Tab/↑↓:switch fields  Enter:create  Esc:cancel")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[5]);
}

fn render_help(f: &mut Frame, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    let title = Paragraph::new("Alert Manager - Help")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let help_text = vec![
        Line::from(Span::styled(
            "Alert Status",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  ✓ [ACTIVE]    ", Style::default().fg(Color::Green)),
            Span::from("- Alert is enabled and monitoring"),
        ]),
        Line::from(vec![
            Span::styled("  🔔 [TRIGGERED]", Style::default().fg(Color::Red)),
            Span::from(" - Alert condition has been met"),
        ]),
        Line::from(vec![
            Span::styled("  ⊗ [DISABLED]  ", Style::default().fg(Color::DarkGray)),
            Span::from(" - Alert is turned off"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  j/↓     - Move down"),
        Line::from("  k/↑     - Move up"),
        Line::from("  g       - Jump to top"),
        Line::from("  G       - Jump to bottom"),
        Line::from(""),
        Line::from(Span::styled(
            "Actions",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  a       - Add new alert"),
        Line::from("  d/Del   - Delete selected alert"),
        Line::from("  t/Space - Toggle alert on/off"),
        Line::from("  c       - Check all alerts against current prices"),
        Line::from("  r       - Refresh alert list"),
        Line::from(""),
        Line::from(Span::styled(
            "Alert Types",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  • Price Above/Below     - Trigger on specific price"),
        Line::from("  • Change Above/Below    - Trigger on % change"),
        Line::from("  • Volume Spike          - Trigger on volume multiplier"),
        Line::from("  • 52W High/Low          - Trigger near 52-week extremes"),
        Line::from("  • Market Cap Above/Below- Trigger on market cap"),
        Line::from("  • Dividend Yield Above  - Trigger on dividend yield"),
        Line::from("  • P/E Ratio Above/Below - Trigger on P/E ratio"),
        Line::from(""),
        Line::from(Span::styled(
            "Other",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  h/?     - Show this help"),
        Line::from("  q/Esc   - Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(help, chunks[1]);

    let status =
        Paragraph::new("Press any key to return").block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
}

fn get_threshold_hint(alert_type: AlertType) -> &'static str {
    match alert_type {
        AlertType::PriceAbove | AlertType::PriceBelow => "e.g., 150 for $150",
        AlertType::ChangeAbove | AlertType::ChangeBelow => "e.g., 5 for 5%",
        AlertType::VolumeSpike => "e.g., 2 for 2x average",
        AlertType::Week52High | AlertType::Week52Low => "e.g., 2 for within 2%",
        AlertType::MarketCapAbove | AlertType::MarketCapBelow => "e.g., 100 for $100B",
        AlertType::DividendYieldAbove => "e.g., 4 for 4%",
        AlertType::PeRatioAbove | AlertType::PeRatioBelow => "e.g., 30",
    }
}
