# cython: boundscheck=False
# cython: wraparound=False
# cython: cdivision=True
# cython: language_level=3

import numpy as np
cimport numpy as np

ctypedef np.float64_t DTYPE_t

def calculate_sma(np.ndarray[DTYPE_t, ndim=1] prices, int period):
    """
    Calculate Simple Moving Average (SMA).

    The SMA is calculated by taking the average of the closing prices over a specified period.

    Formula:
    SMA = (P1 + P2 + ... + Pn) / n

    where P1, P2, ..., Pn are the closing prices over the period.
    """
    cdef int n = <int> len(prices)
    cdef np.ndarray[DTYPE_t, ndim=1] sma_values = np.empty(n, dtype=np.float64)
    sma_values[:] = np.nan

    if n < period:
        return sma_values

    cumsum = np.cumsum(np.insert(prices, 0, 0))
    sma_values[period - 1:] = (cumsum[period:] - cumsum[:-period]) / period

    return sma_values

def calculate_ema(np.ndarray[DTYPE_t, ndim=1] prices, int period):
    """
    Calculate Exponential Moving Average (EMA).

    The EMA is calculated by applying more weight to recent prices, using the following formula:

    EMA_today = (Price_today * Multiplier) + (EMA_yesterday * (1 - Multiplier))

    where Multiplier = 2 / (period + 1)
    """
    cdef int i, n = <int> len(prices)
    cdef np.ndarray[DTYPE_t, ndim=1] ema_values = np.empty(n, dtype=np.float64)
    ema_values[:] = np.nan
    cdef double multiplier = 2.0 / (period + 1)
    cdef double prev_ema

    if n < period:
        return ema_values

    # Initialize with SMA
    prev_ema = np.mean(prices[:period])
    ema_values[period - 1] = prev_ema

    # Calculate EMA
    for i in range(period, n):
        prev_ema = (prices[i] - prev_ema) * multiplier + prev_ema
        ema_values[i] = prev_ema

    return ema_values

def calculate_wma(np.ndarray[DTYPE_t, ndim=1] prices, int period):
    """
    Calculate Weighted Moving Average (WMA).

    The WMA is calculated by assigning a weight to each price, with more recent prices having higher weights, using the following formula:

    WMA = (P1 * W1 + P2 * W2 + ... + Pn * Wn) / (W1 + W2 + ... + Wn)

    where P1, P2, ..., Pn are the prices and W1, W2, ..., Wn are the weights.
    """
    cdef int n = <int> len(prices)
    cdef np.ndarray[DTYPE_t, ndim=1] wma_values = np.empty(n, dtype=np.float64)
    wma_values[:] = np.nan

    if n < period:
        return wma_values

    # Create weights array [period, period-1, ..., 1]
    weights = np.arange(period, 0, -1, dtype=np.float64)
    denominator = np.sum(weights)

    wma_values[period - 1:] = np.convolve(prices, weights[::-1], mode='valid') / denominator

    return wma_values

def calculate_vwma(np.ndarray[DTYPE_t, ndim=1] prices, np.ndarray[DTYPE_t, ndim=1] volumes, int period):
    """
    Calculate Volume Weighted Moving Average (VWMA).

    The VWMA is calculated by weighting prices by their volume, using the following formula:

    VWMA = (P1 * V1 + P2 * V2 + ... + Pn * Vn) / (V1 + V2 + ... + Vn)

    where P1, P2, ..., Pn are the prices and V1, V2, ..., Vn are the volumes.
    """
    cdef int n = <int> len(prices)
    cdef np.ndarray[DTYPE_t, ndim=1] vwma_values = np.empty(n, dtype=np.float64)
    vwma_values[:] = np.nan

    if n < period:
        return vwma_values

    price_volume = prices * volumes

    price_volume_sum = np.cumsum(np.insert(price_volume, 0, 0))
    volume_sum = np.cumsum(np.insert(volumes, 0, 0))

    # Calculate VWMA
    numerator = price_volume_sum[period:] - price_volume_sum[:-period]
    denominator = volume_sum[period:] - volume_sum[:-period]
    vwma_values[period - 1:] = np.where(denominator != 0, numerator / denominator, np.nan)

    return vwma_values
