//! # fq - Finance Query CLI
//!
//! Command-line interface for querying financial data
//!
//! Built on the `finance-query` Rust library, `fq` provides access to:
//! - Real-time and historical stock quotes
//! - Technical indicators and backtesting
//! - Live price streaming via WebSocket
//! - Interactive TUI dashboards and options chains
//! - Company fundamentals, news, and analyst data
//! - Market indices, sectors, and screening
//! - Price alerts and portfolio tracking
//!
//! ## Installation
//!
//! ### From crates.io
//!
//! ```bash
//! cargo install finance-query-cli
//! ```
//!
//! ### From source
//!
//! ```bash
//! git clone https://github.com/Verdenroz/finance-query
//! cd finance-query/finance-query-cli
//! cargo install --path .
//! ```
//!
//! ### Using cargo-binstall
//!
//! ```bash
//! cargo binstall finance-query-cli
//! ```
//!
//! ## Quick Start
//!
//! ```bash
//! # Get a stock quote
//! fq quote AAPL
//!
//! # Get multiple quotes
//! fq quote AAPL MSFT GOOGL
//!
//! # Stream live prices
//! fq stream AAPL TSLA
//!
//! # View historical chart
//! fq chart AAPL --interval 1d --range 1mo
//!
//! # Calculate RSI indicator (TUI)
//! fq indicator AAPL
//!
//! # Backtest a strategy (TUI)
//! fq backtest AAPL
//!
//! # Launch interactive dashboard (TUI)
//! fq dashboard
//!
//! # Explore options chain (TUI)
//! fq options AAPL
//! ```
//!
//! ## Interactive TUI Features
//!
//! Several commands provide interactive terminal user interfaces:
//! - **Dashboard** (`fq dashboard`) - Real-time market watchlist with live updates
//! - **Options Chain** (`fq options SYMBOL`) - Navigate options with Greeks and analytics
//! - **Backtesting** (`fq backtest SYMBOL`) - Visualize strategy performance
//! - **Indicators** (`fq indicator SYMBOL`) - Interactive technical analysis
//!
//! See [screenshots](https://github.com/Verdenroz/finance-query/tree/main/finance-query-cli/assets/screenshots)
//! for examples of the TUI interfaces.
//!
//! ## Output Formats
//!
//! All commands support multiple output formats:
//!
//! ```bash
//! fq quote AAPL --format json    # JSON output
//! fq quote AAPL --format csv     # CSV output
//! fq quote AAPL --format table   # Pretty table (default)
//! ```
//!
//! ## Key Features
//!
//! ### Real-time Data
//! - **Quotes**: Current prices, volume, market cap, P/E ratio
//! - **Streaming**: WebSocket-based live price updates
//! - **Dashboard**: Interactive TUI with real-time watchlist
//!
//! ### Technical Analysis
//! - **40+ Indicators**: SMA, EMA, RSI, MACD, Bollinger Bands, and more
//! - **Backtesting**: Test strategies with pre-built or custom logic
//! - **Chart Data**: OHLCV candles with customizable intervals
//!
//! ### Fundamental Data
//! - **Financials**: Income statements, balance sheets, cash flow
//! - **Profile**: Company info, sector, industry, executives
//! - **Filings**: SEC documents (10-K, 10-Q, 8-K)
//! - **Earnings**: Dates, estimates, history, call transcripts
//!
//! ### Market Intelligence
//! - **Screeners**: Most active, gainers, losers, trending
//! - **Sectors**: Performance tracking and top companies
//! - **Indices**: Global market indices quotes
//! - **News**: Latest company and market news
//!
//! ### Portfolio Management
//! - **Alerts**: Price-based notifications
//! - **Watchlists**: Track your favorite symbols
//! - **Options**: Interactive chain explorer with Greeks
//!
//! ## Commands
//!
//! Run `fq --help` to see all available commands, or `fq <command> --help` for
//! detailed usage information on a specific command.
//!
//! ### Market Data Commands
//! - `quote` - Get current quote data
//! - `chart` - Historical OHLCV data
//! - `stream` - Live price streaming
//! - `market` - Market summary
//! - `trending` - Trending symbols
//! - `indices` - World indices
//! - `sector` - Sector performance
//!
//! ### Analysis Commands
//! - `indicator` - Calculate technical indicators
//! - `backtest` - Test trading strategies
//! - `screener` - Run predefined screeners
//!
//! ### Company Data Commands
//! - `info` - Detailed company information
//! - `profile` - Company profile
//! - `financials` - Financial statements
//! - `earnings` - Earnings data
//! - `filings` - SEC filings
//! - `transcript` - Earnings call transcripts
//! - `news` - Recent news
//! - `recommendations` - Analyst recommendations
//! - `grades` - Upgrade/downgrade history
//! - `holders` - Institutional and insider holdings
//!
//! ### Options & Dividends
//! - `options` - Interactive options chain (TUI)
//! - `dividends` - Dividend history
//! - `splits` - Stock split history
//!
//! ### Utilities
//! - `lookup` - Search for symbols
//! - `hours` - Check market hours
//! - `currencies` - Currency list and rates
//! - `exchanges` - Exchange information
//! - `alerts` - Manage price alerts
//! - `dashboard` - Interactive market dashboard (TUI)
//!
//! ## Configuration
//!
//! ### Environment Variables
//!
//! ```bash
//! # Enable debug logging
//! export RUST_LOG=debug
//!
//! # Disable colored output
//! fq quote AAPL --no-color
//! ```
//!
//! ### Data Storage
//!
//! Alert and portfolio data is stored in:
//! - **Linux/macOS**: `~/.local/share/fq/`
//! - **Windows**: `%APPDATA%\fq\`
//!
//! ## Examples
//!
//! ### Get Quote with Key Metrics
//! ```bash
//! fq quote AAPL MSFT GOOGL --format table
//! ```
//!
//! ### Stream Multiple Symbols
//! ```bash
//! fq stream AAPL MSFT TSLA NVDA
//! ```
//!
//! ### Calculate Multiple Indicators
//! ```bash
//! fq indicator AAPL --rsi 14 --sma 20,50,200 --interval 1d --range 6mo
//! ```
//!
//! ### Backtest SMA Crossover Strategy
//! ```bash
//! fq backtest AAPL \
//!   --strategy sma-crossover \
//!   --fast 10 --slow 20 \
//!   --interval 1d --range 1y \
//!   --initial-capital 10000
//! ```
//!
//! ### Get Earnings Call Transcript
//! ```bash
//! fq transcript AAPL --quarter 2024-Q1
//! ```
//!
//! ### Export Data to CSV
//! ```bash
//! fq chart AAPL --interval 1d --range 1y --format csv > aapl_1y.csv
//! ```
//!
//! ### Set Up Price Alerts
//! ```bash
//! fq alerts add AAPL --above 200 --below 150
//! fq alerts list
//! fq alerts check  # Check all alerts and notify
//! ```
//!

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![allow(missing_docs)]

