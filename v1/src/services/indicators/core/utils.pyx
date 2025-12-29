# cython: boundscheck=False
# cython: wraparound=False
# cython: cdivision=True
# cython: language_level=3

import numpy as np
cimport numpy as np
from libc.math cimport isnan

ctypedef np.float64_t DTYPE_t

def prepare_price_data(dict historical_data):
    """
    Standardized preparation of price data from historical dictionary.
    Returns sorted dates and corresponding numpy arrays for prices.
    """
    dates = sorted(historical_data.keys())
    prices = np.array([float(historical_data[date].close) for date in dates], dtype=np.float64)
    high_prices = np.array([float(historical_data[date].high) for date in dates], dtype=np.float64)
    low_prices = np.array([float(historical_data[date].low) for date in dates], dtype=np.float64)
    volumes = np.array([float(historical_data[date].volume) for date in dates], dtype=np.float64)

    return dates, prices, high_prices, low_prices, volumes

def create_indicator_dict(list dates, np.ndarray[DTYPE_t, ndim=1] values):
    """
    Standardized creation of date-to-value dictionary for indicators.
    """
    return {dates[i]: round(float(values[i]), 2) if not isnan(values[i]) else None for i in range(len(dates))}
