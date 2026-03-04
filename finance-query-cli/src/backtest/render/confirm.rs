use super::super::state::{
    App, OPTIMIZER_FIELD_END, OPTIMIZER_FIELD_START, OPTIMIZER_FIELD_STEP, interval_to_string,
    range_to_string,
};
use super::super::types::{all_optimize_metrics, optimize_metric_label};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub(super) fn render_confirmation(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            " Confirm Backtest Configuration ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
    ]);
    f.render_widget(title, chunks[0]);

    // Summary
    let summary_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Symbol:       ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &app.config.symbol,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Strategy:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.config.strategy.name, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  Interval:     ", Style::default().fg(Color::DarkGray)),
            Span::raw(interval_to_string(app.config.interval)),
        ]),
        Line::from(vec![
            Span::styled("  Range:        ", Style::default().fg(Color::DarkGray)),
            Span::raw(range_to_string(app.config.range)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Capital:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("${:.2}", app.config.capital),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Position:     ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!("{:.0}%", app.config.position_size * 100.0)),
        ]),
        Line::from(vec![
            Span::styled("  Commission:   ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!(
                "{:.2}%{}",
                app.config.commission * 100.0,
                if app.config.commission_flat > 0.0 {
                    format!(" + ${:.2} flat", app.config.commission_flat)
                } else {
                    String::new()
                }
            )),
        ]),
        Line::from(vec![
            Span::styled("  Slippage:     ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!("{:.2}%", app.config.slippage * 100.0)),
        ]),
        Line::from(vec![
            Span::styled("  Stop / TP:    ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!(
                "{}  /  {}{}",
                app.config
                    .stop_loss
                    .map_or("–".to_string(), |v| format!("{:.1}%", v * 100.0)),
                app.config
                    .take_profit
                    .map_or("–".to_string(), |v| format!("{:.1}%", v * 100.0)),
                app.config
                    .trailing_stop
                    .map_or(String::new(), |v| format!("  trail {:.1}%", v * 100.0)),
            )),
        ]),
        Line::from(vec![
            Span::styled("  Short / Divs: ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!(
                "{}  /  {}",
                if app.config.allow_short {
                    "allowed"
                } else {
                    "long-only"
                },
                if app.config.reinvest_dividends {
                    "reinvest"
                } else {
                    "no reinvest"
                },
            )),
        ]),
        Line::from(vec![
            Span::styled("  RF Rate:      ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!("{:.1}%", app.config.risk_free_rate * 100.0)),
        ]),
        {
            if let Some(bm) = &app.config.benchmark {
                Line::from(vec![
                    Span::styled("  Benchmark:    ", Style::default().fg(Color::DarkGray)),
                    Span::styled(bm.as_str(), Style::default().fg(Color::Cyan)),
                ])
            } else {
                Line::from("")
            }
        },
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Entry:",
            Style::default().fg(Color::Green),
        )]),
        Line::from(vec![Span::styled(
            format!("    {}", app.config.strategy.entry_conditions.display()),
            Style::default().fg(Color::White),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Exit:",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from(vec![Span::styled(
            format!("    {}", app.config.strategy.exit_conditions.display()),
            Style::default().fg(Color::White),
        )]),
    ];

    let summary_lines = if let Some(opt) = &app.config.optimizer {
        let mut lines = summary_lines;
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "  Optimizer:",
            Style::default().fg(Color::Magenta),
        )]));
        lines.push(Line::from(vec![
            Span::styled("    Metric:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                optimize_metric_label(opt.metric),
                Style::default().fg(Color::White),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("    Walk-fwd:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if opt.walk_forward { "Yes" } else { "No" },
                Style::default().fg(if opt.walk_forward {
                    Color::Green
                } else {
                    Color::DarkGray
                }),
            ),
        ]));
        if opt.walk_forward {
            lines.push(Line::from(vec![
                Span::styled("    IS/OOS:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} / {} bars", opt.in_sample_bars, opt.out_of_sample_bars),
                    Style::default().fg(Color::White),
                ),
            ]));
        }
        lines
    } else {
        summary_lines
    };

    let summary = Paragraph::new(summary_lines)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Configuration Summary "),
        );
    f.render_widget(summary, chunks[1]);

    // Confirmation prompt
    let confirm = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Run backtest with this configuration?",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );
    f.render_widget(confirm, chunks[2]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" y/Enter", Style::default().fg(Color::Green)),
        Span::styled(":run  ", Style::default().fg(Color::DarkGray)),
        Span::styled("s", Style::default().fg(Color::Magenta)),
        Span::styled(":save preset  ", Style::default().fg(Color::DarkGray)),
        Span::styled("o", Style::default().fg(Color::Cyan)),
        Span::styled(":optimize  ", Style::default().fg(Color::DarkGray)),
        Span::styled("n/Esc", Style::default().fg(Color::Red)),
        Span::styled(":back  ", Style::default().fg(Color::DarkGray)),
        Span::styled("q", Style::default().fg(Color::White)),
        Span::styled(":quit", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[3]);
}

