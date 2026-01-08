use crate::error::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use finance_query::{OptionContract, Ticker};
use ratatui::{Terminal, backend::CrosstermBackend, widgets::TableState};
use std::io;
use tokio::time::Duration;

use super::{input::handle_key_event, render::ui};

/// View mode for the options chain
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    /// Show both calls and puts side by side (straddle view)
    Straddle,
    /// Show only calls
    CallsOnly,
    /// Show only puts
    PutsOnly,
}

impl ViewMode {
    pub fn cycle(&self) -> Self {
        match self {
            ViewMode::Straddle => ViewMode::CallsOnly,
            ViewMode::CallsOnly => ViewMode::PutsOnly,
            ViewMode::PutsOnly => ViewMode::Straddle,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ViewMode::Straddle => "Straddle",
            ViewMode::CallsOnly => "Calls Only",
            ViewMode::PutsOnly => "Puts Only",
        }
    }
}

/// Sort field for options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortField {
    Strike,
    LastPrice,
    Volume,
    OpenInterest,
    ImpliedVolatility,
    Change,
}

impl SortField {
    pub fn cycle(&self) -> Self {
        match self {
            SortField::Strike => SortField::LastPrice,
            SortField::LastPrice => SortField::Volume,
            SortField::Volume => SortField::OpenInterest,
            SortField::OpenInterest => SortField::ImpliedVolatility,
            SortField::ImpliedVolatility => SortField::Change,
            SortField::Change => SortField::Strike,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SortField::Strike => "Strike",
            SortField::LastPrice => "Price",
            SortField::Volume => "Volume",
            SortField::OpenInterest => "OI",
            SortField::ImpliedVolatility => "IV",
            SortField::Change => "Change",
        }
    }
}

/// Application state for options TUI
pub struct OptionsApp {
    pub symbol: String,
    pub underlying_price: Option<f64>,
    pub expiration_dates: Vec<i64>,
    pub selected_expiration_idx: usize,
    pub calls: Vec<OptionContract>,
    pub puts: Vec<OptionContract>,
    pub view_mode: ViewMode,
    pub sort_field: SortField,
    pub sort_ascending: bool,
    pub calls_table_state: TableState,
    pub puts_table_state: TableState,
    /// Which table has focus in straddle view (true = calls, false = puts)
    pub calls_focused: bool,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub should_quit: bool,
    pub show_help: bool,
    /// Filter: show only ITM options
    pub filter_itm_only: bool,
    /// Filter: minimum volume
    pub filter_min_volume: Option<i64>,
}

impl OptionsApp {
    pub async fn new(symbol: &str) -> Result<Self> {
        let mut app = Self {
            symbol: symbol.to_uppercase(),
            underlying_price: None,
            expiration_dates: Vec::new(),
            selected_expiration_idx: 0,
            calls: Vec::new(),
            puts: Vec::new(),
            view_mode: ViewMode::Straddle,
            sort_field: SortField::Strike,
            sort_ascending: true,
            calls_table_state: TableState::default(),
            puts_table_state: TableState::default(),
            calls_focused: true,
            is_loading: true,
            error_message: None,
            should_quit: false,
            show_help: false,
            filter_itm_only: false,
            filter_min_volume: None,
        };

        // Load initial data
        app.load_expirations().await?;

        Ok(app)
    }

