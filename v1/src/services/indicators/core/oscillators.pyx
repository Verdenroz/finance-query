# cython: boundscheck=False
# cython: wraparound=False
# cython: cdivision=True
# cython: language_level=3

import numpy as np
cimport numpy as np
from libc.math cimport fabs

ctypedef np.float64_t DTYPE_t

def calculate_rsi(np.ndarray[DTYPE_t, ndim=1] prices, int period=14):
    """
    Calculate Relative Strength Index (RSI)

    The RSI is calculated using the following formula:

    RSI = 100 - (100 / (1 + RS))

    where RS (Relative Strength) is the average of 'n' days' up closes divided by the average of 'n' days' down closes.

    RS = Average Gain / Average Loss

    Average Gain = (Previous Average Gain * (period - 1) + Current Gain) / period
    Average Loss = (Previous Average Loss * (period - 1) + Current Loss) / period
    """
    cdef int i
    cdef int n = <int> len(prices)
    cdef np.ndarray[DTYPE_t, ndim=1] gains = np.zeros(n - 1, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] losses = np.zeros(n - 1, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] rsi = np.zeros(n, dtype=np.float64)
    cdef double avg_gain, avg_loss, rs

    # Calculate price changes and separate into gains and losses
    for i in range(n - 1):
        change = prices[i + 1] - prices[i]
        if change > 0:
            gains[i] = change
        else:
            losses[i] = -change

    # Skip first period values where RSI isn't calculable
    rsi[:period] = np.nan

    # Calculate first SMA of gains and losses
    avg_gain = np.sum(gains[:period]) / period
    avg_loss = np.sum(losses[:period]) / period

    # Calculate first RSI value
    if avg_loss == 0:
        rsi[period] = 100
    else:
        rs = avg_gain / avg_loss
        rsi[period] = 100 - (100 / (1 + rs))

    # Calculate subsequent values using Wilder's smoothing
    for i in range(period, n - 1):
        avg_gain = ((avg_gain * (period - 1)) + gains[i]) / period
        avg_loss = ((avg_loss * (period - 1)) + losses[i]) / period

        if avg_loss == 0:
            rsi[i + 1] = 100
        else:
            rs = avg_gain / avg_loss
            rsi[i + 1] = 100 - (100 / (1 + rs))

    return rsi

def calculate_stoch_rsi(np.ndarray[DTYPE_t, ndim=1] prices, int rsi_period=14,
                        int stoch_period=14, int smooth=3, int signal_period=3):
    """
    Calculate Stochastic RSI (SRSI)

    The Stochastic RSI is calculated using the following formulas:

    %K = (RSI - Lowest Low RSI) / (Highest High RSI - Lowest Low RSI) * 100
    Smoothed %K = smooth-period SMA of %K
    %D = signal-period SMA of Smoothed %K
    """
    cdef np.ndarray[DTYPE_t, ndim=1] rsi_values = calculate_rsi(prices, rsi_period)
    cdef int n = <int> len(rsi_values)
    cdef np.ndarray[DTYPE_t, ndim=1] raw_k = np.full(n, np.nan, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] smoothed_k = np.full(n, np.nan, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] signal_line = np.full(n, np.nan, dtype=np.float64)
    cdef double highest_high, lowest_low
    cdef int i

    # We need valid RSI values before calculating Stochastic RSI
    # Skip the first rsi_period values where RSI isn't valid
    min_periods = max(rsi_period, stoch_period)

    # Calculate raw Stochastic RSI (%K)
    for i in range(min_periods, n):
        highest_high = np.max(rsi_values[i - stoch_period + 1:i + 1])
        lowest_low = np.min(rsi_values[i - stoch_period + 1:i + 1])

        if highest_high - lowest_low > 0:
            raw_k[i] = (rsi_values[i] - lowest_low) / (highest_high - lowest_low) * 100
        else:
            raw_k[i] = 50  # Default to middle value when range is zero

    # Apply first smoothing to get smoothed %K
    if smooth > 1:
        for i in range(min_periods + smooth - 1, n):
            smoothed_k[i] = np.mean(raw_k[i - smooth + 1:i + 1])
    else:
        smoothed_k = raw_k.copy()

    # Calculate signal line (%D) using signal_period SMA of smoothed %K
    for i in range(min_periods + smooth + signal_period - 2, n):
        signal_line[i] = np.mean(smoothed_k[i - signal_period + 1:i + 1])

    return smoothed_k, signal_line

