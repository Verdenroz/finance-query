from stock_indicators import indicators
from typing_extensions import OrderedDict

from src.schemas.analysis import (MACDData, Analysis, ADXData, AROONData, BBANDSData, OBVData, SuperTrendData,
                                  IchimokuData, Indicator)
from src.schemas.time_series import TimePeriod, Interval
from src.services.get_historical import get_historical_quotes
from src.utils import cache


@cache(expire=60, after_market_expire=600)
async def get_macd(symbol: str, interval: Interval, fast_period: int = 12, slow_period: int = 26,
                   signal_period: int = 9):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    results = indicators.get_macd(quotes, fast_periods=round(fast_period, 2), slow_periods=round(slow_period, 2),
                                  signal_periods=signal_period)

    indicator_data = {result.date.date(): MACDData(value=result.macd, signal=result.signal) for
                      result in results if result.macd is not None and result.signal is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(type=Indicator.MACD, indicators=indicator_data).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)

@cache(expire=60, after_market_expire=600)
async def get_adx(symbol: str, interval: Interval, period: int = 14):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    results = indicators.get_adx(quotes, lookback_periods=period)
    indicator_data = {result.date.date(): ADXData(value=round(result.adx, 2)) for result in results if
                      result.adx is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(type=Indicator.ADX, indicators=indicator_data).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)

@cache(expire=60, after_market_expire=600)
async def get_aroon(symbol: str, interval: Interval, period: int = 25):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    results = indicators.get_aroon(quotes, lookback_periods=period)
    indicator_data = {
        result.date.date(): AROONData(aroon_up=round(result.aroon_up, 2), aroon_down=round(result.aroon_down, 2)) for
        result in results if result.aroon_up is not None and result.aroon_down is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(type=Indicator.AROON, indicators=indicator_data).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)

@cache(expire=60, after_market_expire=600)
async def get_bbands(symbol: str, interval: Interval, period: int = 20, std_dev: int = 2):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    results = indicators.get_bollinger_bands(quotes, lookback_periods=period, standard_deviations=std_dev)
    indicator_data = {
        result.date.date(): BBANDSData(upper_band=round(result.upper_band, 2), lower_band=round(result.lower_band, 2))
        for result in results if result.upper_band is not None and result.lower_band is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(type=Indicator.BBANDS, indicators=indicator_data).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)

@cache(expire=60, after_market_expire=600)
async def get_obv(symbol: str, interval: Interval, sma_periods: int = None):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    results = indicators.get_obv(quotes, sma_periods=sma_periods)
    indicator_data = {result.date.date(): OBVData(value=round(result.obv, 2)) for result in results if
                      result.obv is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(type=Indicator.OBV, indicators=indicator_data).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)

@cache(expire=60, after_market_expire=600)
async def get_super_trend(symbol: str, interval: Interval, period: int = 14, multiplier: int = 3):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    results = indicators.get_super_trend(quotes, lookback_periods=period, multiplier=multiplier)
    indicator_data = {
        result.date.date(): SuperTrendData(value=round(result.super_trend, 2),
                                           trend="DOWN" if result.upper_band else "UP"
                                           )
        for result in results if result.super_trend is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(type=Indicator.SUPER_TREND, indicators=indicator_data).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)

@cache(expire=60, after_market_expire=600)
async def get_ichimoku(symbol: str, interval: Interval, tenkan_period: int = 9, kijun_period: int = 26,
                       senkou_period: int = 52):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
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
    return Analysis(type=Indicator.ICHIMOKU, indicators=indicator_data).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)
