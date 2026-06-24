"""Type stubs for the finance-query Rust extension module.

These stubs cover the user-facing API. Model class attributes are listed for
common fields — full coverage of every PyO3 #[getter] is not necessary for
mypy strictness on call sites.
"""

from collections.abc import Awaitable
from typing import Any, Optional

# -------------------- Module metadata --------------------

__version__: str

def enable_logging(level: str = "INFO") -> None: ...
def edgar_init(email: str) -> None: ...
def edgar_init_with_config(email: str, app_name: str, timeout_seconds: int = 60) -> None: ...

# -------------------- Exceptions --------------------

class FinanceQueryError(Exception): ...
class NetworkError(FinanceQueryError): ...
class RateLimitError(FinanceQueryError): ...
class SymbolNotFound(FinanceQueryError): ...
class ParseError(FinanceQueryError): ...
class ConfigError(FinanceQueryError): ...

# -------------------- Indicator (data-carrying enum wrapper) --------------------

class Indicator:
    """Factory class for creating Indicator values to pass to ``Ticker.indicator()``."""

    @staticmethod
    def sma(period: int) -> "Indicator": ...
    @staticmethod
    def ema(period: int) -> "Indicator": ...
    @staticmethod
    def wma(period: int) -> "Indicator": ...
    @staticmethod
    def dema(period: int) -> "Indicator": ...
    @staticmethod
    def tema(period: int) -> "Indicator": ...
    @staticmethod
    def hma(period: int) -> "Indicator": ...
    @staticmethod
    def vwma(period: int) -> "Indicator": ...
    @staticmethod
    def alma(period: int, offset: float, sigma: float) -> "Indicator": ...
    @staticmethod
    def mcginley_dynamic(period: int) -> "Indicator": ...
    @staticmethod
    def rsi(period: int) -> "Indicator": ...
    @staticmethod
    def cci(period: int) -> "Indicator": ...
    @staticmethod
    def williams_r(period: int) -> "Indicator": ...
    @staticmethod
    def roc(period: int) -> "Indicator": ...
    @staticmethod
    def momentum(period: int) -> "Indicator": ...
    @staticmethod
    def cmo(period: int) -> "Indicator": ...
    @staticmethod
    def stochastic(k_period: int, k_slow: int, d_period: int) -> "Indicator": ...
    @staticmethod
    def stochastic_rsi(
        rsi_period: int, stoch_period: int, k_period: int, d_period: int
    ) -> "Indicator": ...
    @staticmethod
    def awesome_oscillator(fast: int, slow: int) -> "Indicator": ...
    @staticmethod
    def coppock_curve(wma_period: int, long_roc: int, short_roc: int) -> "Indicator": ...
    @staticmethod
    def macd(fast: int, slow: int, signal: int) -> "Indicator": ...
    @staticmethod
    def adx(period: int) -> "Indicator": ...
    @staticmethod
    def aroon(period: int) -> "Indicator": ...
    @staticmethod
    def supertrend(period: int, multiplier: float) -> "Indicator": ...
    @staticmethod
    def ichimoku(
        conversion: int, base: int, lagging: int, displacement: int
    ) -> "Indicator": ...
    @staticmethod
    def parabolic_sar(step: float, max: float) -> "Indicator": ...
    @staticmethod
    def bollinger(period: int, std_dev: float) -> "Indicator": ...
    @staticmethod
    def atr(period: int) -> "Indicator": ...
    @staticmethod
    def keltner_channels(
        period: int, multiplier: float, atr_period: int
    ) -> "Indicator": ...
    @staticmethod
    def donchian_channels(period: int) -> "Indicator": ...
    @staticmethod
    def true_range() -> "Indicator": ...
    @staticmethod
    def choppiness_index(period: int) -> "Indicator": ...
    @staticmethod
    def obv() -> "Indicator": ...
    @staticmethod
    def vwap() -> "Indicator": ...
    @staticmethod
    def mfi(period: int) -> "Indicator": ...
    @staticmethod
    def cmf(period: int) -> "Indicator": ...
    @staticmethod
    def chaikin_oscillator() -> "Indicator": ...
    @staticmethod
    def accumulation_distribution() -> "Indicator": ...
    @staticmethod
    def bull_bear_power(period: int) -> "Indicator": ...
    @staticmethod
    def elder_ray(period: int) -> "Indicator": ...
    @staticmethod
    def balance_of_power(period: Optional[int] = None) -> "Indicator": ...

