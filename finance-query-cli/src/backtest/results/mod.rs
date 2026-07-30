mod charts;
mod comparison;
mod distribution;
mod export;
mod format;
mod monte_carlo;
mod optimizer;
mod overview;
mod periods;
mod portfolio;
mod signals;
mod trades;

use charts::render_charts;
use comparison::render_comparison;
use distribution::render_distribution;
use export::{export_portfolio_csv, export_trades_csv};
use format::return_color;
use monte_carlo::{next_mc_method, render_monte_carlo};
use optimizer::{next_optimize_metric, render_optimizer_results, render_walk_forward_results};
use overview::render_results_overview;
use periods::render_periods;
use portfolio::render_portfolio;
use signals::render_results_signals;
use trades::render_results_trades;

use finance_query::backtesting::portfolio::PortfolioResult;
use finance_query::backtesting::{
    BacktestResult, MonteCarloConfig, MonteCarloMethod, MonteCarloResult, OptimizationReport,
    OptimizeMetric, WalkForwardReport,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};
use std::path::PathBuf;

/// The full result of a run (backtest + optional optimizer/walk-forward outputs).
pub struct RunResult {
    pub backtest: BacktestResult,
    pub optimization: Option<OptimizationReport>,
    pub walk_forward: Option<WalkForwardReport>,
    pub opt_metric: Option<OptimizeMetric>,
    /// Benchmark candles for plotting the actual buy-and-hold equity curve.
    /// When present, the Charts tab uses the real curve instead of a linear
    /// interpolation of the benchmark's total return.
    pub bench_candles: Option<Vec<finance_query::Candle>>,
    /// Portfolio result (set when portfolio mode was used). The `backtest`
    /// field holds the primary symbol's result for single-symbol tabs.
    pub portfolio: Option<PortfolioResult>,
}

impl RunResult {
    pub fn simple(result: BacktestResult) -> Self {
        Self {
            backtest: result,
            optimization: None,
            walk_forward: None,
            opt_metric: None,
            bench_candles: None,
            portfolio: None,
        }
    }
}

/// Results viewer tabs
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ResultsTab {
    #[default]
    Overview,
    Charts,
    Distribution,
    Trades,
    Signals,
    MonteCarlo,
    Periods,
    Comparison,
    Optimizer,
    WalkForward,
    /// Portfolio tab — only shown when a portfolio result is present.
    Portfolio,
}

impl ResultsTab {
    pub fn all_for(
        has_optimizer: bool,
        has_walk_forward: bool,
        has_comparison: bool,
        has_portfolio: bool,
    ) -> Vec<Self> {
        let mut tabs = vec![
            Self::Overview,
            Self::Charts,
            Self::Distribution,
            Self::Trades,
            Self::Signals,
            Self::MonteCarlo,
            Self::Periods,
        ];
        if has_comparison {
            tabs.push(Self::Comparison);
        }
        if has_optimizer {
            tabs.push(Self::Optimizer);
        }
        if has_walk_forward {
            tabs.push(Self::WalkForward);
        }
        if has_portfolio {
            tabs.push(Self::Portfolio);
        }
        tabs
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Charts => "Charts",
            Self::Distribution => "Distribution",
            Self::Trades => "Trades",
            Self::Signals => "Signals",
            Self::MonteCarlo => "Monte Carlo",
            Self::Periods => "Periods",
            Self::Comparison => "Comparison",
            Self::Optimizer => "Optimizer",
            Self::WalkForward => "Walk-Forward",
            Self::Portfolio => "Portfolio",
        }
    }
}

/// Bottom pane of the Charts tab.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartsView {
    #[default]
    Drawdown,
    RollingSharpe,
    RollingWinRate,
}

impl ChartsView {
    fn cycle(self) -> Self {
        match self {
            Self::Drawdown => Self::RollingSharpe,
            Self::RollingSharpe => Self::RollingWinRate,
            Self::RollingWinRate => Self::Drawdown,
        }
    }
}

/// Which period breakdown to display in the Periods tab.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum PeriodsMode {
    #[default]
    Yearly,
    Monthly,
    DayOfWeek,
}

impl PeriodsMode {
    fn cycle(self) -> Self {
        match self {
            Self::Yearly => Self::Monthly,
            Self::Monthly => Self::DayOfWeek,
            Self::DayOfWeek => Self::Yearly,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Yearly => "Yearly",
            Self::Monthly => "Monthly",
            Self::DayOfWeek => "Day of Week",
        }
    }
}

/// Actions from the results TUI
pub enum ResultsAction {
    Quit,
    Retry,
    NewStrategy,
}

