from typing_extensions import OrderedDict

from src.models.indicators import SMAData, TechnicalIndicator, EMAData, WMAData, VWMAData, Indicator
from src.models.historical_data import TimeRange, Interval
from src.services.historical.get_historical import get_historical
from src.services.indicators.core import (calculate_sma, calculate_ema, calculate_wma, calculate_vwma,
                                          prepare_price_data, create_indicator_dict)


async def get_sma(
        symbol: str,
        time_range: TimeRange,
        interval: Interval,
        period: int = 10,
        epoch: bool = False
) -> dict:
    """
    Get the Simple Moving Average (SMA) for a symbol.
    :param symbol: the stock symbol
    :param time_range: the time range of the data (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of data points used in the moving average calculation (default 10). Shorter
                  periods (e.g., 10, 20) are more responsive to recent price changes and commonly used for
                  short-term trading, while longer periods (e.g., 50, 200) smooth out price action and are
                  often used to identify long-term trends
    :param epoch: Whether to return the dates as epoch timestamps (default False)

    :raises HTTPException: with status code 400 on invalid range or interval, 404 if the symbol cannot be found, or 500 for any other error
    """
    quotes = await get_historical(symbol, time_range=time_range, interval=interval, epoch=epoch)

    dates, prices, _, _, _ = prepare_price_data(quotes)

    sma_values = calculate_sma(prices, period=period)

    indicator_data = {
        date: SMAData(value=value)
        for date, value in create_indicator_dict(dates, sma_values).items()
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return TechnicalIndicator(
        type=Indicator.SMA,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_ema(
        symbol: str,
        time_range: TimeRange,
        interval: Interval,
        period: int = 10,
        epoch: bool = False
) -> dict:
    """
    Get the Exponential Moving Average (EMA) for a symbol.
    :param symbol: the stock symbol
    :param time_range: the time range of the data (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of data points used in the exponential moving average calculation (default 10).
                  Unlike SMA, EMA gives more weight to recent prices through an exponential multiplier.
    :param epoch: Whether to return the dates as epoch timestamps (default False)

    :raises HTTPException: with status code 400 on invalid range or interval, 404 if the symbol cannot be found, or 500 for any other error
    """
    quotes = await get_historical(symbol, time_range=time_range, interval=interval, epoch=epoch)

    dates, prices, _, _, _ = prepare_price_data(quotes)
    ema_values = calculate_ema(prices, period=period)

    indicator_data = {
        date: EMAData(value=value)
        for date, value in create_indicator_dict(dates, ema_values).items()
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return TechnicalIndicator(
        type=Indicator.EMA,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_wma(
        symbol: str,
        time_range: TimeRange,
        interval: Interval,
        period: int = 10,
        epoch: bool = False
) -> dict:
    """
    Get the Weighted Moving Average (WMA) for a symbol.
    :param symbol: the stock symbol
    :param time_range: the time range of the data (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
    :param interval: The timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of data points used in the weighted moving average calculation (default 10).
                  WMA assigns a linear weighting that decreases arithmetically (n, n-1, n-2, ..., 1) from
                  the most recent price to the oldest, providing more sensitivity to recent
                  prices than SMA but less aggressive than EMA
    :param epoch: Whether to return the dates as epoch timestamps (default False)

    :raises HTTPException: with status code 400 on invalid range or interval, 404 if the symbol cannot be found, or 500 for any other error
    """
    quotes = await get_historical(symbol, time_range=time_range, interval=interval, epoch=epoch)

    dates, prices, _, _, _ = prepare_price_data(quotes)
    wma_values = calculate_wma(prices, period=period)

    indicator_data = {
        date: WMAData(value=value)
        for date, value in create_indicator_dict(dates, wma_values).items()
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return TechnicalIndicator(
        type=Indicator.WMA,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_vwma(
        symbol: str,
        time_range: TimeRange,
        interval: Interval,
        period: int = 20,
        epoch: bool = False
) -> dict:
    """
    Get the Volume Weighted Moving Average (VWMA) for a symbol.
    :param symbol: the stock symbol
    :param time_range: the time range of the data (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of data points used in the volume-weighted moving average calculation
                  (default 10). VWMA weights each price by its trading volume, giving more importance
                  to prices with higher volume. This helps identify significant price levels where substantial
                  trading activity occurred. High-volume price movements have more impact on VWMA than
                  low-volume moves, making it useful for identifying stronger support/resistance levels
    :param epoch: Whether to return the dates as epoch timestamps (default False)

    :raises HTTPException: with status code 400 on invalid range or interval, 404 if the symbol cannot be found, or 500 for any other error
    """
    quotes = await get_historical(symbol, time_range=time_range, interval=interval, epoch=epoch)

    dates, prices, _, _, volumes = prepare_price_data(quotes)
    vwma_values = calculate_vwma(prices, volumes, period=period)

    indicator_data = {
        date: VWMAData(value=value)
        for date, value in create_indicator_dict(dates, vwma_values).items()
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return TechnicalIndicator(
        type=Indicator.VWMA,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)
