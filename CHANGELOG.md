# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased (draft vs v2.8.0)

<!-- soothfast:notes -->
### Breaking

- `Ticker`, `Tickers`, and the domain handles cache responses by default for the
  lifetime of the handle. Previously caching was off unless `.cache(ttl)` was
  set, so every accessor refetched — which contradicted the documentation. Call
  `.no_cache()` for the old behavior.
- `BacktestEngine::run` and `run_with_dividends` now reject candles that are not
  sorted ascending by timestamp, with the same `InvalidParameter` error already
  used for unsorted dividends. Conditions binary-search the candle slice, so an
  unsorted series previously produced a silently wrong entry index. `GridSearch`,
  `BayesianSearch`, and `WalkForward` apply the same check once per series rather
  than once per candidate.
- A Yahoo HTTP 403 whose body names the crumb now surfaces as
  `AuthenticationFailed` rather than `UnexpectedResponse`, so the shared session
  refreshes instead of failing every later caller on it. A 403 that does *not*
  mention the crumb — a datacenter-IP block or abuse throttle — keeps mapping to
  `UnexpectedResponse`, since no handshake can clear it and retrying would cost
  an extra handshake and a discarded session per blocked request.

### Changed

- `Capability` values combining several bits display as `"quote|chart"`
  instead of `"unknown"`; `Capability::name()` keeps its documented
  single-bit contract.
- When no routed provider supports the specific operation, the error now
  names that operation and the providers that could serve it
  (`NotSupported`) instead of the generic capability-level
  `NoProviderAvailable`.
- Yahoo sessions are now shared per tokio runtime and client configuration.
  The `finance::*` functions and `Ticker`/`Tickers` construction reuse one
  authenticated session instead of running a fresh cookie + crumb handshake per
  call, cutting each `finance::*` call from three HTTP requests to one.

### Added

- **CoinGecko trending, global stats, and coin search** are exposed keylessly on
  the server: `GET /v2/crypto/trending`, `GET /v2/crypto/global`, and
  `GET /v2/crypto/search` (plus the `cryptoTrending`/`cryptoGlobal`/`cryptoSearch`
  GraphQL root fields). The library gains matching `crypto::trending()`,
  `crypto::global()`, and `crypto::search()` shortcuts alongside
  `crypto::coins()`/`crypto::coin()`.
- `CalendarKind::MarketStatus` and `MarketCalendar::market_status()` for live
  exchange open/closed status, which Alpha Vantage serves. It was previously
  mapped onto `CalendarKind::MarketHoliday`, so a holiday-calendar query routed
  to Alpha Vantage silently returned live status rows instead.
- `SymbolMatch` gains `id`, `market_cap_rank`, `thumbnail`, and `image`. `id`
  carries a provider-native identifier when it differs from the ticker — the
  CoinGecko coin id, for instance, is what `Providers::crypto` accepts.
- **World Bank Open Data** (`worldbank` feature, keyless) — `Provider::WorldBank`
  serves `Capability::ECONOMIC` with roughly 1,600 global development and macro
  indicators across 200+ economies, closing the gap left by FRED's US focus.
  Series are addressed as `"<COUNTRY>/<INDICATOR>"`
  (`providers.economic("USA/NY.GDP.MKTP.CD")`); a bare indicator resolves
  against the world aggregate `WLD`. Annual/quarterly/monthly period labels are
  normalised to the `YYYY-MM-DD` start of the period and observations are
  returned oldest-first, matching every other `ECONOMIC` provider.
- **US Treasury FiscalData** (`fiscaldata` feature, keyless) —
  `Provider::FiscalData` serves `Capability::ECONOMIC` from the Treasury's own
  publishing platform: federal debt (`"DEBT_TO_PENNY"`), average interest rates,
  and Daily Treasury Statement cash balances. Six curated series ids cover the
  common cases; any other dataset is reachable through the passthrough form
  `"<dataset path>:<value column>"`. FiscalData encodes numbers as strings and
  marks missing figures with the literal string `"null"` — both are normalised.
- **BLS** (`bls` feature) — `Provider::Bls` serves `Capability::ECONOMIC` with
  CPI, unemployment, payrolls, wages, and PPI from the primary source, using
  native BLS series ids (`providers.economic("CUUR0000SA0")`). The first
  provider with a **keyless/keyed dual mode**: without `BLS_API_KEY` it uses
  the keyless v1 route (25 queries/day, ~3 years); with a key it uses v2 (500
  queries/day, 20 years, plus catalog series titles). BLS's `"-"`
  unpublished-value marker becomes `None`, and annual-aggregate rows (`M13` /
  `Q05` / `S03`) are dropped so a monthly series stays strictly monthly.
- **Frankfurter / ECB reference rates** (`frankfurter` feature, keyless) —
  `Provider::Frankfurter` serves `Capability::FOREX`, the first keyless route
  for that capability: `providers.forex("USD", "EUR").quote()` now works with
  nothing configured, where previously every forex route needed an API key.
  ECB rates are a daily reference fix, so quotes carry a price and a change
  against the previous *published* day but no bid/ask. A pair of identical
  currencies is answered locally with `1.0` rather than Frankfurter's HTTP 422.
- **Binance public market data** (`binance` feature, keyless) —
  `Provider::Binance` serves `Capability::CRYPTO` (rolling 24-hour quotes) and
  `Capability::CHART` (arbitrary-interval OHLCV), the first keyless source of
  exchange-grade crypto data. Symbols normalise across every spelling the
  library uses (`bitcoin`, `BTC`, `BTC-USD`, `BTCUSDT`); note that Binance
  lists no USD spot markets, so `"usd"` maps to the USDT stablecoin. Windows
  longer than Binance's 1000-candle page cap are walked automatically (up to
  10,000 candles). A geo-block (HTTP 451) is reported as such and names Kraken
  as the alternative.
- **Kraken public market data** (`kraken` feature, keyless) —
  `Provider::Kraken` serves `Capability::CRYPTO` and `Capability::CHART` from
  endpoints that are keyless *and* reachable from the US, unlike Binance. With
  both providers, `CRYPTO` finally has a real `Fetch::Sequential` fallback
  chain. Kraken's own conventions (`XBT` for Bitcoin, `XDG` for Dogecoin, the
  legacy `X`/`Z` pair prefixes) are translated in both directions, so callers
  pass normal tickers. Kraken caps `/OHLC` at ~720 candles ending at the
  present with no way to page further back — deep history needs Binance or a
  keyed provider.
- **FINRA short-sale volume** (`finra` feature, keyless) — `Provider::Finra`
  serves the short-volume slice of `Capability::FUNDAMENTALS`, giving
  `Ticker::short_volume()` a keyless provider reading from the primary source
  rather than requiring Polygon. FINRA reports each symbol once per reporting
  facility per day (Nasdaq / NYSE / OTC); the adapter sums them into one figure
  per date, matching FINRA's own consolidated daily file. A symbol with no
  reportable short volume returns an empty series rather than an error. Free
  for non-commercial use — see the provider docs.
- **OpenFIGI identifier mapping** (`openfigi` feature, keyless) — new
  crate-level `openfigi` module resolving a CUSIP, ISIN, SEDOL, or FIGI to the
  instruments carrying it: `openfigi::resolve_cusip("037833100")`,
  `resolve_isin`, `resolve_sedol`, and a positional batch `resolve_many` that
  chunks to OpenFIGI's 10-per-request limit. New public types
  `SecurityMapping` and `SecurityIdKind`. It sits beside `edgar` and `fred`
  rather than behind the Providers API because resolution is not tied to a
  symbol handle and maps onto no `Capability`. `OPENFIGI_API_KEY` is optional
  and only raises the quota.
- **DefiLlama** (`defi` feature, keyless) — the library's first on-chain data.
  `Provider::DefiLlama` adds `CryptoCoin::tvl()` and `.tvl_history()` under
  `Capability::CRYPTO` (protocol TVL, per-chain split, 1d/7d change, market
  cap), and a new crate-level `defi` module carries the market-wide views:
  `defi::chains()` and `defi::stablecoins()`, both ranked largest-first. New
  public models `ProtocolTvl`, `ChainAllocation`, `TvlPoint`, `ChainTvl`, and
  `StablecoinSupply`. Per-chain allocations exclude DefiLlama's breakdown keys
  (`-borrowed`, `pool2`, `staking`), which describe the same capital and would
  double-count. DefiLlama serves no prices, so `quote()` falls through to
  another routed provider.
- **GDELT DOC 2.0** (`gdelt` feature, keyless) — `Provider::Gdelt` serves the
  news slice of `Capability::CORPORATE` from GDELT's worldwide online news
  index (65 languages, updated roughly every 15 minutes), giving `Ticker::news()`
  a keyless alternative to Yahoo's scraper and Alpha Vantage's 25 req/day
  quota. GDELT has no ticker vocabulary, so the symbol itself (quoted for an
  exact phrase match) is the search term — precision over recall. GDELT
  rejects phrases under 5 characters, so shorter tickers (`AAPL`, `TSLA`, …)
  error with `InvalidParameter` and dispatch falls through to another
  CORPORATE provider. GDELT has no corporate calendar, so `fetch_events`
  reports `NotSupported`.
- **CFTC Commitments of Traders** (`cftc` feature, keyless) —
  `Provider::Cftc` serves `Capability::FUTURES` with weekly futures
  positioning by trader category (commercial hedgers, swap dealers, managed
  money, other reportables, small traders) via
  `FuturesContract::commitments_of_traders()`, reading the disaggregated
  futures-only combined report from `publicreporting.cftc.gov`. A curated
  table maps common Yahoo-style continuous futures roots (`"GC=F"`, `"CL=F"`,
  …) to their CFTC contract code; anything else is passed through as a raw
  `cftc_contract_market_code`. New public model `CommitmentsOfTraders` /
  `CotObservation`. Covers physical commodities only (agriculture, energy,
  metals) — CFTC reports financial futures (equity indices, rates,
  currencies) separately in the Traders in Financial Futures report, which
  this adapter does not serve; CFTC publishes no price quotes at all, so
  `fetch_futures_quote` reports `NotSupported` and falls through to another
  routed provider (e.g. Polygon).
- Both keyless providers are exposed on the server: `GET /v2/gdelt/news/{symbol}`
  and `GET /v2/cftc/cot/{symbol}`, plus the `gdeltNews`/`commitmentsOfTraders`
  GraphQL root fields. The library gains `gdelt::news()` and
  `cftc::commitments_of_traders()` shortcuts so neither needs provider routing.
- Alpha Vantage gains the `DISCOVERY` capability and ETF coverage:
  `Ticker::etf_profile()` returns a fund's profile and portfolio holdings
  (heaviest first) — no other wired provider serves ETF composition —
  and `providers.discovery()` now routes to Alpha Vantage for `.search(..)`,
  `.exchanges()`, and the new `.listing_status(active)`, which returns the whole
  listed (or delisted) universe. New public models `EtfProfile` and
  `EtfHolding`.
