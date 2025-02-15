import numpy as np
from typing_extensions import OrderedDict

from src.models.indicators import (MACDData, TechnicalIndicator, ADXData, AROONData, BBANDSData, OBVData, SuperTrendData,
                                   IchimokuData, Indicator)
from src.models.historical_data import TimeRange, Interval
from src.services.historical.get_historical import get_historical
from src.services.indicators.core import (
    prepare_price_data, create_indicator_dict, calculate_macd, calculate_adx,
    calculate_aroon, calculate_bbands,
    calculate_obv, calculate_ichimoku, calculate_supertrend
)


async def get_macd(
        symbol: str,
        time_range: TimeRange,
        interval: Interval,
        fast_period: int = 12,
        slow_period: int = 26,
        signal_period: int = 9,
        epoch: bool = False
) -> dict:
    """
    Get the Moving Average Convergence Divergence (MACD) for a symbol.

    The MACD is a trend-following momentum indicator that shows the relationship between two moving averages of a
    security's price. It is calculated by subtracting the 26-period Exponential Moving Average (EMA) from the 12-period EMA.
    The result of this calculation is the MACD line. A nine-day EMA of the MACD called the "signal line," is then
    plotted above the MACD line, which can act as a trigger for buy and sell signals.

    :param symbol: the stock symbol
    :param time_range: the time range of the data (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param fast_period: the number of periods used to calculate the fast EMA (default 12). A shorter period
    :param slow_period: the number of periods used to calculate the slow EMA (default 26). A longer period
    :param signal_period: the number of periods used to calculate the signal line (default 9). A shorter period
    :param epoch: Whether to return the dates as epoch timestamps (default False)

    :raises HTTPException: with status code 400 on invalid range or interval, 404 if the symbol cannot be found, or 500 for any other error
    """
    quotes = await get_historical(symbol, time_range=time_range, interval=interval, epoch=epoch)
    dates, prices, _, _, _ = prepare_price_data(quotes)
    macd_line, signal_line = calculate_macd(prices, fast_period=fast_period,
                                            slow_period=slow_period, signal_period=signal_period)

    macd_dict = create_indicator_dict(dates, macd_line)
    signal_dict = create_indicator_dict(dates, signal_line)

    # Create indicator data only for dates present in both dictionaries
    indicator_data = {
        date: MACDData(
            value=macd_dict[date],
            signal=signal_dict[date]
        )
        for date in macd_dict.keys() if date in signal_dict
    }

    # Sort the dictionary by date in reverse order
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return TechnicalIndicator(
        type=Indicator.MACD,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_adx(
        symbol: str,
        time_range: TimeRange,
        interval: Interval,
        period: int = 14,
        epoch: bool = False
) -> dict:
    """
    Get the Average Directional Index (ADX) for a symbol.
    The ADX is a trend strength indicator that quantifies the strength of a trend without indicating its direction.
    It is derived from the smoothed moving average of the difference between +DI and -DI, which are directional
    movement indicators. The ADX ranges from 0 to 100, with readings above 20 or 25 typically indicating a strong trend.

    :param symbol: the stock symbol
    :param time_range: the time range of the data (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of periods used to calculate the DMI lines and ADX (default 14). Lower values
                  create a more responsive indicator but may generate more false signals. Values above 25
                  typically indicate a strong trend
                  :param epoch: Whether to return the dates as epoch timestamps (default False)

    :raises HTTPException: with status code 400 on invalid range or interval, 404 if the symbol cannot be found, or 500 for any other error
    """
    quotes = await get_historical(symbol, time_range=time_range, interval=interval, epoch=epoch)

    dates, closes, highs, lows, _ = prepare_price_data(quotes)
    adx_values = calculate_adx(highs, lows, closes, period=period)

    indicator_data = {
        date: ADXData(value=round(value, 2))
        for date, value in create_indicator_dict(dates, adx_values).items()
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return TechnicalIndicator(
        type=Indicator.ADX,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_aroon(symbol: str, interval: Interval, period: int = 25, epoch: bool = False) -> dict:
    """
    Get the Aroon indicator for a symbol.
    The Aroon indicator consists of two lines: Aroon Up and Aroon Down. Aroon Up measures the number of periods
    since the highest high within a specified period, while Aroon Down measures the number of periods since
    the lowest low within the same period. The Aroon indicator oscillates between 0 and 100, with readings above
    70 indicating a strong trend.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The lookback period for finding the highest high and lowest low (default 25). A longer
                  period helps identify more significant trends but may be less responsive to recent price
                  changes. Values range from 0 to 100, with readings above 70 indicating a strong trend
    :param epoch: Whether to return the dates as epoch timestamps (default False)

    :raises HTTPException: with status code 400 on invalid range or interval, 404 if the symbol cannot be found, or 500 for any other error
    """
    quotes = await get_historical(symbol, time_range=TimeRange.YEAR, interval=interval, epoch=epoch)

    dates, _, highs, lows, _ = prepare_price_data(quotes)
    aroon_up, aroon_down = calculate_aroon(highs, lows, period=period)

    up_dict = create_indicator_dict(dates, aroon_up)
    down_dict = create_indicator_dict(dates, aroon_down)

    # Create indicator data for dates present in both dictionaries
    indicator_data = {
        date: AROONData(
            aroon_up=up_dict[date],
            aroon_down=down_dict[date]
        )
        for date in up_dict.keys() if date in down_dict
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return TechnicalIndicator(
        type=Indicator.AROON,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_bbands(
        symbol: str,
        time_range: TimeRange,
        interval: Interval,
        period: int = 20,
        std_dev: int = 2,
        epoch: bool = False
) -> dict:
    """
    Get the Bollinger Bands (BBands) for a symbol.
    Bollinger Bands consist of a middle band (SMA) and two outer bands (standard deviations from the SMA).
    The bands expand and contract based on market volatility, with wider bands indicating higher volatility.

    :param symbol: the stock symbol
    :param time_range: the time range of the data (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of periods used to calculate the simple moving average and standard deviation
                  (default 20). A longer period creates wider, less reactive bands
    :param std_dev: The number of standard deviations for the upper and lower bands (default 2). Higher
                   values create wider bands that identify more extreme price movements. About 95% of price
                   action occurs within 2 standard deviations
    :param epoch: Whether to return the dates as epoch timestamps (default False)

    :raises HTTPException: with status code 400 on invalid range or interval, 404 if the symbol cannot be found, or 500 for any other error
    """
    quotes = await get_historical(symbol, time_range=time_range, interval=interval, epoch=epoch)

    dates, closes, _, _, _ = prepare_price_data(quotes)
    upper_band, middle_band, lower_band = calculate_bbands(closes, period=period, std_dev=std_dev)

    upper_band_dict = create_indicator_dict(dates, upper_band)
    middle_band_dict = create_indicator_dict(dates, middle_band)
    lower_band_dict = create_indicator_dict(dates, lower_band)

    # Create indicator data for dates present in all dictionaries
    indicator_data = {
        date: BBANDSData(
            upper_band=round(upper_band_dict[date], 2),
            middle_band=round(middle_band_dict[date], 2),
            lower_band=round(lower_band_dict[date], 2)
        )
        for date in upper_band_dict.keys() if date in middle_band_dict and date in lower_band_dict
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return TechnicalIndicator(
        type=Indicator.BBANDS,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_obv(symbol: str, time_range: TimeRange, interval: Interval, epoch: bool = False) -> dict:
    """
    Get the On-Balance Volume (OBV) for a symbol.
    The OBV is a volume-based indicator that uses volume flow to predict changes in stock price. It works on the
    principle that volume precedes price movement. If a security closes higher than the previous close, all
    volume for the day is considered up-volume. If it closes lower, all volume is down-volume. The OBV is
    calculated by adding the day's volume to a cumulative total when the close is up and subtracting it when
    the close is down.

    :param symbol: the stock symbol
    :param time_range: the time range of the data (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param epoch: Whether to return the dates as epoch timestamps (default False)

    :raises HTTPException: with status code 400 on invalid range or interval, 404 if the symbol cannot be found, or 500 for any other error
    """
    quotes = await get_historical(symbol, time_range=time_range, interval=interval, epoch=epoch)

    dates, closes, _, _, volumes = prepare_price_data(quotes)
    obv_values = calculate_obv(closes, volumes)

    indicator_data = {
        date: OBVData(value=round(value, 2))
        for date, value in create_indicator_dict(dates, obv_values).items()
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return TechnicalIndicator(
        type=Indicator.OBV,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_super_trend(
        symbol: str,
        time_range: TimeRange,
        interval: Interval,
        period: int = 10,
        multiplier: int = 3,
        epoch: bool = False
) -> dict:
    """
    Get the Super Trend indicator for a symbol.
    The Super Trend indicator is a trend-following indicator that uses the Average True Range (ATR) to determine
    the trend direction and generate buy/sell signals. It consists of two bands: the upper band and the lower band.
    When the price closes above the upper band, it indicates a potential buy signal, and when the price closes
    below the lower band, it indicates a potential sell signal.

    :param symbol: the stock symbol
    :param time_range: the time range of the data (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of periods used to calculate the Average True Range (default 10). A longer
                  period creates a smoother, less reactive indicator
    :param multiplier: The factor applied to the ATR to calculate the bands (default 3). A higher multiplier
                      creates wider bands that generate fewer signals but may reduce false breakouts
    :param epoch: Whether to return the dates as epoch timestamps (default False)

    :raises HTTPException: with status code 400 on invalid range or interval, 404 if the symbol cannot be found, or 500 for any other error
    """
    quotes = await get_historical(symbol, time_range=time_range, interval=interval, epoch=epoch)

    dates, closes, highs, lows, _ = prepare_price_data(quotes)
    supertrend_values, trend = calculate_supertrend(highs, lows, closes, period=period, multiplier=multiplier)

    # Create dictionaries for each Supertrend component
    supertrend_values_dict = create_indicator_dict(dates, supertrend_values)
    trend_dict = create_indicator_dict(dates, trend)

    # Create indicator data only for dates present in both dictionaries
    indicator_data = {
        date: SuperTrendData(
            value=round(value, 2),
            trend="UP" if trend_dict[date] > 0 else "DOWN"
        )
        for date, value in supertrend_values_dict.items() if date in trend_dict
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return TechnicalIndicator(
        type=Indicator.SUPER_TREND,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_ichimoku(
        symbol: str,
        time_range: TimeRange,
        interval: Interval,
        tenkan_period: int = 9,
        kijun_period: int = 26,
        senkou_period: int = 52,
        epoch: bool = False
) -> dict:
    """
    Get the Ichimoku Cloud (Ichimoku Kinko Hyo) for a symbol.
    The Ichimoku Cloud is a comprehensive indicator that defines support and resistance, identifies trend direction,
    gauges momentum, and provides trading signals. It consists of five main components: Tenkan-sen (Conversion Line),
    Kijun-sen (Baseline), Senkou Span A (Leading Span A), Senkou Span B (Leading Span B), and Chikou Span (Lagging Span).
    
    The Tenkan-sen and Kijun-sen lines are used to identify the current trend and potential reversals. The Senkou Span A
    and Senkou Span B lines form the "cloud," which provides dynamic support and resistance levels. The Chikou Span
    line is used to confirm the trend direction. The Ichimoku Cloud is best suited for identifying trends and
    potential reversals in trending markets.

    :param symbol: the stock symbol
    :param time_range: the time range of the data (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param tenkan_period: The period for calculating the Tenkan-sen (default 9). This shorter-term line is
                         more sensitive to price changes
    :param kijun_period: The period for calculating the Kijun-sen (default 26). This longer-term line acts
                        as a medium-term trend indicator
    :param senkou_period: The period for calculating Senkou Span B (default 52). This long-term line helps
                         form the cloud and indicates long-term trend direction
    :param epoch: Whether to return the dates as epoch timestamps (default False)

    :raises HTTPException: with status code 400 on invalid range or interval, 404 if the symbol cannot be found, or 500 for any other error
    """
    quotes = await get_historical(symbol, time_range=time_range, interval=interval, epoch=epoch)
    dates, closes, highs, lows, _ = prepare_price_data(quotes)
    tenkan_sen, kijun_sen, senkou_span_a, senkou_span_b, chikou_span = calculate_ichimoku(
        highs, lows, closes,
        tenkan_period=tenkan_period,
        kijun_period=kijun_period,
        senkou_period=senkou_period
    )
    # Create dictionaries for each Ichimoku component
    tenkan_dict = create_indicator_dict(dates, tenkan_sen)
    kijun_dict = create_indicator_dict(dates, kijun_sen)
    senkou_a_dict = create_indicator_dict(dates, senkou_span_a)
    senkou_b_dict = create_indicator_dict(dates, senkou_span_b)
    chikou_dict = create_indicator_dict(dates, chikou_span)

    # Create indicator data, using the closing price for the Chikou Span if the value is NaN
    indicator_data = {
        date: IchimokuData(
            tenkan_sen=round(tenkan_dict.get(date, np.nan), 2) if not np.isnan(tenkan_dict.get(date, np.nan)) else None,
            kijun_sen=round(kijun_dict.get(date, np.nan), 2) if not np.isnan(kijun_dict.get(date, np.nan)) else None,
            senkou_span_a=round(senkou_a_dict.get(date, np.nan), 2) if not np.isnan(
                senkou_a_dict.get(date, np.nan)) else None,
            senkou_span_b=round(senkou_b_dict.get(date, np.nan), 2) if not np.isnan(
                senkou_b_dict.get(date, np.nan)) else None,
            chikou_span=round(chikou_dict.get(date, np.nan), 2) if not np.isnan(chikou_dict.get(date, np.nan)) else
            closes[-1]
        )
        for date in tenkan_dict.keys()
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return TechnicalIndicator(
        type=Indicator.ICHIMOKU,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)