# -------------------- IndicatorResult --------------------

class IndicatorResult:
    """Result returned by ``Ticker.indicator()``.

    Check ``.kind`` to determine which variant is present, then access the
    corresponding attributes.  Unrelated attributes return ``None``.

    Kinds and their attributes:

    * ``"Series"`` → ``.series: list[float | None]``
    * ``"Macd"`` → ``.macd_line``, ``.signal_line``, ``.histogram``
    * ``"Bollinger"`` / ``"Keltner"`` / ``"Donchian"`` → ``.upper``, ``.middle``, ``.lower``
    * ``"Stochastic"`` → ``.k``, ``.d``
    * ``"Aroon"`` → ``.aroon_up``, ``.aroon_down``
    * ``"SuperTrend"`` → ``.value``, ``.is_uptrend``
    * ``"Ichimoku"`` → ``.conversion_line``, ``.base_line``, ``.leading_span_a``,
      ``.leading_span_b``, ``.lagging_span``
    * ``"BullBearPower"`` / ``"ElderRay"`` → ``.bull_power``, ``.bear_power``
    """

    kind: str
    series: Optional[list[Optional[float]]]
    macd_line: Optional[list[Optional[float]]]
    signal_line: Optional[list[Optional[float]]]
    histogram: Optional[list[Optional[float]]]
    upper: Optional[list[Optional[float]]]
    middle: Optional[list[Optional[float]]]
    lower: Optional[list[Optional[float]]]
    k: Optional[list[Optional[float]]]
    d: Optional[list[Optional[float]]]
    aroon_up: Optional[list[Optional[float]]]
    aroon_down: Optional[list[Optional[float]]]
    value: Optional[list[Optional[float]]]
    is_uptrend: Optional[list[Optional[bool]]]
    conversion_line: Optional[list[Optional[float]]]
    base_line: Optional[list[Optional[float]]]
    leading_span_a: Optional[list[Optional[float]]]
    leading_span_b: Optional[list[Optional[float]]]
    lagging_span: Optional[list[Optional[float]]]
    bull_power: Optional[list[Optional[float]]]
    bear_power: Optional[list[Optional[float]]]

    def to_dict(self) -> dict[str, Any]: ...

# -------------------- Enums --------------------

class Interval:
    OneMinute: "Interval"
    FiveMinutes: "Interval"
    FifteenMinutes: "Interval"
    ThirtyMinutes: "Interval"
    OneHour: "Interval"
    OneDay: "Interval"
    OneWeek: "Interval"
    OneMonth: "Interval"
    ThreeMonths: "Interval"

class TimeRange:
    OneDay: "TimeRange"
    FiveDays: "TimeRange"
    OneMonth: "TimeRange"
    ThreeMonths: "TimeRange"
    SixMonths: "TimeRange"
    OneYear: "TimeRange"
    TwoYears: "TimeRange"
    FiveYears: "TimeRange"
    TenYears: "TimeRange"
    YearToDate: "TimeRange"
    Max: "TimeRange"

class Frequency:
    Annual: "Frequency"
    Quarterly: "Frequency"

class StatementType:
    Income: "StatementType"
    Balance: "StatementType"
    CashFlow: "StatementType"

class ValueFormat:
    Raw: "ValueFormat"
    Pretty: "ValueFormat"
    Both: "ValueFormat"

class Region:
    # ~28 country-level variants. Common ones documented for autocomplete;
    # additional variants exist at runtime.
    UnitedStates: "Region"

class Sector:
    Technology: "Sector"
    FinancialServices: "Sector"
    # ~11 total

class Screener:
    DayGainers: "Screener"
    DayLosers: "Screener"
    # ~15 total

class ExchangeCode:
    # ~20 variants — accessible at runtime.
    ...

class Industry:
    # ~147 variants — accessible at runtime.
    ...

class Provider:
    Yahoo: "Provider"
    Edgar: "Provider"
    # Feature-gated variants available when the corresponding feature is enabled:
    # Polygon, Fmp, AlphaVantage, CoinGecko, Fred

