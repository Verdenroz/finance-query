# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.4.3] - 2026-03-27

### Changed

- **Backtesting engine**: indicator computation now runs in parallel via rayon when ≥4 indicators and ≥1000 bars — no API change, automatic speedup for large backtests
- **Backtesting engine**: price series (highs, lows, volumes, opens) extracted in a single pass over candles rather than separate iterations per series
- **Monte Carlo**: per-simulation passes merged into a single allocation-free loop; no intermediate `Vec` per sim
- **Bayesian optimizer**: reduced allocations in Nadaraya-Watson surrogate evaluation
- **`IndicatorsSummary`**: pre-computed dense intermediates (`rsi_raw`, `atr_raw`) now reused across correlated indicators, eliminating redundant passes through the price series
- **Indicators — O(N) rewrites**: ADX, Keltner Channels, Stochastic, RSI, WMA, ATR, Ichimoku, Supertrend no longer allocate intermediate `Vec` buffers; all operate in a single pass
- **`keltner_channels`**: public function now delegates to `atr_raw` internally, avoiding a `Vec<Option<f64>>` round-trip through the `atr` wrapper
- HTTP clients simplified: deduplicated status-check helpers across Yahoo, EDGAR, and CoinGecko; endpoint parameter handling consolidated

### Fixed

- WebSocket streaming: reconnect delay was hardcoded; now configurable via `subscribe_inner` to allow proper isolation in tests

## [2.4.2] - 2026-03-24

### Fixed

- `CompanyFacts::cik` now deserializes correctly when the SEC EDGAR API returns the field as a zero-padded string (e.g. `"0001835724"`) instead of a number — fixes ~92 symbols that previously failed deserialization
- Bumped `aws-lc-sys` 0.35.0 → 0.39.0 and `rustls-webpki` 0.103.8 → 0.103.10 (security: RUSTSEC-2026-0044 through -0049)

## [2.4.1] - 2026-03-18

### Added

- **`Ticker::chart_range(interval, start, end)`** — fetch chart data using absolute Unix timestamps instead of a named `TimeRange`
  - Auto-chunking: intraday intervals (1m/5m/15m/30m/1h) that exceed Yahoo Finance's native window are automatically split into 7-day chunks, fetched in parallel, and merged (sorted + deduplicated, events accumulated across all chunks)
  - Parameter validation: returns `InvalidParameter` if `start >= end`

### Fixed

- `range_to_cutoff` no longer panics on missing cutoff values — `.unwrap()` replaced with safe fallback

## [2.4.0] - 2026-03-09

### Added

#### Backtesting Engine — Major Expansion

- **Order types** (`Signal`): limit, stop, and stop-limit entry orders in addition to market orders
  - `Signal::buy_limit(ts, px, limit_price)` — fill only if price reaches limit
  - `Signal::buy_stop(ts, px, stop_price)` — fill when price breaks above stop
  - `Signal::buy_stop_limit(ts, px, stop, limit)` — trigger at stop, fill at limit or better
  - `Signal::sell_limit` / `sell_stop` for exit orders
  - `.expires_in_bars(n)` — pending order auto-cancels after N bars (GTC by default)
- **Per-trade bracket orders** on `Signal`: override global config stop-loss / take-profit / trailing-stop per individual signal
  - `.stop_loss(pct)`, `.take_profit(pct)`, `.trailing_stop(pct)` builder methods on `Signal`
- **Scale in / scale out** (`Signal::scale_in(fraction, ...)` / `Signal::scale_out(fraction, ...)`) — pyramid trading with configurable position fraction
- **Signal tagging** (`.tag("name")`) — label signals/trades for post-backtest filtering via `BacktestResult::trades_by_tag` / `metrics_by_tag` / `all_tags`
- **`StrategyBuilder` new methods**:
  - `.regime_filter(condition)` — suppress entry signals unless all regime conditions pass
  - `.with_short(entry, exit)` — define a separate short leg with independent entry/exit conditions
  - `.warmup(bars)` — skip the first N bars before generating signals
