from stock_indicators import indicators
from typing_extensions import OrderedDict

from src.redis import cache
from src.schemas.analysis import SMAData, Analysis, EMAData, WMAData, VWMAData, Indicator
from src.schemas.time_series import TimePeriod, Interval
from src.services.get_historical import get_historical_quotes


@cache(expire=60, market_closed_expire=600)
async def get_sma(symbol: str, interval: Interval, period: int = 10):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    results = indicators.get_sma(quotes, period)
    indicator_data = {result.date.date(): SMAData(value=round(result.sma, 2)) for result in results if
                      result.sma is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(type=Indicator.SMA, indicators=indicator_data).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)

@cache(expire=60, market_closed_expire=600)
async def get_ema(symbol: str, interval: Interval, period: int = 10):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    results = indicators.get_ema(quotes, period).remove_warmup_periods()
    indicator_data = {result.date.date(): EMAData(value=round(result.ema, 2)) for result in results if
                      result.ema is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(type=Indicator.EMA, indicators=indicator_data).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)

@cache(expire=60, market_closed_expire=600)
async def get_wma(symbol: str, interval: Interval, period: int = 10):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    results = indicators.get_wma(quotes, period).remove_warmup_periods()
    indicator_data = {result.date.date(): WMAData(value=round(result.wma, 2)) for result in results if
                      result.wma is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(type=Indicator.WMA, indicators=indicator_data).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)

@cache(expire=60, market_closed_expire=600)
async def get_vwma(symbol: str, interval: Interval, period: int = 20):
    quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    results = indicators.get_vwma(quotes, period).remove_warmup_periods()
    indicator_data = {result.date.date(): VWMAData(value=round(result.vwma, 2)) for result in results
                      if result.vwma is not None}
    indicator_data = OrderedDict(sorted(indicator_data.items(), reverse=True))
    return Analysis(type=Indicator.VWMA, indicators=indicator_data).model_dump(exclude_none=True, by_alias=True, serialize_as_any=True)
