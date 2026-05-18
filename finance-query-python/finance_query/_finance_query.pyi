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

class FearGreedLabel:
    ExtremeFear: "FearGreedLabel"
    Fear: "FearGreedLabel"
    Neutral: "FearGreedLabel"
    Greed: "FearGreedLabel"
    ExtremeGreed: "FearGreedLabel"

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
    def clear_cache(self) -> Awaitable[None]: ...
    def clear_quote_cache(self) -> Awaitable[None]: ...
    def clear_chart_cache(self) -> Awaitable[None]: ...

# -------------------- Tickers --------------------

class Tickers:
    @staticmethod
    def new(symbols: list[str]) -> Awaitable["Tickers"]: ...
    def symbols(self) -> list[str]: ...
    def __len__(self) -> int: ...
    def quotes(self) -> Awaitable[BatchResult]: ...
    def charts(
        self, interval: Interval, range: TimeRange
    ) -> Awaitable[BatchResult]: ...

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
