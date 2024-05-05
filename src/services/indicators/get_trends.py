from stock_indicators import indicators
from typing_extensions import OrderedDict

from src.schemas.analysis import MACDData, Analysis, ADXData, AROONData, BBANDSData, OBVData, SuperTrendData, \
    IchimokuData
from src.schemas.time_series import TimePeriod, Interval
from src.services.get_historical import get_historical_quotes


async def get_macd(symbol: str, fast_period: int = 12, slow_period: int = 26, signal_period: int = 9):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.SIX_MONTHS, interval=Interval.DAILY)
    results = indicators.get_macd(quotes, fast_periods=fast_period, slow_periods=slow_period,
                                  signal_periods=signal_period).remove_warmup_periods()

    indicator_data = {result.date.date(): MACDData(value=result.macd, signal=result.signal) for
                      result in results if result.macd is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(indicators=indicator_data)


async def get_adx(symbol: str, period: int = 14):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.SIX_MONTHS, interval=Interval.DAILY)
    results = indicators.get_adx(quotes, lookback_periods=period).remove_warmup_periods()

    indicator_data = [ADXData(value=result.adx) for result in results if result.adx is not None]
    return indicator_data


async def get_aroon(symbol: str, period: int = 25):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.SIX_MONTHS, interval=Interval.DAILY)
    results = indicators.get_aroon(quotes, lookback_periods=period).remove_warmup_periods()

    indicator_data = [AROONData(aroon_up=result.aroon_up, aroon_down=result.aroon_down) for
                      result in results if result.aroon_up is not None]
    return indicator_data


async def get_bbands(symbol: str, period: int = 14, std_dev: int = 2):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.SIX_MONTHS, interval=Interval.DAILY)
    results = indicators.get_bollinger_bands(quotes, lookback_periods=period,
                                             standard_deviations=std_dev).remove_warmup_periods()

    indicator_data = [
        BBANDSData(upper_band=result.upper_band, lower_band=result.lower_band) for result
        in results if result.upper_band is not None and result.lower_band is not None]
    return indicator_data


async def get_obv(symbol: str, sma_periods: int = None):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.SIX_MONTHS, interval=Interval.DAILY)
    results = indicators.get_obv(quotes, sma_periods=sma_periods).remove_warmup_periods()
    indicator_data = [OBVData(value=result.obv) for result in results if result.obv is not None]
    return indicator_data


async def get_super_trend(symbol: str, period: int = 14, multiplier: int = 3):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.SIX_MONTHS, interval=Interval.DAILY)
    results = indicators.get_super_trend(quotes, lookback_periods=period, multiplier=multiplier).remove_warmup_periods()
    indicator_data = [
        SuperTrendData(value=result.super_trend, upper_band=result.upper_band, lower_band=result.lower_band) for result
        in results if
        result.super_trend is not None and result.upper_band is not None and result.lower_band is not None]
    return indicator_data


async def get_ichimoku(symbol: str, tenkan_period: int = 9, kijun_period: int = 26, senkou_period: int = 52,
                       senkou_offset: int = 26, chikou_offset: int = 26):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.SIX_MONTHS, interval=Interval.DAILY)
    results = indicators.get_ichimoku(
        quotes,
        tenkan_periods=tenkan_period,
        kijun_periods=kijun_period,
        senkou_b_periods=senkou_period,
        senkou_offset=senkou_offset, chikou_offset=chikou_offset
    ).remove_warmup_periods()

    indicator_data = [
        IchimokuData(tenkan_sen=result.tenkan_sen, kijun_sen=result.kijun_sen, senkou_span_a=result.senkou_span_a,
                     senkou_span_b=result.senkou_span_b, chikou_span=result.chikou_span) for result in results if
        result.tenkan_sen is not None]
    return indicator_data
