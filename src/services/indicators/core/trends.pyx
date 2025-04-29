# cython: boundscheck=False
# cython: wraparound=False
# cython: cdivision=True
# cython: language_level=3

import numpy as np
cimport numpy as np

ctypedef np.float64_t DTYPE_t

def calculate_macd(np.ndarray[DTYPE_t, ndim=1] prices, int fast_period=12, int slow_period=26, int signal_period=9):
    """
    Calculate Moving Average Convergence Divergence (MACD).

    MACD = Fast EMA - Slow EMA
    Signal = EMA of MACD

    Formula:
    1. MACD Line = EMA(fast_period) - EMA(slow_period)
    2. Signal Line = EMA(MACD Line, signal_period)
    """
    cdef int n = <int> len(prices)
    cdef np.ndarray[DTYPE_t, ndim=1] macd_line = np.empty(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] signal_line = np.empty(n, dtype=np.float64)
    macd_line[:] = np.nan
    signal_line[:] = np.nan

    if n < max(fast_period, slow_period, signal_period):
        return macd_line, signal_line

    # Calculate EMAs
    cdef double fast_multiplier = 2.0 / (fast_period + 1)
    cdef double slow_multiplier = 2.0 / (slow_period + 1)
    cdef double signal_multiplier = 2.0 / (signal_period + 1)

    # Initialize EMAs with SMA
    cdef double fast_ema = np.mean(prices[:fast_period])
    cdef double slow_ema = np.mean(prices[:slow_period])
    cdef int i

    # Calculate MACD line
    for i in range(slow_period, n):
        fast_ema = (prices[i] - fast_ema) * fast_multiplier + fast_ema
        slow_ema = (prices[i] - slow_ema) * slow_multiplier + slow_ema
        macd_line[i] = fast_ema - slow_ema

    # Calculate Signal line
    cdef double signal_ema = np.mean(macd_line[slow_period:slow_period + signal_period])
    signal_line[slow_period + signal_period - 1] = signal_ema

    for i in range(slow_period + signal_period, n):
        signal_ema = (macd_line[i] - signal_ema) * signal_multiplier + signal_ema
        signal_line[i] = signal_ema

    return macd_line, signal_line

def calculate_adx(np.ndarray[DTYPE_t, ndim=1] high, np.ndarray[DTYPE_t, ndim=1] low,
                  np.ndarray[DTYPE_t, ndim=1] close, int period=14):
    """
    Calculate Average Directional Index (ADX).

    Formula:
    1. TR = max(high - low, |high - prev_close|, |low - prev_close|)
    2. +DM = high - prev_high (if positive, else 0)
    3. -DM = prev_low - low (if positive, else 0)
    4. +DI = 100 * EMA(+DM) / EMA(TR)
    5. -DI = 100 * EMA(-DM) / EMA(TR)
    6. DX = 100 * |+DI - -DI| / (|+DI + -DI|)
    7. ADX = EMA(DX)
    """
    cdef int n = <int> len(high)
    cdef np.ndarray[DTYPE_t, ndim=1] tr = np.zeros(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] plus_dm = np.zeros(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] minus_dm = np.zeros(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] adx = np.empty(n, dtype=np.float64)
    adx[:] = np.nan

    if n < period + 1:
        return adx

    cdef int i
    cdef double up_move, down_move

    # Calculate TR and DM
    for i in range(1, n):
        tr[i] = max(
            high[i] - low[i],
            abs(high[i] - close[i - 1]),
            abs(low[i] - close[i - 1])
        )

        up_move = high[i] - high[i - 1]
        down_move = low[i - 1] - low[i]

        # Positive DM
        if up_move > down_move and up_move > 0:
            plus_dm[i] = up_move
        else:
            plus_dm[i] = 0

        # Negative DM
        if down_move > up_move and down_move > 0:
            minus_dm[i] = down_move
        else:
            minus_dm[i] = 0

    # Calculate smoothed TR and DM
    cdef np.ndarray[DTYPE_t, ndim=1] smoothed_tr = np.zeros(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] smoothed_plus_dm = np.zeros(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] smoothed_minus_dm = np.zeros(n, dtype=np.float64)

    # Initial smoothed values
    smoothed_tr[period] = np.sum(tr[1:period + 1])
    smoothed_plus_dm[period] = np.sum(plus_dm[1:period + 1])
    smoothed_minus_dm[period] = np.sum(minus_dm[1:period + 1])

    # Calculate subsequent smoothed values
    for i in range(period + 1, n):
        smoothed_tr[i] = smoothed_tr[i - 1] - (smoothed_tr[i - 1] / period) + tr[i]
        smoothed_plus_dm[i] = smoothed_plus_dm[i - 1] - (smoothed_plus_dm[i - 1] / period) + plus_dm[i]
        smoothed_minus_dm[i] = smoothed_minus_dm[i - 1] - (smoothed_minus_dm[i - 1] / period) + minus_dm[i]

    # Calculate +DI and -DI
    cdef np.ndarray[DTYPE_t, ndim=1] plus_di = np.zeros(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] minus_di = np.zeros(n, dtype=np.float64)

    for i in range(period, n):
        if smoothed_tr[i] > 0:
            plus_di[i] = 100 * smoothed_plus_dm[i] / smoothed_tr[i]
            minus_di[i] = 100 * smoothed_minus_dm[i] / smoothed_tr[i]

    # Calculate ADX
    cdef double dx
    for i in range(period * 2 - 1, n):
        if (plus_di[i] + minus_di[i]) > 0:
            dx = 100 * abs(plus_di[i] - minus_di[i]) / (plus_di[i] + minus_di[i])
            if i == period * 2 - 1:
                adx[i] = dx
            else:
                adx[i] = (adx[i - 1] * (period - 1) + dx) / period

    return adx