/// Results TUI state
pub struct ResultsApp {
    pub result: RunResult,
    pub monte_carlo: MonteCarloResult,
    pub tab: ResultsTab,
    pub scroll: usize,
    /// Which breakdown to show in the Periods tab.
    pub periods_mode: PeriodsMode,
    /// Which chart to show in the bottom pane of the Charts tab.
    pub charts_view: ChartsView,
    /// Saved results for comparison (label, result). Capped at 5.
    pub saved_results: Vec<(String, BacktestResult)>,
    /// Metric used to rank in the Comparison tab.
    pub comparison_metric: OptimizeMetric,
    /// Monte Carlo resampling method.
    pub mc_method: MonteCarloMethod,
    /// Whether the diagnostics banner is visible (dismissed with d).
    pub show_diagnostics: bool,
    /// Status message shown after export (Some = path exported to, None = idle)
    pub export_status: Option<Result<PathBuf, String>>,
}

impl ResultsApp {
    pub fn new(result: RunResult) -> Self {
        let monte_carlo = MonteCarloConfig::default().run(&result.backtest);
        // Default to Portfolio tab when portfolio mode was used so the user sees
        // the aggregate view immediately rather than a single-symbol overview.
        let tab = if result.portfolio.is_some() {
            ResultsTab::Portfolio
        } else {
            ResultsTab::default()
        };
        Self {
            result,
            monte_carlo,
            tab,
            scroll: 0,
            periods_mode: PeriodsMode::default(),
            charts_view: ChartsView::default(),
            saved_results: Vec::new(),
            comparison_metric: OptimizeMetric::SharpeRatio,
            mc_method: MonteCarloMethod::IidShuffle,
            show_diagnostics: true,
            export_status: None,
        }
    }

    pub fn export_csv(&mut self) {
        let path = if let Some(portfolio) = &self.result.portfolio {
            export_portfolio_csv(portfolio)
        } else {
            export_trades_csv(&self.result.backtest)
        };
        self.export_status = Some(path);
    }

    fn tabs(&self) -> Vec<ResultsTab> {
        ResultsTab::all_for(
            self.result.optimization.is_some(),
            self.result.walk_forward.is_some(),
            self.saved_results.len() >= 2,
            self.result.portfolio.is_some(),
        )
    }

    pub fn next_tab(&mut self) {
        let tabs = self.tabs();
        let idx = tabs.iter().position(|t| *t == self.tab).unwrap_or(0);
        self.tab = tabs[(idx + 1) % tabs.len()];
        self.scroll = 0;
    }

    pub fn prev_tab(&mut self) {
        let tabs = self.tabs();
        let idx = tabs.iter().position(|t| *t == self.tab).unwrap_or(0);
        self.tab = tabs[(idx + tabs.len() - 1) % tabs.len()];
        self.scroll = 0;
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }
}

/// Run the results TUI
pub fn run_results_tui(result: RunResult) -> crate::error::Result<ResultsAction> {
    use crossterm::{
        ExecutableCommand,
        event::{self, Event, KeyCode, KeyEventKind},
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use ratatui::Terminal;
    use ratatui::backend::CrosstermBackend;
    use std::io::stdout;

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = ResultsApp::new(result);
    let action = loop {
        terminal.draw(|f| results_ui(f, &app))?;

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') => break ResultsAction::Quit,
                KeyCode::Char('r') => break ResultsAction::Retry,
                KeyCode::Char('n') => break ResultsAction::NewStrategy,
                KeyCode::Char('e') => app.export_csv(),
                KeyCode::Char('d') => app.show_diagnostics = !app.show_diagnostics,
                KeyCode::Char('c')
                    // Comparison is per-symbol only; skip silently in portfolio mode
                    // to avoid storing a single-symbol result that misrepresents the portfolio.
                    if app.result.portfolio.is_none() && app.saved_results.len() < 5 => {
                        let n = app.saved_results.len() + 1;
                        let label = format!("Run {} — {}", n, app.result.backtest.strategy_name);
                        app.saved_results.push((label, app.result.backtest.clone()));
                    }
                KeyCode::Char('x') => app.saved_results.clear(),
                KeyCode::Char('m') => {
                    // Cycle period breakdown or comparison metric depending on active tab
                    match app.tab {
                        ResultsTab::Periods => {
                            app.periods_mode = app.periods_mode.cycle();
                        }
                        ResultsTab::Comparison => {
                            app.comparison_metric = next_optimize_metric(app.comparison_metric);
                        }
                        _ => {}
                    }
                }
                KeyCode::Char('s') => app.charts_view = app.charts_view.cycle(),
                KeyCode::Char('v') => {
                    app.mc_method = next_mc_method(&app.mc_method);
                    app.monte_carlo = MonteCarloConfig::default()
                        .method(app.mc_method.clone())
                        .run(&app.result.backtest);
                }
                KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => app.next_tab(),
                KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => app.prev_tab(),
                KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                KeyCode::Enter => break ResultsAction::NewStrategy,
                KeyCode::Esc => break ResultsAction::Quit,
                _ => {}
            }
        }
    };

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(action)
}

