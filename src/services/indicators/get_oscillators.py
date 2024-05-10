from stock_indicators import indicators
from typing_extensions import OrderedDict

from src.schemas.analysis import RSIData, Analysis, SRSIData, STOCHData, CCIData, Indicator
from src.schemas.time_series import TimePeriod, Interval
from src.services.get_historical import get_historical_quotes


async def get_rsi(symbol: str, interval: Interval, period: int = 14):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    results = indicators.get_rsi(quotes, lookback_periods=period)
    indicator_data = {result.date.date(): RSIData(value=round(result.rsi, 2)) for result in results if
                      result.rsi is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(type=Indicator.RSI, indicators=indicator_data)


async def get_srsi(symbol: str, interval: Interval, period: int = 14, stoch_period: int = 14, signal_period: int = 3,
                   smooth: int = 3):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    results = indicators.get_stoch_rsi(
        quotes,
        rsi_periods=period,
        stoch_periods=stoch_period,
        signal_periods=signal_period,
        smooth_periods=smooth
    )
    indicator_data = {result.date.date(): SRSIData(k=round(result.stoch_rsi, 2), d=round(result.signal, 2)) for
                      result in results if result.stoch_rsi is not None and result.signal is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(type=Indicator.SRSI, indicators=indicator_data)


async def get_stoch(symbol: str, interval: Interval, period: int = 14, signal_period: int = 3, smooth: int = 3):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    results = indicators.get_stoch(
        quotes,
        lookback_periods=period,
        signal_periods=signal_period,
        smooth_periods=smooth
    )
    indicator_data = {result.date.date(): STOCHData(k=round(result.k, 2), d=round(result.d, 2)) for
                      result in results if result.k is not None and result.d is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(type=Indicator.STOCH, indicators=indicator_data)


async def get_cci(symbol: str, interval: Interval, period: int = 20):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    results = indicators.get_cci(quotes, lookback_periods=period).remove_warmup_periods()
    indicator_data = {result.date.date(): CCIData(value=round(result.cci, 2)) for result in results if
                      result.cci is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(type=Indicator.CCI, indicators=indicator_data)
