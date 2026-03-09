use super::super::state::{App, ConfigField};
use super::super::types::{LongOrderType, ShortOrderType};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::collections::HashSet;

pub(super) fn render_welcome(f: &mut Frame, _app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ╔══════════════════════════════════════╗",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(vec![
            Span::styled("  ║     ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "BACKTEST STRATEGY BUILDER",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("     ║", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![Span::styled(
            "  ╚══════════════════════════════════════╝",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Build and test trading strategies with 40+ technical indicators",
            Style::default().fg(Color::DarkGray),
        )]),
    ];
    let title = Paragraph::new(title_text);
    f.render_widget(title, chunks[0]);

    // Options
    let options = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  [1] ", Style::default().fg(Color::Green)),
            Span::styled(
                "New Strategy",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Configure parameters and build custom conditions"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [2] ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Load Preset",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Start from a pre-configured strategy"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [3] ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "Strategy Builder",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Jump directly to building entry/exit conditions"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [4] ", Style::default().fg(Color::Magenta)),
            Span::styled(
                "Compose Ensemble",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Blend multiple presets into one voting strategy"),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("1", Style::default().fg(Color::White)),
            Span::styled(", ", Style::default().fg(Color::DarkGray)),
            Span::styled("2", Style::default().fg(Color::White)),
            Span::styled(", ", Style::default().fg(Color::DarkGray)),
            Span::styled("3", Style::default().fg(Color::White)),
            Span::styled(", or ", Style::default().fg(Color::DarkGray)),
            Span::styled("4", Style::default().fg(Color::White)),
            Span::styled(" to continue", Style::default().fg(Color::DarkGray)),
        ]),
    ];
    let options_widget = Paragraph::new(options).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Choose an Option "),
    );
    f.render_widget(options_widget, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::White)),
        Span::styled(":quit", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[2]);
}

