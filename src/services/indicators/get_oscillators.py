from typing_extensions import OrderedDict
import numpy as np

from src.models.analysis import RSIData, Analysis, SRSIData, STOCHData, CCIData, Indicator
from src.models.historical_data import TimePeriod, Interval
from src.services.historical.get_historical import get_historical
from src.services.indicators.core import (calculate_rsi, calculate_stoch_rsi, calculate_stoch, calculate_cci,
                                          prepare_price_data, create_indicator_dict)


async def get_rsi(symbol: str, interval: Interval, period: int = 14):
    """
    Get the Relative Strength Index (RSI) for a symbol. RSI measures the speed and magnitude of recent price
    changes to evaluate overbought or oversold conditions. It oscillates between 0 and 100, with traditional
    overbought levels at 70 and oversold levels at 30.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of periods used to calculate the average gains and losses. The default of 14
                  is standard - shorter periods (e.g., 9) create a more volatile indicator that's more
                  sensitive to recent price changes, while longer periods (e.g., 25) produce a smoother
                  RSI that responds more slowly to price changes but may help identify longer-term trends.
                  The first calculation uses a simple average, while subsequent calculations use an
                  exponentially weighted moving average

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical(symbol, period=TimePeriod.YEAR, interval=interval)

    dates, prices, _, _, _ = prepare_price_data(quotes)
    rsi_values = calculate_rsi(prices, period=period)

    indicator_data = {
        date: RSIData(value=value)
        for date, value in create_indicator_dict(dates, rsi_values).items()
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.RSI,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_srsi(
        symbol: str,
        interval: Interval,
        period: int = 14,
        stoch_period: int = 14,
        signal_period: int = 3,
        smooth: int = 3
):
    """
    Get the Stochastic RSI (SRSI) for a symbol. SRSI applies the Stochastic Oscillator formula to RSI values
    instead of price data, resulting in an indicator that measures the relative position of RSI within its
    historical range.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of periods used to calculate the initial RSI value. A larger period creates a
                  smoother RSI line with fewer signals
    :param stoch_period: The lookback window used to find the highest high and lowest low of the RSI values.
                        This determines how many RSI values are used to calculate the %K line of the SRSI
    :param signal_period: The number of periods used to calculate the %D line (the SMA of %K). A smaller value
                         creates a more responsive signal line that may generate more trading signals
    :param smooth: The number of periods used to smooth the %K line before calculating %D. Higher values reduce
                  noise but increase lag in the indicator

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical(symbol, period=TimePeriod.YEAR, interval=interval)

    dates, prices, _, _, _ = prepare_price_data(quotes)
    k_values, d_values = calculate_stoch_rsi(
        prices,
        rsi_period=period,
        stoch_period=stoch_period,
        smooth=smooth,
        signal_period=signal_period
    )

    k_dict = create_indicator_dict(dates, k_values)
    d_dict = create_indicator_dict(dates, d_values)

    indicator_data = {
        date: SRSIData(k=k_dict[date], d=d_dict[date])
        for date in k_dict.keys() & d_dict.keys()
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.SRSI,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_stoch(
        symbol: str,
        interval: Interval,
        period: int = 14,
        smooth: int = 3,
        signal_period: int = 3
):
    """
    Get the Stochastic Oscillator (STOCH) for a symbol. The Stochastic Oscillator measures the position of
    the closing price relative to the high-low range over a specified period, helping identify overbought
    and oversold conditions.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The lookback period used to find the highest high and lowest low for calculating %K.
                  This determines how many periods of price data are used to establish the trading range.
                  A larger period considers more historical price data and may identify longer-term cycles
    :param signal_period: The number of periods used to calculate the %D line (the SMA of %K). A smaller
                         value makes the signal line more responsive to recent price changes but may
                         generate more false signals
    :param smooth: The number of periods used to smooth the %K line before calculating %D. Higher values
                  produce a smoother indicator that's less prone to whipsaws but may delay signal generation

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical(symbol, period=TimePeriod.YEAR, interval=interval)

    dates, prices, highs, lows, _ = prepare_price_data(quotes)

    k_values, d_values = calculate_stoch(
        highs,
        lows,
        prices,
        period=period,
        smooth=smooth,
        signal_period=signal_period
    )

    k_dict = create_indicator_dict(dates, k_values)
    d_dict = create_indicator_dict(dates, d_values)

    indicator_data = {
        date: STOCHData(k=k_dict[date], d=d_dict[date])
        for date in k_dict.keys() & d_dict.keys()
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.STOCH,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_cci(symbol: str, interval: Interval, period: int = 20):
    """
    Get the Commodity Channel Index (CCI) for a symbol. CCI measures the current price level relative to an
    average price level over a given period of time. The indicator oscillates above and below zero, with
    readings above +100 suggesting overbought conditions and below -100 suggesting oversold conditions.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of periods used to calculate both the Simple Moving Average (SMA) of typical
                  prices and the Mean Deviation. The default of 20 is standard - lower values make the
                  indicator more sensitive to price changes but may generate more false signals, while
                  higher values create a smoother line better suited for identifying longer-term trends

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical(symbol, period=TimePeriod.YEAR, interval=interval)

    dates, close_prices, high_prices, low_prices, _ = prepare_price_data(quotes)

    cci_values = calculate_cci(high_prices, low_prices, close_prices, period=period)

    indicator_data = {
        dates[i]: CCIData(value=round(float(cci_values[i]), 2))
        for i in range(len(dates))
        if not np.isnan(cci_values[i])
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.CCI,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)
