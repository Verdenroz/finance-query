use super::indicators::IndicatorCategory;
use super::state::{App, ConditionPanel, ConfigField, Screen, interval_to_string, range_to_string};
use super::types::ComparisonType;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
};

/// Main UI rendering dispatch function
pub fn ui(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::Welcome => render_welcome(f, app),
        Screen::PresetSelect => render_preset_select(f, app),
        Screen::ConfigEditor => render_config_editor(f, app),
        Screen::StrategyBuilder => render_strategy_builder(f, app),
        Screen::IndicatorBrowser => render_indicator_browser(f, app),
        Screen::IndicatorConfig => render_indicator_config(f, app),
        Screen::ComparisonConfig => render_comparison_config(f, app),
        Screen::TargetConfig => render_target_config(f, app),
        Screen::Confirmation => render_confirmation(f, app),
    }
}

fn render_welcome(f: &mut Frame, _app: &App) {
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
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("1", Style::default().fg(Color::White)),
            Span::styled(", ", Style::default().fg(Color::DarkGray)),
            Span::styled("2", Style::default().fg(Color::White)),
            Span::styled(", or ", Style::default().fg(Color::DarkGray)),
            Span::styled("3", Style::default().fg(Color::White)),
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

fn render_preset_select(f: &mut Frame, app: &App) {
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

    // Preset list
    let items: Vec<ListItem> = app
        .presets
        .iter()
        .enumerate()
        .map(|(idx, preset)| {
            let selected = idx == app.preset_idx;
            let style = if selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
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

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" j/k", Style::default().fg(Color::White)),
        Span::styled(":navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::White)),
        Span::styled(":select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::White)),
        Span::styled(":back", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[2]);
}

fn render_config_editor(f: &mut Frame, app: &App) {
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

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {:<14}", field.name()),
                    if selected {
                        style.fg(Color::Yellow)
                    } else {
                        style.fg(Color::White)
                    },
                ),
                Span::styled(
                    value,
                    if selected && app.editing {
                        style.fg(Color::Green)
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
    let strategy_lines = vec![
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
            Span::styled("r", Style::default().fg(Color::White)),
            Span::styled(":run  ", Style::default().fg(Color::DarkGray)),
            Span::styled("q", Style::default().fg(Color::White)),
            Span::styled(":quit", Style::default().fg(Color::DarkGray)),
        ]
    };

    let footer = Paragraph::new(Line::from(footer_text));
    f.render_widget(footer, chunks[3]);
}

fn render_strategy_builder(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Strategy Builder ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " - Navigate with ←/→ and ↑/↓, toggle AND/OR with Enter/c, delete with d",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ]);
    f.render_widget(title, chunks[0]);

    // Main content - split into entry/exit sections
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Determine which panel is active
    let entry_active = app.active_condition_panel == ConditionPanel::Entry;
    let exit_active = app.active_condition_panel == ConditionPanel::Exit;

    // Entry conditions panel
    let entry_conds = &app.config.strategy.entry_conditions;
    let entry_items: Vec<ListItem> = if entry_conds.conditions.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "(no conditions - press 1 to add)",
            Style::default().fg(Color::DarkGray),
        )]))]
    } else {
        entry_conds
            .conditions
            .iter()
            .enumerate()
            .flat_map(|(i, cond)| {
                let mut items = Vec::new();

                // Add the condition line
                let is_selected = entry_active && i == app.entry_condition_idx;
                let prefix = if is_selected { "▶ " } else { "  " };
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(cond.display(), style),
                ])));

                // Add the operator line after this condition (if not the last)
                if i < entry_conds.conditions.len() - 1 {
                    let op_style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("     "),
                        Span::styled(format!("[{}]", cond.next_op.name()), op_style),
                    ])));
                }

                items
            })
            .collect()
    };

    let entry_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if entry_active {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .title(if entry_active {
            " ▶ Entry (Long) - [1] add "
        } else {
            "   Entry (Long) - [1] add "
        });

    let entry_list = List::new(entry_items)
        .block(entry_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(entry_list, main_chunks[0]);

    // Exit conditions panel
    let exit_conds = &app.config.strategy.exit_conditions;
    let exit_items: Vec<ListItem> = if exit_conds.conditions.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "(no conditions - press 2 to add)",
            Style::default().fg(Color::DarkGray),
        )]))]
    } else {
        exit_conds
            .conditions
            .iter()
            .enumerate()
            .flat_map(|(i, cond)| {
                let mut items = Vec::new();

                // Add the condition line
                let is_selected = exit_active && i == app.exit_condition_idx;
                let prefix = if is_selected { "▶ " } else { "  " };
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                items.push(ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(cond.display(), style),
                ])));

                // Add the operator line after this condition (if not the last)
                if i < exit_conds.conditions.len() - 1 {
                    let op_style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("     "),
                        Span::styled(format!("[{}]", cond.next_op.name()), op_style),
                    ])));
                }

                items
            })
            .collect()
    };

    let exit_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if exit_active {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .title(if exit_active {
            " ▶ Exit - [2] add "
        } else {
            "   Exit - [2] add "
        });

    let exit_list = List::new(exit_items)
        .block(exit_block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(exit_list, main_chunks[1]);

    // Footer with updated keybindings
    let can_run = app.can_run();
    let footer = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" ←/→/Tab", Style::default().fg(Color::White)),
            Span::styled(":switch panel  ", Style::default().fg(Color::DarkGray)),
            Span::styled("↑/↓", Style::default().fg(Color::White)),
            Span::styled(":select  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter", Style::default().fg(Color::White)),
            Span::styled(":add condition  ", Style::default().fg(Color::DarkGray)),
            Span::styled("c", Style::default().fg(Color::White)),
            Span::styled(":toggle AND/OR  ", Style::default().fg(Color::DarkGray)),
            Span::styled("d", Style::default().fg(Color::White)),
            Span::styled(":delete  ", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(" 1", Style::default().fg(Color::Green)),
            Span::styled(":add entry  ", Style::default().fg(Color::DarkGray)),
            Span::styled("2", Style::default().fg(Color::Yellow)),
            Span::styled(":add exit  ", Style::default().fg(Color::DarkGray)),
            Span::styled("b", Style::default().fg(Color::White)),
            Span::styled(":back  ", Style::default().fg(Color::DarkGray)),
            if can_run {
                Span::styled("r", Style::default().fg(Color::Green))
            } else {
                Span::styled("r", Style::default().fg(Color::DarkGray))
            },
            Span::styled(":run", Style::default().fg(Color::DarkGray)),
        ]),
    ]);
    f.render_widget(footer, chunks[2]);
}

