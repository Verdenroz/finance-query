import asyncio

from stock_indicators.indicators import get_ema, get_wma, get_vwma, get_rsi, get_stoch_rsi, get_stoch, \
    get_cci, get_macd, get_adx, get_aroon, get_bollinger_bands, get_super_trend, get_ichimoku, get_sma

from src.cache import cache
from src.schemas.analysis import SummaryAnalysis, AROONData, BBANDSData, SuperTrendData, IchimokuData, MACDData
from src.schemas.historical_data import Interval, TimePeriod
from src.services.historical.get_historical import get_historical_quotes


async def get_summary_sma(quotes, periods, sma=None) -> list:
    """
    Recursively calculate the Simple Moving Average (SMA) for a list of periods and return the results
    :param quotes: the historical quotes to calculate the SMA for
    :param periods: the list of periods to calculate the SMA for
    :param sma: the list of SMA values to append to, initially empty
    :return: a list of SMA values for the given periods
    """
    try:
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
    except SystemError:
        # Error within the stock-indicators library itself
        return []


async def get_summary_ema(quotes, periods, ema=None) -> list:
    """
    Recursively calculate the Exponential Moving Average (EMA) for a list of periods and return the results
    :param quotes: the historical quotes to calculate the EMA for
    :param periods: the list of periods to calculate the EMA for
    :param ema: the list of EMA values to append to, initially empty
    :return: the list of EMA values for the given periods
    """
    try:
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
    except SystemError:
        # Error within the stock-indicators library itself
        return []


async def get_summary_wma(quotes, periods, wma=None) -> list:
    """
    Recursively calculate the Weighted Moving Average (WMA) for a list of periods and return the results
    :param quotes: the historical quotes to calculate the WMA for
    :param periods: the list of periods to calculate the WMA for
    :param wma: the list of WMA values to append to, initially empty
    :return: the list of WMA values for the given periods
    """
    try:
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
    except SystemError:
        # Error within the stock-indicators library itself
        return []


async def get_summary_vwma(quotes, period=20) -> float | None:
    """
    Calculates the Volume Weighted Moving Average (VWMA) for a given period
    :param quotes: the historical quotes to calculate the VWMA for
    :param period: the default period to calculate the VWMA for (should not change)
    :return: the VWMA value for the given period or None if it cannot be calculated
    """
    try:
        vwma = get_vwma(quotes, period)[-1].vwma
        if vwma:
            return round(vwma, 2)
        return None
    except SystemError:
        # Error within the stock-indicators library itself
        return None


async def get_summary_rsi(quotes, period=14) -> float | None:
    """
    Calculates the Relative Strength Index (RSI) for a given period
    :param quotes: the historical quotes to calculate the RSI for
    :param period: the default period to calculate the RSI for (should not change)
    :return: the RSI value for the given period or None if it cannot be calculated
    """
    try:
        rsi = get_rsi(quotes, period)[-1].rsi
        if rsi:
            return round(rsi, 2)
        return None
    except SystemError:
        # Error within the stock-indicators library itself
        return None


async def get_summary_srsi(quotes, period=14, stoch_period=14, signal_period=3, smooth=3) -> float | None:
    """
    Calculates the Stochastic RSI (SRSI) for a given period
    :param quotes: the historical quotes to calculate the SRSI for
    :param period: the default period to calculate the RSI for (should not change)
    :param stoch_period: the default period to calculate the Stochastic Oscillator for (should not change)
    :param signal_period: the default period to calculate the signal line for (should not change)
    :param smooth: the default period to smooth the signal line for (should not change)
    :return: the SRSI value for the given period or None if it cannot be calculated
    """
    try:
        srsi = get_stoch_rsi(quotes, rsi_periods=period, stoch_periods=stoch_period,
                             signal_periods=signal_period, smooth_periods=smooth)[-1].stoch_rsi
        if srsi:
            return round(srsi, 2)
        return None
    except SystemError:
        # Error within the stock-indicators library itself
        return None


async def get_summary_stoch(quotes, period=14, signal_period=3, smooth=3) -> float | None:
    """
    Calculates the Stochastic Oscillator (STOCH) for a given period
    :param quotes: the historical quotes to calculate the STOCH for
    :param period: the default period to calculate the STOCH for (should not change)
    :param signal_period: the default period to calculate the signal line for (should not change)
    :param smooth: the default period to smooth the signal line for (should not change)
    :return: the STOCH value for the given period or None if it cannot be calculated
    """
    try:
        stoch = get_stoch(quotes, lookback_periods=period, signal_periods=signal_period, smooth_periods=smooth)[-1].k
        if stoch:
            return round(stoch, 2)
        return None
    except SystemError:
        # Error within the stock-indicators library itself
        return None


