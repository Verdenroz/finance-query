import asyncio

import numpy as np

from src.cache import cache
from src.models.indicators import (SummaryAnalysis, MACDData, AROONData, BBANDSData, SuperTrendData, IchimokuData,
                                   SRSIData, STOCHData, SMAData, EMAData, WMAData, VWMAData, RSIData, CCIData, ADXData)
from src.models.historical_data import Interval, TimeRange
from src.services.historical.get_historical import get_historical
from src.services.indicators.core import (prepare_price_data, calculate_sma, calculate_ema, calculate_wma,
                                          calculate_vwma, calculate_rsi, calculate_stoch_rsi, calculate_stoch,
                                          calculate_cci, calculate_macd, calculate_adx,
                                          calculate_aroon, calculate_bbands, calculate_supertrend, calculate_ichimoku)


@cache(expire=60, market_closed_expire=600)
async def get_summary_analysis(symbol: str, interval: Interval) -> SummaryAnalysis:
    """
    Calculates the summary analysis for a given symbol and interval, where the summary analysis includes the following
    indicators:
    - Simple Moving Average (SMA) for periods 10, 20, 50, 100, 200
    - Exponential Moving Average (EMA) for periods 10, 20, 50, 100, 200
    - Weighted Moving Average (WMA) for periods 10, 20, 50, 100, 200
    - Volume Weighted Moving Average (VWMA) for period 20
    - Relative Strength Index (RSI) for period 14
    - Stochastic RSI (SRSI) for periods 14, 14, 3, 3
    - Stochastic Oscillator (STOCH) for periods 14, 3, 3
    - Commodity Channel Index (CCI) for period 20
    - Moving Average Convergence Divergence (MACD) for periods 12, 26, 9
    - Average Directional Index (ADX) for period 14
    - Aroon Indicator for period 25
    - Bollinger Bands for period 20, standard deviation 2
    - SuperTrend Indicator for period 10, multiplier 3
    - Ichimoku Cloud for periods 9, 26, 52
    :param symbol:
    :param interval:
    :return:
    """
    results = await get_indicator_data(symbol, interval)
    summary = SummaryAnalysis(
        symbol=symbol.upper(),
        sma_10=results[0],
        sma_20=results[1],
        sma_50=results[2],
        sma_100=results[3],
        sma_200=results[4],
        ema_10=results[5],
        ema_20=results[6],
        ema_50=results[7],
        ema_100=results[8],
        ema_200=results[9],
        wma_10=results[10],
        wma_20=results[11],
        wma_50=results[12],
        wma_100=results[13],
        wma_200=results[14],
        vwma=results[15],
        rsi=results[16],
        srsi=results[17],
        stoch=results[18],
        cci=results[19],
        macd=results[20],
        adx=results[21],
        aroon=results[22],
        bbands=results[23],
        supertrend=results[24],
        ichimoku=results[25]
    )

    return summary


async def get_indicator_data(symbol: str, interval: Interval):
    """
    Aggregates all the indicator data for a given symbol and interval

    :return: tuple of indicator data
    """
    quotes = await get_historical(symbol, time_range=TimeRange.YEAR, interval=interval)
    dates, prices, high_prices, low_prices, volumes = prepare_price_data(quotes)

    async def get_sma_data(period):
        sma_values = calculate_sma(prices, period=period)
        return SMAData(value=round(float(sma_values[-1]), 2))

    async def get_ema_data(period):
        ema_values = calculate_ema(prices, period=period)
        return EMAData(value=round(float(ema_values[-1]), 2))

    async def get_wma_data(period):
        wma_values = calculate_wma(prices, period=period)
        return WMAData(value=round(float(wma_values[-1]), 2))

    async def get_vwma_data():
        vwma_values = calculate_vwma(prices, volumes, period=20)
        return VWMAData(value=round(float(vwma_values[-1]), 2))

    async def get_rsi_data():
        rsi_values = calculate_rsi(prices, period=14)
        return RSIData(value=round(float(rsi_values[-1]), 2))

    async def get_srsi_data():
        k_values, d_values = calculate_stoch_rsi(prices, rsi_period=14, stoch_period=14, smooth=3, signal_period=3)
        return SRSIData(k=round(float(k_values[-1]), 2), d=round(float(d_values[-1]), 2))

    async def get_stoch_data():
        k_values, d_values = calculate_stoch(high_prices, low_prices, prices, period=14, smooth=3, signal_period=3)
        return STOCHData(k=round(float(k_values[-1]), 2), d=round(float(d_values[-1]), 2))

    async def get_cci_data():
        cci_values = calculate_cci(high_prices, low_prices, prices, period=20)
        return CCIData(value=round(float(cci_values[-1]), 2))

    async def get_macd_data():
        macd_line, signal_line = calculate_macd(prices, fast_period=12, slow_period=26, signal_period=9)
        return MACDData(value=round(float(macd_line[-1]), 2), signal=round(float(signal_line[-1]), 2))

    async def get_adx_data():
        adx_values = calculate_adx(high_prices, low_prices, prices, period=14)
        return ADXData(value=round(float(adx_values[-1]), 2))

    async def get_aroon_data():
        aroon_up, aroon_down = calculate_aroon(high_prices, low_prices, period=25)
        return AROONData(aroon_up=round(float(aroon_up[-1]), 2), aroon_down=round(float(aroon_down[-1]), 2))

    async def get_bbands_data():
        upper_band, middle_band, lower_band = calculate_bbands(prices, period=20, std_dev=2)
        return BBANDSData(upper_band=round(float(upper_band[-1]), 2), middle_band=round(float(middle_band[-1]), 2),
                          lower_band=round(float(lower_band[-1]), 2))

    async def get_supertrend_data():
        supertrend_values, trend = calculate_supertrend(high_prices, low_prices, prices, period=10, multiplier=3)
        return SuperTrendData(value=round(float(supertrend_values[-1]), 2), trend="UP" if trend[-1] > 0 else "DOWN")

    async def get_ichimoku_data():
        tenkan_sen, kijun_sen, senkou_span_a, senkou_span_b, chikou_span = (
            calculate_ichimoku(high_prices, low_prices, prices, tenkan_period=9, kijun_period=26, senkou_period=52))
        return IchimokuData(
            tenkan_sen=round(float(tenkan_sen[-1]), 2),
            kijun_sen=round(float(kijun_sen[-1]), 2),
            senkou_span_a=round(float(senkou_span_a[-1]), 2),
            senkou_span_b=round(float(senkou_span_b[-1]), 2),
            chikou_span=round(float(chikou_span[-1]), 2) if not np.isnan(chikou_span[-1]) else prices[-1]
        )

    results = await asyncio.gather(
        get_sma_data(10), get_sma_data(20), get_sma_data(50), get_sma_data(100), get_sma_data(200),
        get_ema_data(10), get_ema_data(20), get_ema_data(50), get_ema_data(100), get_ema_data(200),
        get_wma_data(10), get_wma_data(20), get_wma_data(50), get_wma_data(100), get_wma_data(200),
        get_vwma_data(), get_rsi_data(), get_srsi_data(), get_stoch_data(), get_cci_data(),
        get_macd_data(), get_adx_data(), get_aroon_data(), get_bbands_data(), get_supertrend_data(), get_ichimoku_data()
    )

    return results
