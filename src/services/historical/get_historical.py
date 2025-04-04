import pandas as pd
from fastapi import HTTPException
from orjson import orjson

from src.cache import cache
from src.dependencies import fetch
from src.models import HistoricalData, TimeRange, Interval


@cache(expire=60, market_closed_expire=600)
async def get_historical(
        symbol: str,
        time_range: TimeRange,
        interval: Interval,
        epoch: bool = False
) -> dict[str, HistoricalData]:
    """
    Get historical data for a stock symbol based on the time period and interval provided.
    :param symbol: the symbol of the stock to get historical data for
    :param time_range: the time range for the historical data (e.g. 1d, 5d, 7d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max)
    :param interval: the time interval between data points (e.g. 1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo)
    :param epoch: whether to return timestamps as epoch integers or formatted date strings

    :raises HTTPException: with status code 400 if the combination of time period and interval is invalid,
    404 if the symbol cannot be found, or 500 for any other error
    """
    # Validate the combination of time period and interval
    valid_ranges = {
        Interval.ONE_MINUTE: [TimeRange.DAY, TimeRange.FIVE_DAYS],
        Interval.FIVE_MINUTES: [TimeRange.DAY, TimeRange.FIVE_DAYS, TimeRange.ONE_MONTH],
        Interval.FIFTEEN_MINUTES: [TimeRange.DAY, TimeRange.FIVE_DAYS, TimeRange.ONE_MONTH],
        Interval.THIRTY_MINUTES: [TimeRange.DAY, TimeRange.FIVE_DAYS, TimeRange.ONE_MONTH],
        Interval.ONE_HOUR: [TimeRange.DAY, TimeRange.FIVE_DAYS, TimeRange.ONE_MONTH, TimeRange.THREE_MONTHS,
                            TimeRange.SIX_MONTHS, TimeRange.YTD, TimeRange.YEAR],
    }

    if time_range == TimeRange.MAX and interval != Interval.MONTHLY:
        raise HTTPException(status_code=400, detail="If range is max, interval must be 1mo")

    if interval in valid_ranges and time_range not in valid_ranges[interval]:
        raise HTTPException(status_code=400,
                            detail=f"If interval is {interval.value}, range must be {', '.join([r.value for r in valid_ranges[interval]])}")

    base_url = f"https://query1.finance.yahoo.com/v8/finance/chart/{symbol}"

    # Setup request parameters
    params = {
        'interval': interval.value,
        'range': time_range.value,
        'includePrePost': 'false'
    }

    # Construct URL with parameters
    url = f"{base_url}?{'&'.join(f'{k}={v}' for k, v in params.items())}"

    try:
        response_text = await fetch(url=url)
        data = orjson.loads(response_text)

        # Check for error response from Yahoo Finance
        if 'chart' in data:
            if data['chart'].get('error'):
                error = data['chart']['error']
                if error['code'] == 'Not Found':
                    raise HTTPException(status_code=404, detail=error['description'])
                else:
                    raise HTTPException(status_code=500, detail=f"Yahoo Finance API error: {error['description']}")

            if not data['chart'].get('result') or not data['chart']['result'][0]:
                raise HTTPException(status_code=404, detail="No data returned for symbol")
        else:
            raise HTTPException(status_code=500, detail="Invalid response structure from Yahoo Finance API")

        chart_data = data['chart']['result'][0]

        # Extract timestamp and price data
        timestamps = pd.to_datetime(chart_data['timestamp'], unit='s')
        quote = chart_data['indicators']['quote'][0]

        df = pd.DataFrame({
            'open': quote['open'],
            'high': quote['high'],
            'low': quote['low'],
            'close': quote['close'],
            'volume': quote['volume']
        }, index=timestamps)

        # Add adjusted close if available
        if 'adjclose' in chart_data['indicators']:
            df['adjclose'] = chart_data['indicators']['adjclose'][0]['adjclose']

        # Clean missing data
        df.dropna(inplace=True)

        # Sort and determine date format based on interval type
        df.sort_index(ascending=False, inplace=True)
        date_format = '%Y-%m-%d %H:%M:%S' if interval in [
            Interval.ONE_MINUTE,
            Interval.FIVE_MINUTES,
            Interval.FIFTEEN_MINUTES,
            Interval.THIRTY_MINUTES,
            Interval.ONE_HOUR
        ] else '%Y-%m-%d'

        # Convert to TimeSeries format
        history_dict = {}
        for timestamp, row in df.iterrows():
            # Use either formatted date string or epoch timestamp as key
            date_key = timestamp.strftime(date_format) if not epoch else int(timestamp.timestamp())

            history_dict[str(date_key)] = HistoricalData(
                open=round(float(row['open']), 2),
                high=round(float(row['high']), 2),
                low=round(float(row['low']), 2),
                close=round(float(row['close']), 2),
                volume=int(row['volume']),
                adj_close=round(float(row['adjclose']), 2) if 'adjclose' in df.columns else None
            )

        return history_dict

    except orjson.JSONDecodeError:
        raise HTTPException(status_code=500, detail="Invalid response from Yahoo Finance API")
    except Exception as e:
        if "404" in str(e):
            raise HTTPException(status_code=404, detail="Symbol not found")
        raise HTTPException(status_code=500, detail=f"Failed to retrieve historical data: {str(e)}")