async def get_summary_cci(quotes, period=20) -> float | None:
    """
    Calculates the Commodity Channel Index (CCI) for a given period
    :param quotes: the historical quotes to calculate the CCI for
    :param period: the default period to calculate the CCI for (should not change)
    :return: the CCI value for the given period or None if it cannot be calculated
    """
    try:
        cci = get_cci(quotes, period)[-1].cci
        if cci:
            return round(cci, 2)
        return None
    except SystemError:
        # Error within the stock-indicators library itself
        return None


async def get_summary_macd(quotes, fast_period=12, slow_period=26, signal_period=9) -> MACDData | None:
    """
    Calculates the Moving Average Convergence Divergence (MACD) for a given period
    :param quotes: the historical quotes to calculate the MACD for
    :param fast_period: the default period to calculate the fast EMA for (should not change)
    :param slow_period: the default period to calculate the slow EMA for (should not change)
    :param signal_period: the default period to calculate the signal line for (should not change)
    :return: the MACD value for the given period or None if it cannot be calculated
    """
    try:
        macd = get_macd(quotes, fast_periods=fast_period, slow_periods=slow_period, signal_periods=signal_period)[
            -1].macd
        signal = get_macd(quotes, fast_periods=fast_period, slow_periods=slow_period, signal_periods=signal_period)[
            -1].signal
        if macd and signal:
            return MACDData(value=round(macd, 2), signal=round(signal, 2))
        return None
    except SystemError:
        # Error within the stock-indicators library itself
        return None


async def get_summary_adx(quotes, period=14) -> float | None:
    """
    Calculates the Average Directional Index (ADX) for a given period
    :param quotes: the historical quotes to calculate the ADX for
    :param period: the default period to calculate the ADX for (should not change)
    :return:
    """
    try:
        adx = get_adx(quotes, period)[-1].adx
        if adx:
            return round(adx, 2)
        return None
    except SystemError:
        # Error within the stock-indicators library itself
        return None


async def get_summary_aroon(quotes, period=25) -> AROONData | None:
    """
    Calculates the Aroon indicator for a given period
    :param quotes: the historical quotes to calculate the Aroon indicator for
    :param period: the default period to calculate the Aroon indicator for (should not change)
    :return: the Aroon indicator value for the given period or None if it cannot be calculated
    """
    try:
        aroon = get_aroon(quotes, lookback_periods=period)[-1]
        upper_band = aroon.aroon_up
        lower_band = aroon.aroon_down
        if upper_band or lower_band:
            return AROONData(aroon_up=round(upper_band, 2), aroon_down=round(lower_band, 2))
        return None
    except SystemError:
        # Error within the stock-indicators library itself
        return None


async def get_summary_bbands(quotes, period=20, std_dev=2) -> BBANDSData | None:
    """
    Calculates the Bollinger Bands for a given period
    :param quotes: the historical quotes to calculate the Bollinger Bands for
    :param period: the default period to calculate the Bollinger Bands for (should not change)
    :param std_dev: the default standard deviation to calculate the Bollinger Bands for (should not change)
    :return: the Bollinger Bands value for the given period or None if it cannot be calculated
    """
    try:
        bbands = \
        get_bollinger_bands(quotes, lookback_periods=period, standard_deviations=std_dev).remove_warmup_periods()[
            -1]
        upper_band = bbands.upper_band
        lower_band = bbands.lower_band
        if upper_band or lower_band:
            return BBANDSData(upper_band=round(upper_band, 2), lower_band=round(lower_band, 2))
        return None
    except SystemError:
        # Error within the stock-indicators library itself
        return None


async def get_summary_super_trend(quotes, period=10, multiplier=3) -> SuperTrendData | None:
    """
    Calculates the SuperTrend indicator for a given period
    :param quotes: the historical quotes to calculate the SuperTrend indicator for
    :param period: the default period to calculate the SuperTrend indicator for (should not change)
    :param multiplier: the default multiplier to calculate the SuperTrend indicator for (should not change)
    :return: the SuperTrend indicator value for the given period or None if it cannot be calculated
    """
    try:
        super_trend = get_super_trend(quotes, lookback_periods=period, multiplier=multiplier)[-1]
        if not super_trend.super_trend:
            return None

        trend = "DOWN" if super_trend.upper_band else "UP"
        return SuperTrendData(value=round(super_trend.super_trend, 2), trend=trend)
    except SystemError:
        # Error within the stock-indicators library itself
        return None