- FRED series discovery and point-in-time data:
  `providers.economic_catalog()` is a new market-wide handle with `.search(query,
  limit)`, `.categories(parent_id)`, and `.releases()`, so a series id no longer
  has to be known up front. `providers.economic("GDPC1").as_of("2020-06-30")`
  returns the series as it was actually published on that date (ALFRED vintage)
  rather than as currently revised — backtesting against revised macro data is
  look-ahead bias. New public models `EconomicSeriesMatch`, `EconomicCategory`,
  and `EconomicRelease`.
- Keyless crypto price history: `Provider::CoinGecko` now serves
  `Capability::CHART`, so `.route(Capability::CHART, [Provider::CoinGecko])`
  makes `providers.crypto("bitcoin").history("usd", range)` work without an API
  key. CoinGecko picks bar granularity from the range (so `interval` is
  advisory) and its OHLC endpoint reports no volume, so those candles carry
  `volume: 0` rather than an interpolated figure.
- Primary-source ownership data from EDGAR, keyless:
  `providers.filings("AAPL").insider_trades(limit)` parses Form 3/4/5 ownership
  XML into typed `InsiderTrade` rows (insider, role, transaction code, shares,
  price, post-transaction holdings, derivative flag), and
  `providers.filings("BRK-B").institutional_holdings()` parses the latest 13F-HR
  information table into `InstitutionalHolding` rows (issuer, CUSIP, value,
  shares, voting authority). XML is read by a small in-crate element reader —
  no new dependency, same reasoning as the RSS/Atom parser.
- Routed EDGAR full-text search: `providers.filings("AAPL").search(query,
  filters)` searches filing *text* within that filer, and `.search_all(..)`
  across every filer, through the `FILINGS` capability. Returns the flattened
  `FilingSearchHit` (with a derived archive URL per hit) rather than EDGAR's raw
  Elasticsearch envelope, which `edgar::search` still exposes unchanged. New
  public models `FilingSearchHit` and `FilingSearchFilters`.