class FearGreedLabel:
    ExtremeFear: "FearGreedLabel"
    Fear: "FearGreedLabel"
    Neutral: "FearGreedLabel"
    Greed: "FearGreedLabel"
    ExtremeGreed: "FearGreedLabel"

class SentimentLabel:
    Bullish: "SentimentLabel"
    Neutral: "SentimentLabel"
    Bearish: "SentimentLabel"

# -------------------- Models (forward-declared; key fields only) --------------------

class Quote:
    symbol: str
    short_name: Optional[str]
    long_name: Optional[str]
    # Many more fields available at runtime via #[getter]; not all listed.
    def to_dict(self) -> dict[str, Any]: ...

class Candle:
    open: float
    high: float
    low: float
    close: float
    volume: int
    def to_dataframe(self) -> Any: ...  # polars.DataFrame
    def to_dict(self) -> dict[str, Any]: ...

class ChartMeta:
    def to_dict(self) -> dict[str, Any]: ...

class Chart:
    candles: list[Candle]
    meta: ChartMeta
    def to_dataframe(self) -> Any: ...  # polars.DataFrame
    def to_dict(self) -> dict[str, Any]: ...

class Dividend:
    def to_dataframe(self) -> Any: ...
    def to_dict(self) -> dict[str, Any]: ...

class Split:
    def to_dataframe(self) -> Any: ...
    def to_dict(self) -> dict[str, Any]: ...

class CapitalGain:
    def to_dataframe(self) -> Any: ...
    def to_dict(self) -> dict[str, Any]: ...

class FinancialStatement:
    def to_dict(self) -> dict[str, Any]: ...

class News:
    title: str
    link: str
    publisher: str
    def to_dataframe(self) -> Any: ...
    def to_dict(self) -> dict[str, Any]: ...

class Recommendation:
    def to_dict(self) -> dict[str, Any]: ...

class EdgarSubmissions:
    def to_dict(self) -> dict[str, Any]: ...

class ProviderFiling:
    accession_number: Optional[str]
    filing_date: Optional[str]
    filing_type: Optional[str]
    filing_url: Optional[str]
    company_name: Optional[str]
    cik: Optional[str]
    def to_dict(self) -> dict[str, Any]: ...

class ProviderFilings:
    symbol: str
    filings: list[ProviderFiling]
    def to_dict(self) -> dict[str, Any]: ...

class CompanyFacts:
    cik: Optional[int]
    entity_name: Optional[str]
    def to_dict(self) -> Any: ...

class SearchQuote:
    symbol: str
    def to_dataframe(self) -> Any: ...
    def to_dict(self) -> dict[str, Any]: ...

class ScreenerQuote:
    symbol: str
    def to_dict(self) -> dict[str, Any]: ...

class ScreenerResults:
    def to_dict(self) -> dict[str, Any]: ...

class TrendingQuote:
    symbol: str
    def to_dict(self) -> dict[str, Any]: ...

class FearAndGreed:
    value: int
    classification: FearGreedLabel
    timestamp: int
    def to_dict(self) -> dict[str, Any]: ...

class Sentiment:
    label: SentimentLabel
    score: float
    confidence: float
    def to_dict(self) -> dict[str, Any]: ...

class LookupResults:
    def to_dict(self) -> dict[str, Any]: ...

class MarketSummaryQuote:
    def to_dict(self) -> dict[str, Any]: ...

class MarketHours:
    def to_dict(self) -> dict[str, Any]: ...

class SectorData:
    def to_dict(self) -> dict[str, Any]: ...

class Currency:
    def to_dict(self) -> dict[str, Any]: ...

class IndustryData:
    def to_dict(self) -> dict[str, Any]: ...

class Exchange:
    def to_dict(self) -> dict[str, Any]: ...

class Spark:
    def to_dict(self) -> dict[str, Any]: ...

class Options:
    def to_dict(self) -> dict[str, Any]: ...

class StochasticData:
    k: Optional[float]
    d: Optional[float]
    def to_dict(self) -> dict[str, Any]: ...

