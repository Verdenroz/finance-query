# cython: boundscheck=False
# cython: wraparound=False
# cython: cdivision=True
# cython: language_level=3

import numpy as np
cimport numpy as np
from libc.math cimport isnan

ctypedef np.float64_t DTYPE_t

def calculate_sma(dict historical_data, int period) -> dict[str, float]:
    """
    Calculate Simple Moving Average (SMA).

    The SMA is calculated by taking the average of the closing prices over a specified period.

    Formula:
    SMA = (P1 + P2 + ... + Pn) / n

    where P1, P2, ..., Pn are the closing prices over the period.

    :param historical_data: Dictionary containing historical data with dates as keys and data objects as values.
    :param period: The number of periods to calculate the SMA over.

    :return: Dictionary with dates as keys and SMA values as values.
    """
    cdef:
        int i, n
        double[:] prices
        double[:] sma_values
        double window_sum = 0.0

    # Ensure dates are sorted in ascending order
    dates = list(historical_data.keys())
    dates.sort()
    prices = np.array([float(historical_data[date].close) for date in dates], dtype=np.float64)

    n = <int>len(prices)
    sma_values = np.empty(n, dtype=np.float64)
    sma_values[:] = np.nan

    if n < period:
        return {}

    # Calculate initial window sum
    for i in range(period):
        window_sum += prices[i]
    sma_values[period-1] = window_sum / period

    # Calculate remaining values using sliding window
    for i in range(period, n):
        window_sum = window_sum - prices[i-period] + prices[i]
        sma_values[i] = window_sum / period

    return {dates[i]: round(float(str(sma_values[i])), 2)
            for i in range(n) if not isnan(sma_values[i])}

def calculate_ema(dict historical_data, int period) -> dict[str, float]:
    """
    Calculate Exponential Moving Average (EMA).

    The EMA is calculated by applying more weight to recent prices, using the following formula:

    EMA_today = (Price_today * Multiplier) + (EMA_yesterday * (1 - Multiplier))

    where Multiplier = 2 / (period + 1)

    :param historical_data: Dictionary containing historical data with dates as keys and data objects as values.
    :param period: The number of periods to calculate the EMA over.

    :return: Dictionary with dates as keys and EMA values as values.
    """
    cdef:
        int i, n
        double[:] prices
        double[:] ema_values
        double multiplier = 2.0 / (period + 1)
        double prev_ema

    # Ensure dates are sorted in ascending order
    dates = list(historical_data.keys())
    dates.sort()
    prices = np.array([float(historical_data[date].close) for date in dates], dtype=np.float64)

    n = <int>len(prices)
    ema_values = np.empty(n, dtype=np.float64)
    ema_values[:] = np.nan

    if n < period:
        return {}

    # Initialize EMA with SMA for first period
    prev_ema = 0.0
    for i in range(period):
        prev_ema += prices[i]
    prev_ema /= period
    ema_values[period-1] = prev_ema

    # Calculate EMA for remaining periods
    for i in range(period, n):
        prev_ema = (prices[i] - prev_ema) * multiplier + prev_ema
        ema_values[i] = prev_ema

    return {dates[i]: round(float(str(ema_values[i])), 2)
            for i in range(n) if not isnan(ema_values[i])}

def calculate_wma(dict historical_data, int period) -> dict[str, float]:
    """
    Calculate Weighted Moving Average (WMA).

    The WMA is calculated by assigning a weight to each price, with more recent prices having higher weights, using the following formula:

    WMA = (P1 * W1 + P2 * W2 + ... + Pn * Wn) / (W1 + W2 + ... + Wn)

    where P1, P2, ..., Pn are the prices and W1, W2, ..., Wn are the weights.

    :param historical_data: Dictionary containing historical data with dates as keys and data objects as values.
    :param period: The number of periods to calculate the WMA over.

    :return: Dictionary with dates as keys and WMA values as values.
    """
    cdef:
        int i, j, n
        double[:] prices
        double[:] wma_values
        double numerator, denominator
        int[:] weights

    # Ensure dates are sorted in ascending order
    dates = list(historical_data.keys())
    dates.sort()
    prices = np.array([float(historical_data[date].close) for date in dates], dtype=np.float64)

    n = <int>len(prices)
    wma_values = np.empty(n, dtype=np.float64)
    wma_values[:] = np.nan

    if n < period:
        return {}

    # Create weight array [period, period-1, ..., 1]
    weights = np.arange(period, 0, -1, dtype=np.int32)
    denominator = (period * (period + 1)) / 2.0

    # Calculate WMA
    for i in range(period-1, n):
        numerator = 0.0
        for j in range(period):
            numerator += prices[i-j] * weights[j]
        wma_values[i] = numerator / denominator

    return {dates[i]: round(float(str(wma_values[i])), 2)
            for i in range(n) if not isnan(wma_values[i])}

def calculate_vwma(dict historical_data, int period) -> dict[str, float]:
    """
    Calculate Volume Weighted Moving Average (VWMA).

    The VWMA is calculated by weighting prices by their volume, using the following formula:

    VWMA = (P1 * V1 + P2 * V2 + ... + Pn * Vn) / (V1 + V2 + ... + Vn)

    where P1, P2, ..., Pn are the prices and V1, V2, ..., Vn are the volumes.

    :param historical_data: Dictionary containing historical data with dates as keys and data objects as values.
    :param period: The number of periods to calculate the VWMA over.

    :return: Dictionary with dates as keys and VWMA values as values.
    """
    cdef:
        int i, n
        double[:] prices
        double[:] volumes
        double[:] vwma_values
        double price_volume_sum = 0.0
        double volume_sum = 0.0

    # Ensure dates are sorted in ascending order
    dates = list(historical_data.keys())
    dates.sort()
    prices = np.array([float(historical_data[date].close) for date in dates], dtype=np.float64)
    volumes = np.array([float(historical_data[date].volume) for date in dates], dtype=np.float64)

    n = <int>len(prices)
    vwma_values = np.empty(n, dtype=np.float64)
    vwma_values[:] = np.nan

    if n < period:
        return {}

    # Calculate initial sums
    for i in range(period):
        price_volume_sum += prices[i] * volumes[i]
        volume_sum += volumes[i]
    vwma_values[period-1] = price_volume_sum / volume_sum

    # Calculate remaining values using sliding window
    for i in range(period, n):
        price_volume_sum = price_volume_sum - (prices[i-period] * volumes[i-period]) + (prices[i] * volumes[i])
        volume_sum = volume_sum - volumes[i-period] + volumes[i]
        vwma_values[i] = price_volume_sum / volume_sum if volume_sum > 0 else np.nan

    return {dates[i]: round(float(str(vwma_values[i])), 2)
            for i in range(n) if not isnan(vwma_values[i])}