/// Main results UI function
pub fn results_ui(f: &mut Frame, app: &ResultsApp) {
    let area = f.area();

    let diagnostics = &app.result.backtest.diagnostics;
    let diag_height = if app.show_diagnostics && !diagnostics.is_empty() {
        diagnostics.len().min(3) as u16 + 2
    } else {
        0
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Length(diag_height),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    render_results_header(f, app, chunks[0]);
    render_results_tabs(f, app, chunks[1]);

    if diag_height > 0 {
        render_diagnostics_banner(f, app, chunks[2]);
    }

    match app.tab {
        ResultsTab::Overview => render_results_overview(f, app, chunks[3]),
        ResultsTab::Charts => render_charts(f, app, chunks[3]),
        ResultsTab::Distribution => render_distribution(f, app, chunks[3]),
        ResultsTab::Trades => render_results_trades(f, app, chunks[3]),
        ResultsTab::Signals => render_results_signals(f, app, chunks[3]),
        ResultsTab::MonteCarlo => render_monte_carlo(f, app, chunks[3]),
        ResultsTab::Periods => render_periods(f, app, chunks[3]),
        ResultsTab::Comparison => render_comparison(f, app, chunks[3]),
        ResultsTab::Optimizer => render_optimizer_results(f, app, chunks[3]),
        ResultsTab::WalkForward => render_walk_forward_results(f, app, chunks[3]),
        ResultsTab::Portfolio => render_portfolio(f, app, chunks[3]),
    }

    render_results_footer(f, app, chunks[4]);
}

fn render_diagnostics_banner(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let diags = &app.result.backtest.diagnostics;
    let lines: Vec<Line> = diags
        .iter()
        .take(3)
        .map(|msg| {
            Line::from(vec![
                Span::styled(" ⚠ ", Style::default().fg(Color::Yellow)),
                Span::styled(msg.as_str(), Style::default().fg(Color::Yellow)),
            ])
        })
        .collect();

    let banner = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .title(" ⚠ Diagnostics — press d to dismiss "),
    );
    f.render_widget(banner, area);
}

fn render_results_header(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let r = &app.result.backtest;
    let pnl = r.total_pnl();
    let pnl_color = return_color(pnl);

    let title = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " BACKTEST RESULTS ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(&r.symbol, Style::default().fg(Color::Yellow)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(&r.strategy_name, Style::default().fg(Color::White)),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled("P&L: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}{:.2}", if pnl >= 0.0 { "+" } else { "" }, pnl),
                Style::default().fg(pnl_color).add_modifier(Modifier::BOLD),
            ),
        ]),
    ]);
    f.render_widget(title, area);
}

fn render_results_tabs(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let tabs_list = app.tabs();
    let tab_titles: Vec<Line> = tabs_list.iter().map(|t| Line::from(t.name())).collect();
    let idx = tabs_list.iter().position(|t| *t == app.tab).unwrap_or(0);

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(idx)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );
    f.render_widget(tabs, area);
}

fn render_results_footer(f: &mut Frame, app: &ResultsApp, area: ratatui::layout::Rect) {
    let status_line = match &app.export_status {
        Some(Ok(path)) => Line::from(vec![
            Span::styled(" Exported: ", Style::default().fg(Color::Green)),
            Span::styled(
                path.display().to_string(),
                Style::default().fg(Color::White),
            ),
        ]),
        Some(Err(e)) => Line::from(vec![Span::styled(
            format!(" Export failed: {}", e),
            Style::default().fg(Color::Red),
        )]),
        None => Line::from(vec![
            Span::styled(" ←/→", Style::default().fg(Color::White)),
            Span::styled(":tab  ", Style::default().fg(Color::DarkGray)),
            Span::styled("↑/↓", Style::default().fg(Color::White)),
            Span::styled(":scroll  ", Style::default().fg(Color::DarkGray)),
            Span::styled("e", Style::default().fg(Color::Green)),
            Span::styled(":export csv  ", Style::default().fg(Color::DarkGray)),
            Span::styled("r", Style::default().fg(Color::Yellow)),
            Span::styled(":retry  ", Style::default().fg(Color::DarkGray)),
            Span::styled("n", Style::default().fg(Color::Cyan)),
            Span::styled(":new strategy  ", Style::default().fg(Color::DarkGray)),
            Span::styled("q", Style::default().fg(Color::White)),
            Span::styled(":quit", Style::default().fg(Color::DarkGray)),
        ]),
    };
    f.render_widget(Paragraph::new(status_line), area);
}