class MacdData:
    macd: Optional[float]
    signal: Optional[float]
    histogram: Optional[float]
    def to_dict(self) -> dict[str, Any]: ...

class AroonData:
    aroon_up: Optional[float]
    aroon_down: Optional[float]
    def to_dict(self) -> dict[str, Any]: ...

class BollingerBandsData:
    upper: Optional[float]
    middle: Optional[float]
    lower: Optional[float]
    def to_dict(self) -> dict[str, Any]: ...

class SuperTrendData:
    value: Optional[float]
    trend: Optional[str]
    def to_dict(self) -> dict[str, Any]: ...

class IchimokuData:
    conversion_line: Optional[float]
    base_line: Optional[float]
    leading_span_a: Optional[float]
    leading_span_b: Optional[float]
    lagging_span: Optional[float]
    def to_dict(self) -> dict[str, Any]: ...

class KeltnerChannelsData:
    upper: Optional[float]
    middle: Optional[float]
    lower: Optional[float]
    def to_dict(self) -> dict[str, Any]: ...

class DonchianChannelsData:
    upper: Optional[float]
    middle: Optional[float]
    lower: Optional[float]
    def to_dict(self) -> dict[str, Any]: ...

class BullBearPowerData:
    bull_power: Optional[float]
    bear_power: Optional[float]
    def to_dict(self) -> dict[str, Any]: ...

class ElderRayData:
    bull_power: Optional[float]
    bear_power: Optional[float]
    def to_dict(self) -> dict[str, Any]: ...

class RiskSummary:
    var_95: float
    var_99: float
    parametric_var_95: float
    sharpe: Optional[float]
    sortino: Optional[float]
    calmar: Optional[float]
    beta: Optional[float]
    max_drawdown: float
    max_drawdown_recovery_periods: Optional[int]
    def to_dict(self) -> dict[str, Any]: ...

class IndicatorsSummary:
    sma_10: Optional[float]
    sma_20: Optional[float]
    sma_50: Optional[float]
    sma_100: Optional[float]
    sma_200: Optional[float]
    ema_10: Optional[float]
    ema_20: Optional[float]
    ema_50: Optional[float]
    ema_100: Optional[float]
    ema_200: Optional[float]
    wma_10: Optional[float]
    wma_20: Optional[float]
    wma_50: Optional[float]
    wma_100: Optional[float]
    wma_200: Optional[float]
    dema_20: Optional[float]
    tema_20: Optional[float]
    hma_20: Optional[float]
    vwma_20: Optional[float]
    alma_9: Optional[float]
    mcginley_dynamic_20: Optional[float]
    rsi_14: Optional[float]
    stochastic: Optional[StochasticData]
    stochastic_rsi: Optional[StochasticData]
    cci_20: Optional[float]
    williams_r_14: Optional[float]
    roc_12: Optional[float]
    momentum_10: Optional[float]
    cmo_14: Optional[float]
    awesome_oscillator: Optional[float]
    coppock_curve: Optional[float]
    macd: Optional[MacdData]
    adx_14: Optional[float]
    aroon: Optional[AroonData]
    supertrend: Optional[SuperTrendData]
    ichimoku: Optional[IchimokuData]
    parabolic_sar: Optional[float]
    bull_bear_power: Optional[BullBearPowerData]
    elder_ray_index: Optional[ElderRayData]
    bollinger_bands: Optional[BollingerBandsData]
    atr_14: Optional[float]
    keltner_channels: Optional[KeltnerChannelsData]
    donchian_channels: Optional[DonchianChannelsData]
    true_range: Optional[float]
    choppiness_index_14: Optional[float]
    obv: Optional[float]
    mfi_14: Optional[float]
    cmf_20: Optional[float]
    chaikin_oscillator: Optional[float]
    accumulation_distribution: Optional[float]
    vwap: Optional[float]
    balance_of_power: Optional[float]
    def to_dataframe(self) -> Any: ...  # polars.DataFrame
    def to_dict(self) -> dict[str, Any]: ...

class DividendAnalytics:
    total_paid: float
    payment_count: int
    average_payment: float
    cagr: Optional[float]
    last_payment: Optional[Dividend]
    first_payment: Optional[Dividend]
    def to_dict(self) -> dict[str, Any]: ...

