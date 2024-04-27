import pandas as pd
import yfinance as yf

from src.utils import TimePeriod, Interval

from decimal import Decimal
from src.schemas import HistoricalData, TimeSeries


async def scrape_historical(symbol: str, time: TimePeriod, interval: Interval):
    stock = yf.Ticker(symbol)
    data = stock.history(period=time.value, interval=interval.value, rounding=True)

    # Ensure the DataFrame's index is a DatetimeIndex
    if not isinstance(data.index, pd.DatetimeIndex):
        data.index = pd.to_datetime(data.index)

    # Convert the DataFrame's index to string
    if interval in [Interval.FIFTEEN_MINUTES, Interval.THIRTY_MINUTES, Interval.ONE_HOUR] and time == TimePeriod.DAY:
        data.index = data.index.strftime('%I:%M %p')
    elif interval in [Interval.FIFTEEN_MINUTES, Interval.THIRTY_MINUTES, Interval.ONE_HOUR]:
        data.index = data.index.strftime('%Y-%m-%d %I:%M %p')
    else:
        data.index = data.index.date.astype(str)

    # Convert the DataFrame to a dictionary
    data_dict = data.to_dict(orient='index')
    data_dict = dict(reversed(list(data_dict.items())))

    for date, data in data_dict.items():
        data_dict[date] = HistoricalData(
            open=Decimal(data['Open']),
            high=Decimal(data['High']),
            low=Decimal(data['Low']),
            adj_close=Decimal(data['Close']),
            volume=int(data['Volume'])
        )

    # Return a TimeSeries object
    return TimeSeries(history=data_dict)