- **Ensemble Strategy** (`src/backtesting/strategy/ensemble.rs`):
  - `EnsembleStrategy` — combine 2+ member strategies with configurable voting
  - `EnsembleMode` enum: `WeightedMajority` (default), `Unanimous`, `AnySignal`, `StrongestSignal`
- **Higher-Timeframe (HTF) Conditions** (`src/backtesting/refs/htf.rs`):
  - `htf(interval, condition)` — evaluate a condition on a coarser timeframe within a lower-TF strategy
  - `htf_region(interval, region, condition)` — with explicit exchange region
  - `resample(candles, interval, utc_offset_secs)` utility for manual candle aggregation
- **Advanced performance metrics** on `PerformanceMetrics` (all `#[non_exhaustive]`, non-breaking):
  - `winning_trades`, `losing_trades`, `largest_win`, `largest_loss`
  - `max_consecutive_wins`, `max_consecutive_losses`
  - `total_signals`, `executed_signals`
  - `avg_trade_return_pct`
  - `kelly_criterion`, `sqn` (System Quality Number), `expectancy`
  - `omega_ratio`, `tail_ratio`, `recovery_factor`, `ulcer_index`, `serenity_ratio`
- **`BacktestResult` extensions**:
  - `diagnostics: Vec<String>` — engine warnings and notes (e.g., rejected orders, skipped bars)
  - `rolling_sharpe(window)`, `drawdown_series()`, `rolling_win_rate(window)` — rolling analytics
  - `by_year()`, `by_month()`, `by_day_of_week()` — temporal breakdown returning `PerformanceMetrics`
  - `trades_by_tag(tag)`, `metrics_by_tag(tag)`, `all_tags()` — tag-based filtering
- **`BacktestConfig` new fields** (all `#[non_exhaustive]`, non-breaking):
  - `spread_pct` — bid-ask spread cost, half applied per side
  - `transaction_tax_pct` — one-time purchase tax (e.g. UK stamp duty, buy-side only)
  - `max_positions: Option<usize>` — cap concurrent open positions across the engine
  - `bars_per_year: f64` — annualisation denominator (default `252.0`)
  - `commission_fn: Option<CommissionFn>` — custom `fn(size, price) -> commission` overrides flat + pct
  - `BacktestConfig::zero_cost()` — convenience constructor with all friction zeroed
- **`BacktestComparison`** (`src/backtesting/comparison.rs`): rank multiple `BacktestResult` values side-by-side
  - `BacktestComparison::new().add(label, result).ranked_by(metric)` → `ComparisonReport`
  - `ComparisonReport`: `winner()`, `table()` → `&[ComparisonRow]`, `winner_row()`
- **Parameter Optimizer** (`src/backtesting/optimizer/`):
  - `GridSearch` — exhaustive search over all parameter combinations, parallelised with rayon
  - `BayesianSearch` (SAMBO) — Latin Hypercube Sampling init → Nadaraya-Watson surrogate → UCB acquisition; efficient for large/continuous parameter spaces
  - `ParamRange`: `int_range` / `float_range` (grid), `int_bounds` / `float_bounds` (Bayesian)
  - `OptimizeMetric` enum: `TotalReturn`, `SharpeRatio`, `SortinoRatio`, `CalmarRatio`, `ProfitFactor`, `WinRate`, `MinDrawdown`
  - `OptimizationReport`: `best`, `results` (sorted), `convergence_curve`, `n_evaluations`, `skipped_errors`
- **Walk-Forward Validation** (`src/backtesting/walk_forward.rs`):
  - `WalkForwardConfig::new(grid, config).in_sample_bars(n).out_of_sample_bars(n).run(...)`
  - `WalkForwardReport`: `aggregate_metrics`, `consistency_ratio`, `windows` (per-window IS/OOS results)