use clap::{Parser, Subcommand};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub(crate) mod alerts;
mod backtest;
mod commands;
mod dashboard;
mod error;
mod indicator;
pub(crate) mod options;
mod output;
pub(crate) mod portfolio;

use error::Result;

/// Finance Query CLI
#[derive(Parser)]
#[command(
    name = "fq",
    version,
    about = "Command-line interface for querying financial data",
    long_about = None,
    arg_required_else_help = true
)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get quote data for one or more symbols
    Quote(commands::quote::QuoteArgs),

    /// Backtest a trading strategy on historical data
    Backtest(commands::backtest::BacktestArgs),

    /// Stream live prices for symbols via WebSocket
    Stream(commands::stream::StreamArgs),

    /// Get historical chart data (OHLCV)
    Chart(commands::chart::ChartArgs),

    /// Calculate technical indicators
    Indicator(commands::indicator::IndicatorArgs),

    /// Get dividend history for a symbol
    Dividends(commands::dividends::DividendsArgs),

    /// Get earnings dates, estimates, and history for a symbol
    Earnings(commands::earnings::EarningsArgs),

    /// Get stock split history for a symbol
    Splits(commands::splits::SplitsArgs),

    /// Get analyst recommendations and similar symbols
    Recommendations(commands::recommendations::RecommendationsArgs),

    /// Interactive options chain explorer with calls, puts, IV, and volume (TUI)
    Options(commands::options::OptionsArgs),

    /// Get company profile (sector, industry, description, executives)
    Profile(commands::profile::ProfileArgs),

    /// Get SEC filings (10-K, 10-Q, 8-K)
    Filings(commands::filings::FilingsArgs),

    /// Get financial statements (income, balance, cash flow)
    Financials(commands::financials::FinancialsArgs),

    /// Get analyst upgrade/downgrade history
    Grades(commands::grades::GradesArgs),

    /// Lookup symbols by asset type (equity, ETF, crypto, etc.)
    Lookup(commands::lookup::LookupArgs),

    /// Get earnings call transcripts
    Transcript(commands::transcript::TranscriptArgs),

    /// Check market hours and status
    Hours(commands::hours::HoursArgs),

    /// Get institutional and insider holder information
    Holders(commands::holders::HoldersArgs),

    /// Get world market indices quotes
    Indices(commands::indices::IndicesArgs),

    /// Get detailed company information
    Info(commands::info::InfoArgs),

    /// Get recent news for a symbol
    News(commands::news::NewsArgs),

    /// Run predefined screeners (most actives, gainers, losers, etc.)
    Screener(commands::screener::ScreenerArgs),

    /// Get sector performance and top companies
    Sector(commands::sector::SectorArgs),

    /// List available currencies and exchange rates
    Currencies(commands::currencies::CurrenciesArgs),

    /// List supported exchanges with metadata
    Exchanges(commands::exchanges::ExchangesArgs),

    /// Get market summary (indices, futures, currencies)
    Market(commands::market::MarketArgs),

    /// Get trending symbols by region
    Trending(commands::trending::TrendingArgs),

    /// Manage price alerts for stocks (add, list, check, remove)
    Alerts(commands::alerts::AlertsArgs),

    /// Live market dashboard with real-time streaming (TUI)
    Dashboard(commands::dashboard::DashboardArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing/logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("warn")
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Disable colors if requested
    if cli.no_color {
        colored::control::set_override(false);
    }

    // Execute the appropriate command
    match cli.command {
        Commands::Quote(args) => commands::quote::execute(args).await,
        Commands::Backtest(args) => commands::backtest::execute(args).await,
        Commands::Stream(args) => commands::stream::execute(args).await,
        Commands::Chart(args) => commands::chart::execute(args).await,
        Commands::Indicator(args) => commands::indicator::execute(args).await,
        Commands::Dividends(args) => commands::dividends::execute(args).await,
        Commands::Earnings(args) => commands::earnings::execute(args).await,
        Commands::Splits(args) => commands::splits::execute(args).await,
        Commands::Recommendations(args) => commands::recommendations::execute(args).await,
        Commands::Options(args) => commands::options::execute(args).await,
        Commands::Profile(args) => commands::profile::execute(args).await,
        Commands::Filings(args) => commands::filings::execute(args).await,
        Commands::Financials(args) => commands::financials::execute(args).await,
        Commands::Grades(args) => commands::grades::execute(args).await,
        Commands::Lookup(args) => commands::lookup::execute(args).await,
        Commands::Transcript(args) => commands::transcript::execute(args).await,
        Commands::Hours(args) => commands::hours::execute(args).await,
        Commands::Holders(args) => commands::holders::execute(args).await,
        Commands::Indices(args) => commands::indices::execute(args).await,
        Commands::Info(args) => commands::info::execute(args).await,
        Commands::News(args) => commands::news::execute(args).await,
        Commands::Screener(args) => commands::screener::execute(args).await,
        Commands::Sector(args) => commands::sector::execute(args).await,
        Commands::Currencies(args) => commands::currencies::execute(args).await,
        Commands::Exchanges(args) => commands::exchanges::execute(args).await,
        Commands::Market(args) => commands::market::execute(args).await,
        Commands::Trending(args) => commands::trending::execute(args).await,
        Commands::Alerts(args) => commands::alerts::execute(args).await,
        Commands::Dashboard(args) => commands::dashboard::execute(args).await,
    }
}