def calculate_aroon(np.ndarray[DTYPE_t, ndim=1] high, np.ndarray[DTYPE_t, ndim=1] low, int period=25):
    """
    Calculate Aroon indicator.

    Formula:
    Aroon Up = ((period - periods since highest high) / period) * 100
    Aroon Down = ((period - periods since lowest low) / period) * 100

    Both oscillators range between 0 and 100, with readings above 70 indicating a strong trend.
    """
    cdef int n = <int> len(high)
    cdef np.ndarray[DTYPE_t, ndim=1] aroon_up = np.empty(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] aroon_down = np.empty(n, dtype=np.float64)
    aroon_up[:] = np.nan
    aroon_down[:] = np.nan

    if n < period:
        return aroon_up, aroon_down

    cdef int i, j
    cdef int high_index, low_index
    cdef double max_high, min_low

    for i in range(period - 1, n):
        # Initialize for current window
        max_high = high[i - period + 1]  # Start of window
        min_low = low[i - period + 1]  # Start of window
        high_index = i - period + 1
        low_index = i - period + 1

        # Scan the window for highest high and lowest low
        for j in range(i - period + 1, i + 1):
            if high[j] >= max_high:
                max_high = high[j]
                high_index = j
            if low[j] <= min_low:
                min_low = low[j]
                low_index = j

        # Calculate periods since highest high and lowest low
        days_since_high = i - high_index
        days_since_low = i - low_index

        # Calculate Aroon values (ensure floating point division)
        aroon_up[i] = ((period - days_since_high) * 100.0) / period
        aroon_down[i] = ((period - days_since_low) * 100.0) / period

    return aroon_up, aroon_down

def calculate_bbands(np.ndarray[DTYPE_t, ndim=1] prices, int period=20, double std_dev=2.0):
    """
    Calculate Bollinger Bands.

    Formula:
    Middle Band = SMA(price, period)
    Upper Band = Middle Band + (std_dev * Standard Deviation)
    Lower Band = Middle Band - (std_dev * Standard Deviation)

    The bands expand and contract based on volatility, with wider bands indicating
    higher volatility and narrower bands indicating lower volatility.
    """
    cdef int n = <int> len(prices)
    cdef np.ndarray[DTYPE_t, ndim=1] upper_band = np.empty(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] middle_band = np.empty(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] lower_band = np.empty(n, dtype=np.float64)
    upper_band[:] = np.nan
    middle_band[:] = np.nan
    lower_band[:] = np.nan

    if n < period:
        return upper_band, middle_band, lower_band

    cdef int i
    cdef double std

    # Calculate rolling mean and standard deviation
    for i in range(period - 1, n):
        window = prices[i - period + 1:i + 1]
        middle_band[i] = np.mean(window)
        std = np.std(window, ddof=1)  # ddof=1 for sample standard deviation
        upper_band[i] = middle_band[i] + (std_dev * std)
        lower_band[i] = middle_band[i] - (std_dev * std)

    return upper_band, middle_band, lower_band

