from datetime import datetime, time
from decimal import Decimal

import pandas as pd
from async_lru import alru_cache
from fastapi import HTTPException
from requests.exceptions import RetryError
from stock_indicators.indicators.common.quote import Quote
from typing_extensions import List
from yahooquery import Ticker

from src.redis import cache
from src.schemas import HistoricalData, TimeSeries
from src.schemas.time_series import Interval, TimePeriod


@cache(expire=60, market_closed_expire=600)
async def get_historical(symbol: str, time: TimePeriod, interval: Interval) -> TimeSeries:
    try:
        stock = Ticker(symbol, asynchronous=True, retry=3, status_forcelist=[404, 429, 500, 502, 503, 504])
        data = stock.history(period=time.value, interval=interval.value)

        if interval in [Interval.ONE_MINUTE, Interval.FIVE_MINUTES, Interval.FIFTEEN_MINUTES,
                        Interval.THIRTY_MINUTES, Interval.ONE_HOUR]:
            # Reset the index
            data.reset_index(inplace=True)

            # Convert the 'date' column to datetime
            data['date'] = pd.to_datetime(data['date'])

            # Sort the DataFrame by the 'date' column in ascending order
            data.sort_values(by='date', ascending=False, inplace=True)

            # Set the index back to ['symbol', 'date']
            data.set_index(['symbol', 'date'], inplace=True)
        else:
            data = data.sort_index(ascending=False)
            if not isinstance(data.index.get_level_values('date'), pd.DatetimeIndex):
                data.index = data.index.set_levels(pd.to_datetime(data.index.get_level_values('date'), utc=True),
                                                   level='date')
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
                close=round(Decimal(row['close']), 2),
                adj_close=round(Decimal(row['adjclose']), 2) if 'adjclose' in data.columns else None,
                volume=int(row['volume'])
            )
        return TimeSeries(history=data_dict)

    except RetryError as e:
        if '404' in str(e):
            raise HTTPException(status_code=404, detail="Stock not found")
        else:
            raise HTTPException(status_code=500, detail="Internal server error")


@alru_cache(maxsize=512, ttl=60)
async def get_historical_quotes(symbol: str, timePeriod: TimePeriod, interval: Interval) -> List[Quote]:
    try:
        stock = Ticker(symbol, asynchronous=True, retry=3, status_forcelist=[404, 429, 500, 502, 503, 504])
        data = stock.history(period=timePeriod.value, interval=interval.value)
        data = data.sort_index(ascending=False)
        quotes = []
        for _, row in data.iterrows():
            if row.name[1] is not None:
                date = row.name[1]
                if not isinstance(date, datetime):
                    date = datetime.combine(date, time())  # Convert date to datetime.datetime
                if date is not None:
                    quotes.append(
                        Quote(date=date, open=row['open'], high=row['high'], low=row['low'], close=row['close'],
                              volume=row['volume'])
                    )
        return quotes

    except RetryError as e:
        if '404' in str(e):
            raise HTTPException(status_code=404, detail="Stock not found")
        else:
            raise HTTPException(status_code=500, detail="Internal server error")
