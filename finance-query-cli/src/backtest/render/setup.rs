use super::super::state::{App, ConfigField};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

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

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {:<16}", field.name()),
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