def calculate_obv(np.ndarray[DTYPE_t, ndim=1] closes, np.ndarray[DTYPE_t, ndim=1] volumes):
    """
    Calculate On-Balance Volume (OBV).

    Formula:
    If closing price > prior close: OBV = Previous OBV + Current Volume
    If closing price < prior close: OBV = Previous OBV - Current Volume
    If closing price = prior close: OBV = Previous OBV

    OBV is a cumulative indicator that shows volume flow, potentially indicating
    price movements before they occur.
    """
    cdef int n = <int> len(closes)
    cdef np.ndarray[DTYPE_t, ndim=1] obv = np.zeros(n, dtype=np.float64)

    if n < 2:
        return obv

    cdef int i
    obv[0] = volumes[0]

    for i in range(1, n):
        if closes[i] > closes[i - 1]:
            obv[i] = obv[i - 1] + volumes[i]
        elif closes[i] < closes[i - 1]:
            obv[i] = obv[i - 1] - volumes[i]
        else:
            obv[i] = obv[i - 1]

    return obv

def calculate_supertrend(np.ndarray[DTYPE_t, ndim=1] high, np.ndarray[DTYPE_t, ndim=1] low,
                         np.ndarray[DTYPE_t, ndim=1] close, int period=10, double multiplier=3.0):
    """
    Calculate Super Trend indicator.

    Formula:
    1. ATR = Average True Range
    2. Basic Upper Band = (High + Low) / 2 + multiplier * ATR
    3. Basic Lower Band = (High + Low) / 2 - multiplier * ATR
    4. Final Bands are adjusted based on previous Super Trend value
    5. Trend changes when price crosses the SuperTrend line
    """
    cdef int n = <int> len(high)
    cdef np.ndarray[DTYPE_t, ndim=1] supertrend = np.empty(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] trend = np.zeros(n, dtype=np.float64)
    supertrend[:] = np.nan
    trend[:] = np.nan

    if n < period:
        return supertrend, trend

    # Calculate TR and ATR
    cdef np.ndarray[DTYPE_t, ndim=1] tr = np.zeros(n)
    cdef np.ndarray[DTYPE_t, ndim=1] atr = np.zeros(n)
    cdef int i

    # Calculate True Range
    tr[0] = high[0] - low[0]  # First TR
    for i in range(1, n):
        tr[i] = max(
            high[i] - low[i],
            abs(high[i] - close[i - 1]),
            abs(low[i] - close[i - 1])
        )

    # Calculate ATR
    atr[period - 1] = np.mean(tr[:period])
    for i in range(period, n):
        atr[i] = (atr[i - 1] * (period - 1) + tr[i]) / period

    # Calculate basic upper and lower bands
    cdef np.ndarray[DTYPE_t, ndim=1] basic_upperband = np.zeros(n)
    cdef np.ndarray[DTYPE_t, ndim=1] basic_lowerband = np.zeros(n)
    cdef np.ndarray[DTYPE_t, ndim=1] final_upperband = np.zeros(n)
    cdef np.ndarray[DTYPE_t, ndim=1] final_lowerband = np.zeros(n)

    for i in range(period, n):
        basic_upperband[i] = ((high[i] + low[i]) / 2) + (multiplier * atr[i])
        basic_lowerband[i] = ((high[i] + low[i]) / 2) - (multiplier * atr[i])

    # Initialize final bands and trend
    final_upperband[period] = basic_upperband[period]
    final_lowerband[period] = basic_lowerband[period]

    # Initialize trend based on close price position relative to the middle of the bands
    if close[period] <= (final_upperband[period] + final_lowerband[period]) / 2:
        trend[period] = -1
        supertrend[period] = final_upperband[period]
    else:
        trend[period] = 1
        supertrend[period] = final_lowerband[period]

    # Calculate final upper and lower bands
    for i in range(period + 1, n):
        # Calculate final upper band
        if basic_upperband[i] < final_upperband[i - 1] or close[i - 1] > final_upperband[i - 1]:
            final_upperband[i] = basic_upperband[i]
        else:
            final_upperband[i] = final_upperband[i - 1]

        # Calculate final lower band
        if basic_lowerband[i] > final_lowerband[i - 1] or close[i - 1] < final_lowerband[i - 1]:
            final_lowerband[i] = basic_lowerband[i]
        else:
            final_lowerband[i] = final_lowerband[i - 1]

        # Determine trend
        if trend[i - 1] == 1:  # Previous trend was UP
            if close[i] < final_lowerband[i]:
                trend[i] = -1  # Switch to DOWN
                supertrend[i] = final_upperband[i]
            else:
                trend[i] = 1  # Maintain UP
                supertrend[i] = final_lowerband[i]
        else:  # Previous trend was DOWN
            if close[i] > final_upperband[i]:
                trend[i] = 1  # Switch to UP
                supertrend[i] = final_lowerband[i]
            else:
                trend[i] = -1  # Maintain DOWN
                supertrend[i] = final_upperband[i]

    return supertrend, trend

