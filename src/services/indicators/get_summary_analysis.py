import asyncio

from stock_indicators.indicators import get_ema, get_wma, get_vwma, get_rsi, get_stoch_rsi, get_stoch, \
    get_cci, get_macd, get_adx, get_aroon, get_bollinger_bands, get_obv, get_super_trend, get_ichimoku, get_sma

from src.schemas.analysis import SummaryAnalysis, AROONData, BBANDSData, SuperTrendData, IchimokuData, MACDData
from src.schemas.time_series import Interval, TimePeriod
from src.services.get_historical import get_historical_quotes
from src.utils import cache


async def get_summary_sma(quotes, periods, sma=None):
    if sma is None:
        sma = []

    if not periods:
        return sma

    period = periods[0]
    remaining_periods = periods[1:]
    remaining_quotes = quotes[:int(len(quotes) / 2)]
    sma_value = get_sma(quotes, period)[-1].sma
    sma.append(round(sma_value, 2)) if sma_value else sma.append(None)

    return await get_summary_sma(remaining_quotes, remaining_periods, sma)


async def get_summary_ema(quotes, periods, ema=None):
    if ema is None:
        ema = []

    if not periods:
        return ema

    period = periods[0]
    remaining_periods = periods[1:]
    remaining_quotes = quotes[:int(len(quotes) / 2)]
    ema_value = get_ema(quotes, period)[-1].ema
    ema.append(round(ema_value, 2)) if ema_value else ema.append(None)
    return await get_summary_ema(remaining_quotes, remaining_periods, ema)


async def get_summary_wma(quotes, periods, wma=None):
    if wma is None:
        wma = []

    if not periods:
        return wma

    period = periods[0]
    remaining_periods = periods[1:]
    remaining_quotes = quotes[:int(len(quotes) / 2)]
    wma_value = get_wma(quotes, period)[-1].wma
    wma.append(round(wma_value, 2)) if wma_value else wma.append(None)
    return await get_summary_wma(remaining_quotes, remaining_periods, wma)


async def get_summary_vwma(quotes, period=20):
    vma = get_vwma(quotes, period)[-1].vwma
    if vma:
        return round(vma, 2)
    return None


async def get_summary_rsi(quotes, period=14):
    rsi = get_rsi(quotes, period)[-1].rsi
    if rsi:
        return round(rsi, 2)
    return None


async def get_summary_srsi(quotes, period=14, stoch_period=14, signal_period=3, smooth=3):
    srsi = get_stoch_rsi(quotes, rsi_periods=period, stoch_periods=stoch_period,
                         signal_periods=signal_period, smooth_periods=smooth)[-1].stoch_rsi
    if srsi:
        return round(srsi, 2)
    return None


async def get_summary_stoch(quotes, period=14, signal_period=3, smooth=3):
    stoch = get_stoch(quotes, lookback_periods=period, signal_periods=signal_period, smooth_periods=smooth)[-1].k
    if stoch:
        return round(stoch, 2)
    return None


async def get_summary_cci(quotes, period=20):
    cci = get_cci(quotes, period)[-1].cci
    if cci:
        return round(cci, 2)
    return None


async def get_summary_macd(quotes, fast_period=12, slow_period=26, signal_period=9):
    macd = get_macd(quotes, fast_periods=fast_period, slow_periods=slow_period, signal_periods=signal_period)[-1].macd
    signal = get_macd(quotes, fast_periods=fast_period, slow_periods=slow_period, signal_periods=signal_period)[-1].signal
    if macd:
        return MACDData(value=round(macd, 2), signal=round(signal, 2))
    return None


async def get_summary_adx(quotes, period=14):
    adx = get_adx(quotes, period)[-1].adx
    if adx:
        return round(adx, 2)
    return None


async def get_summary_obv(quotes, period=20):
    obv = get_obv(quotes, sma_periods=period)[-1].obv
    if obv:
        return round(obv, 2)
    return None


async def get_summary_aroon(quotes, period=25):
    aroon = get_aroon(quotes, lookback_periods=period)[-1]
    upper_band = aroon.aroon_up
    lower_band = aroon.aroon_down

    if upper_band or lower_band:
        return AROONData(aroon_up=round(upper_band, 2), aroon_down=round(lower_band, 2))
    return None