    pub async fn load_expirations(&mut self) -> Result<()> {
        self.is_loading = true;
        self.error_message = None;

        match Ticker::new(&self.symbol).await {
            Ok(ticker) => {
                // Get quote for underlying price
                if let Ok(quote) = ticker.quote(false).await {
                    self.underlying_price = quote.regular_market_price.as_ref().and_then(|v| v.raw);
                }

                // Get options expirations
                match ticker.options(None).await {
                    Ok(options) => {
                        self.expiration_dates = options.expiration_dates();
                        if !self.expiration_dates.is_empty() {
                            // Load the first expiration
                            self.load_chain_for_expiration().await?;
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load options: {}", e));
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Invalid symbol: {}", e));
            }
        }

        self.is_loading = false;
        Ok(())
    }

    pub async fn load_chain_for_expiration(&mut self) -> Result<()> {
        if self.expiration_dates.is_empty() {
            return Ok(());
        }

        self.is_loading = true;
        let exp_date = self.expiration_dates[self.selected_expiration_idx];

        match Ticker::new(&self.symbol).await {
            Ok(ticker) => match ticker.options(Some(exp_date)).await {
                Ok(options) => {
                    self.calls = options.calls().0;
                    self.puts = options.puts().0;
                    self.sort_contracts();

                    // Reset table selection
                    if !self.calls.is_empty() {
                        self.calls_table_state.select(Some(0));
                    }
                    if !self.puts.is_empty() {
                        self.puts_table_state.select(Some(0));
                    }

                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load chain: {}", e));
                }
            },
            Err(e) => {
                self.error_message = Some(format!("Error: {}", e));
            }
        }

        self.is_loading = false;
        Ok(())
    }

    pub fn sort_contracts(&mut self) {
        let sorter = |a: &OptionContract, b: &OptionContract| -> std::cmp::Ordering {
            let cmp = match self.sort_field {
                SortField::Strike => a
                    .strike
                    .partial_cmp(&b.strike)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortField::LastPrice => {
                    let a_price = a.last_price.unwrap_or(0.0);
                    let b_price = b.last_price.unwrap_or(0.0);
                    a_price
                        .partial_cmp(&b_price)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
                SortField::Volume => {
                    let a_vol = a.volume.unwrap_or(0);
                    let b_vol = b.volume.unwrap_or(0);
                    a_vol.cmp(&b_vol)
                }
                SortField::OpenInterest => {
                    let a_oi = a.open_interest.unwrap_or(0);
                    let b_oi = b.open_interest.unwrap_or(0);
                    a_oi.cmp(&b_oi)
                }
                SortField::ImpliedVolatility => {
                    let a_iv = a.implied_volatility.unwrap_or(0.0);
                    let b_iv = b.implied_volatility.unwrap_or(0.0);
                    a_iv.partial_cmp(&b_iv).unwrap_or(std::cmp::Ordering::Equal)
                }
                SortField::Change => {
                    let a_chg = a.percent_change.unwrap_or(0.0);
                    let b_chg = b.percent_change.unwrap_or(0.0);
                    a_chg
                        .partial_cmp(&b_chg)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            };
            if self.sort_ascending {
                cmp
            } else {
                cmp.reverse()
            }
        };

        self.calls.sort_by(sorter);
        self.puts.sort_by(sorter);
    }

    /// Get filtered calls based on current filters
    pub fn filtered_calls(&self) -> Vec<&OptionContract> {
        self.calls
            .iter()
            .filter(|c| self.passes_filter(c))
            .collect()
    }

    /// Get filtered puts based on current filters
    pub fn filtered_puts(&self) -> Vec<&OptionContract> {
        self.puts.iter().filter(|c| self.passes_filter(c)).collect()
    }

    fn passes_filter(&self, contract: &OptionContract) -> bool {
        // ITM filter
        if self.filter_itm_only
            && let Some(itm) = contract.in_the_money
            && !itm
        {
            return false;
        }

        // Volume filter
        if let Some(min_vol) = self.filter_min_volume {
            let vol = contract.volume.unwrap_or(0);
            if vol < min_vol {
                return false;
            }
        }

        true
    }

    pub fn next_expiration(&mut self) {
        if !self.expiration_dates.is_empty() {
            self.selected_expiration_idx =
                (self.selected_expiration_idx + 1) % self.expiration_dates.len();
        }
    }

    pub fn prev_expiration(&mut self) {
        if !self.expiration_dates.is_empty() {
            if self.selected_expiration_idx == 0 {
                self.selected_expiration_idx = self.expiration_dates.len() - 1;
            } else {
                self.selected_expiration_idx -= 1;
            }
        }
    }

    pub fn move_selection_down(&mut self) {
        match self.view_mode {
            ViewMode::Straddle => {
                if self.calls_focused {
                    let len = self.filtered_calls().len();
                    if len > 0 {
                        let i = self.calls_table_state.selected().unwrap_or(0);
                        self.calls_table_state.select(Some((i + 1).min(len - 1)));
                    }
                } else {
                    let len = self.filtered_puts().len();
                    if len > 0 {
                        let i = self.puts_table_state.selected().unwrap_or(0);
                        self.puts_table_state.select(Some((i + 1).min(len - 1)));
                    }
                }
            }
            ViewMode::CallsOnly => {
                let len = self.filtered_calls().len();
                if len > 0 {
                    let i = self.calls_table_state.selected().unwrap_or(0);
                    self.calls_table_state.select(Some((i + 1).min(len - 1)));
                }
            }
            ViewMode::PutsOnly => {
                let len = self.filtered_puts().len();
                if len > 0 {
                    let i = self.puts_table_state.selected().unwrap_or(0);
                    self.puts_table_state.select(Some((i + 1).min(len - 1)));
                }
            }
        }
    }

    pub fn move_selection_up(&mut self) {
        match self.view_mode {
            ViewMode::Straddle => {
                if self.calls_focused {
                    let i = self.calls_table_state.selected().unwrap_or(0);
                    self.calls_table_state.select(Some(i.saturating_sub(1)));
                } else {
                    let i = self.puts_table_state.selected().unwrap_or(0);
                    self.puts_table_state.select(Some(i.saturating_sub(1)));
                }
            }
            ViewMode::CallsOnly => {
                let i = self.calls_table_state.selected().unwrap_or(0);
                self.calls_table_state.select(Some(i.saturating_sub(1)));
            }
            ViewMode::PutsOnly => {
                let i = self.puts_table_state.selected().unwrap_or(0);
                self.puts_table_state.select(Some(i.saturating_sub(1)));
            }
        }
    }

    pub fn toggle_focus(&mut self) {
        if self.view_mode == ViewMode::Straddle {
            self.calls_focused = !self.calls_focused;
        }
    }
}

pub async fn run_options_tui(symbol: &str) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = OptionsApp::new(symbol).await?;

    let result = run_event_loop(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut OptionsApp,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if app.should_quit {
            break;
        }

        // Handle keyboard input with a small poll timeout
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                while event::poll(Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        handle_key_event(app, key).await?;
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