def calculate_stoch(np.ndarray[DTYPE_t, ndim=1] high_prices,
                    np.ndarray[DTYPE_t, ndim=1] low_prices,
                    np.ndarray[DTYPE_t, ndim=1] close_prices,
                    int period=14, int smooth=3, int signal_period=3):
    """
    Calculate Stochastic Oscillator.

    %K = (Current Close - Lowest Low) / (Highest High - Lowest Low) * 100
    Smoothed %K = smooth-period SMA of %K
    %D = signal-period SMA of Smoothed %K
    """
    cdef int n = <int> len(close_prices)
    cdef np.ndarray[DTYPE_t, ndim=1] raw_k = np.empty(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] smoothed_k = np.empty(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] d_line = np.empty(n, dtype=np.float64)
    raw_k[:] = np.nan
    smoothed_k[:] = np.nan
    d_line[:] = np.nan

    if n < period:
        return smoothed_k, d_line

    # Calculate raw %K
    for i in range(period - 1, n):
        highest_high = np.max(high_prices[i - period + 1:i + 1])
        lowest_low = np.min(low_prices[i - period + 1:i + 1])

        if highest_high - lowest_low > 0:
            raw_k[i] = ((close_prices[i] - lowest_low) /
                        (highest_high - lowest_low)) * 100
        else:
            raw_k[i] = 50  # Default to middle value when range is zero

    # Calculate smoothed %K
    if n >= period + smooth - 1:
        for i in range(period + smooth - 2, n):
            smoothed_k[i] = np.mean(raw_k[i - smooth + 1:i + 1])

    # Calculate %D (signal line)
    if n >= period + smooth + signal_period - 2:
        for i in range(period + smooth + signal_period - 3, n):
            d_line[i] = np.mean(smoothed_k[i - signal_period + 1:i + 1])

    return smoothed_k, d_line

def calculate_cci(np.ndarray[DTYPE_t, ndim=1] high_prices,
                  np.ndarray[DTYPE_t, ndim=1] low_prices,
                  np.ndarray[DTYPE_t, ndim=1] close_prices,
                  int period=20):
    """
    Calculate Commodity Channel Index (CCI)

    The CCI is calculated using the following formulas:

    Typical Price (TP) = (High + Low + Close) / 3
    SMA = Simple Moving Average of Typical Price
    Mean Deviation = Average of the absolute differences between the Typical Price and the SMA
    CCI = (Typical Price - SMA) / (0.015 * Mean Deviation)
    """
    cdef int n = <int> len(close_prices)
    cdef np.ndarray[DTYPE_t, ndim=1] cci = np.zeros(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] tp = np.zeros(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] sma = np.zeros(n, dtype=np.float64)
    cdef double mean_dev
    cdef int i, j

    # Calculate Typical Price
    for i in range(n):
        tp[i] = (high_prices[i] + low_prices[i] + close_prices[i]) / 3.0

    # Calculate SMA of Typical Price
    for i in range(period - 1, n):
        sma[i] = np.mean(tp[i - period + 1:i + 1])

        # Calculate Mean Deviation
        mean_dev = 0
        for j in range(i - period + 1, i + 1):
            mean_dev += fabs(tp[j] - sma[i])
        mean_dev /= period

        # Calculate CCI
        if mean_dev == 0:
            cci[i] = 0
        else:
            cci[i] = (tp[i] - sma[i]) / (0.015 * mean_dev)

    return cci