def calculate_ichimoku(np.ndarray[DTYPE_t, ndim=1] high, np.ndarray[DTYPE_t, ndim=1] low,
                      np.ndarray[DTYPE_t, ndim=1] close, int tenkan_period=9,
                      int kijun_period=26, int senkou_period=52):
    """
    Calculate Ichimoku Cloud components.

    Formula:
    Tenkan-sen (Conversion Line) = (Highest High + Lowest Low) / 2 for the last tenkan_period periods
    Kijun-sen (Base Line) = (Highest High + Lowest Low) / 2 for the last kijun_period periods
    Senkou Span A = (Tenkan-sen + Kijun-sen) / 2 shifted forward kijun_period periods
    Senkou Span B = (Highest High + Lowest Low) / 2 for the last senkou_period periods, shifted forward kijun_period periods
    Chikou Span = Current closing price shifted backward kijun_period periods
    """
    cdef int n = <int> len(high)
    cdef np.ndarray[DTYPE_t, ndim=1] tenkan_sen = np.empty(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] kijun_sen = np.empty(n, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] senkou_span_a = np.empty(n + kijun_period, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] senkou_span_b = np.empty(n + kijun_period, dtype=np.float64)
    cdef np.ndarray[DTYPE_t, ndim=1] chikou_span = np.empty(n, dtype=np.float64)

    tenkan_sen[:] = np.nan
    kijun_sen[:] = np.nan
    senkou_span_a[:] = np.nan
    senkou_span_b[:] = np.nan
    chikou_span[:] = np.nan

    cdef int i, j
    cdef double period_high, period_low

    # Calculate Tenkan-sen (Conversion Line)
    for i in range(tenkan_period - 1, n):
        period_high = -np.inf  # Initialize to negative infinity
        period_low = np.inf    # Initialize to positive infinity

        # Calculate highest high and lowest low for the period
        for j in range(i - tenkan_period + 1, i + 1):
            if high[j] > period_high:
                period_high = high[j]
            if low[j] < period_low:
                period_low = low[j]
        tenkan_sen[i] = (period_high + period_low) / 2

    # Calculate Kijun-sen (Base Line)
    for i in range(kijun_period - 1, n):
        period_high = -np.inf
        period_low = np.inf

        # Calculate highest high and lowest low for the period
        for j in range(i - kijun_period + 1, i + 1):
            if high[j] > period_high:
                period_high = high[j]
            if low[j] < period_low:
                period_low = low[j]
        kijun_sen[i] = (period_high + period_low) / 2

    # Calculate Senkou Span A (Leading Span A)
    # Calculate the spans first, then shift them forward
    for i in range(max(tenkan_period, kijun_period) - 1, n):  # Removed - kijun_period
        if i + kijun_period - 1 < n:  # Ensure we don't write beyond array bounds
            senkou_span_a[i + kijun_period - 1] = (tenkan_sen[i] + kijun_sen[i]) / 2

    # Calculate Senkou Span B (Leading Span B)
    # Calculate for senkou_period, then shift forward
    for i in range(senkou_period - 1, n):  # Removed - kijun_period
        if i + kijun_period - 1 < n:  # Ensure we don't write beyond array bounds
            period_high = -np.inf
            period_low = np.inf
            for j in range(i - senkou_period + 1, i + 1):
                if high[j] > period_high:
                    period_high = high[j]
                if low[j] < period_low:
                    period_low = low[j]
            senkou_span_b[i + kijun_period - 1] = (period_high + period_low) / 2

    # Calculate Chikou Span (Lagging Span)
    # Shifted backwards by kijun_period
    chikou_span[:-kijun_period + 1] = close[kijun_period - 1:]

    return tenkan_sen, kijun_sen, senkou_span_a[:n], senkou_span_b[:n], chikou_span
