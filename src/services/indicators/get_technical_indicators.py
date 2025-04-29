import asyncio

import numpy as np

from src.models.historical_data import Interval, TimeRange
from src.models.indicators import (
    ADXData,
    AROONData,
    BBANDSData,
    CCIData,
    EMAData,
    IchimokuData,
    Indicator,
    IndicatorData,
    MACDData,
    RSIData,
    SMAData,
    SRSIData,
    STOCHData,
    SuperTrendData,
    VWMAData,
    WMAData,
)
from src.services.historical.get_historical import get_historical
from src.services.indicators.core import (
    calculate_adx,
    calculate_aroon,
    calculate_bbands,
    calculate_cci,
    calculate_ema,
    calculate_ichimoku,
    calculate_macd,
    calculate_rsi,
    calculate_sma,
    calculate_stoch,
    calculate_stoch_rsi,
    calculate_supertrend,
    calculate_vwma,
    calculate_wma,
    prepare_price_data,
)


async def get_technical_indicators(
    symbol: str,
    interval: Interval,
    indicators: list[Indicator] = None,
) -> dict[str, IndicatorData]:
    # Default to all indicators if none specified
    if not indicators:
        indicators = list(Indicator)

    # Any other interval will be mapped to 5 years
    interval_to_range = {
        Interval.ONE_MINUTE: TimeRange.FIVE_DAYS,
        Interval.FIVE_MINUTES: TimeRange.ONE_MONTH,
        Interval.FIFTEEN_MINUTES: TimeRange.ONE_MONTH,
        Interval.THIRTY_MINUTES: TimeRange.ONE_MONTH,
        Interval.ONE_HOUR: TimeRange.ONE_MONTH,
    }

    time_range = interval_to_range.get(interval, TimeRange.FIVE_YEARS)

    quotes = await get_historical(symbol, time_range=time_range, interval=interval)
    dates, prices, high_prices, low_prices, volumes = prepare_price_data(quotes)
    tasks = []

    for indicator in indicators:
        if indicator == Indicator.SMA:
            for period in [10, 20, 50, 100, 200]:
                tasks.append((f"SMA({period})", get_sma_data(prices, period)))
        elif indicator == Indicator.EMA:
            for period in [10, 20, 50, 100, 200]:
                tasks.append((f"EMA({period})", get_ema_data(prices, period)))
        elif indicator == Indicator.WMA:
            for period in [10, 20, 50, 100, 200]:
                tasks.append((f"WMA({period})", get_wma_data(prices, period)))
        elif indicator == Indicator.VWMA:
            tasks.append(("VWMA(20)", get_vwma_data(prices, volumes)))
        elif indicator == Indicator.RSI:
            tasks.append(("RSI(14)", get_rsi_data(prices)))
        elif indicator == Indicator.SRSI:
            tasks.append(("SRSI(3,3,14,14)", get_srsi_data(prices)))
        elif indicator == Indicator.STOCH:
            tasks.append(("STOCH %K(14,3,3)", get_stoch_data(high_prices, low_prices, prices)))
        elif indicator == Indicator.CCI:
            tasks.append(("CCI(20)", get_cci_data(high_prices, low_prices, prices)))
        elif indicator == Indicator.MACD:
            tasks.append(("MACD(12,26)", get_macd_data(prices)))
        elif indicator == Indicator.ADX:
            tasks.append(("ADX(14)", get_adx_data(high_prices, low_prices, prices)))
        elif indicator == Indicator.AROON:
            tasks.append(("Aroon(25)", get_aroon_data(high_prices, low_prices)))
        elif indicator == Indicator.BBANDS:
            tasks.append(("BBANDS(20,2)", get_bbands_data(prices)))
        elif indicator == Indicator.SUPER_TREND:
            tasks.append(("Super Trend", get_supertrend_data(high_prices, low_prices, prices)))
        elif indicator == Indicator.ICHIMOKU:
            tasks.append(("Ichimoku Cloud", get_ichimoku_data(high_prices, low_prices, prices)))

    task_results = await asyncio.gather(*[task[1] for task in tasks])
    return {name: result for (name, _), result in zip(tasks, task_results, strict=False)}


# Helper functions for individual indicators
async def get_sma_data(prices, period):
    sma_values = calculate_sma(prices, period=period)
    return SMAData(value=round(float(sma_values[-1]), 2) if not np.isnan(sma_values[-1]) else None)


async def get_ema_data(prices, period):
    ema_values = calculate_ema(prices, period=period)
    return EMAData(value=round(float(ema_values[-1]), 2) if not np.isnan(ema_values[-1]) else None)