class FormattedValueF64:
    raw: Optional[float]
    fmt: Optional[str]
    long_fmt: Optional[str]

class FormattedValueI64:
    raw: Optional[int]
    fmt: Optional[str]
    long_fmt: Optional[str]

class FormattedValueU64:
    raw: Optional[int]
    fmt: Optional[str]
    long_fmt: Optional[str]

class FormattedValueString:
    raw: Optional[str]
    fmt: Optional[str]
    long_fmt: Optional[str]

# -------------------- BacktestResult --------------------

class BacktestResult:
    """Result of a single-symbol backtest run."""
    symbol: str
    strategy_name: str
    start_timestamp: int
    end_timestamp: int
    initial_capital: float
    final_equity: float
    total_return_pct: float
    annualized_return_pct: float
    sharpe_ratio: float
    sortino_ratio: float
    max_drawdown_pct: float
    win_rate: float
    profit_factor: float
    total_trades: int
    winning_trades: int
    losing_trades: int
    calmar_ratio: float
    sqn: float
    expectancy: float
    diagnostics: list[str]
    def to_dict(self) -> dict[str, Any]: ...

# -------------------- PortfolioResult --------------------

class PortfolioResult:
    """Result of a multi-symbol portfolio backtest run."""
    initial_capital: float
    final_equity: float
    total_return_pct: float
    sharpe_ratio: float
    max_drawdown_pct: float
    total_trades: int
    symbols: list[str]
    def symbol_results(self) -> dict[str, BacktestResult]: ...
    def to_dict(self) -> dict[str, Any]: ...

# -------------------- BatchResult --------------------

class BatchResult:
    data: dict[str, Any]
    errors: dict[str, str]

# -------------------- TickerBuilder --------------------

class TickerBuilder:
    def lang(self, lang: str) -> "TickerBuilder": ...
    def region_code(self, region: str) -> "TickerBuilder": ...
    def region(self, region: Region) -> "TickerBuilder": ...
    def timeout(self, seconds: int) -> "TickerBuilder": ...
    def proxy(self, proxy: str) -> "TickerBuilder": ...
    def cache(self, ttl_seconds: int) -> "TickerBuilder": ...
    def logo(self) -> "TickerBuilder": ...
    def build(self) -> Awaitable["Ticker"]: ...

# -------------------- Ticker --------------------

class Ticker:
    symbol: str

    @staticmethod
    def new(symbol: str) -> Awaitable["Ticker"]: ...
    @staticmethod
    def builder(symbol: str) -> TickerBuilder: ...
    def quote(self) -> Awaitable[Quote]: ...
    def chart(self, interval: Interval, range: TimeRange) -> Awaitable[Chart]: ...
    def chart_range(
        self, interval: Interval, start: int, end: int
    ) -> Awaitable[Chart]: ...
    def dividends(self, range: TimeRange) -> Awaitable[list[Dividend]]: ...
    def splits(self, range: TimeRange) -> Awaitable[list[Split]]: ...
    def capital_gains(self, range: TimeRange) -> Awaitable[list[CapitalGain]]: ...
    def financials(
        self, statement: StatementType, frequency: Frequency
    ) -> Awaitable[FinancialStatement]: ...
    def news(self) -> Awaitable[list[News]]: ...
    def recommendations(self, limit: int) -> Awaitable[Recommendation]: ...
    def edgar_submissions(self) -> Awaitable[EdgarSubmissions]: ...
    def options(self, date: int | None = None) -> Awaitable[Options]: ...
    def dividend_analytics(self, range: TimeRange) -> Awaitable[DividendAnalytics]: ...
    def filings(self) -> Awaitable[ProviderFilings]: ...
    def edgar_company_facts(self) -> Awaitable[CompanyFacts]: ...
    def news_sentiment(self) -> Awaitable[Sentiment]: ...
    def indicators(self, interval: Interval, range: TimeRange) -> Awaitable[IndicatorsSummary]: ...
    def indicator(
        self, indicator: Indicator, interval: Interval, range: TimeRange
    ) -> Awaitable[IndicatorResult]: ...
    def risk(
        self, interval: Interval, range: TimeRange, benchmark: str | None = None
    ) -> Awaitable[RiskSummary]: ...
    def backtest(
        self, strategy: object, interval: Interval, range: TimeRange
    ) -> Awaitable[BacktestResult]: ...
    def backtest_with_benchmark(
        self, strategy: object, interval: Interval, range: TimeRange, benchmark: str
    ) -> Awaitable[BacktestResult]: ...