- **Monte Carlo Simulation** (`src/backtesting/monte_carlo.rs`):
  - `MonteCarloConfig::new().num_simulations(n).method(m).seed(s).run(&result)`
  - `MonteCarloMethod` enum: `IidShuffle` (default), `BlockBootstrap { block_size }`, `StationaryBootstrap { mean_block_size }`, `Parametric`
  - `MonteCarloResult`: `total_return`, `max_drawdown`, `sharpe_ratio`, `profit_factor` — each a `PercentileStats` (p5/p25/p50/p75/p95/mean)
  - Internal `Xorshift64` PRNG — no `rand` dependency
- **Portfolio Backtesting** (`src/backtesting/portfolio/`): full multi-symbol portfolio engine
  - `PortfolioEngine::new(config).run(&symbol_data, factory)` → `PortfolioResult`
  - `PortfolioConfig`: wraps `BacktestConfig` with `max_total_positions`, `max_allocation_per_symbol`, `rebalance`
  - `RebalanceMode` enum: `AvailableCapital` (default), `EqualWeight`, `CustomWeights(HashMap<String, f64>)`
  - `SymbolData::new(symbol, candles).with_dividends(divs)` — per-symbol data with optional dividend reinvestment
  - `PortfolioResult`: `symbols: HashMap<String, BacktestResult>`, `portfolio_equity_curve`, `portfolio_metrics`, `allocation_history`
  - `Tickers::backtest(interval, range, config, factory)` — fetches charts and dividends automatically, then runs `PortfolioEngine`

### Fixed

- Commission and slippage now correctly account for bid-ask spread as a separate cost component
- Portfolio engine dividend cash accounting — dividends now correctly added to available cash when `reinvest_dividends` is false
- Indicator smoothing: all indicators now accept fully customisable periods; no hardcoded defaults remain in public API

## [2.3.0] - 2026-02-25

### Added
- **Fear & Greed Index** (`finance::fear_and_greed()`): CNN Fear & Greed index via alternative.me — keyless, no init required
  - `FearAndGreed` response struct with score, label, and timestamp
  - `GET /v2/fear-and-greed` server endpoint
- **FRED Module** (`finance_query::fred`): Federal Reserve Economic Data integration (feature: `fred`)
  - `fred::init(api_key)` / `fred::init_with_timeout(api_key, timeout)` for one-time setup
  - `fred::series(id)` — any FRED time series by ID (e.g., `"FEDFUNDS"`, `"CPIAUCSL"`, `"GDP"`)
  - `fred::treasury_yields(year)` — daily US Treasury yield curve from treasury.gov (keyless)
  - `TreasuryYield` with full maturity ladder: 1m, 3m, 6m, 1y, 2y, 3y, 5y, 7y, 10y, 20y, 30y (all `Option<f64>`)
  - Rate limited to 2 req/sec per FRED guidelines
  - `GET /v2/fred/series/{id}` and `GET /v2/fred/treasury-yields?year=<u32>` server endpoints
- **CoinGecko Module** (`finance_query::crypto`): CoinGecko cryptocurrency market data (feature: `crypto`)
  - Keyless, lazy-init singleton — no init required
  - `crypto::coins(vs_currency, count)` — top N coins by market cap
  - `crypto::coin(id, vs_currency)` — single coin by CoinGecko ID
  - Rate limited to 30 req/min (CoinGecko free tier)
  - `GET /v2/crypto/coins?vs_currency=usd&count=50` and `GET /v2/crypto/coins/{id}` server endpoints
- **RSS/Atom Feeds Module** (`finance_query::feeds`): News feed aggregation (feature: `rss`)
  - `feeds::fetch(source)` — single named or custom feed
  - `feeds::fetch_all(sources)` — concurrent fetch, deduplicated, sorted newest-first
  - 30+ named `FeedSource` variants: `FederalReserve`, `SecPressReleases`, `SecFilings(form_type)`, `MarketWatch`, `Cnbc`, `Bloomberg`, `FinancialTimes`, `NytBusiness`, `GuardianBusiness`, `Investing`, `Bea`, `Ecb`, `Cfpb`, `WsjMarkets`, `Fortune`, `BusinessWire`, `CoinDesk`, `CoinTelegraph`, `TechCrunch`, `HackerNews`, `OilPrice`, `CalculatedRisk`, `Scmp`, `NikkeiAsia`, `BankOfEngland`, `VentureBeat`, `YCombinator`, `TheEconomist`, `FinancialPost`, `FtLex`, `RitholtzBigPicture`, `Custom(url)`
  - `FeedEntry` fields: `title`, `url`, `published` (RFC 3339), `summary`, `source`
  - `GET /v2/feeds?sources=<csv>&form_type=<str>` server endpoint