fn render_indicator_browser(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Select Indicator ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" for ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.condition_target.name(),
                Style::default()
                    .fg(app.condition_target.color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" condition", Style::default().fg(Color::DarkGray)),
        ]),
    ]);
    f.render_widget(title, chunks[0]);

    // Category tabs with indicator counts
    let categories = IndicatorCategory::all();
    let tab_titles: Vec<Line> = categories
        .iter()
        .map(|cat| {
            let count = app.indicator_count_by_category(*cat);
            Line::from(Span::styled(
                format!("{} ({})", cat.name(), count),
                Style::default().fg(cat.color()),
            ))
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(app.category_idx)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
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
                let ps: Vec<String> = ind.params.iter().map(|p| p.name.to_string()).collect();
                format!(" ({})", ps.join(", "))
            };

            let prefix = if selected { "▶ " } else { "  " };

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
                    Span::styled(
                        format!("{} ", ind.name),
                        if selected {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        },
                    ),
                    Span::styled(params_str, Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![Span::styled(
                    format!("     {}", ind.description),
                    Style::default().fg(Color::DarkGray),
                )]),
            ])
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.current_category().color()))
            .title(format!(" {} ", app.current_category().name())),
    );

    let mut list_state = ListState::default();
    list_state.select(Some(app.indicator_idx));
    f.render_stateful_widget(list, chunks[2], &mut list_state);

    // Info panel
    let info_lines = if let Some(ind) = app.current_indicator() {
        let mut lines = vec![Line::from(vec![
            Span::styled(" Code: ", Style::default().fg(Color::DarkGray)),
            Span::styled(ind.code, Style::default().fg(Color::Cyan)),
        ])];

        if let Some((low, high)) = ind.typical_range {
            lines.push(Line::from(vec![
                Span::styled(" Range: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.0} to {:.0}", low, high),
                    Style::default().fg(Color::White),
                ),
            ]));
        }

        if !ind.params.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(" Parameters: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}", ind.params.len()),
                    Style::default().fg(Color::White),
                ),
            ]));
        }

        lines
    } else {
        vec![Line::from("")]
    };

    let info = Paragraph::new(info_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Info "),
    );
    f.render_widget(info, chunks[3]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ←/→", Style::default().fg(Color::White)),
        Span::styled(":category  ", Style::default().fg(Color::DarkGray)),
        Span::styled("↑/↓", Style::default().fg(Color::White)),
        Span::styled(":navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::styled(":select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::White)),
        Span::styled(":back", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[4]);
}