# -------------------- Tickers --------------------

class Tickers:
    @staticmethod
    def new(symbols: list[str]) -> Awaitable["Tickers"]: ...
    def symbols(self) -> list[str]: ...
    def __len__(self) -> int: ...
    def quotes(self) -> Awaitable[BatchResult]: ...
    def chart(self, symbol: str, interval: Interval, range: TimeRange) -> Awaitable[Chart]: ...
    def charts(
        self, interval: Interval, range: TimeRange
    ) -> Awaitable[BatchResult]: ...
    def charts_range(self, interval: Interval, start: int, end: int) -> Awaitable[BatchResult]: ...
    def dividends(self, range: TimeRange) -> Awaitable[BatchResult]: ...
    def splits(self, range: TimeRange) -> Awaitable[BatchResult]: ...
    def capital_gains(self, range: TimeRange) -> Awaitable[BatchResult]: ...
    def recommendations(self, limit: int) -> Awaitable[BatchResult]: ...
    def financials(self, statement: StatementType, frequency: Frequency) -> Awaitable[BatchResult]: ...
    def spark(self, interval: Interval, range: TimeRange) -> Awaitable[BatchResult]: ...
    def options(self, date: int | None = None) -> Awaitable[BatchResult]: ...
    def news(self) -> Awaitable[BatchResult]: ...
    def quote(self, symbol: str) -> Awaitable[Quote]: ...
    def clear_cache(self) -> Awaitable[None]: ...
    def clear_quote_cache(self) -> Awaitable[None]: ...
    def clear_chart_cache(self) -> Awaitable[None]: ...
    def indicators(self, interval: Interval, range: TimeRange) -> Awaitable[BatchResult]: ...
    def backtest(
        self, strategy: object, interval: Interval, range: TimeRange
    ) -> Awaitable[PortfolioResult]: ...

# -------------------- finance submodule --------------------

class _FinanceSubmodule:
    def search(self, query: str) -> Awaitable[list[SearchQuote]]: ...
    def screener(
        self, screener: Screener, count: Optional[int] = ...
    ) -> Awaitable[ScreenerResults]: ...
    def trending(
        self, region: Optional[Region] = ...
    ) -> Awaitable[list[TrendingQuote]]: ...
    def fear_and_greed(self) -> Awaitable[FearAndGreed]: ...
    def lookup(self, query: str) -> Awaitable[LookupResults]: ...
    def market_summary(
        self, region: Optional[Region] = ...
    ) -> Awaitable[list[MarketSummaryQuote]]: ...
    def hours(self, region: Optional[str] = ...) -> Awaitable[MarketHours]: ...
    def sector(self, sector_type: Sector) -> Awaitable[SectorData]: ...
    def currencies(self) -> Awaitable[list[Currency]]: ...
    def news(self) -> Awaitable[list[News]]: ...
    def industry(self, industry_key: str) -> Awaitable[IndustryData]: ...
    def exchanges(self) -> Awaitable[list[Exchange]]: ...

finance: _FinanceSubmodule

# -------------------- Prebuilt backtest strategy classes --------------------

class SmaCrossover:
    """Dual SMA crossover trend-following strategy."""
    def __init__(self, fast_period: int, slow_period: int) -> None: ...

class RsiReversal:
    """RSI mean-reversion strategy."""
    def __init__(self, period: int) -> None: ...

class MacdSignal:
    """MACD line crossover strategy."""
    def __init__(self, fast: int, slow: int, signal: int) -> None: ...

class BollingerMeanReversion:
    """Bollinger Bands mean-reversion strategy."""
    def __init__(self, period: int, std_dev: float) -> None: ...

class SuperTrendFollow:
    """SuperTrend trend-following strategy."""
    def __init__(self, period: int, multiplier: float) -> None: ...

class DonchianBreakout:
    """Donchian channel breakout strategy."""
    def __init__(self, period: int) -> None: ...