- **Risk Analytics Module** (`finance_query::risk`): Standalone risk metrics (feature: `risk`)
  - `Ticker::risk(interval, range, benchmark)` — full risk summary via `RiskSummary`
  - `RiskSummary` fields: `var_95`, `var_99`, `parametric_var_95`, `sharpe`, `sortino`, `calmar`, `beta`, `max_drawdown`, `max_drawdown_recovery_periods`
  - Standalone functions: `historical_var`, `parametric_var`, `sharpe_ratio`, `sortino_ratio`, `calmar_ratio`, `beta`, `max_drawdown`
  - Pure computation on `&[f64]` or `&[Candle]` — no network calls
  - `GET /v2/risk/{symbol}?interval=&range=&benchmark=` server endpoint
- **Dividend Analytics** (`Ticker::dividend_analytics(range)`): Pure computed analytics over dividend history
  - `DividendAnalytics` fields: `total_paid`, `payment_count`, `average_payment`, `cagr`, `last_payment`, `first_payment`
  - No additional network call — computed from cached dividend data
  - Injected into `GET /v2/dividends/{symbol}` response
- **Typed Screener Query API**: Fully type-safe screener query builder replacing stringly-typed API
  - `EquityScreenerQuery` and `FundScreenerQuery` builders with typed field enums
  - `EquityField` enum: ~80 fields (price, volume, PE, PEG, debt ratios, ESG, etc.)
  - `FundField` enum: ~10 fund-specific fields
  - `ScreenerFieldExt` trait: `.eq_str()`, `.gt()`, `.lt()`, `.between()`, and more operators
  - `ConditionValue`, `QueryCondition`, `QueryGroup`, `QueryOperand` moved to `models::screeners::condition`
  - `QuoteType`, `SortType` moved to `models::screeners::query`
  - `ScreenerFundCategory`, `ScreenerPeerGroup` value enums added

### Changed
- **Breaking**: `Sector` response struct renamed to `SectorData`
  - Update imports: `use finance_query::Sector` → `use finance_query::SectorData`
- **Breaking**: `Industry` response struct renamed to `IndustryData`
  - Update imports: `use finance_query::Industry` → `use finance_query::IndustryData`
- **Breaking**: `SectorType` enum renamed to `Sector` (selector enum in `constants`)
  - Update imports: `use finance_query::SectorType` → `use finance_query::Sector`
- **Breaking**: `ScreenerType` enum renamed to `Screener`
  - Update imports: `use finance_query::ScreenerType` → `use finance_query::Screener`
- **Breaking**: `Operator`, `LogicalOperator`, `QuoteType`, `SortType` moved out of `constants::screener_query` into `models::screeners`
  - `constants::screener_query` module removed; all types re-exported from `finance_query` root
- **Breaking**: `ScreenerQuery` replaced by `EquityScreenerQuery` / `FundScreenerQuery` for typed screener queries
  - Old `QueryCondition::new("field", Operator::Gt).value(n)` → `EquityField::Price.gt(n)`
  - `finance::custom_screener(query)` now accepts `impl Into<ScreenerQuery>`

## [2.2.1] - 2026-02-21

