from stock_indicators import indicators
from typing_extensions import OrderedDict

from src.cache import cache
from src.models.analysis import SMAData, Analysis, EMAData, WMAData, VWMAData, Indicator
from src.models.historical_data import TimePeriod, Interval
from src.services.historical.get_historical import get_historical_quotes


@cache(expire=60, market_closed_expire=600)
async def get_sma(symbol: str, interval: Interval, period: int = 10) -> dict:
    """
    Get the Simple Moving Average (SMA) for a symbol.
    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of data points used in the moving average calculation (default 10). Shorter
                  periods (e.g., 10, 20) are more responsive to recent price changes and commonly used for
                  short-term trading, while longer periods (e.g., 50, 200) smooth out price action and are
                  often used to identify long-term trends

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical_quotes(symbol, period=TimePeriod.MAX, interval=interval)
    results = indicators.get_sma(quotes, period)
    indicator_data = {result.date.date(): SMAData(value=round(result.sma, 2)) for result in results if
                      result.sma is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.SMA,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


@cache(expire=60, market_closed_expire=600)
async def get_ema(symbol: str, interval: Interval, period: int = 10):
    """
    Get the Exponential Moving Average (EMA) for a symbol.
    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of data points used in the exponential moving average calculation (default 10).
                  Unlike SMA, EMA gives more weight to recent prices through an exponential multiplier.

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical_quotes(symbol, period=TimePeriod.MAX, interval=interval)
    results = indicators.get_ema(quotes, period).remove_warmup_periods()
    indicator_data = {result.date.date(): EMAData(value=round(result.ema, 2)) for result in results if
                      result.ema is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.EMA,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


@cache(expire=60, market_closed_expire=600)
async def get_wma(symbol: str, interval: Interval, period: int = 10):
    """
    Get the Weighted Moving Average (WMA) for a symbol.
    :param symbol: the stock symbol
    :param interval: The timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of data points used in the weighted moving average calculation (default 10).
                  WMA assigns a linear weighting that decreases arithmetically (n, n-1, n-2, ..., 1) from
                  the most recent price to the oldest, providing more sensitivity to recent
                  prices than SMA but less aggressive than EMA

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical_quotes(symbol, period=TimePeriod.MAX, interval=interval)
    results = indicators.get_wma(quotes, period).remove_warmup_periods()
    indicator_data = {result.date.date(): WMAData(value=round(result.wma, 2)) for result in results if
                      result.wma is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.WMA,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


@cache(expire=60, market_closed_expire=600)
async def get_vwma(symbol: str, interval: Interval, period: int = 20):
    """
    Get the Volume Weighted Moving Average (VWMA) for a symbol.
    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of data points used in the volume-weighted moving average calculation
                  (default 10). VWMA weights each price by its trading volume, giving more importance
                  to prices with higher volume. This helps identify significant price levels where substantial
                  trading activity occurred. High-volume price movements have more impact on VWMA than
                  low-volume moves, making it useful for identifying stronger support/resistance levels

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical_quotes(symbol, period=TimePeriod.MAX, interval=interval)
    results = indicators.get_vwma(quotes, period).remove_warmup_periods()
    indicator_data = {result.date.date(): VWMAData(value=round(result.vwma, 2)) for result in results
                      if result.vwma is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.VWMA,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)
