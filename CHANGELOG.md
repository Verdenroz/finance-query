# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased (draft vs v2.8.0)

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
| `finance_query::bt_base_to_htf_index` | n/a | 28.64µs | 31.94µs | 1 | n/a |
| `finance_query::bt_bayesian_search` | n/a | 3.04ms | 3.11ms | 2382 | n/a |
| `finance_query::bt_grid_search` | n/a | 642.16µs | 717.46µs | 3012 | n/a |
| `finance_query::bt_monte_carlo` | n/a | 177.63µs | 183.85µs | 6 | n/a |
| `finance_query::bt_resample` | n/a | 110.48µs | 113.07µs | 11 | n/a |
| `finance_query::bt_sma_crossover` | n/a | 947.51µs | 1.49ms | 2373 | n/a |
| `finance_query::bt_strategy_builder` | n/a | 164.83µs | 176.37µs | 330 | n/a |
| `finance_query::capability_name` | n/a | 2.8ns | 4.3ns | 0 | n/a |
| `finance_query::cond_bench_always_false` | n/a | 18.12µs | 18.55µs | 6 | n/a |
| `finance_query::cond_bench_always_true` | n/a | 407.41µs | 425.94µs | 4045 | n/a |
| `finance_query::cond_bench_has_position` | n/a | 405.00µs | 428.81µs | 4045 | n/a |
| `finance_query::cond_bench_held_for_bars` | n/a | 157.64µs | 159.51µs | 1041 | n/a |
| `finance_query::cond_bench_in_loss` | n/a | 415.96µs | 432.94µs | 4045 | n/a |
| `finance_query::cond_bench_in_profit` | n/a | 47.20µs | 49.34µs | 142 | n/a |
| `finance_query::cond_bench_is_long` | n/a | 404.61µs | 442.22µs | 4045 | n/a |
| `finance_query::cond_bench_is_short` | n/a | 32.61µs | 35.77µs | 13 | n/a |
| `finance_query::cond_bench_no_position` | n/a | 32.81µs | 33.33µs | 13 | n/a |
| `finance_query::cond_bench_stop_loss` | n/a | 40.38µs | 43.82µs | 24 | n/a |
| `finance_query::cond_bench_take_profit` | n/a | 39.03µs | 39.09µs | 13 | n/a |
| `finance_query::cond_bench_trailing_stop` | n/a | 49.03µs | 49.40µs | 92 | n/a |
| `finance_query::cond_bench_trailing_take_profit` | n/a | 48.09µs | 52.93µs | 72 | n/a |
| `finance_query::de_chart` | n/a | 9.47µs | 9.59µs | 13 | n/a |
| `finance_query::de_crypto_coins` | n/a | 33.50µs | 33.80µs | 205 | n/a |
| `finance_query::de_currencies` | n/a | 55.55µs | 57.85µs | 647 | n/a |
| `finance_query::de_edgar_facts` | n/a | 1.36ms | 1.53ms | 11899 | n/a |
| `finance_query::de_edgar_submissions` | n/a | 454.10µs | 474.73µs | 6784 | n/a |
| `finance_query::de_fear_and_greed` | n/a | 119.2ns | 133.6ns | 0 | n/a |
| `finance_query::de_fear_and_greed_crypto_history` | n/a | 1.36µs | 1.37µs | 3 | n/a |
| `finance_query::de_financials` | n/a | 20.52µs | 20.66µs | 206 | n/a |
| `finance_query::de_fred_series` | n/a | 103.94µs | 106.88µs | 873 | n/a |
| `finance_query::de_hours` | n/a | 690.6ns | 703.4ns | 10 | n/a |
| `finance_query::de_market_summary` | n/a | 29.65µs | 30.92µs | 228 | n/a |
| `finance_query::de_news` | n/a | 3.85µs | 3.90µs | 53 | n/a |
| `finance_query::de_options` | n/a | 1.11µs | 1.13µs | 6 | n/a |
| `finance_query::de_quote` | n/a | 1.04ms | 1.09ms | 8235 | n/a |
| `finance_query::de_screener` | n/a | 421.24µs | 456.57µs | 4498 | n/a |
| `finance_query::de_search` | n/a | 5.03µs | 5.35µs | 55 | n/a |
| `finance_query::de_treasury_yields` | n/a | 79.06µs | 80.59µs | 119 | n/a |
| `finance_query::de_trending` | n/a | 1.43µs | 1.45µs | 24 | n/a |
| `finance_query::dispatch_select` | n/a | 4.0ns | 5.0ns | 0 | n/a |
| `finance_query::ind_accumulation_distribution` | n/a | 3.81µs | 3.98µs | 1 | n/a |
| `finance_query::ind_alma` | n/a | 5.12µs | 5.24µs | 2 | n/a |
| `finance_query::ind_aroon` | n/a | 12.98µs | 13.04µs | 9 | n/a |
| `finance_query::ind_atr` | n/a | 9.73µs | 9.88µs | 2 | n/a |
| `finance_query::ind_awesome_oscillator` | n/a | 8.49µs | 8.61µs | 4 | n/a |
| `finance_query::ind_balance_of_power` | n/a | 1.91µs | 1.92µs | 2 | n/a |
| `finance_query::ind_bull_bear_power` | n/a | 7.11µs | 7.33µs | 3 | n/a |
| `finance_query::ind_cci` | n/a | 17.01µs | 17.74µs | 2 | n/a |
| `finance_query::ind_chaikin_oscillator` | n/a | 15.98µs | 16.10µs | 4 | n/a |
| `finance_query::ind_choppiness_index` | n/a | 22.73µs | 22.96µs | 8 | n/a |
| `finance_query::ind_cmf` | n/a | 4.00µs | 4.04µs | 2 | n/a |
| `finance_query::ind_cmo` | n/a | 4.97µs | 5.00µs | 2 | n/a |
| `finance_query::ind_coppock_curve` | n/a | 7.81µs | 8.44µs | 3 | n/a |
| `finance_query::ind_dema` | n/a | 11.93µs | 12.02µs | 3 | n/a |
| `finance_query::ind_donchian_channels` | n/a | 12.08µs | 13.46µs | 9 | n/a |
| `finance_query::ind_elder_ray` | n/a | 7.09µs | 7.35µs | 3 | n/a |
| `finance_query::ind_fibonacci_pivot_points` | n/a | 2.40µs | 2.48µs | 1 | n/a |
| `finance_query::ind_fibonacci_retracement` | n/a | 11.42µs | 11.93µs | 9 | n/a |
| `finance_query::ind_heikin_ashi` | n/a | 14.59µs | 14.68µs | 9 | n/a |
| `finance_query::ind_hma` | n/a | 13.65µs | 13.74µs | 5 | n/a |
| `finance_query::ind_ichimoku` | n/a | 35.67µs | 36.55µs | 27 | n/a |
| `finance_query::ind_keltner_channels` | n/a | 17.50µs | 17.79µs | 5 | n/a |
| `finance_query::ind_last_value` | n/a | 3.1ns | 3.4ns | 0 | n/a |
| `finance_query::ind_macd` | n/a | 20.40µs | 20.61µs | 7 | n/a |
| `finance_query::ind_mcginley_dynamic` | n/a | 12.03µs | 12.13µs | 1 | n/a |
| `finance_query::ind_mfi` | n/a | 7.30µs | 7.54µs | 3 | n/a |
| `finance_query::ind_momentum` | n/a | 127.02µs | 130.43µs | 52 | n/a |
| `finance_query::ind_momentum_single` | n/a | 529.5ns | 561.5ns | 1 | n/a |
| `finance_query::ind_moving_averages` | n/a | 85.75µs | 87.11µs | 27 | n/a |
| `finance_query::ind_parabolic_sar` | n/a | 2.53µs | 2.56µs | 1 | n/a |
| `finance_query::ind_patterns` | n/a | 106.75µs | 113.76µs | 1 | n/a |
| `finance_query::ind_pivot_points` | n/a | 2.85µs | 2.92µs | 1 | n/a |
| `finance_query::ind_roc` | n/a | 1.62µs | 1.65µs | 1 | n/a |
| `finance_query::ind_sma` | n/a | 380.28µs | 403.08µs | 1 | n/a |
| `finance_query::ind_stochastic` | n/a | 16.57µs | 17.02µs | 11 | n/a |
| `finance_query::ind_stochastic_rsi` | n/a | 28.42µs | 29.07µs | 12 | n/a |
| `finance_query::ind_supertrend` | n/a | 13.24µs | 13.33µs | 4 | n/a |
| `finance_query::ind_tema` | n/a | 17.57µs | 17.68µs | 4 | n/a |
| `finance_query::ind_trend` | n/a | 93.90µs | 99.31µs | 48 | n/a |
| `finance_query::ind_true_range` | n/a | 1.27µs | 1.28µs | 1 | n/a |
| `finance_query::ind_volatility` | n/a | 73.39µs | 74.67µs | 28 | n/a |
| `finance_query::ind_volume` | n/a | 40.66µs | 41.23µs | 14 | n/a |
| `finance_query::ind_vwap` | n/a | 3.88µs | 3.95µs | 1 | n/a |
| `finance_query::ind_vwma` | n/a | 3.10µs | 3.13µs | 1 | n/a |
| `finance_query::ind_williams_r` | n/a | 10.97µs | 12.08µs | 7 | n/a |
| `finance_query::ind_wma` | n/a | 4.63µs | 4.98µs | 7 | n/a |
| `finance_query::ind_zigzag` | n/a | 1.73µs | 1.81µs | 4 | n/a |
| `finance_query::ref_bench_accumulation_distribution` | n/a | 48.38µs | 48.96µs | 23 | n/a |
| `finance_query::ref_bench_adx` | n/a | 505.03µs | 518.76µs | 4930 | n/a |
| `finance_query::ref_bench_alma` | n/a | 548.19µs | 555.36µs | 5019 | n/a |
| `finance_query::ref_bench_aroon` | n/a | 511.03µs | 525.01µs | 4950 | n/a |
| `finance_query::ref_bench_atr` | n/a | 508.65µs | 537.98µs | 4996 | n/a |
| `finance_query::ref_bench_awesome_oscillator` | n/a | 314.87µs | 434.38µs | 2636 | n/a |
| `finance_query::ref_bench_balance_of_power` | n/a | 48.29µs | 50.00µs | 25 | n/a |
| `finance_query::ref_bench_bear_power` | n/a | 140.21µs | 143.29µs | 806 | n/a |
| `finance_query::ref_bench_bollinger` | n/a | 552.07µs | 588.12µs | 4968 | n/a |
| `finance_query::ref_bench_bull_power` | n/a | 476.22µs | 500.58µs | 4238 | n/a |
| `finance_query::ref_bench_candle_body` | n/a | 18.74µs | 19.14µs | 6 | n/a |
| `finance_query::ref_bench_candle_range` | n/a | 470.37µs | 482.26µs | 5043 | n/a |
| `finance_query::ref_bench_cci` | n/a | 301.96µs | 340.87µs | 2605 | n/a |
| `finance_query::ref_bench_chaikin_oscillator` | n/a | 64.37µs | 65.45µs | 26 | n/a |
| `finance_query::ref_bench_choppiness_index` | n/a | 525.93µs | 658.43µs | 5002 | n/a |
| `finance_query::ref_bench_close` | n/a | 476.00µs | 486.68µs | 5043 | n/a |
| `finance_query::ref_bench_cmf` | n/a | 49.30µs | 50.29µs | 25 | n/a |
| `finance_query::ref_bench_cmo` | n/a | 287.20µs | 296.44µs | 2598 | n/a |
| `finance_query::ref_bench_coppock_curve` | n/a | 307.87µs | 313.76µs | 2559 | n/a |
| `finance_query::ref_bench_dema` | n/a | 499.82µs | 508.75µs | 4875 | n/a |
| `finance_query::ref_bench_donchian` | n/a | 556.52µs | 567.83µs | 4976 | n/a |
| `finance_query::ref_bench_elder_bear_power` | n/a | 141.08µs | 144.14µs | 806 | n/a |
| `finance_query::ref_bench_elder_bull_power` | n/a | 477.92µs | 505.69µs | 4238 | n/a |
| `finance_query::ref_bench_ema` | n/a | 497.67µs | 507.92µs | 4963 | n/a |
| `finance_query::ref_bench_gap_pct` | n/a | 253.70µs | 265.16µs | 2489 | n/a |
| `finance_query::ref_bench_high` | n/a | 469.29µs | 480.50µs | 5043 | n/a |
| `finance_query::ref_bench_hma` | n/a | 506.32µs | 518.59µs | 4957 | n/a |
| `finance_query::ref_bench_htf` | n/a | 595.94µs | 605.76µs | 7039 | n/a |
| `finance_query::ref_bench_htf_region` | n/a | 593.94µs | 629.09µs | 7039 | n/a |
| `finance_query::ref_bench_ichimoku` | n/a | 597.87µs | 616.54µs | 4836 | n/a |
| `finance_query::ref_bench_ichimoku_custom` | n/a | 599.31µs | 738.72µs | 4836 | n/a |
| `finance_query::ref_bench_is_bearish` | n/a | 18.23µs | 18.58µs | 6 | n/a |
| `finance_query::ref_bench_is_bullish` | n/a | 21.22µs | 23.10µs | 6 | n/a |
| `finance_query::ref_bench_keltner` | n/a | 559.41µs | 579.54µs | 4972 | n/a |
| `finance_query::ref_bench_low` | n/a | 470.77µs | 480.38µs | 5043 | n/a |
| `finance_query::ref_bench_macd` | n/a | 321.79µs | 330.39µs | 2541 | n/a |
| `finance_query::ref_bench_mcginley` | n/a | 507.81µs | 599.66µs | 4963 | n/a |
| `finance_query::ref_bench_median_price` | n/a | 474.02µs | 490.95µs | 5043 | n/a |
| `finance_query::ref_bench_mfi` | n/a | 500.56µs | 578.36µs | 4998 | n/a |
| `finance_query::ref_bench_momentum` | n/a | 285.26µs | 296.17µs | 2552 | n/a |
| `finance_query::ref_bench_obv` | n/a | 57.77µs | 59.87µs | 30 | n/a |
| `finance_query::ref_bench_open` | n/a | 471.87µs | 487.96µs | 5043 | n/a |
| `finance_query::ref_bench_parabolic_sar` | n/a | 555.96µs | 585.27µs | 5056 | n/a |
| `finance_query::ref_bench_price` | n/a | 477.12µs | 489.38µs | 5043 | n/a |
| `finance_query::ref_bench_price_change_pct` | n/a | 276.12µs | 294.87µs | 2489 | n/a |
| `finance_query::ref_bench_relative_volume` | n/a | 527.00µs | 547.38µs | 4949 | n/a |
| `finance_query::ref_bench_roc` | n/a | 283.65µs | 301.00µs | 2587 | n/a |
| `finance_query::ref_bench_rsi` | n/a | 503.61µs | 563.19µs | 4994 | n/a |
| `finance_query::ref_bench_sma` | n/a | 496.99µs | 505.18µs | 4963 | n/a |
| `finance_query::ref_bench_stochastic` | n/a | 560.78µs | 579.75µs | 4987 | n/a |
| `finance_query::ref_bench_stochastic_rsi` | n/a | 569.46µs | 606.60µs | 4831 | n/a |
| `finance_query::ref_bench_supertrend` | n/a | 561.73µs | 577.67µs | 5021 | n/a |
| `finance_query::ref_bench_tema` | n/a | 500.45µs | 559.18µs | 4781 | n/a |
| `finance_query::ref_bench_true_range` | n/a | 500.08µs | 518.67µs | 5059 | n/a |
| `finance_query::ref_bench_typical_price` | n/a | 511.91µs | 523.11µs | 5043 | n/a |
| `finance_query::ref_bench_volume` | n/a | 471.03µs | 492.60µs | 5043 | n/a |
| `finance_query::ref_bench_vwap` | n/a | 508.54µs | 514.42µs | 5060 | n/a |
| `finance_query::ref_bench_vwma` | n/a | 502.57µs | 520.46µs | 4964 | n/a |
| `finance_query::ref_bench_williams_r` | n/a | 58.16µs | 59.25µs | 29 | n/a |
| `finance_query::ref_bench_wma` | n/a | 494.22µs | 506.03µs | 4969 | n/a |
| `finance_query::risk_beta` | n/a | 61.32µs | 61.70µs | 0 | n/a |
| `finance_query::risk_calmar` | n/a | 16.5ns | 16.8ns | 0 | n/a |
| `finance_query::risk_historical_cvar` | n/a | 339.15µs | 345.14µs | 2 | n/a |
| `finance_query::risk_historical_var` | n/a | 2.55ms | 2.73ms | 2 | n/a |
| `finance_query::risk_information_ratio` | n/a | 35.77µs | 36.45µs | 1 | n/a |
| `finance_query::risk_kelly_criterion` | n/a | 4.19µs | 4.30µs | 16 | n/a |
| `finance_query::risk_max_drawdown` | n/a | 8.03µs | 8.14µs | 1 | n/a |
| `finance_query::risk_omega_ratio` | n/a | 30.57µs | 30.72µs | 0 | n/a |
| `finance_query::risk_parametric_cvar` | n/a | 30.61µs | 30.78µs | 0 | n/a |
| `finance_query::risk_parametric_var` | n/a | 30.59µs | 30.75µs | 0 | n/a |
| `finance_query::risk_sharpe` | n/a | 30.60µs | 30.88µs | 0 | n/a |
| `finance_query::risk_sortino` | n/a | 30.61µs | 30.68µs | 0 | n/a |
| `finance_query::risk_tracking_error` | n/a | 36.12µs | 36.46µs | 1 | n/a |
| `finance_query::risk_ulcer_index` | n/a | 174.59µs | 175.29µs | 2 | n/a |
| `finance_query::risk_win_loss_stats` | n/a | 168.41µs | 171.95µs | 25 | n/a |
| `finance_query::rss_parse` | n/a | 5.48µs | 5.95µs | 48 | n/a |
| `finance_query::score_news` | n/a | 83.95µs | 86.09µs | 976 | n/a |
| `finance_query::score_transcript` | n/a | 4.62ms | 4.78ms | 42100 | n/a |
| `finance_query::ser_currencies` | n/a | 16.87µs | 17.75µs | 8 | n/a |
| `finance_query::stream_deserialize` | n/a | 1.21µs | 1.39µs | 4 | n/a |
| `finance_query::stream_serialize` | n/a | 1.01µs | 1.21µs | 4 | n/a |
| `finance_query::ticker_chart_then_cached` | n/a | 1.26µs | 1.30µs | 20 | n/a |
| `finance_query::ticker_quote_then_cached` | n/a | 12.51µs | 13.43µs | 4 | n/a |
| `finance_query::tickers_batch_quote_then_cached` | n/a | 9.72µs | 10.43µs | 16 | n/a |
| `finance_query::translate_dictionary` | n/a | 1.96µs | 2.01µs | 49 | n/a |
| `finance_query::translation_set_backend` | n/a | 21.4ns | 23.9ns | 1 | n/a |
| `finance_query::translation_translate` | n/a | 3.56µs | 3.75µs | 79 | n/a |
| `finance_query::translation_translate_with` | n/a | 3.46µs | 3.53µs | 77 | n/a |


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
