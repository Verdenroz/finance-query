use crate::error::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use finance_query::{EdgarFiling, EdgarSearchResults, EdgarSubmissions};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
pub enum AppMode {
    Symbol {
        symbol: String,
        submissions: Box<EdgarSubmissions>,
    },
    Search {
        query: String,
        total_results: usize,
        page_size: usize,
        current_offset: usize,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    SearchInput,
    SymbolInput,
}
pub struct App {
    pub mode: AppMode,
    pub filings: Vec<EdgarFiling>,
    pub unfiltered_filings: Vec<EdgarFiling>, // Store original for re-filtering
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub filter_form: Option<String>,
    pub input_mode: InputMode,
    pub search_input: String,
    pub symbol_input: String,
    pub error_message: Option<String>,
}
impl App {
    pub fn new_empty() -> Self {
        Self {
            mode: AppMode::Search {
                query: String::new(),
                total_results: 0,
                page_size: 100,
                current_offset: 0,
            },
            filings: Vec::new(),
            unfiltered_filings: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            filter_form: None,
            input_mode: InputMode::SearchInput, // Start in search mode
            search_input: String::new(),
            symbol_input: String::new(),
            error_message: None,
        }
    }
    pub fn new_symbol(symbol: String, submissions: EdgarSubmissions) -> Self {
        let filings = submissions
            .filings
            .as_ref()
            .and_then(|f| f.recent.as_ref())
            .map(|recent| recent.to_filings())
            .unwrap_or_default();
        Self {
            mode: AppMode::Symbol {
                symbol,
                submissions: Box::new(submissions),
            },
            unfiltered_filings: filings.clone(),
            filings,
            selected_index: 0,
            scroll_offset: 0,
            filter_form: None,
            input_mode: InputMode::Normal,
            search_input: String::new(),
            symbol_input: String::new(),
            error_message: None,
        }
    }
    pub fn new_search(query: String, search_results: EdgarSearchResults) -> Self {
        let total_results = search_results
            .hits
            .as_ref()
            .and_then(|h| h.total.as_ref())
            .and_then(|t| t.value)
            .unwrap_or(0) as usize;
        // Convert search hits to EdgarFiling
        let filings: Vec<EdgarFiling> = search_results
            .hits
            .as_ref()
            .map(|h| &h.hits)
            .map(|hits| {
                hits.iter()
                    .filter_map(|hit| hit._source.as_ref())
                    .map(|source| {
                        EdgarFiling::new(
                            source.adsh.clone().unwrap_or_default(),
                            source.file_date.clone().unwrap_or_default(),
                            source.period_ending.clone().unwrap_or_default(),
                            String::new(), // acceptance_date_time not available in search
                            source.form.clone().unwrap_or_default(),
                            0,             // size not available in search results
                            false,         // is_xbrl not available
                            false,         // is_inline_xbrl not available
                            String::new(), // primary_document not available
                            source.display_names.first().cloned().unwrap_or_default(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self {
            mode: AppMode::Search {
                query,
                total_results,
                page_size: 100,
                current_offset: 0,
            },
            unfiltered_filings: filings.clone(),
            filings,
            selected_index: 0,
            scroll_offset: 0,
            filter_form: None,
            input_mode: InputMode::Normal,
            search_input: String::new(),
            symbol_input: String::new(),
            error_message: None,
        }
    }
    pub fn next_filing(&mut self) {
        if self.selected_index < self.filings.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }
    pub fn prev_filing(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }
    pub fn page_down(&mut self, page_size: usize) {
        self.selected_index =
            (self.selected_index + page_size).min(self.filings.len().saturating_sub(1));
    }
    pub fn page_up(&mut self, page_size: usize) {
        self.selected_index = self.selected_index.saturating_sub(page_size);
    }
    pub fn jump_to_top(&mut self) {
        self.selected_index = 0;
    }
    pub fn jump_to_bottom(&mut self) {
        self.selected_index = self.filings.len().saturating_sub(1);
    }
    pub fn open_selected_filing(&self) -> Result<()> {
        if let Some(filing) = self.filings.get(self.selected_index) {
            let url = filing.edgar_url();
            open_url(&url)?;
        }
        Ok(())
    }
    pub fn start_search_input(&mut self) {
        self.input_mode = InputMode::SearchInput;
        self.search_input.clear();
    }
    pub fn start_symbol_input(&mut self) {
        self.input_mode = InputMode::SymbolInput;
        self.symbol_input.clear();
    }
    pub fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_input.clear();
        self.symbol_input.clear();
        self.error_message = None;
    }
    pub fn push_input_char(&mut self, c: char) {
        match self.input_mode {
            InputMode::SearchInput => self.search_input.push(c),
            InputMode::SymbolInput => self.symbol_input.push(c.to_uppercase().next().unwrap_or(c)),
            InputMode::Normal => {}
        }
    }
    pub fn pop_input_char(&mut self) {
        match self.input_mode {
            InputMode::SearchInput => {
                self.search_input.pop();
            }
            InputMode::SymbolInput => {
                self.symbol_input.pop();
            }
            InputMode::Normal => {}
        }
    }
    pub fn execute_search_sync(&mut self) -> Result<()> {
        if self.search_input.is_empty() {
            return Ok(());
        }
        // Clear any previous error
        self.error_message = None;
        // Use tokio::task::block_in_place to safely block in async context
        let search_input = self.search_input.clone();
        let results = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                finance_query::edgar::search(&search_input, None, None, None, None, None).await
            })
        })?;
        let total_results = results
            .hits
            .as_ref()
            .and_then(|h| h.total.as_ref())
            .and_then(|t| t.value)
            .unwrap_or(0) as usize;
        // Convert search hits to EdgarFiling
        let filings: Vec<EdgarFiling> = results
            .hits
            .as_ref()
            .map(|h| &h.hits)
            .map(|hits| {
                hits.iter()
                    .filter_map(|hit| hit._source.as_ref())
                    .map(|source| {
                        EdgarFiling::new(
                            source.adsh.clone().unwrap_or_default(),
                            source.file_date.clone().unwrap_or_default(),
                            source.period_ending.clone().unwrap_or_default(),
                            String::new(),
                            source.form.clone().unwrap_or_default(),
                            0,
                            false,
                            false,
                            String::new(),
                            source.display_names.first().cloned().unwrap_or_default(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        // Update app state
        self.mode = AppMode::Search {
            query: self.search_input.clone(),
            total_results,
            page_size: 100,
            current_offset: 0,
        };
        self.unfiltered_filings = filings.clone();
        self.filings = filings;
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.filter_form = None; // Reset any active filter
        self.input_mode = InputMode::Normal;
        self.search_input.clear();
        Ok(())
    }
    pub fn execute_symbol_lookup(&mut self) -> Result<()> {
        if self.symbol_input.is_empty() {
            return Ok(());
        }
        // Clear any previous error
        self.error_message = None;
        // Use tokio::task::block_in_place to safely block in async context
        let symbol = self.symbol_input.clone().to_uppercase();
        let symbol_for_closure = symbol.clone();
        let (_cik, submissions) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let cik = finance_query::edgar::resolve_cik(&symbol_for_closure)
                    .await
                    .map_err(crate::error::CliError::FinanceQuery)?;
                let submissions = finance_query::edgar::submissions(cik)
                    .await
                    .map_err(crate::error::CliError::FinanceQuery)?;
                Ok::<_, crate::error::CliError>((cik, submissions))
            })
        })?;
        // Convert to filings
        let filings: Vec<EdgarFiling> = submissions
            .filings
            .as_ref()
            .and_then(|f| f.recent.as_ref())
            .map(|recent| recent.to_filings())
            .unwrap_or_default();
        // Update app state to symbol mode
        self.mode = AppMode::Symbol {
            symbol,
            submissions: Box::new(submissions),
        };
        self.unfiltered_filings = filings.clone();
        self.filings = filings;
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.filter_form = None; // Reset any active filter
        self.input_mode = InputMode::Normal;
        self.symbol_input.clear();
        Ok(())
    }
    pub fn apply_filter(&mut self, form_type: Option<String>) {
        self.filter_form = form_type.clone();
        // Always filter from unfiltered_filings to avoid re-filtering filtered results
        self.filings = if let Some(filter) = &form_type {
            self.unfiltered_filings
                .iter()
                .filter(|f| f.form.to_uppercase().contains(&filter.to_uppercase()))
                .cloned()
                .collect()
        } else {
            self.unfiltered_filings.clone()
        };
        self.selected_index = 0;
        self.scroll_offset = 0;
    }
    pub fn load_next_page(&mut self) -> Result<()> {
        let (query, new_offset) = if let AppMode::Search {
            query,
            total_results,
            page_size,
            current_offset,
        } = &self.mode
        {
            let new_offset = current_offset + page_size;
            if new_offset >= *total_results {
                // Already on last page
                return Ok(());
            }
            (query.clone(), new_offset)
        } else {
            return Ok(());
        };
        self.load_page(query, new_offset)?;
        Ok(())
    }
    pub fn load_prev_page(&mut self) -> Result<()> {
        let (query, new_offset) = if let AppMode::Search {
            query,
            current_offset,
            page_size,
            ..
        } = &self.mode
        {
            if *current_offset == 0 {
                // Already on first page
                return Ok(());
            }
            let new_offset = current_offset.saturating_sub(*page_size);
            (query.clone(), new_offset)
        } else {
            return Ok(());
        };
        self.load_page(query, new_offset)?;
        Ok(())
    }
    fn load_page(&mut self, query: String, offset: usize) -> Result<()> {
        // Clear any previous error
        self.error_message = None;
        let query_for_closure = query.clone();
        let results = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                finance_query::edgar::search(
                    &query_for_closure,
                    None,
                    None,
                    None,
                    Some(offset),
                    Some(100),
                )
                .await
            })
        })?;
        let total_results = results
            .hits
            .as_ref()
            .and_then(|h| h.total.as_ref())
            .and_then(|t| t.value)
            .unwrap_or(0) as usize;
        // Convert search hits to EdgarFiling
        let filings: Vec<EdgarFiling> = results
            .hits
            .as_ref()
            .map(|h| &h.hits)
            .map(|hits| {
                hits.iter()
                    .filter_map(|hit| hit._source.as_ref())
                    .map(|source| {
                        EdgarFiling::new(
                            source.adsh.clone().unwrap_or_default(),
                            source.file_date.clone().unwrap_or_default(),
                            source.period_ending.clone().unwrap_or_default(),
                            String::new(),
                            source.form.clone().unwrap_or_default(),
                            0,
                            false,
                            false,
                            String::new(),
                            source.display_names.first().cloned().unwrap_or_default(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        // Update app state
        self.mode = AppMode::Search {
            query,
            total_results,
            page_size: 100,
            current_offset: offset,
        };
        self.unfiltered_filings = filings.clone();
        self.filings = filings;
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.filter_form = None; // Reset any active filter
        Ok(())
    }
}
pub fn run_symbol(symbol: String, submissions: EdgarSubmissions) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Create app state
    let mut app = App::new_symbol(symbol, submissions);
    // Cleanup helper
    let cleanup = || -> Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    };
    // Main loop
    let result = run_app(&mut terminal, &mut app);
    // Always cleanup terminal
    cleanup()?;
    result
}
pub fn run_search(query: String, search_results: EdgarSearchResults) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Create app state
    let mut app = App::new_search(query, search_results);
    // Cleanup helper
    let cleanup = || -> Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    };
    // Main loop
    let result = run_app(&mut terminal, &mut app);
    // Always cleanup terminal
    cleanup()?;
    result
}
pub fn run_empty() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Create app state - starts in search input mode
    let mut app = App::new_empty();
    // Cleanup helper
    let cleanup = || -> Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    };
    // Main loop
    let result = run_app(&mut terminal, &mut app);
    // Always cleanup terminal
    cleanup()?;
    result
}
fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal
            .draw(|f| {
                super::render::render_ui(f, app);
            })
            .map_err(crate::error::CliError::Io)?;
        // Handle input with polling
        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match app.input_mode {
                InputMode::Normal => {
                    // If there's an error, any key dismisses it
                    if app.error_message.is_some() {
                        app.error_message = None;
                        continue;
                    }
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break;
                        }
                        KeyCode::Up | KeyCode::Char('k') => app.prev_filing(),
                        KeyCode::Down | KeyCode::Char('j') => app.next_filing(),
                        KeyCode::PageDown | KeyCode::Char('d')
                            if key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            app.page_down(10);
                        }
                        KeyCode::PageUp | KeyCode::Char('u')
                            if key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            app.page_up(10);
                        }
                        KeyCode::Home | KeyCode::Char('g') => app.jump_to_top(),
                        KeyCode::End | KeyCode::Char('G') => app.jump_to_bottom(),
                        KeyCode::Enter | KeyCode::Char('o') => {
                            // Disable raw mode temporarily
                            disable_raw_mode()?;
                            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                            // Open URL
                            let _ = app.open_selected_filing();
                            // Re-enable raw mode
                            execute!(io::stdout(), EnterAlternateScreen)?;
                            enable_raw_mode()?;
                            terminal.clear().map_err(crate::error::CliError::Io)?;
                        }
                        KeyCode::Char('f') => {
                            // Cycle through all form type filters
                            let next_filter = match app.filter_form.as_deref() {
                                None => Some("10-K".to_string()),
                                Some("10-K") => Some("10-Q".to_string()),
                                Some("10-Q") => Some("8-K".to_string()),
                                Some("8-K") => Some("4".to_string()),
                                Some("4") => Some("S-1".to_string()),
                                Some("S-1") => Some("DEF 14A".to_string()),
                                Some("DEF 14A") => Some("10-K/A".to_string()),
                                Some("10-K/A") => Some("10-Q/A".to_string()),
                                Some("10-Q/A") => Some("S-3".to_string()),
                                Some("S-3") => Some("20-F".to_string()),
                                Some("20-F") => None, // Back to all
                                _ => None,
                            };
                            app.apply_filter(next_filter);
                        }
                        KeyCode::Char('r') => {
                            // Reset filter
                            app.apply_filter(None);
                        }
                        KeyCode::Char('/') => {
                            // Enter full-text search mode
                            app.start_search_input();
                        }
                        KeyCode::Char('s') => {
                            // Enter symbol lookup mode
                            app.start_symbol_input();
                        }
                        KeyCode::Char('n') | KeyCode::Right => {
                            // Next page (only in search mode)
                            if matches!(app.mode, AppMode::Search { .. })
                                && let Err(e) = app.load_next_page()
                            {
                                app.error_message =
                                    Some(format!("Failed to load next page: {}", e));
                            }
                        }
                        KeyCode::Char('p') | KeyCode::Left => {
                            // Previous page (only in search mode)
                            if matches!(app.mode, AppMode::Search { .. })
                                && let Err(e) = app.load_prev_page()
                            {
                                app.error_message =
                                    Some(format!("Failed to load previous page: {}", e));
                            }
                        }
                        _ => {}
                    }
                }
                InputMode::SearchInput => match key.code {
                    KeyCode::Esc => {
                        app.cancel_input();
                    }
                    KeyCode::Enter => {
                        // Execute search synchronously
                        if let Err(e) = app.execute_search_sync() {
                            // Store error message for display
                            app.error_message = Some(format!("Search failed: {}", e));
                            app.input_mode = InputMode::Normal;
                            app.search_input.clear();
                        }
                    }
                    KeyCode::Backspace => {
                        app.pop_input_char();
                    }
                    KeyCode::Char(c) => {
                        app.push_input_char(c);
                    }
                    _ => {}
                },
                InputMode::SymbolInput => match key.code {
                    KeyCode::Esc => {
                        app.cancel_input();
                    }
                    KeyCode::Enter => {
                        // Execute symbol lookup synchronously
                        if let Err(e) = app.execute_symbol_lookup() {
                            // Store error message for display
                            app.error_message = Some(format!("Symbol lookup failed: {}", e));
                            app.input_mode = InputMode::Normal;
                            app.symbol_input.clear();
                        }
                    }
                    KeyCode::Backspace => {
                        app.pop_input_char();
                    }
                    KeyCode::Char(c) => {
                        app.push_input_char(c);
                    }
                    _ => {}
                },
            }
        }
    }
    Ok(())
}
fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(url).spawn()?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("cmd")
        .args(["/C", "start", url])
        .spawn()?;
    Ok(())
}