async def get_summary_bbands(quotes, period=20, std_dev=2):
    bbands = get_bollinger_bands(quotes, lookback_periods=period, standard_deviations=std_dev).remove_warmup_periods()[
        -1]
    upper_band = bbands.upper_band
    lower_band = bbands.lower_band
    if upper_band or lower_band:
        return BBANDSData(upper_band=round(upper_band, 2), lower_band=round(lower_band, 2))
    return None


async def get_summary_super_trend(quotes, period=14, multiplier=3):
    super_trend = get_super_trend(quotes, lookback_periods=period, multiplier=multiplier)[-1]
    if not super_trend:
        return None

    trend = "DOWN" if super_trend.upper_band else "UP"
    return SuperTrendData(value=round(super_trend.super_trend, 2), trend=trend)


async def get_summary_ichimoku(quotes):
    ichimoku = get_ichimoku(quotes)[-1]
    tenkan_sen = ichimoku.tenkan_sen
    kijun_sen = ichimoku.kijun_sen
    senkou_span_a = ichimoku.senkou_span_a
    senkou_span_b = ichimoku.senkou_span_b
    if tenkan_sen or kijun_sen or senkou_span_a or senkou_span_b:
        return IchimokuData(tenkan_sen=round(tenkan_sen, 2), kijun_sen=round(kijun_sen, 2),
                        senkou_span_a=round(senkou_span_a, 2), senkou_span_b=round(senkou_span_b, 2))

@cache(expire=60, after_market_expire=600)
async def get_summary_analysis(symbol: str, interval: Interval):
    if interval == Interval.FIFTEEN_MINUTES or interval == Interval.THIRTY_MINUTES:
        quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.ONE_MONTH, interval=interval)
    elif interval == Interval.ONE_HOUR:
        quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.YEAR, interval=interval)
    else:
        quotes = await get_historical_quotes(symbol, timePeriod=TimePeriod.MAX, interval=interval)
    summary = SummaryAnalysis(symbol=symbol.upper())
    tasks = [
        get_summary_sma(quotes[:200], [200, 100, 50, 20, 10]),
        get_summary_ema(quotes[:750], [200, 100, 50, 20, 10]),
        get_summary_wma(quotes[:200], [200, 100, 50, 20, 10]),
        get_summary_vwma(quotes[:25], 20),
        get_summary_rsi(quotes[:100], 14),
        get_summary_srsi(quotes[:100], 14),
        get_summary_stoch(quotes[:16], 14),
        get_summary_cci(quotes[:20], 20),
        get_summary_macd(quotes[:75], 12, 26, 9),
        get_summary_adx(quotes[:100], 14),
        get_summary_obv(quotes, 20),
        get_summary_aroon(quotes[:30], 25),
        get_summary_bbands(quotes[:20], 20, 2),
        get_summary_super_trend(quotes[:92], 14, 3),
        get_summary_ichimoku(quotes[:78]),
    ]
    # Run the tasks concurrently and unpack the results
    sma, ema, wma, vwma, rsi, srsi, stoch, cci, macd, adx, obv, aroon, bbands, super_trend, ichimoku = await (
        asyncio.gather(*tasks))

    summary.sma_10 = sma[4]
    summary.sma_20 = sma[3]
    summary.sma_50 = sma[2]
    summary.sma_100 = sma[1]
    summary.sma_200 = sma[0]
    summary.ema_10 = ema[4]
    summary.ema_20 = ema[3]
    summary.ema_50 = ema[2]
    summary.ema_100 = ema[1]
    summary.ema_200 = ema[0]
    summary.wma_10 = wma[4]
    summary.wma_20 = wma[3]
    summary.wma_50 = wma[2]
    summary.wma_100 = wma[1]
    summary.wma_200 = wma[0]
    summary.vwma = vwma
    summary.rsi = rsi
    summary.srsi = srsi
    summary.stoch = stoch
    summary.cci = cci
    summary.macd = macd
    summary.adx = adx
    summary.obv = obv
    summary.aroon = aroon
    summary.bbands = bbands
    summary.supertrend = super_trend
    summary.ichimoku = ichimoku

    return summary.dict(by_alias=True)
