use crate::dashboard::state::{App, FocusPane, InputMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub(super) fn render_alerts_tab(f: &mut Frame, app: &App, area: Rect) {
    // Check if we're in add alert mode
    if app.input_mode == InputMode::AddAlert {
        render_add_alert_form(f, app, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left side: Alert list
    let items: Vec<ListItem> = app
        .alerts
        .iter()
        .enumerate()
        .map(|(idx, alert)| {
            let triggered = alert.last_triggered.is_some();
            let icon = if triggered { "*" } else { "o" };
            let status_color = if triggered {
                Color::Red
            } else if !alert.enabled {
                Color::DarkGray
            } else {
                Color::Green
            };
            let is_selected = idx == app.selected_alert_idx;

            let text = format!(
                "{} {:<6} {:<12} {}",
                icon,
                alert.symbol,
                alert.alert_type.short_display(),
                alert.alert_type.format_threshold(alert.threshold)
            );

            let style = if is_selected {
                Style::default()
                    .fg(status_color)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(status_color)
            };

            ListItem::new(Line::from(vec![Span::styled(text, style)]))
        })
        .collect();

    let list_title = format!(
        "Alerts ({}) [a:add d:del e:toggle j/k:nav]",
        app.alerts.len()
    );

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(list_title)
            .border_style(Style::default().fg(if app.focus_pane == FocusPane::Left {
                Color::Yellow
            } else {
                Color::White
            })),
    );

    f.render_widget(list, chunks[0]);

    // Right side: Selected alert details or help
    let detail_lines = if app.alerts.is_empty() {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "No alerts configured",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from("Press 'a' to add your first alert."),
            Line::from(""),
            Line::from(Span::styled(
                "Available Alert Types:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("  Price >    - Price goes above value"),
            Line::from("  Price <    - Price goes below value"),
            Line::from("  Chg% >     - Daily change exceeds %"),
            Line::from("  Chg% <     - Daily change below %"),
            Line::from("  Vol spike  - Volume exceeds average"),
            Line::from("  52W High   - Near 52-week high"),
            Line::from("  52W Low    - Near 52-week low"),
            Line::from("  MCap >/<   - Market cap threshold"),
            Line::from("  DivYld >   - Dividend yield above"),
            Line::from("  P/E >/<    - P/E ratio threshold"),
        ]
    } else if app.selected_alert_idx < app.alerts.len() {
        let alert = &app.alerts[app.selected_alert_idx];
        let status = if alert.last_triggered.is_some() {
            Span::styled(
                "TRIGGERED",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )
        } else if !alert.enabled {
            Span::styled("Disabled", Style::default().fg(Color::DarkGray))
        } else {
            Span::styled("Active", Style::default().fg(Color::Green))
        };

        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Symbol:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &alert.symbol,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Type:      ", Style::default().fg(Color::DarkGray)),
                Span::raw(alert.alert_type.display()),
            ]),
            Line::from(vec![
                Span::styled("Threshold: ", Style::default().fg(Color::DarkGray)),
                Span::raw(alert.alert_type.format_threshold(alert.threshold)),
            ]),
            Line::from(vec![
                Span::styled("Status:    ", Style::default().fg(Color::DarkGray)),
                status,
            ]),
            Line::from(vec![
                Span::styled("Created:   ", Style::default().fg(Color::DarkGray)),
                Span::raw(alert.created_at.format("%Y-%m-%d %H:%M").to_string()),
            ]),
        ];

        if let Some(label) = &alert.label {
            lines.push(Line::from(vec![
                Span::styled("Label:     ", Style::default().fg(Color::DarkGray)),
                Span::raw(label),
            ]));
        }

        if let Some(last_triggered) = alert.last_triggered {
            lines.push(Line::from(vec![
                Span::styled("Triggered: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    last_triggered.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default().fg(Color::Yellow),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Count:     ", Style::default().fg(Color::DarkGray)),
                Span::raw(format!("{} time(s)", alert.trigger_count)),
            ]));
        }

        // Show current value if we have quote data
        if let Some(quote) = app.quotes.get(&alert.symbol) {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "─ Current Value ─",
                Style::default().fg(Color::DarkGray),
            )));
            let current = alert.format_current_value(quote);
            let triggered_now = alert.check(quote);
            let current_style = if triggered_now {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(vec![
                Span::styled("Current:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(current, current_style),
            ]));
        }

        lines
    } else {
        vec![Line::from("Select an alert")]
    };

    let detail_block = Paragraph::new(detail_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Alert Details")
            .border_style(Style::default().fg(if app.focus_pane == FocusPane::Right {
                Color::Yellow
            } else {
                Color::White
            })),
    );

    f.render_widget(detail_block, chunks[1]);
}

fn render_add_alert_form(f: &mut Frame, app: &App, area: Rect) {
    use crate::alerts::AlertType;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        "Create New Alert",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Form fields
    let all_types = AlertType::all();
    let alert_type = all_types[app.alert_form_type_idx];

    let form_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(chunks[1]);

    // Symbol field
    let symbol_style = if app.alert_form_field == 0 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let symbol_text = if app.alert_form_field == 0 {
        format!("{}▌", app.alert_form_symbol)
    } else if app.alert_form_symbol.is_empty() {
        "Enter symbol...".to_string()
    } else {
        app.alert_form_symbol.clone()
    };
    let symbol_field = Paragraph::new(symbol_text)
        .style(symbol_style)
        .block(Block::default().borders(Borders::ALL).title("Symbol"));
    f.render_widget(symbol_field, form_chunks[0]);

    // Type field
    let type_style = if app.alert_form_field == 1 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let type_text = if app.alert_form_field == 1 {
        format!("< {} >", alert_type.display())
    } else {
        alert_type.display().to_string()
    };
    let type_field = Paragraph::new(type_text)
        .style(type_style)
        .block(Block::default().borders(Borders::ALL).title("Type (←/→)"));
    f.render_widget(type_field, form_chunks[1]);

    // Threshold field
    let threshold_style = if app.alert_form_field == 2 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let threshold_text = if app.alert_form_field == 2 {
        format!("{}▌", app.alert_form_threshold)
    } else if app.alert_form_threshold.is_empty() {
        "Enter value...".to_string()
    } else {
        app.alert_form_threshold.clone()
    };
    let threshold_field = Paragraph::new(threshold_text)
        .style(threshold_style)
        .block(Block::default().borders(Borders::ALL).title("Value"));
    f.render_widget(threshold_field, form_chunks[2]);

    // Help text
    let help_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(": Next field   "),
            Span::styled("←/→", Style::default().fg(Color::Yellow)),
            Span::raw(": Change type   "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": Create   "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": Cancel"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Alert Types:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Price >/< : Alert when price crosses threshold (e.g., 150.00)"),
        Line::from("  Chg% >/<  : Alert when daily % change crosses threshold (e.g., 5.0)"),
        Line::from("  Vol spike : Alert when volume exceeds multiplier of average (e.g., 2.0)"),
        Line::from("  52W High/Low : Alert when within % of 52-week high/low (e.g., 5.0)"),
        Line::from("  MCap >/<  : Alert when market cap crosses threshold in billions (e.g., 100)"),
        Line::from("  DivYld >  : Alert when dividend yield exceeds % (e.g., 4.0)"),
        Line::from("  P/E >/<   : Alert when P/E ratio crosses threshold (e.g., 30)"),
    ];

    let help = Paragraph::new(help_lines)
        .block(Block::default().borders(Borders::ALL).title("Instructions"));
    f.render_widget(help, chunks[2]);
}