fn render_indicator_config(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let ind = app.building_indicator.as_ref();
    let ind_name = ind.map(|i| i.indicator.name).unwrap_or("Unknown");

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Configure Parameters ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                ind_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ]);
    f.render_widget(title, chunks[0]);

    // Parameters
    let param_lines: Vec<Line> = if let Some(ind) = ind {
        if ind.indicator.params.is_empty() {
            vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    " This indicator has no configurable parameters.",
                    Style::default().fg(Color::DarkGray),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    " Press Enter to continue.",
                    Style::default().fg(Color::White),
                )]),
            ]
        } else {
            let mut lines = vec![Line::from("")];
            for (idx, param) in ind.indicator.params.iter().enumerate() {
                let selected = idx == app.param_idx;
                let value = app.param_values.get(idx).copied().unwrap_or(param.default);

                let style = if selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {:<15}", param.name),
                        if selected {
                            style.fg(Color::Yellow)
                        } else {
                            style.fg(Color::White)
                        },
                    ),
                    Span::styled(
                        format!(
                            " {} ",
                            if value.fract() == 0.0 {
                                format!("{:.0}", value)
                            } else {
                                format!("{:.2}", value)
                            }
                        ),
                        style.fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("  ({:.0}-{:.0})", param.min, param.max),
                        style.fg(Color::DarkGray),
                    ),
                ]));

                if selected {
                    lines.push(Line::from(vec![Span::styled(
                        format!("                 {}", param.description),
                        style.fg(Color::DarkGray),
                    )]));
                }
            }
            lines
        }
    } else {
        vec![Line::from("")]
    };

    let params = Paragraph::new(param_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Parameters "),
    );
    f.render_widget(params, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" j/k", Style::default().fg(Color::White)),
        Span::styled(":select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("←/→", Style::default().fg(Color::White)),
        Span::styled(":adjust value  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::White)),
        Span::styled(":next  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::White)),
        Span::styled(":back", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[2]);
}

fn render_comparison_config(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let ind_name = app
        .building_indicator
        .as_ref()
        .map(|i| i.display_name())
        .unwrap_or_else(|| "Unknown".to_string());

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Select Comparison ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" for ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &ind_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ]);
    f.render_widget(title, chunks[0]);

    // Comparison options as a scrollable list
    let items: Vec<ListItem> = ComparisonType::all()
        .iter()
        .enumerate()
        .map(|(idx, comp)| {
            let selected = idx == app.comparison_idx;
            let prefix = if selected { "▶ " } else { "  " };

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
                    Span::styled(
                        format!("{} ", comp.symbol()),
                        if selected {
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Cyan)
                        },
                    ),
                    Span::styled(
                        comp.name(),
                        if selected {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        },
                    ),
                ]),
                Line::from(vec![Span::styled(
                    format!("     {}", comp.description()),
                    Style::default().fg(Color::DarkGray),
                )]),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Comparison Type "),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut list_state = ListState::default();
    list_state.select(Some(app.comparison_idx));
    f.render_stateful_widget(list, chunks[1], &mut list_state);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↑/↓", Style::default().fg(Color::White)),
        Span::styled(":navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::White)),
        Span::styled(":select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::White)),
        Span::styled(":back", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[2]);
}

fn render_target_config(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(area);

    let comp_name = app
        .building_comparison
        .map(|c| c.name())
        .unwrap_or("Unknown");

    // Title
    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Set Target Value ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                comp_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ]);
    f.render_widget(title, chunks[0]);

    // Target value configuration - check if we need two values
    let needs_two_values = app
        .building_comparison
        .map(|c| c.needs_range())
        .unwrap_or(false);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            " Current condition: ",
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(""),
    ];

    if let Some(ref ind) = app.building_indicator {
        let preview = if needs_two_values {
            format!(
                "   {:.2} < {} < {:.2}",
                app.target_value,
                ind.display_name(),
                app.target_value2
            )
        } else {
            format!(
                "   {} {} {:.2}",
                ind.display_name(),
                app.building_comparison.map(|c| c.symbol()).unwrap_or("?"),
                app.target_value
            )
        };

        lines.push(Line::from(vec![Span::styled(
            preview,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Primary value display
    let primary_selected = app.editing_target_value;
    let primary_editing = app.target_input_mode && app.editing_target_value;

    let primary_value_display = if primary_editing {
        format!("{}▌", app.target_edit_buffer)
    } else {
        format!("{:.2}", app.target_value)
    };

    lines.push(Line::from(vec![
        Span::styled(
            if primary_selected { " ▶ " } else { "   " },
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(
            "Primary value: ",
            if primary_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
        Span::styled(
            primary_value_display,
            if primary_editing {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if primary_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        ),
    ]));

    if needs_two_values {
        lines.push(Line::from(""));

        let secondary_selected = !app.editing_target_value;
        let secondary_editing = app.target_input_mode && !app.editing_target_value;

        let secondary_value_display = if secondary_editing {
            format!("{}▌", app.target_edit_buffer)
        } else {
            format!("{:.2}", app.target_value2)
        };

        lines.push(Line::from(vec![
            Span::styled(
                if secondary_selected { " ▶ " } else { "   " },
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                "Secondary value: ",
                if secondary_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Span::styled(
                secondary_value_display,
                if secondary_editing {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if secondary_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            ),
        ]));
    }

    // Show typical range if available
    if let Some(ref ind) = app.building_indicator
        && let Some((low, high)) = ind.indicator.typical_range
    {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!(" Typical range: {:.0} to {:.0}", low, high),
            Style::default().fg(Color::DarkGray),
        )]));
    }

    let content = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Target "),
    );
    f.render_widget(content, chunks[1]);

    // Footer - different based on input mode
    let footer = if app.target_input_mode {
        Paragraph::new(vec![Line::from(vec![
            Span::styled(" Type number", Style::default().fg(Color::White)),
            Span::styled(" then ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::styled(":apply  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::White)),
            Span::styled(":cancel", Style::default().fg(Color::DarkGray)),
        ])])
    } else {
        let mut spans = vec![
            Span::styled(" Type number", Style::default().fg(Color::Yellow)),
            Span::styled(" or ", Style::default().fg(Color::DarkGray)),
            Span::styled("←/→", Style::default().fg(Color::White)),
            Span::styled(":±1  ", Style::default().fg(Color::DarkGray)),
            Span::styled("↑/↓", Style::default().fg(Color::White)),
            Span::styled(":±0.1  ", Style::default().fg(Color::DarkGray)),
        ];
        if needs_two_values {
            spans.push(Span::styled("Tab", Style::default().fg(Color::White)));
            spans.push(Span::styled(
                ":switch  ",
                Style::default().fg(Color::DarkGray),
            ));
        }
        spans.push(Span::styled("Enter", Style::default().fg(Color::Green)));
        spans.push(Span::styled(
            ":confirm",
            Style::default().fg(Color::DarkGray),
        ));
        Paragraph::new(vec![Line::from(spans)])
    };
    f.render_widget(footer, chunks[2]);
}

fn render_confirmation(f: &mut Frame, app: &App) {
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
        Span::styled("n/Esc", Style::default().fg(Color::Red)),
        Span::styled(":back  ", Style::default().fg(Color::DarkGray)),
        Span::styled("q", Style::default().fg(Color::White)),
        Span::styled(":quit", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[3]);
}
