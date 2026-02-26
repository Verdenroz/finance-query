# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/Verdenroz/finance-query/compare/v2.3.0...HEAD
[2.3.0]: https://github.com/Verdenroz/finance-query/compare/v2.2.1...v2.3.0
[2.2.1]: https://github.com/Verdenroz/finance-query/compare/v2.2.0...v2.2.1
[2.2.0]: https://github.com/Verdenroz/finance-query/compare/v2.1.0...v2.2.0
[2.1.0]: https://github.com/Verdenroz/finance-query/compare/v2.0.1...v2.1.0
[2.0.1]: https://github.com/Verdenroz/finance-query/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/Verdenroz/finance-query/releases/tag/v2.0.0
