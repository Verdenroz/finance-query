from stock_indicators import indicators
from typing_extensions import OrderedDict

from src.cache import cache
from src.schemas.analysis import (MACDData, Analysis, ADXData, AROONData, BBANDSData, OBVData, SuperTrendData,
                                  IchimokuData, Indicator)
from src.schemas.historical_data import TimePeriod, Interval
from src.services.historical.get_historical import get_historical_quotes


@cache(expire=60, market_closed_expire=600)
async def get_macd(symbol: str, interval: Interval, fast_period: int = 12, slow_period: int = 26,
                   signal_period: int = 9):
    """
    Get the Moving Average Convergence Divergence (MACD) for a symbol. MACD is a trend-following momentum
    indicator that shows the relationship between two moving averages of a security's price, revealing changes
    in the strength, direction, momentum, and duration of a trend.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param fast_period: The number of periods for the faster EMA calculation (default 12). A shorter period
                       makes this EMA more responsive to recent price changes
    :param slow_period: The number of periods for the slower EMA calculation (default 26). A longer period
                       provides a more smoothed view of the longer-term trend
    :param signal_period: The number of periods used to calculate the signal line (default 9). This EMA of
                         the MACD line helps identify potential trade signals when the MACD crosses above
                         or below it

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical_quotes(symbol, period=TimePeriod.MAX, interval=interval)
    results = indicators.get_macd(quotes, fast_periods=round(fast_period, 2), slow_periods=round(slow_period, 2),
                                  signal_periods=signal_period)

    indicator_data = {result.date.date(): MACDData(value=result.macd, signal=result.signal) for
                      result in results if result.macd is not None and result.signal is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.MACD,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


@cache(expire=60, market_closed_expire=600)
async def get_adx(symbol: str, interval: Interval, period: int = 14):
    """
    Get the Average Directional Index (ADX) for a symbol. ADX identifies the strength of a trend (regardless
    of direction) by measuring the expanding price range in the direction of the trend. It combines the
    Positive Directional Indicator (+DI) and Negative Directional Indicator (-DI).

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of periods used to calculate the DMI lines and ADX (default 14). Lower values
                  create a more responsive indicator but may generate more false signals. Values above 25
                  typically indicate a strong trend, while values below 20 suggest a weak or non-trending
                  market

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical_quotes(symbol, period=TimePeriod.MAX, interval=interval)
    results = indicators.get_adx(quotes, lookback_periods=period)
    indicator_data = {result.date.date(): ADXData(value=round(result.adx, 2)) for result in results if
                      result.adx is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.ADX,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


@cache(expire=60, market_closed_expire=600)
async def get_aroon(symbol: str, interval: Interval, period: int = 25):
    """
    Get the Aroon indicator for a symbol. The Aroon indicator consists of two lines (Aroon Up and Aroon Down)
    that measure the time since the last highest high and lowest low within a specified period, helping to
    identify the start of new trends and determine their strength.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The lookback period for finding the highest high and lowest low (default 25). A longer
                  period helps identify more significant trends but may be less responsive to recent price
                  changes. Values range from 0 to 100, with readings above 70 indicating a strong trend

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical_quotes(symbol, period=TimePeriod.MAX, interval=interval)
    results = indicators.get_aroon(quotes, lookback_periods=period)
    indicator_data = {
        result.date.date(): AROONData(aroon_up=round(result.aroon_up, 2), aroon_down=round(result.aroon_down, 2)) for
        result in results if result.aroon_up is not None and result.aroon_down is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.AROON,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


@cache(expire=60, market_closed_expire=600)
async def get_bbands(symbol: str, interval: Interval, period: int = 20, std_dev: int = 2):
    """
    Get the Bollinger Bands (BBands) for a symbol. Bollinger Bands consist of three lines: a middle band
    (simple moving average) and an upper and lower band that are standard deviations away from the middle
    band. They help identify volatility and potential overbought/oversold conditions.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of periods used to calculate the simple moving average and standard deviation
                  (default 20). A longer period creates wider, less reactive bands
    :param std_dev: The number of standard deviations for the upper and lower bands (default 2). Higher
                   values create wider bands that identify more extreme price movements. About 95% of price
                   action occurs within 2 standard deviations

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical_quotes(symbol, period=TimePeriod.MAX, interval=interval)
    results = indicators.get_bollinger_bands(quotes, lookback_periods=period, standard_deviations=std_dev)
    indicator_data = {
        result.date.date(): BBANDSData(upper_band=round(result.upper_band, 2), lower_band=round(result.lower_band, 2))
        for result in results if result.upper_band is not None and result.lower_band is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.BBANDS,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


@cache(expire=60, market_closed_expire=600)
async def get_obv(symbol: str, interval: Interval, sma_periods: int = None):
    """
    Get the On-Balance Volume (OBV) for a symbol. OBV is a cumulative indicator that combines volume and
    price movement to show how volume flows into and out of a security. It adds volume on up days and
    subtracts volume on down days.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param sma_periods: Optional number of periods for calculating a simple moving average of the OBV line.
                       When provided, returns both raw OBV and its SMA for trend confirmation. The SMA helps
                       smooth out the OBV line and can help identify significant trend changes

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical_quotes(symbol, period=TimePeriod.MAX, interval=interval)
    results = indicators.get_obv(quotes, sma_periods=sma_periods)
    indicator_data = {result.date.date(): OBVData(value=round(result.obv, 2)) for result in results if
                      result.obv is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.OBV,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


@cache(expire=60, market_closed_expire=600)
async def get_super_trend(symbol: str, interval: Interval, period: int = 10, multiplier: int = 3):
    """
    Get the Super Trend indicator for a symbol. Super Trend is a trend-following indicator that combines
    Average True Range (ATR) with basic support/resistance levels to identify the current trend direction
    and potential reversal points.

    :param symbol: the stock symbol
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param period: The number of periods used to calculate the Average True Range (default 10). A longer
                  period creates a smoother, less reactive indicator
    :param multiplier: The factor applied to the ATR to calculate the bands (default 3). A higher multiplier
                      creates wider bands that generate fewer signals but may reduce false breakouts

    :raises HTTPException: with status code 404 if the symbol cannot be found or code 500 for any other error
    """
    quotes = await get_historical_quotes(symbol, period=TimePeriod.MAX, interval=interval)
    results = indicators.get_super_trend(quotes, lookback_periods=period, multiplier=multiplier)
    indicator_data = {
        result.date.date(): SuperTrendData(value=round(result.super_trend, 2),
                                           trend="DOWN" if result.upper_band else "UP"
                                           )
        for result in results if result.super_trend is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.SUPER_TREND,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)


@cache(expire=60, market_closed_expire=600)
async def get_ichimoku(symbol: str, interval: Interval, tenkan_period: int = 9, kijun_period: int = 26,
                       senkou_period: int = 52):
    """
    Get the Ichimoku Cloud (Ichimoku Kinko Hyo) for a symbol. This comprehensive indicator combines multiple 
    technical strategies to generate signals through the interaction of five lines, with the space between 
    Senkou spans forming a "cloud" that can indicate trend direction and strength.

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
    quotes = await get_historical_quotes(symbol, period=TimePeriod.MAX, interval=interval)
    results = indicators.get_ichimoku(
        quotes,
        tenkan_periods=tenkan_period,
        kijun_periods=kijun_period,
        senkou_b_periods=senkou_period,
    )
    # Shift senkou spans and chikou span by 1 to match TradingView's Ichimoku Cloud
    senkou_span_a_shifted = [result.senkou_span_a for result in results[1:]] + [None]
    senkou_span_b_shifted = [result.senkou_span_b for result in results[1:]] + [None]
    chikou_span_shifted = [None] + [result.chikou_span for result in results[:-1]]

    # Update the results with the shifted values
    for i, result in enumerate(results):
        if i == len(results) - 1:  # If this is the last result
            # Calculate senkou_span_a and senkou_span_b manually
            recent_quotes = quotes[:52]
            highest_high = max(quote.high for quote in recent_quotes)
            lowest_low = min(quote.low for quote in recent_quotes)
            result.senkou_span_a = (result.tenkan_sen + result.kijun_sen) / 2
            result.senkou_span_b = (highest_high + lowest_low) / 2
        else:
            if senkou_span_a_shifted[i] is not None:
                result.senkou_span_a = float(senkou_span_a_shifted[i])
            if senkou_span_b_shifted[i] is not None:
                result.senkou_span_b = float(senkou_span_b_shifted[i])
        if chikou_span_shifted[i] is not None:
            result.chikou_span = float(chikou_span_shifted[i])

    indicator_data = {
        result.date.date(): IchimokuData(
            tenkan_sen=round(result.tenkan_sen, 2) if result.tenkan_sen is not None else None,
            kijun_sen=round(result.kijun_sen, 2) if result.kijun_sen is not None else None,
            chikou_span=round(result.chikou_span, 2) if result.chikou_span is not None else None,
            senkou_span_a=round(result.senkou_span_a, 2) if result.senkou_span_a is not None else None,
            senkou_span_b=round(result.senkou_span_b, 2) if result.senkou_span_b is not None else None,
        )
        for result in results
    }
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(
        type=Indicator.ICHIMOKU,
        indicators=indicator_data
    ).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)
