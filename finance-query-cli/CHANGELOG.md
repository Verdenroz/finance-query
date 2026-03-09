# Changelog - finance-query-cli

All notable changes to the Finance Query CLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-03-09

### Added

#### `backtest` — Major Expansion

- **Portfolio mode**: set `Portfolio Symbols` (comma-separated) to run the strategy across multiple symbols with a shared capital pool
  - `Rebalance Mode` field cycles between `Available Capital` and `Equal Weight`
  - `Max Allocation Per Symbol` field caps exposure to a single symbol
  - New **Portfolio** result tab shows per-symbol equity curves and allocation history
- **Parameter optimizer** (`Ctrl+O` in config editor):
  - Configure start/end/step ranges per indicator parameter
  - Choose search method: **Grid** (exhaustive, parallel) or **Bayesian/SAMBO** (adaptive, efficient for large spaces)
  - Choose optimisation metric: Sharpe Ratio, Total Return, Sortino, Calmar, Profit Factor, Win Rate, Min Drawdown
  - Optional **walk-forward validation** toggle (in-sample/out-of-sample windows)
  - New **Optimizer** result tab shows best parameters, convergence curve, and per-window walk-forward results
- **Ensemble strategy composition** (`4` / `c` on welcome screen):
  - Combine multiple strategies with configurable member weights
  - `h/l` keys adjust weight by step; `w` types an exact value
  - Voting modes: Weighted Majority, Unanimous, Any Signal, Strongest Signal
- **Order type configuration** in config editor:
  - `Entry Order Type` — Market, Limit Below, Stop Above, Stop-Limit Above
  - `Entry Price Offset`, `Entry Stop-Limit Gap`, `Entry Expiry Bars`
  - Equivalent Short order fields for short-leg strategies
- **Per-trade bracket orders** in config editor:
  - `Entry Bracket SL`, `Entry Bracket TP`, `Entry Bracket Trail`
  - `Short Bracket SL`, `Short Bracket TP`, `Short Bracket Trail`
- **Strategy builder new condition tabs**: Scale-In (add to position) and Scale-Out (partial exit), each with configurable fraction via `e` key; Regime Filter tab
- **Higher-timeframe (HTF) scope**: press `t` in the indicator config to cycle the HTF interval (e.g. daily RSI filter inside a 15-minute strategy)
- **User-saved presets**: `Ctrl+S` in config editor saves the current strategy locally; saved presets appear in the preset list and can be deleted with `d`
- **New result tabs**: Monte Carlo (1,000-run simulation, p5–p95 bands), Periods (breakdown by year / month / day of week), Comparison (strategy vs benchmark)
- **Cost profile quick-select**: cycle between Free / Realistic / Aggressive / Custom cost presets in config editor
- **New config fields**: `Spread %`, `Transaction Tax %`, `Max Positions`, `Bars / Year`, `Warmup Bars`
- **New presets** (10 total, up from 6): `ichimoku`, `volume`, `keltner`, `ema-momentum` added; `mean-reversion`, `conservative`, `aggressive` replaced by strategy-named equivalents (`rsi`, `macd`, `bollinger`)

### Changed

- **Backtest presets renamed** for clarity — update any saved workflows:
  - `mean-reversion` → `rsi`
  - `conservative` → `bollinger`
  - `aggressive` → `macd`
- Config editor now uses **tabbed sections** (`Tab` / `Shift+Tab`) for faster navigation across the expanded field set

## [0.2.0] - 2026-02-14

### Added
- **`edgar` command**: Unified TUI for SEC EDGAR data, replacing `filings`
  - `fq edgar submissions <SYMBOL>` — full filing history with interactive TUI viewer
  - `fq edgar facts <SYMBOL>` — XBRL company facts (us-gaap, ifrs, dei taxonomies)
  - `fq edgar search <QUERY>` — full-text filing search with form type and date filters
  - Email address persisted to `~/.config/fq/config.toml` — no need to re-enter each session