async def get_summary_ichimoku(quotes) -> IchimokuData | None:
    """
    Calculates the Ichimoku Cloud indicator
    :param quotes: the historical quotes to calculate the Ichimoku Cloud for
    :return: the Ichimoku Cloud indicator values or None if it cannot be calculated
    """
    try:
        ichimoku = get_ichimoku(quotes)[-1]
        tenkan_sen = ichimoku.tenkan_sen
        kijun_sen = ichimoku.kijun_sen
        senkou_span_a = ichimoku.senkou_span_a
        senkou_span_b = ichimoku.senkou_span_b
        if tenkan_sen and kijun_sen and senkou_span_a and senkou_span_b:
            return IchimokuData(tenkan_sen=round(tenkan_sen, 2), kijun_sen=round(kijun_sen, 2),
                                senkou_span_a=round(senkou_span_a, 2), senkou_span_b=round(senkou_span_b, 2))
    except SystemError:
        # Error within the stock-indicators library itself
        return None


@cache(expire=60, market_closed_expire=600)
async def get_summary_analysis(symbol: str, interval: Interval) -> dict:
    """
    Get a summary analysis of technical indicators for a stock symbol, based on the interval provided.
    Includes Simple Moving Average (SMA), Exponential Moving Average (EMA), Weighted Moving Average (WMA),
    Volume Weighted Moving Average (VWMA), Relative Strength Index (RSI), Stochastic RSI (SRSI),
    Stochastic Oscillator (STOCH), Commodity Channel Index (CCI), Moving Average Convergence Divergence (MACD),
    Average Directional Index (ADX), Aroon indicator, Bollinger Bands, SuperTrend, and Ichimoku Cloud.
    :param symbol: the symbol of the stock to get the summary analysis for
    :param interval: the timeframe between each data point (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :return: the serialized summary analysis as a dictionary
    """
    # Get the most historical data available given the interval
    # (1m, 5m, 15m, 30m -> One month)
    if interval == Interval.ONE_MINUTE or interval == Interval.FIVE_MINUTES or interval == Interval.FIFTEEN_MINUTES or interval == Interval.THIRTY_MINUTES:
        quotes = await get_historical_quotes(symbol, period=TimePeriod.ONE_MONTH, interval=interval)
    # (1h -> One year)
    elif interval == Interval.ONE_HOUR:
        quotes = await get_historical_quotes(symbol, period=TimePeriod.YEAR, interval=interval)
    # (1d, 1wk, 1mo, 3mo -> Max)
    else:
        quotes = await get_historical_quotes(symbol, period=TimePeriod.MAX, interval=interval)

    summary = SummaryAnalysis(symbol=symbol.upper())
    # Define the tasks to run concurrently
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
        get_summary_aroon(quotes[:30], 25),
        get_summary_bbands(quotes[:20], 20, 2),
        get_summary_super_trend(quotes[:92], 14, 3),
        get_summary_ichimoku(quotes[:78]),
    ]
    # Run the tasks concurrently and unpack the results
    sma, ema, wma, vwma, rsi, srsi, stoch, cci, macd, adx, aroon, bbands, super_trend, ichimoku = await (
        asyncio.gather(*tasks)
    )

    summary.sma_10 = sma[4] if sma else None
    summary.sma_20 = sma[3] if sma else None
    summary.sma_50 = sma[2] if sma else None
    summary.sma_100 = sma[1] if sma else None
    summary.sma_200 = sma[0] if sma else None
    summary.ema_10 = ema[4] if ema else None
    summary.ema_20 = ema[3] if ema else None
    summary.ema_50 = ema[2] if ema else None
    summary.ema_100 = ema[1] if ema else None
    summary.ema_200 = ema[0] if ema else None
    summary.wma_10 = wma[4] if wma else None
    summary.wma_20 = wma[3] if wma else None
    summary.wma_50 = wma[2] if wma else None
    summary.wma_100 = wma[1] if wma else None
    summary.wma_200 = wma[0] if wma else None
    summary.vwma = vwma
    summary.rsi = rsi
    summary.srsi = srsi
    summary.stoch = stoch
    summary.cci = cci
    summary.macd = macd
    summary.adx = adx
    summary.aroon = aroon
    summary.bbands = bbands
    summary.supertrend = super_trend
    summary.ichimoku = ichimoku

    return summary.model_dump(by_alias=True)