pub(super) fn render_save_preset(f: &mut Frame, app: &App) {
    let area = f.area();

    // Center a dialog box in the middle of the screen
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(10),
            Constraint::Min(0),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(vertical[1]);
    let dialog_area = horizontal[1];

    // Dialog content
    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Strategy: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.config.strategy.name.as_str(),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Name: ", Style::default().fg(Color::White)),
            Span::styled(
                app.save_preset_buffer.as_str(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
    ];

    if let Some(err) = &app.save_preset_error {
        lines.push(Line::from(vec![Span::styled(
            format!("  ✗ {}", err),
            Style::default().fg(Color::Red),
        )]));
    }

    // Split the dialog area: content + footer row
    let dialog_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(dialog_area);

    let dialog = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .title(" Save Strategy as Preset "),
    );
    f.render_widget(dialog, dialog_chunks[0]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Enter", Style::default().fg(Color::Green)),
        Span::styled(":save  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::White)),
        Span::styled(":cancel", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, dialog_chunks[1]);
}

pub(super) fn render_optimizer_setup(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            " Parameter Optimizer ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
    ]);
    f.render_widget(title, chunks[0]);

    // Params table
    let param_items: Vec<ListItem> = if app.optimizer_params.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "  No optimizable parameters found in current strategy.",
            Style::default().fg(Color::DarkGray),
        )]))]
    } else {
        app.optimizer_params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let is_selected = i == app.optimizer_param_idx;
                let enabled_mark = if p.enabled { "✓" } else { "○" };
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(if p.enabled {
                        Color::White
                    } else {
                        Color::DarkGray
                    })
                };

                let start_style = if is_selected && app.optimizer_field_idx == OPTIMIZER_FIELD_START
                {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::UNDERLINED)
                } else {
                    Style::default().fg(Color::Green)
                };
                let end_style = if is_selected && app.optimizer_field_idx == OPTIMIZER_FIELD_END {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::UNDERLINED)
                } else {
                    Style::default().fg(Color::Green)
                };
                let step_style = if is_selected && app.optimizer_field_idx == OPTIMIZER_FIELD_STEP {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::UNDERLINED)
                } else {
                    Style::default().fg(Color::Green)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", enabled_mark), style),
                    Span::styled(format!("{:<35}", p.name), style),
                    Span::styled(" start:", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{:<7}", p.start), start_style),
                    Span::styled(" end:", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{:<7}", p.end), end_style),
                    Span::styled(" step:", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{}", p.step), step_style),
                ]))
            })
            .collect()
    };

    let param_list = List::new(param_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Parameters (Space=toggle, Enter=edit, ←/→=field) "),
    );
    f.render_widget(param_list, chunks[1]);

    // Options panel: metric + walk-forward
    let metrics = all_optimize_metrics();
    let metric_label = optimize_metric_label(
        metrics
            .get(app.optimizer_metric_idx)
            .copied()
            .unwrap_or(metrics[0]),
    );
    let wf_label = if app.optimizer_walk_forward {
        format!(
            "Yes  (IS: {} bars, OOS: {} bars)",
            app.optimizer_in_sample, app.optimizer_oos
        )
    } else {
        "No".to_string()
    };

    let opts = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Optimize for:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                metric_label,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  (m to cycle)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  Walk-forward:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                wf_label,
                Style::default().fg(if app.optimizer_walk_forward {
                    Color::Green
                } else {
                    Color::DarkGray
                }),
            ),
            Span::styled("  (w to toggle)", Style::default().fg(Color::DarkGray)),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .title(" Options "),
    );
    f.render_widget(opts, chunks[2]);

    // Footer
    let edit_hint = if app.editing {
        format!(" Editing: {} ", app.edit_buffer)
    } else {
        String::new()
    };
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↑/↓", Style::default().fg(Color::White)),
        Span::styled(":param  ", Style::default().fg(Color::DarkGray)),
        Span::styled("←/→", Style::default().fg(Color::White)),
        Span::styled(":field  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::White)),
        Span::styled(":edit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Space", Style::default().fg(Color::White)),
        Span::styled(":toggle  ", Style::default().fg(Color::DarkGray)),
        Span::styled("m", Style::default().fg(Color::Cyan)),
        Span::styled(":metric  ", Style::default().fg(Color::DarkGray)),
        Span::styled("w", Style::default().fg(Color::Magenta)),
        Span::styled(":walk-fwd  ", Style::default().fg(Color::DarkGray)),
        Span::styled("r", Style::default().fg(Color::Green)),
        Span::styled(":run  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::styled(":back", Style::default().fg(Color::DarkGray)),
        Span::styled(edit_hint, Style::default().fg(Color::Yellow)),
    ]));
    f.render_widget(footer, chunks[3]);
}