- Cross-market snapshots: `providers.snapshot().get(&["AAPL", "X:BTCUSD",
  "I:SPX"])` answers a mixed watchlist in one request through the `QUOTE` route
  (Polygon's `/v3/snapshot`, max 250 symbols) instead of one request per asset
  class. Unresolvable symbols come back as rows with `error` set rather than
  being dropped. New public models `MarketSnapshot` and `AssetClass`, and a new
  `Snapshot` handle.
- Analyst consensus on `Ticker`: `price_target_consensus()` (high/low/mean/median
  target), `price_target_summary()` (how many targets were published last
  month/quarter/year/all time and their averages), and `rating_consensus()`
  (grade distribution plus a headline label) via the `FUNDAMENTALS` route (FMP).
  These are the provider's own panel-wide rollups, not the raw per-analyst grade
  actions already available through `UpgradeDowngradeHistory`.
- TTM fundamentals snapshots on `Ticker`: `key_metrics_ttm()` and `ratios_ttm()`
  return a single always-current trailing-twelve-month rollup via the
  `FUNDAMENTALS` route (FMP), instead of requiring callers to fetch the latest
  fiscal period and reason about whether it is still current. New public models
  `KeyMetricsTtm` and `FinancialRatiosTtm`.
- Ownership/governance on `Ticker`: `executive_compensation()` and
  `employee_count()` via the `CORPORATE` route (FMP), both extracted from the
  company's own SEC filings and returned newest-first. FMP also now serves
  `share_float()` on the `FUNDAMENTALS` route, so it works when FMP is routed
  ahead of Yahoo. New public models `ExecutiveCompensation` and `EmployeeCount`.
- Index constituents: `providers.index("^GSPC").constituents()` and
  `.constituent_changes()` list the current members and membership history of
  the S&P 500, Nasdaq 100, and Dow Jones (FMP; changes are S&P 500 only).
  `MajorIndex::from_symbol` maps common symbol spellings.
- Short data on `Ticker`: `short_interest()`, `short_volume()`, and
  `share_float()` via the `FUNDAMENTALS` route. The default Yahoo route
  derives short interest (current + prior-month snapshots) and float from
  key statistics keylessly; Polygon adds the full history and daily short
  volume.
- Filing text: `providers.filings("AAPL").sections(accession, form)` returns
  the sectioned 10-K/8-K text of a filing and `.risk_factors()` the extracted
  risk factors (Polygon; EDGAR still serves metadata).
- `Ticker::press_releases(limit)` — the company's own releases, distinct from
  press coverage via `news()` (FMP).
- `providers.calendar().holidays()` — upcoming market holidays and early
  closes as a new `CalendarKind::MarketHoliday` (Polygon).
- `providers.market().sector_performance_history(limit)` (FMP), and market
  movers now work on the default keyless route: Yahoo serves
  `gainers()`/`losers()`/`most_active()` derived from its predefined
  screeners, with Alpha Vantage as a second keyed route. `providers.market()`
  and the mover/sector models are available without any provider feature.
- Three market-wide capabilities and handles on the Providers API (each needs
  at least one of the `fmp`/`polygon`/`alphavantage` features):
  `Capability::DISCOVERY` with `providers.discovery()` (symbol search,
  reference data, exchanges, screeners), `Capability::CALENDAR` with
  `providers.calendar()` (market-wide earnings/IPO/dividend/split/economic
  calendars), and `Capability::MARKET` with `providers.market()`
  (sector/industry performance and movers).
- Batch quotes for FMP, Polygon, and Alpha Vantage. Routing `QUOTE` to a
  non-Yahoo provider previously fell back to one request per symbol; a
  10-symbol batch now costs one request instead of ten.
- `TickerBuilder::no_cache()`, `TickersBuilder::no_cache()`, and `no_cache()`
  on the domain handles.
- `backtesting::PositionExtremes` and `StrategyContext::extremes` — the highest
  and lowest bar high/low/close since the open position was entered. The engine
  folds these once per bar, so `TrailingStop` and `TrailingTakeProfit` read one
  shared running value instead of rescanning the candle history per bar
  (O(bars²) → O(bars)). Both conditions stay `Copy`. Custom conditions can read
  `ctx.extremes`; it is `None` outside the engine's bar loop.
- Automatic crumb refresh: a request that fails authentication re-runs the
  Yahoo handshake once and retries. If the refresh itself fails, the shared
  session is dropped so the next caller builds a fresh one.

### Fixed

- `FinanceError::RateLimited` rendered as "Rate limited (retry after Nones)"
  when no retry hint was available, and "Some(30)s" when there was one.
- GDELT's throttle response carries its retry interval in a plain-text body
  rather than a `Retry-After` header; that interval is now parsed into
  `RateLimited::retry_after` and the message logged instead of discarded.
- EDGAR is no longer dropped from the provider set when Alpha Vantage is
  configured. Auto-injection tested `capabilities().contains(FILINGS)`, and
  Alpha Vantage advertises FILINGS only to serve insider transactions — so
  EDGAR, the sole real filings source, was suppressed and every `filings()`
  call failed. Injection is now keyed on provider identity.
- CoinGecko symbol search puts the ticker in `SymbolMatch::symbol` (uppercased),
  matching every other DISCOVERY provider; it previously put the CoinGecko coin
  id there, which broke the provider-neutral contract for consumers searching
  across providers.
- `RetryPolicy` honored an error's `retry_after` hint with no ceiling, so a
  hostile or buggy upstream could park a caller indefinitely. Capped by the
  new `max_retry_after` (default 5 minutes), kept separate from `max_delay`
  so legitimate multi-minute hints are still honored.
- A stream whose session came up and then failed a few seconds in — auth
  rejection, subscription error, idle kill — reset the reconnect counter
  every cycle and retried forever despite `max_reconnect_attempts`. The
  healthy-session threshold is now 60s, independent of the (short) base
  delay it was keyed to.
- A zero base delay produced the maximum delay instead of zero at high
  attempt counts (`0.0 * inf` is NaN, and `f64::min` returns the other
  operand for NaN).

- `finance::hours()` no longer labels every region's market "U.S. markets".
  Yahoo returns correct per-region session times but hardcodes the U.S.
  label; the name (and its occurrence in the status message) is now derived
  from the market id, so `region=JP` reads "Japanese markets".
- `vwma` is computed with rolling sums instead of rescanning the window each
  bar. Results may differ from previous releases in the last few significant
  digits; a window whose volumes span extreme magnitudes is now rebuilt rather
  than returning a value derived from a cancelled-out denominator.
- A batch quote response whose `result` array contains a non-object element now
  fails that batch instead of yielding an all-empty quote for it.
<!-- /soothfast:notes -->

### API surface

```
ADDED    finance_query::AssetClass
ADDED    finance_query::CalendarDetail
ADDED    finance_query::CalendarKind
ADDED    finance_query::CommitmentsOfTraders
ADDED    finance_query::CotObservation
ADDED    finance_query::Discovery
ADDED    finance_query::EconomicCatalog
ADDED    finance_query::EconomicCategory
ADDED    finance_query::EconomicRelease
ADDED    finance_query::EconomicSeriesMatch
ADDED    finance_query::EmployeeCount
ADDED    finance_query::EtfHolding
ADDED    finance_query::EtfProfile
ADDED    finance_query::ExchangeInfo
ADDED    finance_query::ExecutiveCompensation
ADDED    finance_query::FilingSearchFilters
ADDED    finance_query::FilingSearchHit
ADDED    finance_query::FilingSection
ADDED    finance_query::FilingSectionForm
ADDED    finance_query::FinancialRatiosTtm
ADDED    finance_query::IndexConstituent
ADDED    finance_query::IndexConstituentChange
ADDED    finance_query::IndustryPe
ADDED    finance_query::InsiderTrade
ADDED    finance_query::InstitutionalHolding
ADDED    finance_query::KeyMetricsTtm
ADDED    finance_query::MajorIndex
ADDED    finance_query::Market
ADDED    finance_query::MarketCalendar
ADDED    finance_query::MarketCalendarEntry
ADDED    finance_query::MarketSnapshot
ADDED    finance_query::MoverDirection
ADDED    finance_query::MoverQuote
ADDED    finance_query::PressRelease
ADDED    finance_query::PriceTargetConsensus
ADDED    finance_query::PriceTargetSummary
ADDED    finance_query::ProviderHealth
ADDED    finance_query::RatingConsensus
ADDED    finance_query::RetryPolicy
ADDED    finance_query::RiskFactor
ADDED    finance_query::ScreenerFilters
ADDED    finance_query::ScreenerMatch
ADDED    finance_query::SectorPe
ADDED    finance_query::SectorPerformance
ADDED    finance_query::SectorPerformanceHistory
ADDED    finance_query::ShareFloat
ADDED    finance_query::ShortInterest
ADDED    finance_query::ShortVolume
ADDED    finance_query::Snapshot
ADDED    finance_query::SymbolDetails
ADDED    finance_query::SymbolMatch
ADDED    finance_query::adapters::cftc::futures::fetch_commitments_of_traders_response
ADDED    finance_query::adapters::coingecko::discovery::fetch_symbol_search_response
ADDED    finance_query::adapters::coingecko::fetch_crypto_global_response
ADDED    finance_query::adapters::coingecko::fetch_crypto_trending_response
ADDED    finance_query::adapters::gdelt::corporate::fetch_news_response
ADDED    finance_query::adapters::yahoo::client::ClientConfig
ADDED    finance_query::adapters::yahoo::client::YahooClient
ADDED    finance_query::backtesting::PositionExtremes
ADDED    finance_query::backtesting::position::Position::close
ADDED    finance_query::backtesting::strategy::PositionExtremes
ADDED    finance_query::cftc
ADDED    finance_query::cftc::CommitmentsOfTraders
ADDED    finance_query::cftc::CotObservation
ADDED    finance_query::cftc::commitments_of_traders
ADDED    finance_query::constants::industries::Industry::as_slug
ADDED    finance_query::crypto::GlobalCryptoStats
ADDED    finance_query::crypto::SymbolMatch
ADDED    finance_query::crypto::TrendingCoin
ADDED    finance_query::crypto::global
ADDED    finance_query::crypto::search
ADDED    finance_query::crypto::trending
ADDED    finance_query::defi
ADDED    finance_query::defi::ChainAllocation
ADDED    finance_query::defi::ChainTvl
ADDED    finance_query::defi::ProtocolTvl
ADDED    finance_query::defi::StablecoinSupply
ADDED    finance_query::defi::TvlPoint
ADDED    finance_query::defi::chains
ADDED    finance_query::defi::stablecoins
ADDED    finance_query::domains::Discovery
ADDED    finance_query::domains::EconomicCatalog
ADDED    finance_query::domains::Market
ADDED    finance_query::domains::MarketCalendar
ADDED    finance_query::domains::Snapshot
ADDED    finance_query::domains::crypto::CryptoCoin::tvl
ADDED    finance_query::domains::crypto::CryptoCoin::tvl_history
ADDED    finance_query::domains::discovery::Discovery
ADDED    finance_query::domains::discovery::Discovery::search
ADDED    finance_query::domains::economic::EconomicCatalog
ADDED    finance_query::domains::economic::EconomicIndicator::series
ADDED    finance_query::domains::filings::Filings::get
ADDED    finance_query::domains::filings::Filings::search_all
ADDED    finance_query::domains::forex::ForexPair::quote
ADDED    finance_query::domains::futures::FuturesContract::commitments_of_traders
ADDED    finance_query::domains::indices::Index::constituents
ADDED    finance_query::domains::market::Market
ADDED    finance_query::domains::market::Market::crypto_global
ADDED    finance_query::domains::market::Market::crypto_trending
ADDED    finance_query::domains::market::Market::grouped_daily
ADDED    finance_query::domains::market::MarketCalendar
ADDED    finance_query::domains::snapshot::Snapshot
ADDED    finance_query::finance::fear_and_greed_crypto
ADDED    finance_query::gdelt
ADDED    finance_query::gdelt::News
ADDED    finance_query::gdelt::news
ADDED    finance_query::indicators::FibonacciLevels
ADDED    finance_query::indicators::PivotPoints
ADDED    finance_query::indicators::ZigZagPoint
ADDED    finance_query::indicators::fibonacci_pivot_points
ADDED    finance_query::indicators::fibonacci_retracement
ADDED    finance_query::indicators::fibonacci_retracement::FibonacciLevels
ADDED    finance_query::indicators::fibonacci_retracement::fibonacci_retracement
ADDED    finance_query::indicators::heikin_ashi
ADDED    finance_query::indicators::heikin_ashi::heikin_ashi
ADDED    finance_query::indicators::pivot_points
ADDED    finance_query::indicators::pivot_points::PivotPoints
ADDED    finance_query::indicators::pivot_points::fibonacci_pivot_points
ADDED    finance_query::indicators::pivot_points::pivot_points
ADDED    finance_query::indicators::zigzag
ADDED    finance_query::indicators::zigzag::ZigZagPoint
ADDED    finance_query::indicators::zigzag::zigzag
ADDED    finance_query::models::calendar::market::CalendarDetail
ADDED    finance_query::models::calendar::market::CalendarKind
ADDED    finance_query::models::calendar::market::MarketCalendarEntry
ADDED    finance_query::models::chart::data::Chart::pivot_points
ADDED    finance_query::models::corporate::governance::EmployeeCount
ADDED    finance_query::models::corporate::governance::ExecutiveCompensation
ADDED    finance_query::models::corporate::press_release::PressRelease
ADDED    finance_query::models::crypto::GlobalCryptoStats
ADDED    finance_query::models::crypto::TrendingCoin
ADDED    finance_query::models::crypto::defi::ChainAllocation
ADDED    finance_query::models::crypto::defi::ChainTvl
ADDED    finance_query::models::crypto::defi::ProtocolTvl
ADDED    finance_query::models::crypto::defi::StablecoinSupply
ADDED    finance_query::models::crypto::defi::TvlPoint
ADDED    finance_query::models::discovery::figi::SecurityIdKind
ADDED    finance_query::models::discovery::figi::SecurityMapping
ADDED    finance_query::models::discovery::reference::ExchangeInfo
ADDED    finance_query::models::discovery::reference::ScreenerFilters
ADDED    finance_query::models::discovery::reference::ScreenerFilters::new
ADDED    finance_query::models::discovery::reference::ScreenerMatch
ADDED    finance_query::models::discovery::reference::SymbolDetails
ADDED    finance_query::models::discovery::reference::SymbolMatch
ADDED    finance_query::models::economic::catalog::EconomicCategory
ADDED    finance_query::models::economic::catalog::EconomicRelease
ADDED    finance_query::models::economic::catalog::EconomicSeriesMatch
ADDED    finance_query::models::filings::full_text::FilingSearchFilters
ADDED    finance_query::models::filings::full_text::FilingSearchHit
ADDED    finance_query::models::filings::ownership::InsiderTrade
ADDED    finance_query::models::filings::ownership::InstitutionalHolding
ADDED    finance_query::models::filings::sections::FilingSection
ADDED    finance_query::models::filings::sections::FilingSectionForm
ADDED    finance_query::models::filings::sections::RiskFactor
ADDED    finance_query::models::fundamentals::consensus::PriceTargetConsensus
ADDED    finance_query::models::fundamentals::consensus::PriceTargetSummary
ADDED    finance_query::models::fundamentals::consensus::RatingConsensus
ADDED    finance_query::models::fundamentals::etf::EtfHolding
ADDED    finance_query::models::fundamentals::etf::EtfProfile
ADDED    finance_query::models::fundamentals::short_activity::ShareFloat
ADDED    finance_query::models::fundamentals::short_activity::ShortInterest
ADDED    finance_query::models::fundamentals::short_activity::ShortVolume
ADDED    finance_query::models::fundamentals::ttm::FinancialRatiosTtm
ADDED    finance_query::models::fundamentals::ttm::KeyMetricsTtm
ADDED    finance_query::models::futures::cot::CommitmentsOfTraders
ADDED    finance_query::models::futures::cot::CotObservation
ADDED    finance_query::models::indices::IndexConstituent
ADDED    finance_query::models::indices::IndexConstituentChange
ADDED    finance_query::models::indices::MajorIndex
ADDED    finance_query::models::market::performance::IndustryPe
ADDED    finance_query::models::market::performance::MoverDirection
ADDED    finance_query::models::market::performance::MoverQuote
ADDED    finance_query::models::market::performance::SectorPe
ADDED    finance_query::models::market::performance::SectorPerformance
ADDED    finance_query::models::market::performance::SectorPerformanceHistory
ADDED    finance_query::models::quote::response::QuoteSummaryResponse
ADDED    finance_query::models::quote::snapshot::AssetClass
ADDED    finance_query::models::quote::snapshot::MarketSnapshot
ADDED    finance_query::openfigi
ADDED    finance_query::openfigi::SecurityIdKind
ADDED    finance_query::openfigi::SecurityMapping
ADDED    finance_query::openfigi::resolve
ADDED    finance_query::openfigi::resolve_cusip
ADDED    finance_query::openfigi::resolve_isin
ADDED    finance_query::openfigi::resolve_many
ADDED    finance_query::openfigi::resolve_sedol
ADDED    finance_query::providers::Capability::name
ADDED    finance_query::providers::Routes
ADDED    finance_query::providers::config::Providers::calendar
ADDED    finance_query::providers::config::Providers::discovery
ADDED    finance_query::providers::config::Providers::economic_catalog
ADDED    finance_query::providers::config::Providers::market
ADDED    finance_query::providers::config::Providers::snapshot
ADDED    finance_query::providers::config::ProvidersBuilder::retry
ADDED    finance_query::providers::health::ProviderHealth
ADDED    finance_query::providers::retry::RetryPolicy
ADDED    finance_query::providers::retry::RetryPolicy::base_delay
ADDED    finance_query::providers::retry::RetryPolicy::jitter
ADDED    finance_query::providers::retry::RetryPolicy::max_delay
ADDED    finance_query::providers::retry::RetryPolicy::max_retry_after
ADDED    finance_query::providers::retry::RetryPolicy::multiplier
ADDED    finance_query::risk::cvar::historical_cvar
ADDED    finance_query::risk::cvar::parametric_cvar
ADDED    finance_query::risk::historical_cvar
ADDED    finance_query::risk::information_ratio
ADDED    finance_query::risk::kelly_criterion
ADDED    finance_query::risk::omega_ratio
ADDED    finance_query::risk::parametric_cvar
ADDED    finance_query::risk::ratios::information_ratio
ADDED    finance_query::risk::ratios::kelly_criterion
ADDED    finance_query::risk::ratios::omega_ratio
ADDED    finance_query::risk::ratios::tracking_error
ADDED    finance_query::risk::ratios::ulcer_index
ADDED    finance_query::risk::ratios::win_loss_stats
ADDED    finance_query::risk::tracking_error
ADDED    finance_query::risk::ulcer_index
ADDED    finance_query::risk::win_loss_stats
ADDED    finance_query::streaming::AlertCondition
ADDED    finance_query::streaming::AlertConditionKind
ADDED    finance_query::streaming::AlertEvaluator
ADDED    finance_query::streaming::AlertEvent
ADDED    finance_query::streaming::AlertExt
ADDED    finance_query::streaming::AlertRule
ADDED    finance_query::streaming::AlertStream
ADDED    finance_query::streaming::AssetClass
ADDED    finance_query::streaming::Batched
ADDED    finance_query::streaming::BookLevel
ADDED    finance_query::streaming::DepthStream
ADDED    finance_query::streaming::DepthStreamBuilder
ADDED    finance_query::streaming::EconomicStream
ADDED    finance_query::streaming::EconomicStreamBuilder
ADDED    finance_query::streaming::Greeks
ADDED    finance_query::streaming::OptionContractUpdate
ADDED    finance_query::streaming::OptionsChainStream
ADDED    finance_query::streaming::OptionsChainStreamBuilder
ADDED    finance_query::streaming::OrderBookUpdate
ADDED    finance_query::streaming::PriceSource
ADDED    finance_query::streaming::SeriesUpdate
ADDED    finance_query::streaming::StreamBatchExt
ADDED    finance_query::streaming::TradeStream
ADDED    finance_query::streaming::TradeStreamBuilder
ADDED    finance_query::streaming::TradeTick
ADDED    finance_query::streaming::alerts::AlertCondition
ADDED    finance_query::streaming::alerts::AlertConditionKind
ADDED    finance_query::streaming::alerts::AlertEvaluator
ADDED    finance_query::streaming::alerts::AlertEvent
ADDED    finance_query::streaming::alerts::AlertExt
ADDED    finance_query::streaming::alerts::AlertRule
ADDED    finance_query::streaming::alerts::AlertStream
ADDED    finance_query::streaming::batch::Batched
ADDED    finance_query::streaming::batch::StreamBatchExt
ADDED    finance_query::streaming::book::BookLevel
ADDED    finance_query::streaming::book::DepthStream
ADDED    finance_query::streaming::book::DepthStreamBuilder
ADDED    finance_query::streaming::book::DepthStreamBuilder::max_reconnect_attempts
ADDED    finance_query::streaming::book::OrderBookUpdate
ADDED    finance_query::streaming::client::PriceSource
ADDED    finance_query::streaming::client::PriceStreamBuilder::max_reconnect_attempts
ADDED    finance_query::streaming::economic::EconomicStream
ADDED    finance_query::streaming::economic::EconomicStreamBuilder
ADDED    finance_query::streaming::economic::SeriesUpdate
ADDED    finance_query::streaming::options::Greeks
ADDED    finance_query::streaming::options::OptionContractUpdate
ADDED    finance_query::streaming::options::OptionsChainStream
ADDED    finance_query::streaming::options::OptionsChainStreamBuilder
ADDED    finance_query::streaming::options::OptionsChainStreamBuilder::greeks_refresh
ADDED    finance_query::streaming::options::OptionsChainStreamBuilder::max_reconnect_attempts
ADDED    finance_query::streaming::polygon::AssetClass
ADDED    finance_query::streaming::trades::TradeStream
ADDED    finance_query::streaming::trades::TradeStreamBuilder
ADDED    finance_query::streaming::trades::TradeStreamBuilder::build
ADDED    finance_query::streaming::trades::TradeStreamBuilder::max_reconnect_attempts
ADDED    finance_query::streaming::trades::TradeTick
ADDED    finance_query::ticker::core::Ticker::calendar
ADDED    finance_query::ticker::core::Ticker::news
ADDED    finance_query::ticker::core::Ticker::recommendations
ADDED    finance_query::ticker::core::TickerBuilder::cache
ADDED    finance_query::ticker::core::TickerBuilder::no_cache
ADDED    finance_query::tickers::core::Tickers::client_handle
REMOVED  finance_query::streaming::pricing::OptionTypeProto
CHANGED  finance_query::CryptoCoin (body)
CHANGED  finance_query::FinanceError (body)
CHANGED  finance_query::Frequency (body)
CHANGED  finance_query::FuturesContract (body)
CHANGED  finance_query::Indicator (body)
CHANGED  finance_query::IndicatorResult (body)
CHANGED  finance_query::IndicatorsSummary (body)
CHANGED  finance_query::IndicesRegion (body)
CHANGED  finance_query::Industry (body)
CHANGED  finance_query::Interval (body)
CHANGED  finance_query::Operation (body)
CHANGED  finance_query::Provider (body)
CHANGED  finance_query::ProvidersBuilder (body)
CHANGED  finance_query::QuoteType (body)
CHANGED  finance_query::Region (body)
CHANGED  finance_query::Screener (body)
CHANGED  finance_query::Sector (body)
CHANGED  finance_query::SortType (body)
CHANGED  finance_query::StatementType (body)
CHANGED  finance_query::Ticker (body)
CHANGED  finance_query::TickerBuilder (body)
CHANGED  finance_query::Tickers (body)
CHANGED  finance_query::TickersBuilder (body)
CHANGED  finance_query::TimeRange (body)
CHANGED  finance_query::ValueFormat (body)
CHANGED  finance_query::adapters::fred::series (body)
CHANGED  finance_query::backtesting::BenchmarkMetrics (body)
CHANGED  finance_query::backtesting::Signal (body)
CHANGED  finance_query::backtesting::SignalRecord (body)
CHANGED  finance_query::backtesting::Strategy (body)
CHANGED  finance_query::backtesting::StrategyContext (body)
CHANGED  finance_query::backtesting::condition::Condition (body)
CHANGED  finance_query::backtesting::engine::BacktestEngine::run_with_dividends (body)
CHANGED  finance_query::backtesting::refs::HtfCondition (body)
CHANGED  finance_query::backtesting::refs::htf (body)
CHANGED  finance_query::backtesting::refs::htf::HtfCondition (body)
CHANGED  finance_query::backtesting::refs::htf::htf (body)
CHANGED  finance_query::backtesting::refs::htf::htf_region (body)
CHANGED  finance_query::backtesting::refs::htf_region (body)
CHANGED  finance_query::backtesting::result::BenchmarkMetrics (body)
CHANGED  finance_query::backtesting::result::SignalRecord (body)
CHANGED  finance_query::backtesting::signal::Signal (body)
CHANGED  finance_query::backtesting::strategy::Strategy (body)
CHANGED  finance_query::backtesting::strategy::StrategyContext (body)
CHANGED  finance_query::backtesting::walk_forward::WalkForwardConfig::run (body)
CHANGED  finance_query::constants::Frequency (body)
CHANGED  finance_query::constants::Interval (body)
CHANGED  finance_query::constants::Interval::as_str (body)
CHANGED  finance_query::constants::Region (body)
CHANGED  finance_query::constants::StatementType (body)
CHANGED  finance_query::constants::TimeRange (body)
CHANGED  finance_query::constants::ValueFormat (body)
CHANGED  finance_query::constants::indices::Region (body)
CHANGED  finance_query::constants::industries::Industry (body)
CHANGED  finance_query::constants::screeners::Screener (body)
CHANGED  finance_query::constants::sectors::Sector (body)
CHANGED  finance_query::domains::CryptoCoin (body)
CHANGED  finance_query::domains::FuturesContract (body)
CHANGED  finance_query::domains::commodities::Commodity::quote (body)
CHANGED  finance_query::domains::crypto::CryptoCoin (body)
CHANGED  finance_query::domains::crypto::CryptoCoin::quote (body)
CHANGED  finance_query::domains::futures::FuturesContract (body)
CHANGED  finance_query::error::FinanceError (body)
CHANGED  finance_query::finance::currencies (body)
CHANGED  finance_query::finance::custom_screener (body)
CHANGED  finance_query::finance::earnings_transcript (body)
CHANGED  finance_query::finance::earnings_transcripts (body)
CHANGED  finance_query::finance::hours (body)
CHANGED  finance_query::finance::industry (body)
CHANGED  finance_query::finance::lookup (body)
CHANGED  finance_query::finance::market_summary (body)
CHANGED  finance_query::finance::screener (body)
CHANGED  finance_query::finance::search (body)
CHANGED  finance_query::finance::sector (body)
CHANGED  finance_query::finance::trending (body)
CHANGED  finance_query::fred::series (body)
CHANGED  finance_query::indicators::Indicator (body)
CHANGED  finance_query::indicators::Indicator::warmup_bars (body)
CHANGED  finance_query::indicators::IndicatorResult (body)
CHANGED  finance_query::indicators::IndicatorType (body)
CHANGED  finance_query::indicators::IndicatorsSummary (body)
CHANGED  finance_query::indicators::summary::IndicatorsSummary (body)
CHANGED  finance_query::indicators::vwma (body)
CHANGED  finance_query::indicators::vwma::vwma (body)
CHANGED  finance_query::models::discovery::screeners::query::QuoteType (body)
CHANGED  finance_query::models::discovery::screeners::query::SortType (body)
CHANGED  finance_query::providers::Operation (body)
CHANGED  finance_query::providers::Operation::capability (body)
CHANGED  finance_query::providers::Provider (body)
CHANGED  finance_query::providers::config::ProvidersBuilder (body)
CHANGED  finance_query::risk::RiskSummary (body)
CHANGED  finance_query::risk::historical_var (body)
CHANGED  finance_query::risk::parametric_var (body)
CHANGED  finance_query::risk::ratios::sharpe_ratio (body)
CHANGED  finance_query::risk::sharpe_ratio (body)
CHANGED  finance_query::risk::var::historical_var (body)
CHANGED  finance_query::risk::var::parametric_var (body)
CHANGED  finance_query::streaming::PriceStream (body)
CHANGED  finance_query::streaming::PriceStreamBuilder (body)
CHANGED  finance_query::streaming::client::PriceStream (body)
CHANGED  finance_query::streaming::client::PriceStreamBuilder (body)
CHANGED  finance_query::ticker::core::Ticker (body)
CHANGED  finance_query::ticker::core::Ticker::edgar_company_facts (body)
CHANGED  finance_query::ticker::core::Ticker::edgar_submissions (body)
CHANGED  finance_query::ticker::core::Ticker::filings (body)
CHANGED  finance_query::ticker::core::Ticker::financials (body)
CHANGED  finance_query::ticker::core::Ticker::quote (body)
CHANGED  finance_query::ticker::core::Ticker::risk (body)
CHANGED  finance_query::ticker::core::TickerBuilder (body)
CHANGED  finance_query::ticker::core::TickerBuilder::build (body)
CHANGED  finance_query::tickers::core::Tickers (body)
CHANGED  finance_query::tickers::core::Tickers::charts (body)
CHANGED  finance_query::tickers::core::TickersBuilder (body)
```

### Performance

| item | instructions | median | p99 | allocs | polls |
|---|---:|---:|---:|---:|---:|
| `finance_query::bt_base_to_htf_index` | n/a | 21.36µs | 22.24µs | 1 | n/a |
| `finance_query::bt_bayesian_search` | n/a | 2.31ms | 2.34ms | 2382 | n/a |
| `finance_query::bt_grid_search` | n/a | 509.24µs | 582.21µs | 3012 | n/a |
| `finance_query::bt_monte_carlo` | n/a | 140.37µs | 142.30µs | 6 | n/a |
| `finance_query::bt_resample` | n/a | 90.30µs | 93.46µs | 11 | n/a |
| `finance_query::bt_sma_crossover` | n/a | 755.38µs | 813.83µs | 2373 | n/a |
| `finance_query::bt_strategy_builder` | n/a | 130.88µs | 140.01µs | 330 | n/a |
| `finance_query::capability_name` | n/a | 2.4ns | 2.6ns | 0 | n/a |
| `finance_query::cond_bench_always_false` | n/a | 18.45µs | 18.58µs | 6 | n/a |
| `finance_query::cond_bench_always_true` | n/a | 348.28µs | 360.98µs | 4045 | n/a |
| `finance_query::cond_bench_has_position` | n/a | 346.01µs | 352.49µs | 4045 | n/a |
| `finance_query::cond_bench_held_for_bars` | n/a | 131.51µs | 150.50µs | 1041 | n/a |
| `finance_query::cond_bench_in_loss` | n/a | 349.81µs | 391.68µs | 4045 | n/a |
| `finance_query::cond_bench_in_profit` | n/a | 40.29µs | 45.54µs | 142 | n/a |
| `finance_query::cond_bench_is_long` | n/a | 347.62µs | 353.98µs | 4045 | n/a |
| `finance_query::cond_bench_is_short` | n/a | 27.81µs | 28.52µs | 13 | n/a |
| `finance_query::cond_bench_no_position` | n/a | 27.90µs | 28.31µs | 13 | n/a |
| `finance_query::cond_bench_stop_loss` | n/a | 34.86µs | 39.22µs | 24 | n/a |
| `finance_query::cond_bench_take_profit` | n/a | 33.68µs | 34.69µs | 13 | n/a |
| `finance_query::cond_bench_trailing_stop` | n/a | 40.44µs | 42.33µs | 92 | n/a |
| `finance_query::cond_bench_trailing_take_profit` | n/a | 40.01µs | 42.25µs | 72 | n/a |
| `finance_query::de_chart` | n/a | 7.78µs | 7.89µs | 13 | n/a |
| `finance_query::de_crypto_coins` | n/a | 25.13µs | 25.57µs | 205 | n/a |
| `finance_query::de_currencies` | n/a | 46.40µs | 47.89µs | 647 | n/a |
| `finance_query::de_edgar_facts` | n/a | 1.16ms | 1.22ms | 11899 | n/a |
| `finance_query::de_edgar_submissions` | n/a | 360.25µs | 365.08µs | 6784 | n/a |
| `finance_query::de_fear_and_greed` | n/a | 95.5ns | 98.7ns | 0 | n/a |
| `finance_query::de_fear_and_greed_crypto_history` | n/a | 1.11µs | 1.93µs | 3 | n/a |
| `finance_query::de_financials` | n/a | 17.06µs | 17.33µs | 206 | n/a |
| `finance_query::de_fred_series` | n/a | 86.46µs | 89.35µs | 873 | n/a |
| `finance_query::de_hours` | n/a | 609.1ns | 632.4ns | 10 | n/a |
| `finance_query::de_market_summary` | n/a | 25.39µs | 25.60µs | 228 | n/a |
| `finance_query::de_news` | n/a | 3.22µs | 3.27µs | 53 | n/a |
| `finance_query::de_options` | n/a | 845.4ns | 931.7ns | 6 | n/a |
| `finance_query::de_quote` | n/a | 860.05µs | 882.45µs | 8235 | n/a |
| `finance_query::de_screener` | n/a | 317.47µs | 334.38µs | 4498 | n/a |
| `finance_query::de_search` | n/a | 4.28µs | 4.50µs | 55 | n/a |
| `finance_query::de_treasury_yields` | n/a | 63.10µs | 65.24µs | 119 | n/a |
| `finance_query::de_trending` | n/a | 1.09µs | 1.12µs | 24 | n/a |
| `finance_query::dispatch_select` | n/a | 3.2ns | 3.5ns | 0 | n/a |
| `finance_query::ind_accumulation_distribution` | n/a | 3.32µs | 3.73µs | 1 | n/a |
| `finance_query::ind_alma` | n/a | 4.31µs | 4.43µs | 2 | n/a |
| `finance_query::ind_aroon` | n/a | 9.26µs | 9.48µs | 9 | n/a |
| `finance_query::ind_atr` | n/a | 8.48µs | 8.53µs | 2 | n/a |
| `finance_query::ind_awesome_oscillator` | n/a | 7.35µs | 7.39µs | 4 | n/a |
| `finance_query::ind_balance_of_power` | n/a | 1.71µs | 1.74µs | 2 | n/a |
| `finance_query::ind_bull_bear_power` | n/a | 6.02µs | 6.23µs | 3 | n/a |
| `finance_query::ind_cci` | n/a | 14.68µs | 14.74µs | 2 | n/a |
| `finance_query::ind_chaikin_oscillator` | n/a | 13.94µs | 14.04µs | 4 | n/a |
| `finance_query::ind_choppiness_index` | n/a | 16.35µs | 17.43µs | 8 | n/a |
| `finance_query::ind_cmf` | n/a | 3.50µs | 3.53µs | 2 | n/a |
| `finance_query::ind_cmo` | n/a | 4.50µs | 4.65µs | 2 | n/a |
| `finance_query::ind_coppock_curve` | n/a | 6.85µs | 8.85µs | 3 | n/a |
| `finance_query::ind_dema` | n/a | 10.31µs | 10.85µs | 3 | n/a |
| `finance_query::ind_donchian_channels` | n/a | 8.62µs | 8.93µs | 9 | n/a |
| `finance_query::ind_elder_ray` | n/a | 6.03µs | 6.10µs | 3 | n/a |
| `finance_query::ind_fibonacci_pivot_points` | n/a | 2.19µs | 2.23µs | 1 | n/a |
| `finance_query::ind_fibonacci_retracement` | n/a | 8.74µs | 8.96µs | 9 | n/a |
| `finance_query::ind_heikin_ashi` | n/a | 12.77µs | 13.18µs | 9 | n/a |
| `finance_query::ind_hma` | n/a | 11.88µs | 11.93µs | 5 | n/a |
| `finance_query::ind_ichimoku` | n/a | 25.75µs | 26.99µs | 27 | n/a |
| `finance_query::ind_keltner_channels` | n/a | 14.76µs | 14.84µs | 5 | n/a |
| `finance_query::ind_last_value` | n/a | 2.7ns | 3.0ns | 0 | n/a |
| `finance_query::ind_macd` | n/a | 17.26µs | 17.82µs | 7 | n/a |
| `finance_query::ind_mcginley_dynamic` | n/a | 10.73µs | 10.84µs | 1 | n/a |
| `finance_query::ind_mfi` | n/a | 5.47µs | 6.02µs | 3 | n/a |
| `finance_query::ind_momentum` | n/a | 102.66µs | 106.19µs | 52 | n/a |
| `finance_query::ind_momentum_single` | n/a | 459.7ns | 465.8ns | 1 | n/a |
| `finance_query::ind_moving_averages` | n/a | 74.63µs | 75.62µs | 27 | n/a |
| `finance_query::ind_parabolic_sar` | n/a | 2.01µs | 2.06µs | 1 | n/a |
| `finance_query::ind_patterns` | n/a | 79.26µs | 82.45µs | 1 | n/a |
| `finance_query::ind_pivot_points` | n/a | 2.51µs | 2.54µs | 1 | n/a |
| `finance_query::ind_roc` | n/a | 1.45µs | 1.54µs | 1 | n/a |
| `finance_query::ind_sma` | n/a | 331.36µs | 355.63µs | 1 | n/a |
| `finance_query::ind_stochastic` | n/a | 13.02µs | 13.39µs | 11 | n/a |
| `finance_query::ind_stochastic_rsi` | n/a | 21.43µs | 22.51µs | 12 | n/a |
| `finance_query::ind_supertrend` | n/a | 11.38µs | 11.90µs | 4 | n/a |
| `finance_query::ind_tema` | n/a | 15.23µs | 15.33µs | 4 | n/a |
| `finance_query::ind_trend` | n/a | 71.50µs | 73.76µs | 48 | n/a |
| `finance_query::ind_true_range` | n/a | 1.07µs | 1.09µs | 1 | n/a |
| `finance_query::ind_volatility` | n/a | 55.24µs | 56.91µs | 28 | n/a |
| `finance_query::ind_volume` | n/a | 34.56µs | 34.84µs | 14 | n/a |
| `finance_query::ind_vwap` | n/a | 3.39µs | 3.41µs | 1 | n/a |
| `finance_query::ind_vwma` | n/a | 2.54µs | 2.57µs | 1 | n/a |
| `finance_query::ind_williams_r` | n/a | 8.49µs | 8.69µs | 7 | n/a |
| `finance_query::ind_wma` | n/a | 4.01µs | 4.02µs | 7 | n/a |
| `finance_query::ind_zigzag` | n/a | 1.40µs | 1.46µs | 4 | n/a |
| `finance_query::ref_bench_accumulation_distribution` | n/a | 39.54µs | 40.87µs | 23 | n/a |
| `finance_query::ref_bench_adx` | n/a | 423.61µs | 475.43µs | 4930 | n/a |
| `finance_query::ref_bench_alma` | n/a | 452.02µs | 535.59µs | 5019 | n/a |
| `finance_query::ref_bench_aroon` | n/a | 432.21µs | 441.80µs | 4950 | n/a |
| `finance_query::ref_bench_atr` | n/a | 425.20µs | 432.45µs | 4996 | n/a |
| `finance_query::ref_bench_awesome_oscillator` | n/a | 265.14µs | 274.66µs | 2636 | n/a |
| `finance_query::ref_bench_balance_of_power` | n/a | 43.97µs | 44.65µs | 25 | n/a |
| `finance_query::ref_bench_bear_power` | n/a | 113.53µs | 117.75µs | 806 | n/a |
| `finance_query::ref_bench_bollinger` | n/a | 447.02µs | 450.62µs | 4968 | n/a |
| `finance_query::ref_bench_bull_power` | n/a | 391.53µs | 399.82µs | 4238 | n/a |
| `finance_query::ref_bench_candle_body` | n/a | 18.58µs | 18.90µs | 6 | n/a |
| `finance_query::ref_bench_candle_range` | n/a | 401.88µs | 409.24µs | 5043 | n/a |
| `finance_query::ref_bench_cci` | n/a | 251.05µs | 256.34µs | 2605 | n/a |
| `finance_query::ref_bench_chaikin_oscillator` | n/a | 53.90µs | 54.99µs | 26 | n/a |
| `finance_query::ref_bench_choppiness_index` | n/a | 434.92µs | 440.70µs | 5002 | n/a |
| `finance_query::ref_bench_close` | n/a | 400.03µs | 405.57µs | 5043 | n/a |
| `finance_query::ref_bench_cmf` | n/a | 39.65µs | 40.43µs | 25 | n/a |
| `finance_query::ref_bench_cmo` | n/a | 236.48µs | 240.75µs | 2598 | n/a |
| `finance_query::ref_bench_coppock_curve` | n/a | 252.71µs | 273.21µs | 2559 | n/a |
| `finance_query::ref_bench_dema` | n/a | 414.40µs | 417.79µs | 4875 | n/a |
| `finance_query::ref_bench_donchian` | n/a | 450.42µs | 470.50µs | 4976 | n/a |
| `finance_query::ref_bench_elder_bear_power` | n/a | 113.31µs | 116.77µs | 806 | n/a |
| `finance_query::ref_bench_elder_bull_power` | n/a | 389.51µs | 427.78µs | 4238 | n/a |
| `finance_query::ref_bench_ema` | n/a | 413.63µs | 535.95µs | 4963 | n/a |
| `finance_query::ref_bench_gap_pct` | n/a | 212.89µs | 222.08µs | 2489 | n/a |
| `finance_query::ref_bench_high` | n/a | 394.82µs | 403.11µs | 5043 | n/a |
| `finance_query::ref_bench_hma` | n/a | 422.98µs | 430.78µs | 4957 | n/a |
| `finance_query::ref_bench_htf` | n/a | 476.13µs | 494.40µs | 7039 | n/a |
| `finance_query::ref_bench_htf_region` | n/a | 479.26µs | 484.90µs | 7039 | n/a |
| `finance_query::ref_bench_ichimoku` | n/a | 471.83µs | 490.12µs | 4836 | n/a |
| `finance_query::ref_bench_ichimoku_custom` | n/a | 471.37µs | 485.55µs | 4836 | n/a |
| `finance_query::ref_bench_is_bearish` | n/a | 18.49µs | 18.62µs | 6 | n/a |
| `finance_query::ref_bench_is_bullish` | n/a | 18.50µs | 18.98µs | 6 | n/a |
| `finance_query::ref_bench_keltner` | n/a | 451.36µs | 460.32µs | 4972 | n/a |
| `finance_query::ref_bench_low` | n/a | 397.42µs | 452.71µs | 5043 | n/a |
| `finance_query::ref_bench_macd` | n/a | 261.96µs | 277.22µs | 2541 | n/a |
| `finance_query::ref_bench_mcginley` | n/a | 425.91µs | 442.92µs | 4963 | n/a |
| `finance_query::ref_bench_median_price` | n/a | 400.75µs | 406.55µs | 5043 | n/a |
| `finance_query::ref_bench_mfi` | n/a | 425.05µs | 535.53µs | 4998 | n/a |
| `finance_query::ref_bench_momentum` | n/a | 234.39µs | 236.33µs | 2552 | n/a |
| `finance_query::ref_bench_obv` | n/a | 48.16µs | 51.04µs | 30 | n/a |
| `finance_query::ref_bench_open` | n/a | 394.80µs | 404.12µs | 5043 | n/a |
| `finance_query::ref_bench_parabolic_sar` | n/a | 449.43µs | 456.34µs | 5056 | n/a |
| `finance_query::ref_bench_price` | n/a | 400.10µs | 415.68µs | 5043 | n/a |
| `finance_query::ref_bench_price_change_pct` | n/a | 219.66µs | 222.77µs | 2489 | n/a |
| `finance_query::ref_bench_relative_volume` | n/a | 431.05µs | 444.08µs | 4949 | n/a |
| `finance_query::ref_bench_roc` | n/a | 236.21µs | 240.72µs | 2587 | n/a |
| `finance_query::ref_bench_rsi` | n/a | 416.56µs | 424.46µs | 4994 | n/a |
| `finance_query::ref_bench_sma` | n/a | 411.34µs | 427.55µs | 4963 | n/a |
| `finance_query::ref_bench_stochastic` | n/a | 451.23µs | 467.91µs | 4987 | n/a |
| `finance_query::ref_bench_stochastic_rsi` | n/a | 458.08µs | 466.71µs | 4831 | n/a |
| `finance_query::ref_bench_supertrend` | n/a | 457.69µs | 493.28µs | 5021 | n/a |
| `finance_query::ref_bench_tema` | n/a | 415.54µs | 418.67µs | 4781 | n/a |
| `finance_query::ref_bench_true_range` | n/a | 422.88µs | 430.92µs | 5059 | n/a |
| `finance_query::ref_bench_typical_price` | n/a | 426.82µs | 432.08µs | 5043 | n/a |
| `finance_query::ref_bench_volume` | n/a | 399.49µs | 403.46µs | 5043 | n/a |
| `finance_query::ref_bench_vwap` | n/a | 424.37µs | 435.65µs | 5060 | n/a |
| `finance_query::ref_bench_vwma` | n/a | 415.81µs | 423.91µs | 4964 | n/a |
| `finance_query::ref_bench_williams_r` | n/a | 47.66µs | 48.73µs | 29 | n/a |
| `finance_query::ref_bench_wma` | n/a | 413.72µs | 425.42µs | 4969 | n/a |
| `finance_query::risk_beta` | n/a | 53.52µs | 53.72µs | 0 | n/a |
| `finance_query::risk_calmar` | n/a | 12.6ns | 12.7ns | 0 | n/a |
| `finance_query::risk_historical_cvar` | n/a | 243.86µs | 251.31µs | 2 | n/a |
| `finance_query::risk_historical_var` | n/a | 2.16ms | 2.19ms | 2 | n/a |
| `finance_query::risk_information_ratio` | n/a | 30.21µs | 30.35µs | 1 | n/a |
| `finance_query::risk_kelly_criterion` | n/a | 3.26µs | 3.32µs | 16 | n/a |
| `finance_query::risk_max_drawdown` | n/a | 7.02µs | 7.06µs | 1 | n/a |
| `finance_query::risk_omega_ratio` | n/a | 26.77µs | 27.05µs | 0 | n/a |
| `finance_query::risk_parametric_cvar` | n/a | 26.79µs | 27.07µs | 0 | n/a |
| `finance_query::risk_parametric_var` | n/a | 26.79µs | 26.88µs | 0 | n/a |
| `finance_query::risk_sharpe` | n/a | 26.78µs | 27.06µs | 0 | n/a |
| `finance_query::risk_sortino` | n/a | 26.78µs | 27.41µs | 0 | n/a |
| `finance_query::risk_tracking_error` | n/a | 30.20µs | 30.39µs | 1 | n/a |
| `finance_query::risk_ulcer_index` | n/a | 157.18µs | 157.92µs | 2 | n/a |
| `finance_query::risk_win_loss_stats` | n/a | 49.07µs | 51.96µs | 25 | n/a |
| `finance_query::rss_parse` | n/a | 3.99µs | 4.07µs | 48 | n/a |
| `finance_query::score_news` | n/a | 57.07µs | 57.77µs | 976 | n/a |
| `finance_query::score_transcript` | n/a | 3.69ms | 3.76ms | 42100 | n/a |
| `finance_query::ser_currencies` | n/a | 12.34µs | 12.52µs | 8 | n/a |
| `finance_query::stream_deserialize` | n/a | 1.00µs | 1.07µs | 4 | n/a |
| `finance_query::stream_serialize` | n/a | 798.9ns | 825.3ns | 4 | n/a |
| `finance_query::ticker_chart_then_cached` | n/a | 1.10µs | 1.12µs | 20 | n/a |
| `finance_query::ticker_quote_then_cached` | n/a | 8.91µs | 9.02µs | 4 | n/a |
| `finance_query::tickers_batch_quote_then_cached` | n/a | 7.88µs | 8.20µs | 16 | n/a |
| `finance_query::translate_dictionary` | n/a | 1.55µs | 1.58µs | 49 | n/a |
| `finance_query::translation_set_backend` | n/a | 18.3ns | 19.8ns | 1 | n/a |
| `finance_query::translation_translate` | n/a | 2.72µs | 2.75µs | 79 | n/a |
| `finance_query::translation_translate_with` | n/a | 2.68µs | 2.81µs | 77 | n/a |


## [2.8.0] - 2026-07-10

Domain handles (`ForexPair`, `CryptoCoin`, `Index`, `FuturesContract`,
`Commodity`) gain chart/history, indicators, and risk analytics, moving them
toward parity with `Ticker`. A new financial event calendar aggregates
earnings, dividends, options expirations, and (with `fred`) economic releases
across symbols. The `streaming` module gains `NewsStream`, a polled RSS/Atom
counterpart to `PriceStream`. Several dependencies pulling in outsized
transitive weight (`feed-rs`, `scraper`, `governor`, `indicatif`) were
replaced with small hand-rolled implementations or removed outright. Includes
a handful of breaking API changes — see Changed/Removed below.

### Added

- **`chart()` / `history()` on domain handles** — `ForexPair`, `CryptoCoin`,
  `Index`, `FuturesContract`, and `Commodity` now expose historical OHLCV data,
  bringing them toward parity with `Ticker`. Both route through
  `Capability::CHART` (Yahoo by default) and cache per `(symbol, interval, range)`
  when `.cache(ttl)` is set. `history(range)` is sugar for
  `chart(range.default_interval(), range)`.
  - `CryptoCoin::chart`/`history` take a `vs_currency` and build the chart symbol
    as `"{ID}-{VS}"` (e.g. `"BTC-USD"`), which resolves on the default Yahoo route
    for ticker-style ids; CoinGecko-id coins should route `Capability::CHART` to a
    crypto-aware provider.
- **`TimeRange::default_interval()`** — the per-range default candle interval used
  by `history()` (finer granularity for short ranges, coarser for long ones).
- **`indicators()` / `indicator()` / `risk()` on domain handles** — `ForexPair`,
  `CryptoCoin`, `Index`, `FuturesContract`, and `Commodity` now expose the same
  technical-indicator (`indicators` feature) and risk (`risk` feature) analytics
  as `Ticker`, computed over each handle's cached chart. `CryptoCoin` takes a
  leading `vs_currency` argument on all three (matching its `chart()`).
  - `risk()` annualizes with the handle's asset-class trading calendar — 252 days
    for exchange-traded (index/futures/commodity), ~260 for forex, 365 for crypto
    (24/7) — and intraday intervals scale by session length, so Sharpe/Sortino/
    Calmar are correct across asset classes and intervals. `beta` is always `None`
    on domain handles (no benchmark is fetched).
- **Financial event calendar** — `Ticker::calendar(range)` / `Tickers::calendar(range)`
  → `Vec<CalendarEvent>`, aggregating upcoming earnings, ex-dividend/dividend-payment
  dates, and standard monthly options expirations across one or more symbols into a
  single time-sorted list. With the `fred` feature, `fred::release_dates()` appends a
  curated set of market-moving economic releases (CPI, NFP, GDP, …). New public
  `CalendarEvent` / `EventKind` types.
- `Region::Japan` / `Region::Korea` / `Region::Mexico` / `Region::Qatar` variants
  (verified live against Yahoo's market-time endpoint).
- `ProvidersBuilder::region_code` for parity with `TickerBuilder`/`TickersBuilder`.
- **`NewsStream` / `NewsStreamBuilder`** (`streaming` module) — polls RSS/Atom
  feeds in the background and broadcasts newly discovered entries (deduplicated
  by URL), mirroring the existing `PriceStream` pattern but for pull-only
  sources. `PriceStream` and `NewsStream` now share a generic internal
  `Subscription<T>` broadcast primitive instead of duplicating channel plumbing.
- `feeds::parse_bytes(bytes, source_name)` — parses already-fetched RSS/Atom
  bytes without a network round-trip; used internally by `NewsStream` and
  useful for callers with their own HTTP client/cache/proxy.

### Changed

- Extracted the per-`Indicator` dispatch out of `Ticker::indicator` into a shared
  internal `indicators::compute_indicator(indicator, &chart)`, now reused by both
  `Ticker` and the domain handles (no behavior change). `risk` summaries are now
  computed via an annualization-factor-aware path; `Ticker::risk` is unchanged
  (still the daily 252-period calendar).
- `Tickers::add_symbols`/`remove_symbols`, `PriceStream::subscribe`/`add_symbols`/
  `remove_symbols`, `PriceStreamBuilder::symbols`, `NewsStream::subscribe`/
  `add_sources`/`remove_sources`, `NewsStreamBuilder::sources`, `feeds::fetch_all`,
  `ProvidersBuilder::route`, and `translation::translate_texts` now accept
  `impl IntoIterator<Item = impl Into<String>>` (or `Item = Provider`/`FeedSource`
  where applicable) instead of requiring a `&[...]` slice reference.
- **Breaking**: `finance::hours()` now takes `Option<Region>` instead of
  `Option<&str>`, matching the typed sibling `market_summary`/`trending`/`indices`
  functions.
  - Migration: `finance::hours(Some("JP"))` → `finance::hours(Some(Region::Japan))`
- **Breaking**: `FinanceError::NotSupported`'s `provider`/`operation` fields changed
  from `&'static str` to the typed `Provider`/`Operation` enums, and gained a new
  `candidates: Vec<Provider>` field listing which providers could have served the
  capability. Code matching or destructuring these fields as strings will need
  updating; the `Display` output text is unchanged.
- Internal: the `scraper` crate (html5ever/selectors/cssparser, 56 transitive
  crates) replaced with a minimal hand-rolled HTML element matcher
  (`src/scrapers/html.rs`) behind the Yahoo exchanges/stockanalysis scrapers —
  same approach as the earlier `feed-rs` → hand-rolled RSS parser move. No
  public API change.

### Fixed

- Options chain parsing (`Ticker::options`/`Tickers::options`) now correctly reads
  calls/puts from the per-expiration `options[]` array instead of the top-level
  chain result, which could silently return an empty or incomplete contract list.
- `FinanceError::with_context`/`category`/`is_retriable` now cover `MacroDataError`/
  `FeedParseError`/`ExternalApiError`, which had matching fields but were previously
  silently skipped.

### Removed

- **Breaking**: `Region::cors_domain()` removed (unused, zero call sites anywhere).
- `ClientConfigBuilder`/`ClientConfig::builder()` — dead code, unreachable outside
  the crate and fully superseded by the direct `.timeout()`/`.proxy()`/`.lang()`/
  `.region()`/`.region_code()` setters already on every builder.

### Security

- `feed-rs` (and its `quick-xml`/XML dependency chain) removed entirely; RSS/Atom
  parsing is now a hand-rolled, dependency-free extractor (`src/feeds/parser.rs`),
  resolving RUSTSEC-2026-0195.
- Bumped `crossbeam-epoch` 0.9.18 → 0.9.20 (RUSTSEC-2026-0204).
- Bumped `anyhow` 1.0.102 → 1.0.103 (RUSTSEC-2026-0190, unsoundness in
  `Error::downcast_mut()`).
- Bumped `cxx` 1.0.194 → 1.0.197 (RUSTSEC-2026-0202, unsound), pulled in
  transitively via `ct2rs` under the `translation-offline` feature.

## [2.7.1] - 2026-06-20

Maintenance and internal-architecture release. Domain handles gain opt-in
caching, the spark and streaming paths are routed through the provider
abstraction, and a long-deprecated `Fetch` variant is removed. The default API
surface is otherwise unchanged.

### Added

- **Opt-in per-handle caching for domain handles** (`ForexPair`, `CryptoCoin`,
  `EconomicIndicator`, `Index`, `FuturesContract`, `Commodity`, `Filings`),
  which were previously stateless on every call. New `DomainCache<V>`
  (`src/domains/mod.rs`) is a `String`-keyed, TTL'd response cache with a fetch
  guard that collapses concurrent identical misses. Enable per handle with
  `.cache(ttl)`; default behavior is unchanged (stateless).
- `ProviderAdapter::fetch_spark` trait method (defaulting to `NotSupported`,
  mirroring `fetch_quotes_batch`), letting batch spark data flow through
  `CHART` routing.
- `PriceStream::subscribe_with_source()` — a generic entry point over the new
  pluggable `StreamSource` trait, enabling non-Yahoo real-time price streams.
  `subscribe()` continues to default to Yahoo.

### Changed

- `Tickers::spark()` now dispatches through
  `ProviderSet::fetch(Capability::CHART, ..)` (via `fetch_spark`) instead of
  calling Yahoo directly, so it honors `CHART` routing like every other chart
  path. Caching, fetch-guard dedup, and missing-symbol error tracking are
  preserved; Yahoo remains the default route.
- Internal: extracted a pluggable `StreamSource` trait
  (`src/streaming/source.rs`) with a `YahooStreamSource` reference impl from the
  Yahoo WebSocket client; `run_stream_loop` drives any source with
  auto-reconnect. The public `PriceStream` API is unchanged.

### Removed

- **`Fetch::All`** — deprecated since v2.6.0 as an alias for `Fetch::Parallel`.
  Replace `Fetch::All` with `Fetch::Parallel` (identical behavior). Also removed
  a duplicate `[profile.release]` from the CLI manifest that Cargo ignored while
  warning on every build.

### Security

No publicly known run-time vulnerabilities with a CVE or RUSTSEC assignment were
fixed in the library or its direct dependencies in this release.

## [2.7.0] - 2026-06-18

Adds two opt-in, offline-capable enrichment layers — response-field translation
and VADER sentiment scoring — plus a fully local machine-translation backend.
Both are feature-gated and default-off, so the default API surface is unchanged.

### Added

- **Translation** (`translation` feature) — post-processes responses so the
  existing `.lang()`/`.region()` builder surface actually localizes Yahoo's
  English-only natural-language fields (company summaries, sector/industry
  names, news titles, officer titles, transcripts). Symbols, codes, URLs, and
  numbers are never touched.
  - New `translation` module: `Lang` (BCP 47 tag parsing/normalization),
    `Translatable` trait (implemented by all text-bearing response models, and
    composes over `Vec<T>`/`Option<T>`), `TranslationBackend` trait +
    `set_backend` for plugging a custom engine, `translate(&mut value, lang)`
    for standalone values, and `preload()`.
  - Two-tier strategy: a zero-latency built-in dictionary (sector names,
    security types, officer titles across 11 languages) always applies; an
    optional ML backend handles free-form text. With no backend, free-form
    fields stay English and the dictionary tier still applies — enabling
    `translation` alone never breaks a response.
  - `ProvidersBuilder::lang()`/`.region()` set the language once and are
    inherited by every `ticker()`/`tickers()` handle; the tag is validated
    fail-fast at `build()` before any network call.
- **`translation-offline` feature** — a fully local CPU machine-translation
  backend built on opus-mt bilingual models (~48 languages) run through
  CTranslate2 with int8 weights, distributed as Argos packages. A small
  per-language model (~80–210 MB) is downloaded on first use and cached;
  every subsequent run is offline with no API key. Heavy native build (compiles
  CTranslate2 + SentencePiece from source — needs `cmake` and a C++ toolchain).
- **Sentiment scoring** (`sentiment` feature) — offline, keyless VADER
  lexicon-based scoring of news titles and earnings-transcript paragraphs.
  - New public types: `Sentiment` (`label`, `score`, `confidence`),
    `SentimentLabel` (`Bullish`/`Neutral`/`Bearish`), and `analyze_sentiment`
    for scoring arbitrary text.
  - `News` articles gain an optional `sentiment` field, populated automatically
    when the feature is enabled; transcript paragraphs are scored in place.
  - `Ticker::news_sentiment()` returns the average sentiment across recent
    headlines; `Transcript::overall_sentiment()` returns a length-weighted
    aggregate across a whole call.

### Security

No publicly known run-time vulnerabilities with a CVE or RUSTSEC assignment were fixed in the library or its direct dependencies in this release. The offline translation backend downloads opus-mt models over HTTPS from the Argos package server on first use and caches them locally; no network call is made unless `translation-offline` is enabled and a non-English language is requested. Sentiment scoring is fully offline (bundled VADER lexicon) with no network access.

## [2.6.1] - 2026-05-27

### Added

- **Fuzz testing suite** (`fuzz/`): 10 fuzz targets covering core library types and indicators
  - `fuzz_quote`, `fuzz_chart`, `fuzz_financials`, `fuzz_options`, `fuzz_edgar`, `fuzz_discovery` — deserialization fuzzing for all major response types
  - `fuzz_indicators_ohlcv`, `fuzz_indicators_series`, `fuzz_patterns`, `fuzz_atr` — indicator computation fuzzing with arbitrary OHLCV inputs
- **`CONTRIBUTING.md`**: contribution guide covering bug reports, feature requests, dev setup, code style, and PR process

### Changed

- **`Quote<F: Format>`** — `Quote` (and the `FinancialData`, `DefaultKeyStatistics`, and `Price` sub-structs) is now generic over a compile-time `Format` type parameter (`Raw`, `Pretty`, or `Both`). Format selection moves from a runtime builder method to a type parameter at the call site: `ticker.quote::<Raw>()`, `ticker.quote::<Pretty>()`, `ticker.quote::<Both>()`.
  - Default format changed from `Both` to `Raw`
  - `finance-query-derive` is now a direct (non-optional) dependency; the `dataframe` feature no longer re-enables it
- Updated `SECURITY.md` supported version table: `2.5.x` → `2.6.x`

### Security

No publicly known run-time vulnerabilities with a CVE or RUSTSEC assignment were fixed in the library or its direct dependencies in this release. The following supply-chain and infrastructure hardening changes were made:

- Docker runtime stages now run `apt-get upgrade` on every image build so OS-level packages (including `libgnutls30`, `libkrb5support0`, `libgcrypt20`) receive available security patches regardless of the pinned base image digest
- All GitHub Actions workflow steps pinned to exact release-tag SHAs (`harden-runner` → v2.19.3, `actions/checkout` → v6.0.2, `docker/setup-buildx-action` → v3.12.0, `actions/upload-artifact` → v6.0.0, `codeql-action/upload-sarif` → v3.36.0, `cargo-deny-action` → v2.0.19, `rust-cache` → v2.9.1) so `zizmor` ref-version-mismatch checks pass
- `once_cell` replaced with `std::LazyLock` from the standard library, removing the external dependency for lazy initialization
- Documentation build pinned with `pip install --require-hashes` from `docs/requirements.txt` (generated with `pip-compile --generate-hashes`), closing a Scorecard Pinned-Dependencies finding

## [2.6.0] - 2026-05-21

Introduces a multi-provider data-aggregation architecture. **This is a breaking change to the public API** — see Migration below.

### Added

- **`Providers` / `ProvidersBuilder`** (`finance_query::Providers`) — central entry point: configure providers once, then create many domain handles that share the same connections.
  - Capability routing: `.route(Capability::QUOTE, &[Provider::Polygon, Provider::Yahoo])`
  - Fetch strategies: `.fetch(Fetch::Sequential)` (try in order, first success wins) or `Fetch::Parallel` (race concurrently)
  - Shared knobs: `.timeout(...)`, `.proxy(...)`, etc.
- **New public types:** `Provider`, `Fetch`, `Capability`, `Providers`, `ProvidersBuilder`.
- **Domain handles**, all constructed via the `Providers` factory and feature-gated:
  - `ForexPair` (`providers.forex("USD", "EUR")`) → `ForexQuote`
  - `CryptoCoin` (`providers.crypto("bitcoin")`) → `CryptoQuote`
  - `EconomicIndicator` (`providers.economic("REAL_GDP")`) → `EconomicSeries`
  - `Index` (`providers.index("SPY")`) → `IndexQuote`
  - `FuturesContract` (`providers.futures("NQ=F")`) → `FuturesQuote`
  - `Commodity` (`providers.commodity("WHEAT")`) → `CommodityQuote`
  - `Filings` (`providers.filings("AAPL")`) → `ProviderFilings` (SEC EDGAR, always available)
- **Multi-provider batch API** via `providers.tickers([...])`.
- `Ticker::financials()` for financial statements through provider dispatch.
- Automatic EDGAR routing for the `FILINGS` capability when no other provider is configured.

### Changed

- **Provider routing now lives on `ProvidersBuilder`.** `Ticker`/`Tickers` builders are the Yahoo-only fast path; routed instances are created with `providers.ticker(symbol)` / `providers.tickers([...])`.
- Adapters reorganized into capability subdirectories (`src/adapters/<provider>/<capability>/`) and are now `pub(crate)` — they are no longer part of the public API.
- Public models reorganized into capability-based directories.
- `quote()` returns a single unified `Quote` instead of provider-specific result types.
- Quotes, charts, events, financials, and indicators are lazily fetched and cached through the provider layer.

### Removed

- **Per-provider `Ticker` handles introduced in 2.5.1**: `Ticker::polygon()`, `Ticker::fmp()`, `Ticker::alphavantage()` and the `PolygonHandle` / `FmpHandle` / `AlphaVantageHandle` types. Provider selection is now declared via capability routing on `ProvidersBuilder`.
- Direct public access to adapter modules (adapters are now `pub(crate)`).
- Legacy client modules and provider-specific public types, replaced by the unified provider layer.

### Migration

```rust
// Before (2.5.x): per-provider handle on Ticker
let ticker = Ticker::new("AAPL").await?;
let snapshot = ticker.polygon().snapshot().await?;

// After (2.6.0): route capabilities on Providers, then create the ticker
let providers = Providers::builder()
    .route(Capability::QUOTE, &[Provider::Polygon, Provider::Yahoo])
    .fetch(Fetch::Sequential)
    .build().await?;
let ticker = providers.ticker("AAPL").build().await?;
let quote = ticker.quote().await?;
```

The Yahoo-only fast path is unchanged: `Ticker::new("AAPL")` and `Ticker::builder("AAPL")` still work without configuring providers.

## [2.5.1] - 2026-05-06

The `Ticker` adapter handle integration in this release was contributed by [@Johnson-f](https://github.com/Johnson-f) in [#133](https://github.com/Verdenroz/finance-query/pull/133).

### Added

- **Typed adapter handles on `Ticker`** for explicit provider access:
  - `Ticker::polygon()` → `PolygonHandle` (feature: `polygon`)
  - `Ticker::fmp()` → `FmpHandle` (feature: `fmp`)
  - `Ticker::alphavantage()` → `AlphaVantageHandle` (feature: `alphavantage`)
- **Curated single-symbol adapter methods** on the new handles (28 total):
  - `PolygonHandle`: `snapshot`, `aggregates`, `previous_close`, `last_trade`, `news`, `dividends`, `splits`, `financials`, `details`
  - `FmpHandle`: `quote`, `historical`, `intraday`, `income_statement`, `balance_sheet`, `cash_flow`, `key_metrics`, `ratios`, `profile`, `news`
  - `AlphaVantageHandle`: `quote`, `intraday`, `daily`, `daily_adjusted`, `weekly`, `weekly_adjusted`, `monthly`, `monthly_adjusted`, `overview`

### Changed

- Improved type safety for adapter handle parameters:
  - `FmpHandle::intraday(...)` now uses `IntradayInterval` instead of a free-form string
  - `PolygonHandle::financials(...)` now uses `FinancialPeriod` instead of a free-form string
  - New enums are re-exported from the crate root for downstream use

## [2.5.0] - 2026-05-02

The adapter additions in this release were contributed by [@Johnson-f](https://github.com/Johnson-f) in [#132](https://github.com/Verdenroz/finance-query/pull/132).

### Added

- **Alpha Vantage adapter** (`finance_query::adapters::alphavantage`, feature: `alphavantage`)
  - Core stocks: intraday/daily/weekly/monthly time series (raw + adjusted), global quote, bulk quotes, symbol search, market status
  - Options: realtime and historical chains with greeks
  - Forex: exchange rates and FX time series
  - Crypto: intraday, daily, weekly, and monthly OHLCV series
  - Commodities: WTI, Brent, natural gas, copper, aluminum, wheat, corn, cotton, sugar, coffee, composite index
  - Economic indicators: GDP, CPI, Treasury yield, Fed funds rate, unemployment, nonfarm payroll
  - Alpha Intelligence: news sentiment, earnings call transcripts, top gainers/losers/most-active
  - Fundamentals: company overview, ETF profile, income/balance/cash flow statements, earnings history, dividends, splits, IPO/earnings calendars
  - Technical indicators: 50+ typed wrappers — SMA, EMA, RSI, MACD, BBANDS, STOCH, ADX, and more
  - Singleton: `alphavantage::init(api_key)` / `init_with_timeout(api_key, timeout)` — call once at startup; rate-limited to 1 req/sec by default

- **Polygon.io adapter** (`finance_query::adapters::polygon`, feature: `polygon`)
  - Stocks: aggregate bars, tick-level trades, NBBO quotes, snapshots, fundamentals (balance sheets, income, cash flow, ratios, short interest), corporate actions (dividends, splits, IPOs), SEC filings (10-K, 8-K, risk factors), news with sentiment, technical indicators
  - Options: aggregates, contracts, chain/contract snapshots with greeks, trades/quotes, indicators
  - Forex: aggregates, quotes, currency conversion, snapshots, indicators
  - Crypto: aggregates, trades, snapshots, indicators
  - Indices: aggregates, snapshots, indicators
  - Futures: aggregates, contracts/products/schedules, snapshots, trades/quotes
  - Economy: inflation, inflation expectations, labor market data, Treasury yields
  - Reference: ticker details, types, related tickers, exchanges, condition codes, market holidays/status
  - Partner data: Benzinga analyst ratings/insights/consensus/guidance/earnings/news; ETF Global analytics, constituents, flows, profiles
  - WebSocket streaming (`polygon::websocket`): real-time trades, quotes, per-second/minute aggregates for all asset classes
  - Cursor-based pagination via `get_all_pages<T>()` with configurable safety cap
  - Singleton: `polygon::init(api_key)` / `init_with_timeout(api_key, timeout)`

- **Financial Modeling Prep adapter** (`finance_query::adapters::fmp`, feature: `fmp`)
  - Fundamentals: income statement, balance sheet, cash flow (standard, as-reported, and full financial)
  - Analysis: financial ratios, key metrics, enterprise value, DCF, company rating, financial growth
  - Company: profile, key executives, market cap, outlook, peers, delisted companies
  - Prices: real-time quotes, batch quotes, historical daily/intraday (1min–4hour)
  - Dividends and splits: full historical records
  - Technical indicators: SMA, EMA, WMA, DEMA, TEMA, Williams %R, RSI, ADX, MACD (daily + intraday)
  - Calendars: earnings, IPO, stock split, dividend, economic
  - News: stock news, FMP articles, press releases, crypto/forex news
  - Insider trading: SEC Form 4, CIK mapper, insider RSS, fail-to-deliver, congressional trading
  - Institutional: holders, ETF/mutual fund holders, Form 13F
  - Fund holdings: ETF sector/country weightings and holdings
  - Estimates: analyst estimates/recommendations, earnings surprises, stock grades, earnings transcripts
  - Market performance: sector/industry PE ratios, sector performance, gainers, losers, most active
  - Crypto, forex, commodities: quotes, available symbols, historical daily/intraday
  - ETF and mutual funds: quotes, available lists, historical prices
  - Indexes: major indexes, S&P 500/Nasdaq/Dow constituents (current + historical)
  - Screener: stock screener (market cap, sector, country, etc.), symbol search, CIK search
  - Advanced: SIC codes, COT reports/analysis
  - Bulk: bulk income statements, balance sheets, cash flow, ratios, key metrics, profiles
  - Singleton: `fmp::init(api_key)` / `init_with_timeout(api_key, timeout)`

- **Shared adapter infrastructure** (`finance_query::adapters`):
  - Consistent singleton pattern across all three providers: `OnceLock` + shared token-bucket `RateLimiter`
  - Percent-encoded URL path segments (`adapters::common`) for safe handling of complex symbols (options tickers, forex pairs)
  - All adapters feature-gated — zero compile-time cost when the feature is not enabled
  - Full mockito test coverage — no API key or network access required to run tests

### Fixed

- `ScreenerQuote::short_name` now defaults to `""` when Yahoo Finance omits the `shortName` field, preventing parse failures for predefined screeners (`DayGainers`, `MostActives`, etc.)
- `TimeRange::Max` with `Interval::OneDay` or `Interval::OneWeek` now chunks requests into sequential 10-year periods to avoid Yahoo Finance response truncation
- Security: bumped `rustls-webpki` 0.103.10 → 0.103.13 (RUSTSEC-2026-0098, RUSTSEC-2026-0099, RUSTSEC-2026-0104: name constraint and CRL parsing vulnerabilities in TLS certificate verification)
- Security: bumped `rand` 0.8.5 → 0.8.6, 0.9.2 → 0.9.4, 0.10.0 → 0.10.1 (RUSTSEC-2026-0097: unsoundness with custom global loggers)

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

[Unreleased]: https://github.com/Verdenroz/finance-query/compare/v2.7.1...HEAD
[2.7.1]: https://github.com/Verdenroz/finance-query/compare/v2.7.0...v2.7.1
[2.7.0]: https://github.com/Verdenroz/finance-query/compare/v2.6.1...v2.7.0
[2.6.1]: https://github.com/Verdenroz/finance-query/compare/v2.6.0...v2.6.1
[2.6.0]: https://github.com/Verdenroz/finance-query/compare/v2.5.1...v2.6.0
[2.5.1]: https://github.com/Verdenroz/finance-query/compare/v2.5.0...v2.5.1
[2.5.0]: https://github.com/Verdenroz/finance-query/compare/v2.4.3...v2.5.0
[2.4.3]: https://github.com/Verdenroz/finance-query/compare/v2.4.2...v2.4.3
[2.4.2]: https://github.com/Verdenroz/finance-query/compare/v2.4.1...v2.4.2
[2.4.1]: https://github.com/Verdenroz/finance-query/compare/v2.4.0...v2.4.1
[2.4.0]: https://github.com/Verdenroz/finance-query/compare/v2.3.0...v2.4.0
[2.3.0]: https://github.com/Verdenroz/finance-query/compare/v2.2.1...v2.3.0
[2.2.1]: https://github.com/Verdenroz/finance-query/compare/v2.2.0...v2.2.1
[2.2.0]: https://github.com/Verdenroz/finance-query/compare/v2.1.0...v2.2.0
[2.1.0]: https://github.com/Verdenroz/finance-query/compare/v2.0.1...v2.1.0
[2.0.1]: https://github.com/Verdenroz/finance-query/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/Verdenroz/finance-query/releases/tag/v2.0.0