pub(super) fn render_preset_select(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            " Select a Strategy Preset ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
    ]);
    f.render_widget(title, chunks[0]);

    // Combined built-in + user preset list
    let built_in_count = app.presets.len();
    let mut items: Vec<ListItem> = app
        .presets
        .iter()
        .enumerate()
        .map(|(idx, preset)| {
            let selected = idx == app.preset_idx;
            let fg = if selected {
                Color::Yellow
            } else {
                Color::White
            };
            let style = if selected {
                Style::default().fg(fg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(fg)
            };
            ListItem::new(vec![
                Line::from(vec![Span::styled(format!(" {} ", preset.name), style)]),
                Line::from(vec![Span::styled(
                    format!("   {}", preset.description),
                    Style::default().fg(Color::DarkGray),
                )]),
                Line::from(""),
            ])
        })
        .collect();

    if !app.user_presets.is_empty() {
        // Separator between built-in and user presets
        items.push(ListItem::new(vec![
            Line::from(vec![Span::styled(
                " ── Saved Presets ──────────────────────",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(""),
        ]));
        for (offset, preset) in app.user_presets.iter().enumerate() {
            let idx = built_in_count + offset;
            let selected = idx == app.preset_idx;
            let fg = if selected {
                Color::Magenta
            } else {
                Color::White
            };
            let style = if selected {
                Style::default().fg(fg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(fg)
            };
            items.push(ListItem::new(vec![
                Line::from(vec![
                    Span::styled(" ★ ", Style::default().fg(Color::Magenta)),
                    Span::styled(preset.name.as_str(), style),
                ]),
                Line::from(vec![Span::styled(
                    format!("   {}", preset.description),
                    Style::default().fg(Color::DarkGray),
                )]),
                Line::from(""),
            ]));
        }
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.preset_idx));
    f.render_stateful_widget(list, chunks[1], &mut list_state);

    // Footer — show delete hint only when a user preset is selected
    let mut footer_spans = vec![
        Span::styled(" j/k", Style::default().fg(Color::White)),
        Span::styled(":navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::White)),
        Span::styled(":select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("c", Style::default().fg(Color::Magenta)),
        Span::styled(":compose  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::White)),
        Span::styled(":back", Style::default().fg(Color::DarkGray)),
    ];
    if app.is_user_preset(app.preset_idx) && !app.user_presets.is_empty() {
        footer_spans.push(Span::styled("  d", Style::default().fg(Color::Red)));
        footer_spans.push(Span::styled(
            ":delete",
            Style::default().fg(Color::DarkGray),
        ));
    }
    let footer = Paragraph::new(Line::from(footer_spans));
    f.render_widget(footer, chunks[2]);
}

pub(super) fn render_ensemble_compose(f: &mut Frame, app: &App) {
    let area = f.area();
    let total = app.total_preset_count();
    let selected_set: HashSet<usize> = app.ensemble_selected.iter().copied().collect();
    let total_weight: f64 = app
        .ensemble_selected
        .iter()
        .map(|idx| app.ensemble_weight_for(*idx))
        .sum();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(4),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Compose Ensemble ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - mode: ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.ensemble_mode.name(), Style::default().fg(Color::Cyan)),
            Span::styled(" - selected: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", app.ensemble_selected.len()),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(" - total weight: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.2}", total_weight),
                Style::default().fg(Color::Green),
            ),
        ]),
    ]);
    f.render_widget(title, chunks[0]);

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(64), Constraint::Percentage(36)])
        .split(chunks[1]);

    let mut items: Vec<ListItem> = Vec::new();
    if total == 0 {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            " No presets found.",
            Style::default().fg(Color::DarkGray),
        )])));
    } else {
        for idx in 0..total {
            let (name, description, user_preset) = if idx < app.presets.len() {
                let preset = &app.presets[idx];
                (
                    preset.name.to_string(),
                    preset.description.to_string(),
                    false,
                )
            } else {
                let user_idx = idx - app.presets.len();
                if let Some(preset) = app.user_presets.get(user_idx) {
                    (preset.name.clone(), preset.description.clone(), true)
                } else {
                    continue;
                }
            };

            let selected = selected_set.contains(&idx);
            let focused = idx == app.ensemble_cursor_idx;
            let marker = if selected { "[x]" } else { "[ ]" };
            let lead = if focused { ">" } else { " " };
            let weight = app.ensemble_weight_for(idx);

            let name_style = if focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if user_preset {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default().fg(Color::White)
            };

            items.push(ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!(" {} {} ", lead, marker), name_style),
                    Span::styled(
                        if selected {
                            format!("{}  w={:.2}", name, weight)
                        } else {
                            name
                        },
                        name_style,
                    ),
                ]),
                Line::from(vec![Span::styled(
                    format!("    {}", description),
                    Style::default().fg(Color::DarkGray),
                )]),
                Line::from(""),
            ]));
        }
    }

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .title(" Presets (Space to toggle) "),
    );

    let mut list_state = ListState::default();
    if total > 0 {
        list_state.select(Some(app.ensemble_cursor_idx.min(total - 1)));
    }
    f.render_stateful_widget(list, main[0], &mut list_state);

    let mut selected_lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Voting Mode: ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.ensemble_mode.name(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Members:",
            Style::default().fg(Color::Yellow),
        )]),
    ];

    if app.ensemble_selected.is_empty() {
        selected_lines.push(Line::from(vec![Span::styled(
            "  (none selected)",
            Style::default().fg(Color::DarkGray),
        )]));
    } else {
        for idx in &app.ensemble_selected {
            if let Some((name, _, _)) = app.preset_entry(*idx) {
                selected_lines.push(Line::from(vec![Span::styled(
                    format!("  - {} (w={:.2})", name, app.ensemble_weight_for(*idx)),
                    Style::default().fg(Color::White),
                )]));
            }
        }
    }

    let selected_panel = Paragraph::new(selected_lines)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Selection "),
        );
    f.render_widget(selected_panel, main[1]);

    let info_line = if app.editing {
        if let Some((name, _, _)) = app.preset_entry(app.ensemble_cursor_idx) {
            Line::from(vec![
                Span::styled(" Weight input: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{} = {}█", name, app.edit_buffer),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "  (Enter save, Esc cancel)",
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        } else {
            Line::from(vec![Span::styled(
                " Weight input mode  (Enter save, Esc cancel)",
                Style::default().fg(Color::DarkGray),
            )])
        }
    } else if let Some(err) = &app.edit_error {
        Line::from(vec![
            Span::styled(" Error: ", Style::default().fg(Color::Red)),
            Span::styled(err, Style::default().fg(Color::Red)),
        ])
    } else if app.ensemble_selected.len() < 2 {
        Line::from(vec![Span::styled(
            " Select at least 2 presets to build an ensemble.",
            Style::default().fg(Color::Yellow),
        )])
    } else {
        Line::from(vec![Span::styled(
            " Use h/l (or <-/->) for step adjust, w for exact input, then Enter to apply.",
            Style::default().fg(Color::DarkGray),
        )])
    };

    let info = Paragraph::new(vec![Line::from(""), info_line]).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(info, chunks[2]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" j/k", Style::default().fg(Color::White)),
        Span::styled(":navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Space", Style::default().fg(Color::White)),
        Span::styled(":toggle  ", Style::default().fg(Color::DarkGray)),
        Span::styled("a", Style::default().fg(Color::Cyan)),
        Span::styled(":all  ", Style::default().fg(Color::DarkGray)),
        Span::styled("c", Style::default().fg(Color::Cyan)),
        Span::styled(":clear  ", Style::default().fg(Color::DarkGray)),
        Span::styled("m", Style::default().fg(Color::Magenta)),
        Span::styled(":mode  ", Style::default().fg(Color::DarkGray)),
        Span::styled("h/l", Style::default().fg(Color::Yellow)),
        Span::styled(":weight  ", Style::default().fg(Color::DarkGray)),
        Span::styled("w", Style::default().fg(Color::Yellow)),
        Span::styled(":type  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::styled(":apply  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::styled(":back", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[3]);
}

pub(super) fn render_config_editor(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let symbol_str = if app.config.symbol.is_empty() {
        "No Symbol".to_string()
    } else {
        app.config.symbol.clone()
    };
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Configure Backtest ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                symbol_str,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ]);
    f.render_widget(title, chunks[0]);

    // Split main area into two columns
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Left column - config fields
    let fields = ConfigField::all();
    let items: Vec<ListItem> = fields
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            let selected = idx == app.config_field_idx;
            let style = if selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let value = if app.editing && selected {
                format!("{}▌", app.edit_buffer)
            } else {
                app.format_field_value(*field)
            };

            // Dim offset/gap fields when their order type doesn't require them.
            let entry_order_type = app.config.strategy.entry_order_type;
            let short_order_type = app.config.strategy.short_order_type;
            let is_inactive_offset = matches!(field, ConfigField::EntryPriceOffset)
                && !LongOrderType::needs_offset(entry_order_type)
                || matches!(field, ConfigField::EntryStopLimitGap)
                    && !LongOrderType::needs_gap(entry_order_type)
                || matches!(field, ConfigField::ShortPriceOffset)
                    && !ShortOrderType::needs_offset(short_order_type);

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {:<16}", field.name()),
                    if selected {
                        style.fg(Color::Yellow)
                    } else if is_inactive_offset {
                        style.fg(Color::DarkGray)
                    } else {
                        style.fg(Color::White)
                    },
                ),
                Span::styled(
                    value,
                    if selected && app.editing {
                        style.fg(Color::Green)
                    } else if is_inactive_offset {
                        style.fg(Color::DarkGray)
                    } else {
                        style.fg(Color::Cyan)
                    },
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Parameters "),
    );

    // Use stateful rendering so the list scrolls to keep selection visible
    let mut list_state = ListState::default();
    list_state.select(Some(app.config_field_idx));
    f.render_stateful_widget(list, main_chunks[0], &mut list_state);

    // Right column - strategy preview
    let mut strategy_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Strategy: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &app.config.strategy.name,
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Entry Conditions:",
            Style::default().fg(Color::Green),
        )]),
        Line::from(vec![Span::styled(
            format!("   {}", app.config.strategy.entry_conditions.display()),
            Style::default().fg(Color::White),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Exit Conditions:",
            Style::default().fg(Color::Yellow),
        )]),
        Line::from(vec![Span::styled(
            format!("   {}", app.config.strategy.exit_conditions.display()),
            Style::default().fg(Color::White),
        )]),
    ];

    if let Some(ensemble) = &app.config.ensemble {
        let names = ensemble
            .members
            .iter()
            .map(|m| format!("{} (w={:.2})", m.name, m.weight))
            .collect::<Vec<_>>()
            .join(", ");
        strategy_lines.insert(
            5,
            Line::from(vec![
                Span::styled(" Ensemble: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{} ({})", ensemble.mode.name(), ensemble.members.len()),
                    Style::default().fg(Color::Magenta),
                ),
            ]),
        );
        strategy_lines.insert(
            6,
            Line::from(vec![Span::styled(
                format!("   {}", names),
                Style::default().fg(Color::DarkGray),
            )]),
        );
        strategy_lines.insert(7, Line::from(""));
    }

    let strategy_preview = Paragraph::new(strategy_lines)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Strategy Preview "),
        );
    f.render_widget(strategy_preview, main_chunks[1]);

    // Help area
    let help_text = if let Some(ref err) = app.edit_error {
        vec![Line::from(vec![
            Span::styled(" Error: ", Style::default().fg(Color::Red)),
            Span::raw(err),
        ])]
    } else {
        vec![Line::from(vec![Span::styled(
            format!(" {}", app.current_config_field().help()),
            Style::default().fg(Color::DarkGray),
        )])]
    };

    let help = Paragraph::new(help_text).wrap(Wrap { trim: true }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(help, chunks[2]);

    // Footer
    let footer_text = if app.editing {
        vec![
            Span::styled(" Enter", Style::default().fg(Color::White)),
            Span::styled(":save  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::White)),
            Span::styled(":cancel", Style::default().fg(Color::DarkGray)),
        ]
    } else {
        vec![
            Span::styled(" j/k", Style::default().fg(Color::White)),
            Span::styled(":nav  ", Style::default().fg(Color::DarkGray)),
            Span::styled("e", Style::default().fg(Color::White)),
            Span::styled(":edit  ", Style::default().fg(Color::DarkGray)),
            Span::styled("s", Style::default().fg(Color::White)),
            Span::styled(":strategy  ", Style::default().fg(Color::DarkGray)),
            Span::styled("p", Style::default().fg(Color::White)),
            Span::styled(":presets  ", Style::default().fg(Color::DarkGray)),
            Span::styled("c", Style::default().fg(Color::Magenta)),
            Span::styled(":compose  ", Style::default().fg(Color::DarkGray)),
            Span::styled("r", Style::default().fg(Color::White)),
            Span::styled(":run  ", Style::default().fg(Color::DarkGray)),
            Span::styled("q", Style::default().fg(Color::White)),
            Span::styled(":quit", Style::default().fg(Color::DarkGray)),
        ]
    };

    let footer = Paragraph::new(Line::from(footer_text));
    f.render_widget(footer, chunks[3]);
}
