import numpy as np
from typing_extensions import OrderedDict

from src.models.analysis import (MACDData, Analysis, ADXData, AROONData, BBANDSData, OBVData, SuperTrendData,
                                 IchimokuData, Indicator)
from src.models.historical_data import TimePeriod, Interval
from src.services.historical.get_historical import get_historical
from src.services.indicators.core import (
    prepare_price_data, create_indicator_dict, calculate_macd, calculate_adx,
    calculate_aroon, calculate_bbands,
    calculate_obv, calculate_ichimoku, calculate_supertrend
)


async def get_macd(symbol: str, interval: Interval, fast_period: int = 12, slow_period: int = 26,
                   signal_period: int = 9) -> dict:
    quotes = await get_historical(symbol, period=TimePeriod.YEAR, interval=interval)
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
    return Analysis(
        type=Indicator.MACD,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_adx(symbol: str, interval: Interval, period: int = 14) -> dict:
    """
    Get the Average Directional Index (ADX) for a symbol.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of periods used to calculate the DMI lines and ADX (default 14). Lower values
                  create a more responsive indicator but may generate more false signals. Values above 25
                  typically indicate a strong trend

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical(symbol, period=TimePeriod.YEAR, interval=interval)

    dates, closes, highs, lows, _ = prepare_price_data(quotes)
    adx_values = calculate_adx(highs, lows, closes, period=period)

    indicator_data = {
        date: ADXData(value=round(value, 2))
        for date, value in create_indicator_dict(dates, adx_values).items()
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.ADX,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_aroon(symbol: str, interval: Interval, period: int = 25) -> dict:
    """
    Get the Aroon indicator for a symbol.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The lookback period for finding the highest high and lowest low (default 25). A longer
                  period helps identify more significant trends but may be less responsive to recent price
                  changes. Values range from 0 to 100, with readings above 70 indicating a strong trend

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical(symbol, period=TimePeriod.YEAR, interval=interval)

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
    return Analysis(
        type=Indicator.AROON,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_bbands(symbol: str, interval: Interval, period: int = 20, std_dev: int = 2) -> dict:
    """
    Get the Bollinger Bands (BBands) for a symbol.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of periods used to calculate the simple moving average and standard deviation
                  (default 20). A longer period creates wider, less reactive bands
    :param std_dev: The number of standard deviations for the upper and lower bands (default 2). Higher
                   values create wider bands that identify more extreme price movements. About 95% of price
                   action occurs within 2 standard deviations

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical(symbol, period=TimePeriod.YEAR, interval=interval)

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
    return Analysis(
        type=Indicator.BBANDS,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_obv(symbol: str, interval: Interval) -> dict:
    """
    Get the On-Balance Volume (OBV) for a symbol.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical(symbol, period=TimePeriod.YEAR, interval=interval)

    dates, closes, _, _, volumes = prepare_price_data(quotes)
    obv_values = calculate_obv(closes, volumes)

    indicator_data = {
        date: OBVData(value=round(value, 2))
        for date, value in create_indicator_dict(dates, obv_values).items()
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.OBV,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_super_trend(symbol: str, interval: Interval, period: int = 10, multiplier: int = 3) -> dict:
    """
    Get the Super Trend indicator for a symbol.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of periods used to calculate the Average True Range (default 10). A longer
                  period creates a smoother, less reactive indicator
    :param multiplier: The factor applied to the ATR to calculate the bands (default 3). A higher multiplier
                      creates wider bands that generate fewer signals but may reduce false breakouts

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical(symbol, period=TimePeriod.YEAR, interval=interval)

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
    return Analysis(
        type=Indicator.SUPER_TREND,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


async def get_ichimoku(
        symbol: str,
        interval: Interval,
        tenkan_period: int = 9,
        kijun_period: int = 26,
        senkou_period: int = 52
) -> dict:
    """
    Get the Ichimoku Cloud (Ichimoku Kinko Hyo) for a symbol.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param tenkan_period: The period for calculating the Tenkan-sen (default 9). This shorter-term line is
                         more sensitive to price changes
    :param kijun_period: The period for calculating the Kijun-sen (default 26). This longer-term line acts
                        as a medium-term trend indicator
    :param senkou_period: The period for calculating Senkou Span B (default 52). This long-term line helps
                         form the cloud and indicates long-term trend direction

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical(symbol, period=TimePeriod.YEAR, interval=interval)
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
            chikou_span=round(chikou_dict.get(date, np.nan), 2) if not np.isnan(chikou_dict.get(date, np.nan)) else closes[-1]
        )
        for date in tenkan_dict.keys()
    }

    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.ICHIMOKU,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)