- **`portfolio` command**: Portfolio tracking with local SQLite database
- **Config persistence** (`src/config.rs`): User preferences saved across sessions

### Changed
- **Breaking**: `filings` command replaced by `edgar` — update any scripts or aliases using `fq filings`

### Removed
- `filings` command (use `fq edgar submissions <SYMBOL>` instead)

### Documentation
- Updated command reference with `edgar` subcommand examples
- New CLI examples page (`docs/cli/examples.md`)
- Installation guide updates
- Added EDGAR TUI screenshot

## [0.1.0] - 2026-01-13

### Added
- **Initial Release**: Complete command-line interface for financial data
- **Interactive Dashboard**: Real-time market watchlist with live streaming
  - Customizable watchlist management
  - Real-time price updates via WebSocket
  - Sparkline charts for quick trend visualization
  - Pre-market and after-hours indicators
  - Keyboard navigation and search
- **Technical Indicators**: 40+ indicators with interactive TUI
  - RSI, MACD, SMA, EMA, Bollinger Bands, ADX, Stochastic, and more
  - Interactive indicator selection and parameter configuration
  - Export to CSV, JSON, or table formats
  - Latest value display for quick checks
  - Custom time ranges and intervals
- **Backtesting Engine**: Strategy testing with visual results
  - Interactive TUI for strategy configuration
  - 6 preset strategies: swing, day, trend, mean-reversion, conservative, aggressive
  - Performance metrics with visual charts
  - Trade-by-trade analysis
  - JSON export for programmatic analysis
- **Price Alerts**: Desktop notifications for price movements
  - Multiple alert types: price thresholds, percent changes, volume spikes
  - Background monitoring with system service integration
  - Alert history and management
  - Support for Linux (systemd) and macOS (launchd)
- **Options Chain Explorer**: Interactive options analysis
  - Calls and puts with Greeks
  - Implied volatility visualization
  - Volume and open interest analysis
  - Multiple expiration dates
- **Market Data Commands**:
  - `quote`: Current prices and key metrics
  - `chart`: Historical OHLCV data with interactive TUI
  - `stream`: Real-time price streaming
  - `market`: Market summary (indices, futures, crypto)
  - `trending`: Trending symbols by region
  - `indices`: World market indices
  - `sector`: Sector performance and top companies
  - `screener`: Pre-built stock screeners
- **Company Information Commands**:
  - `info`: Detailed company information
  - `profile`: Company description and executives
  - `financials`: Financial statements
  - `earnings`: Earnings history and estimates
  - `news`: Recent news articles
  - `recommendations`: Analyst ratings
  - `holders`: Institutional and insider ownership
  - `filings`: SEC filings (10-K, 10-Q, 8-K)
  - `transcript`: Earnings call transcripts
  - `grades`: Analyst upgrade/downgrade history
- **Utility Commands**:
  - `lookup`: Symbol search
  - `hours`: Market hours and status
  - `currencies`: Currency exchange rates
  - `exchanges`: Supported exchanges
  - `dividends`: Dividend history
  - `splits`: Stock split history
- **Output Formats**: Table, JSON, CSV for all commands
- **Cross-Platform**: Linux, macOS, Windows support
- **Pre-built Installers**: Shell script installers for easy installation

### Documentation
- Complete command reference
- Usage examples for all features
- Installation guide for all platforms
- Screenshots of TUI interfaces

[Unreleased]: https://github.com/Verdenroz/finance-query/compare/cli-v0.3.0...HEAD
[0.3.0]: https://github.com/Verdenroz/finance-query/compare/cli-v0.2.0...cli-v0.3.0
[0.2.0]: https://github.com/Verdenroz/finance-query/compare/cli-v0.1.0...cli-v0.2.0
[0.1.0]: https://github.com/Verdenroz/finance-query/releases/tag/cli-v0.1.0
