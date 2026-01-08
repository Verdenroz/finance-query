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