### Added
- **Candlestick Pattern Recognition** (`finance_query::indicators::patterns`): Detects 20 common single-, double-, and triple-bar patterns across OHLCV candle data
  - `patterns(&candles)` — standalone function returning `Vec<Option<CandlePattern>>` aligned 1:1 with the input slice
  - `Chart::patterns()` — extension method on `Chart` for ergonomic use
  - `CandlePattern` enum (20 variants, `#[non_exhaustive]`, serde-serializable): `MorningStar`, `EveningStar`, `ThreeWhiteSoldiers`, `ThreeBlackCrows`, `BullishEngulfing`, `BearishEngulfing`, `BullishHarami`, `BearishHarami`, `PiercingLine`, `DarkCloudCover`, `TweezerTop`, `TweezerBottom`, `Hammer`, `InvertedHammer`, `HangingMan`, `ShootingStar`, `BullishMarubozu`, `BearishMarubozu`, `Doji`, `SpinningTop`
  - `PatternSentiment` enum (`Bullish` / `Bearish` / `Neutral`) accessible via `CandlePattern::sentiment()`
  - Precedence rule: three-bar patterns take priority over two-bar, which take priority over one-bar
  - Re-exported from `finance_query` root under the `indicators` feature flag
- **`?patterns=true` query parameter** on `GET /v2/chart/{symbol}` and `GET /v2/charts`: injects a per-candle `patterns` array into the JSON response; `null` entries mean no pattern was detected on that bar
- OpenAPI spec updated with new query parameter and 20-variant nullable string enum schema

### Changed
- Updated `polars` dependency `0.52 → 0.53` with associated type conversion fix in DataFrame operations

## [2.2.0] - 2026-02-14

### Added
- **EDGAR Module** (`finance_query::edgar`): Complete SEC EDGAR integration
  - Singleton client with automatic rate limiting (10 req/sec per SEC guidelines)
  - `edgar::init(email)` / `edgar::init_with_config(email, app, timeout)` for one-time setup
  - `edgar::resolve_cik(symbol)` — resolves ticker symbols to CIK numbers (cached)
  - `edgar::submissions(cik)` — full filing history + company metadata (~1000 recent filings)
  - `edgar::company_facts(cik)` — structured XBRL financial data (us-gaap, ifrs, dei taxonomies)
  - `edgar::search(query, forms, start_date, end_date)` — full-text filing search with pagination (`from`, `size`)
  - DataFrame conversion methods for all EDGAR models (feature-gated via `dataframe`)
- **Extended `Tickers` Batch API**: New batch methods with caching
  - `dividends(range)`, `splits(range)`, `capital_gains(range)`
  - `financials(statement, freq)`, `news()`, `recommendations(limit)`, `options(date)`
  - `indicators(interval, range)` (feature-gated via `indicators`)
  - `charts_range(interval, start, end)` for Unix timestamp-based batch chart fetching
- **`Ticker::chart_range(interval, start, end)`**: Single-ticker chart fetch by Unix timestamps
- **Builder enhancements**: `TickersBuilder::cache(ttl)`, `TickersBuilder::max_concurrency(n)`, `TickersBuilder::client(handle)`
- **New server batch endpoints** mirroring extended `Tickers` API:
  - `GET /v2/charts`, `/v2/dividends`, `/v2/splits`, `/v2/capital-gains`
  - `GET /v2/financials`, `/v2/recommendations`, `/v2/options`, `/v2/indicators`
- **New EDGAR server endpoints**:
  - `GET /v2/edgar/cik/{symbol}`, `GET /v2/edgar/submissions/{symbol}`
  - `GET /v2/edgar/facts/{symbol}`, `GET /v2/edgar/search`
  - Requires `EDGAR_EMAIL` environment variable
- **CLI `edgar` command** (`fq edgar`): Unified TUI viewer for submissions, company facts, and search
  - Replaces the `filings` command
  - Email persisted to local config (`~/.config/fq/config.toml`)

### Changed
- **Breaking**: `YahooError` renamed to `FinanceError` to reflect multi-source data
  - `type Error = FinanceError` alias updated accordingly
  - Update any direct imports: `use finance_query::YahooError` → `use finance_query::FinanceError`
- **Breaking**: `Ticker::quote()` and `Tickers::quotes()` / `Tickers::quote()` no longer accept `include_logo: bool`
  - Request logos via the builder instead: `Ticker::builder("AAPL").logo().build().await?`