async def get_wma_data(prices, period):
    wma_values = calculate_wma(prices, period=period)
    return WMAData(value=round(float(wma_values[-1]), 2) if not np.isnan(wma_values[-1]) else None)


async def get_vwma_data(prices, volumes):
    vwma_values = calculate_vwma(prices, volumes, period=20)
    return VWMAData(value=round(float(vwma_values[-1]), 2) if not np.isnan(vwma_values[-1]) else None)


async def get_rsi_data(prices):
    rsi_values = calculate_rsi(prices, period=14)
    return RSIData(value=round(float(rsi_values[-1]), 2) if not np.isnan(rsi_values[-1]) else None)


async def get_srsi_data(prices):
    k_values, d_values = calculate_stoch_rsi(prices, rsi_period=14, stoch_period=14, smooth=3, signal_period=3)
    return SRSIData(
        k=round(float(k_values[-1]), 2) if not np.isnan(k_values[-1]) else None,
        d=round(float(d_values[-1]), 2) if not np.isnan(d_values[-1]) else None,
    )


async def get_stoch_data(high_prices, low_prices, prices):
    k_values, d_values = calculate_stoch(high_prices, low_prices, prices, period=14, smooth=3, signal_period=3)
    return STOCHData(
        k=round(float(k_values[-1]), 2) if not np.isnan(k_values[-1]) else None,
        d=round(float(d_values[-1]), 2) if not np.isnan(d_values[-1]) else None,
    )


async def get_cci_data(high_prices, low_prices, prices):
    cci_values = calculate_cci(high_prices, low_prices, prices, period=20)
    return CCIData(value=round(float(cci_values[-1]), 2) if not np.isnan(cci_values[-1]) else None)


async def get_macd_data(prices):
    macd_line, signal_line = calculate_macd(prices, fast_period=12, slow_period=26, signal_period=9)
    return MACDData(
        value=round(float(macd_line[-1]), 2) if not np.isnan(macd_line[-1]) else None,
        signal=round(float(signal_line[-1]), 2) if not np.isnan(signal_line[-1]) else None,
    )


async def get_adx_data(high_prices, low_prices, prices):
    adx_values = calculate_adx(high_prices, low_prices, prices, period=14)
    return ADXData(value=round(float(adx_values[-1]), 2) if not np.isnan(adx_values[-1]) else None)


async def get_aroon_data(high_prices, low_prices):
    aroon_up, aroon_down = calculate_aroon(high_prices, low_prices, period=25)
    return AROONData(
        aroon_up=round(float(aroon_up[-1]), 2) if not np.isnan(aroon_up[-1]) else None,
        aroon_down=round(float(aroon_down[-1]), 2) if not np.isnan(aroon_down[-1]) else None,
    )


async def get_bbands_data(prices):
    upper_band, middle_band, lower_band = calculate_bbands(prices, period=20, std_dev=2)
    return BBANDSData(
        upper_band=round(float(upper_band[-1]), 2) if not np.isnan(upper_band[-1]) else None,
        middle_band=round(float(middle_band[-1]), 2) if not np.isnan(middle_band[-1]) else None,
        lower_band=round(float(lower_band[-1]), 2) if not np.isnan(lower_band[-1]) else None,
    )


async def get_supertrend_data(high_prices, low_prices, prices):
    supertrend_values, trend = calculate_supertrend(high_prices, low_prices, prices, period=10, multiplier=3)
    return SuperTrendData(
        value=round(float(supertrend_values[-1]), 2) if not np.isnan(supertrend_values[-1]) else None,
        trend="UP" if trend[-1] > 0 else "DOWN",
    )


async def get_ichimoku_data(high_prices, low_prices, prices):
    tenkan_sen, kijun_sen, senkou_span_a, senkou_span_b, chikou_span = calculate_ichimoku(
        high_prices, low_prices, prices, tenkan_period=9, kijun_period=26, senkou_period=52
    )
    return IchimokuData(
        tenkan_sen=round(float(tenkan_sen[-1]), 2) if not np.isnan(tenkan_sen[-1]) else prices[-1],
        kijun_sen=round(float(kijun_sen[-1]), 2) if not np.isnan(kijun_sen[-1]) else prices[-1],
        senkou_span_a=round(float(senkou_span_a[-1]), 2) if not np.isnan(senkou_span_a[-1]) else prices[-1],
        senkou_span_b=round(float(senkou_span_b[-1]), 2) if not np.isnan(senkou_span_b[-1]) else prices[-1],
        chikou_span=round(float(chikou_span[-1]), 2) if not np.isnan(chikou_span[-1]) else prices[-1],
    )
