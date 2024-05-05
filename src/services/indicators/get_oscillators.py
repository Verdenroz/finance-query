from stock_indicators import indicators
from typing_extensions import OrderedDict

from src.schemas.analysis import RSIData, Analysis, SRSIData, STOCHData, CCIData
from src.schemas.time_series import TimePeriod, Interval
from src.services.get_historical import get_historical_quotes


async def get_rsi(symbol: str, period: int = 14):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.SIX_MONTHS, interval=Interval.DAILY)
    results = indicators.get_rsi(quotes, lookback_periods=period).remove_warmup_periods()
    indicator_data = {result.date.date(): RSIData(value=result.rsi) for result in results if
                      result.rsi is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(indicators=indicator_data)


async def get_srsi(symbol: str, period: int = 14, stoch_period: int = 14, signal_period: int = 3, smooth: int = 3):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.SIX_MONTHS, interval=Interval.DAILY)
    results = indicators.get_stoch_rsi(
        quotes,
        rsi_periods=period,
        stoch_periods=stoch_period,
        signal_periods=signal_period,
        smooth_periods=smooth
    ).remove_warmup_periods()
    indicator_data = {result.date.date(): SRSIData(k=result.stoch_rsi, d=result.signal) for
                      result in results if result.stoch_rsi is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(indicators=indicator_data)


async def get_stoch(symbol: str, period: int = 14, signal: int = 3, smooth: int = 3):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.SIX_MONTHS, interval=Interval.DAILY)
    results = indicators.get_stoch(
        quotes,
        lookback_periods=period,
        signal_periods=signal,
        smooth_periods=smooth
    ).remove_warmup_periods()
    indicator_data = [STOCHData(k=result.k, d=result.d) for result in results if result.k is not None]
    return indicator_data


async def get_cci(symbol: str, period: int = 20):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.SIX_MONTHS, interval=Interval.DAILY)
    results = indicators.get_cci(quotes, lookback_periods=period).remove_warmup_periods()
    indicator_data = [CCIData(value=result.cci) for result in results if result.cci is not None]
    return indicator_data
