import pandas as pd
from yahooquery import Ticker

from decimal import Decimal
from src.schemas import HistoricalData, TimeSeries
from src.schemas.time_series import Interval, TimePeriod


async def get_historical(symbol: str, time: TimePeriod, interval: Interval):
    try:
        stock = Ticker(symbol, asynchronous=True, retry=3, status_forcelist=[404, 429, 500, 502, 503, 504])
        data = stock.history(period=time.value, interval=interval.value, adj_ohlc=True)
        data = data.sort_index(ascending=False)

        if not isinstance(data.index.get_level_values('date'), pd.DatetimeIndex):
            data.index = data.index.set_levels(pd.to_datetime(data.index.get_level_values('date'), utc=True),
                                               level='date')

        # Convert the 'date' level of the DataFrame's index to string
        if interval in [Interval.FIFTEEN_MINUTES, Interval.THIRTY_MINUTES,
                        Interval.ONE_HOUR] and time == TimePeriod.DAY:
            data.index = data.index.set_levels(data.index.get_level_values('date').strftime('%I:%M %p'), level='date')
        elif interval in [Interval.FIFTEEN_MINUTES, Interval.THIRTY_MINUTES, Interval.ONE_HOUR]:
            data.index = data.index.set_levels(data.index.get_level_values('date').strftime('%Y-%m-%d %I:%M %p'),
                                               level='date')
        else:
            data.index = data.index.set_levels(data.index.get_level_values('date').date.astype(str), level='date')

        # Convert the DataFrame to a dictionary
        data_dict = {}
        for date, row in data.iterrows():
            if isinstance(date[1], pd.Timestamp):
                date_str = date[1].strftime('%Y-%m-%d %H:%M:%S')
            else:
                date_str = date[1]
            data_dict[date_str] = HistoricalData(
                open=round(Decimal(row['open']), 2),
                high=round(Decimal(row['high']), 2),
                low=round(Decimal(row['low']), 2),
                adj_close=round(Decimal(row['close']), 2),
                volume=int(row['volume'])
            )
        return TimeSeries(history=data_dict)

    except RetryError as e:
        if '404' in str(e):
            raise HTTPException(status_code=404, detail="Stock not found")
        else:
            raise HTTPException(status_code=500, detail="Internal server error")

    return TimeSeries(history=data_dict)
