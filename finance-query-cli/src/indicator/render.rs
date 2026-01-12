use super::state::{
    App, ChartField, IndicatorCategory, Screen, interval_to_string, range_to_string,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs},
};

/// Main UI rendering dispatch function
pub fn ui(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::IndicatorSelect => render_indicator_select(f, app),
        Screen::ParamConfig => render_param_config(f, app),
        Screen::ChartConfig => render_chart_config(f, app),
        Screen::Confirmation => render_confirmation(f, app),
    }
}

fn render_indicator_select(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Title + description
            Constraint::Length(3), // Category tabs
            Constraint::Min(0),    // Indicator list
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // Title
    let title_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ╔════════════════════════════════════════╗",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(vec![
            Span::styled("  ║       ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "TECHNICAL INDICATOR CALCULATOR",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("       ║", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![Span::styled(
            "  ╚════════════════════════════════════════╝",
            Style::default().fg(Color::Cyan),
        )]),
    ];
    let title = Paragraph::new(title_text);
    f.render_widget(title, chunks[0]);

    // Category tabs with indicator counts
    let categories = IndicatorCategory::all();
    let tab_titles: Vec<Line> = categories
        .iter()
        .enumerate()
        .map(|(i, cat)| {
            let count = app.indicator_count_by_category(*cat);
            let style = if i == app.category_idx {
                Style::default()
                    .fg(cat.color())
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Line::from(Span::styled(format!(" {} ({}) ", cat.name(), count), style))
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .select(app.category_idx)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[1]);

    // Indicator list for current category
    let indicators = app.indicators_in_category();
    let items: Vec<ListItem> = indicators
        .iter()
        .enumerate()
        .map(|(idx, ind)| {
            let selected = idx == app.indicator_idx;

            let params_str = if ind.params.is_empty() {
                String::new()
            } else {
                let defaults: Vec<String> = ind
                    .params
                    .iter()
                    .map(|p| {
                        if p.default.fract() == 0.0 {
                            format!("{:.0}", p.default)
                        } else {
                            format!("{:.2}", p.default)
                        }
                    })
                    .collect();
                format!(" ({})", defaults.join(", "))
            };

            let prefix = if selected { "▶ " } else { "  " };
            let name_style = if selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let desc_style = if selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        prefix,
                        if selected {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        },
                    ),
                    Span::styled(ind.name, name_style),
                    Span::styled(params_str, Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(ind.description, desc_style),
                ]),
            ])
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(format!(
            " {} Indicators ",
            categories
                .get(app.category_idx)
                .map(|c| c.name())
                .unwrap_or("All")
        ));

    let list = List::new(items)
        .block(list_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut list_state = ListState::default();
    list_state.select(Some(app.indicator_idx));
    f.render_stateful_widget(list, chunks[2], &mut list_state);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ←/→", Style::default().fg(Color::White)),
        Span::styled(":category  ", Style::default().fg(Color::DarkGray)),
        Span::styled("↑/↓", Style::default().fg(Color::White)),
        Span::styled(":select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::styled(":configure  ", Style::default().fg(Color::DarkGray)),
        Span::styled("1-5", Style::default().fg(Color::White)),
        Span::styled(":jump  ", Style::default().fg(Color::DarkGray)),
        Span::styled("q", Style::default().fg(Color::Red)),
        Span::styled(":quit", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[3]);
}

fn render_param_config(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Title
            Constraint::Min(0),    // Parameters
            Constraint::Length(5), // Preview
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // Title with indicator name
    let ind_name = app
        .selected_indicator
        .as_ref()
        .map(|i| i.name)
        .unwrap_or("Unknown");

    let title_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Configure Parameters for ",
                Style::default().fg(Color::White),
            ),
            Span::styled(
                ind_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];
    let title = Paragraph::new(title_text);
    f.render_widget(title, chunks[0]);

    // Parameters list with sliders
    if let Some(ref ind) = app.selected_indicator {
        let items: Vec<ListItem> = ind
            .params
            .iter()
            .enumerate()
            .map(|(idx, param)| {
                let selected = idx == app.param_idx;
                let value = app.param_values.get(idx).copied().unwrap_or(param.default);

                let prefix = if selected { "▶ " } else { "  " };
                let name_style = if selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                // Value display
                let value_str = if value.fract() == 0.0 {
                    format!("{:.0}", value)
                } else {
                    format!("{:.2}", value)
                };

                // Create a simple text-based slider
                let range = param.max - param.min;
                let pct = ((value - param.min) / range * 100.0) as u16;
                let bar_width = 20;
                let filled = (pct as usize * bar_width / 100).min(bar_width);
                let empty = bar_width - filled;
                let slider = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(prefix, name_style),
                        Span::styled(format!("{}: ", param.name), name_style),
                        Span::styled(
                            value_str,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(
                            slider,
                            if selected {
                                Style::default().fg(Color::Yellow)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            },
                        ),
                        Span::styled(
                            format!("  ({:.0}-{:.0})", param.min, param.max),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(param.description, Style::default().fg(Color::DarkGray)),
                    ]),
                    Line::from(""),
                ])
            })
            .collect();

        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Parameters (←/→ to adjust, d to reset) ");

        let list = List::new(items).block(list_block);
        f.render_widget(list, chunks[1]);
    }

    // Preview showing what will be calculated
    let preview = build_preview_string(app);
    let preview_widget = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Preview: ", Style::default().fg(Color::DarkGray)),
            Span::styled(preview, Style::default().fg(Color::Green)),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(preview_widget, chunks[2]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↑/↓", Style::default().fg(Color::White)),
        Span::styled(":select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("←/→", Style::default().fg(Color::White)),
        Span::styled(":adjust  ", Style::default().fg(Color::DarkGray)),
        Span::styled("H/L", Style::default().fg(Color::White)),
        Span::styled(":±10  ", Style::default().fg(Color::DarkGray)),
        Span::styled("d", Style::default().fg(Color::White)),
        Span::styled(":default  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::styled(":continue  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::styled(":back", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[3]);
}

fn render_chart_config(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(5), // Fields (3 items, 1 line each + borders)
            Constraint::Length(3), // Summary
            Constraint::Length(3), // Footer
            Constraint::Min(0),    // Remaining space
        ])
        .split(area);

    // Title
    let ind_name = app
        .selected_indicator
        .as_ref()
        .map(|i| i.name)
        .unwrap_or("Unknown");

    let title_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " ═══ Chart Settings for ",
                Style::default().fg(Color::White),
            ),
            Span::styled(
                ind_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ═══", Style::default().fg(Color::White)),
        ]),
    ];
    let title = Paragraph::new(title_text);
    f.render_widget(title, chunks[0]);

    // Fields
    let fields = ChartField::all();
    let items: Vec<ListItem> = fields
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            let selected = idx == app.chart_field_idx;
            let value = app.get_chart_field_value(*field);

            let prefix = if selected { "▶ " } else { "  " };
            let name_style = if selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Show edit buffer if editing this field
            let display_value = if app.editing && selected {
                format!("{}▌", app.edit_buffer)
            } else {
                value
            };

            let value_style = if *field == ChartField::Symbol && app.symbol.is_empty() {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Cyan)
            };

            ListItem::new(vec![Line::from(vec![
                Span::styled(prefix, name_style),
                Span::styled(format!("{}: ", field.name()), name_style),
                Span::styled(display_value, value_style),
                Span::raw("  "),
                Span::styled(field.help(), Style::default().fg(Color::DarkGray)),
            ])])
        })
        .collect();

    let mut list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Chart Configuration ");

    // Show error if any
    if let Some(ref err) = app.edit_error {
        list_block = list_block.title(Span::styled(
            format!(" Error: {} ", err),
            Style::default().fg(Color::Red),
        ));
    }

    let list = List::new(items).block(list_block);
    f.render_widget(list, chunks[1]);

    // Summary (compact: single line)
    let preview = build_full_preview_string(app);
    let summary = Paragraph::new(vec![Line::from(vec![
        Span::styled(" Will calculate: ", Style::default().fg(Color::DarkGray)),
        Span::styled(preview, Style::default().fg(Color::Green)),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(summary, chunks[2]);

    // Footer
    let can_run = app.can_run();
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↑/↓", Style::default().fg(Color::White)),
        Span::styled(":select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::White)),
        Span::styled(":edit  ", Style::default().fg(Color::DarkGray)),
        if can_run {
            Span::styled("r", Style::default().fg(Color::Green))
        } else {
            Span::styled("r", Style::default().fg(Color::DarkGray))
        },
        Span::styled(":run  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::styled(":back", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[3]);
}

fn render_confirmation(f: &mut Frame, app: &App) {
    let area = f.area();

    // Center the confirmation dialog
    let popup_area = centered_rect(60, 50, area);
    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Details
            Constraint::Length(3), // Buttons
        ])
        .split(popup_area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            " Confirm Indicator Calculation ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
    ])
    .block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(title, chunks[0]);

    // Details
    let ind_name = app
        .selected_indicator
        .as_ref()
        .map(|i| i.name)
        .unwrap_or("Unknown");

    let params_str = build_preview_string(app);

    let details = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Indicator: ", Style::default().fg(Color::White)),
            Span::styled(ind_name, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  Parameters: ", Style::default().fg(Color::White)),
            Span::styled(params_str, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  Symbol: ", Style::default().fg(Color::White)),
            Span::styled(&app.symbol, Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("  Interval: ", Style::default().fg(Color::White)),
            Span::styled(
                interval_to_string(app.interval),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Range: ", Style::default().fg(Color::White)),
            Span::styled(range_to_string(app.range), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
    ])
    .block(
        Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(details, chunks[1]);

    // Buttons
    let buttons = Paragraph::new(Line::from(vec![
        Span::styled("  [y/Enter] ", Style::default().fg(Color::Green)),
        Span::styled("Run  ", Style::default().fg(Color::White)),
        Span::styled("[n/Esc] ", Style::default().fg(Color::Red)),
        Span::styled("Cancel", Style::default().fg(Color::White)),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(buttons, chunks[2]);
}

/// Build a preview string showing indicator with params
fn build_preview_string(app: &App) -> String {
    if let Some(ref ind) = app.selected_indicator {
        if ind.params.is_empty() {
            ind.code.to_uppercase()
        } else {
            let params: Vec<String> = app
                .param_values
                .iter()
                .map(|v| {
                    if v.fract() == 0.0 {
                        format!("{:.0}", v)
                    } else {
                        format!("{:.2}", v)
                    }
                })
                .collect();
            format!("{}({})", ind.code.to_uppercase(), params.join(", "))
        }
    } else {
        "No indicator selected".to_string()
    }
}

/// Build a full preview string with symbol and time range
fn build_full_preview_string(app: &App) -> String {
    let indicator = build_preview_string(app);
    let symbol = if app.symbol.is_empty() {
        "???".to_string()
    } else {
        app.symbol.clone()
    };

    format!(
        "{} on {} ({}, {})",
        indicator,
        symbol,
        interval_to_string(app.interval),
        range_to_string(app.range)
    )
}

/// Helper to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