- **Breaking**: `Tickers::symbols()` now returns `Vec<&str>` instead of `&[String]`
- **Breaking**: `Candle` and `Exchange` JSON serialization now uses `camelCase` field names
- CLI version bumped to `0.2.0`

### Deprecated
- `Ticker::sec_filings()` — use `edgar::submissions(cik)` for comprehensive SEC filing data

### Fixed
- Empty string deserialization in JSON responses no longer causes parse failures

## [2.1.0] - 2026-01-13

### Added
- **Backtesting Framework**: Comprehensive strategy testing engine
  - Signal-based entry/exit conditions with flexible configuration
  - Performance metrics including Sharpe ratio, max drawdown, win rate, profit factor
  - Pre-built strategies: SMA crossover, RSI, Bollinger Bands, MACD, trend following
  - Position management with configurable sizing and commission
  - Detailed trade-by-trade analysis
- **Indicators Module**: Refactored indicator calculations into dedicated module
  - 40+ technical indicators (RSI, MACD, Bollinger Bands, ADX, Stochastic, etc.)
  - Three usage patterns: summary API, chart extensions, direct functions
  - Optimized performance with vectorized calculations
  - Support for custom periods and parameters
- **Spark Endpoint**: Batch sparkline data retrieval
  - Efficient mini-chart data for multiple symbols
  - Optimized for watchlist displays
  - `/v2/spark` endpoint for server integration
- **Market Hours Enhancements**: Overnight trading hours display
  - Pre-market and after-hours session information
  - Real-time market status updates

### Changed
- **Breaking**: Indicators moved from `models::indicators` to top-level `indicators` module
  - Update imports: `use finance_query::models::indicators::*` → `use finance_query::indicators::*`
  - Old module still works but is deprecated
- **Breaking**: Indicator API simplified and more flexible
  - Chart extension methods now take period parameters
  - Summary API provides pre-computed indicators with standard periods
- Improved error messages for invalid time ranges and intervals

### Fixed
- Chart rendering with dynamic range and interval selection
- Indicator calculations for edge cases with insufficient data

### Documentation
- Complete rewrite of indicators documentation
- Added backtesting guide with examples
- Updated all code examples to use new indicator module

## [2.0.1] - 2025-12-31

### Added
- Production hosting at https://finance-query.com
  - Automatic HTTPS with Caddy reverse proxy
  - REST API at `/v2/*`
  - WebSocket streaming at `/v2/stream`

### Changed
- Updated all documentation to reference new hosted API

### Deprecated
- Legacy AWS endpoint (`https://43pk30s7aj.execute-api.us-east-2.amazonaws.com/prod`)
- Legacy Render endpoint (`https://finance-query.onrender.com`)

### Fixed
- Health check endpoint routing
- Docker image naming consistency

## [2.0.0] - 2025-12-31

### Added
- Initial v2.0 release with major API redesign
- Comprehensive quote data with 30+ modules
- Historical chart data with multiple intervals
- Real-time WebSocket streaming
- Company fundamentals and financials
- Options chain data
- News and analyst recommendations

[Unreleased]: https://github.com/Verdenroz/finance-query/compare/v2.4.2...HEAD
[2.4.2]: https://github.com/Verdenroz/finance-query/compare/v2.4.1...v2.4.2
[2.4.1]: https://github.com/Verdenroz/finance-query/compare/v2.4.0...v2.4.1
[2.4.0]: https://github.com/Verdenroz/finance-query/compare/v2.3.0...v2.4.0
[2.3.0]: https://github.com/Verdenroz/finance-query/compare/v2.2.1...v2.3.0
[2.2.1]: https://github.com/Verdenroz/finance-query/compare/v2.2.0...v2.2.1
[2.2.0]: https://github.com/Verdenroz/finance-query/compare/v2.1.0...v2.2.0
[2.1.0]: https://github.com/Verdenroz/finance-query/compare/v2.0.1...v2.1.0
[2.0.1]: https://github.com/Verdenroz/finance-query/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/Verdenroz/finance-query/releases/tag/v2.0.0
