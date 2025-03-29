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

        # Validate response structure
        if 'chart' not in data:
            raise HTTPException(status_code=500, detail="Invalid response structure from Yahoo Finance API")

        chart = data['chart']

        # Check for API errors
        if chart.get('error'):
            error = chart['error']
            if error['code'] == 'Not Found':
                raise HTTPException(status_code=404, detail=error['description'])
            raise HTTPException(status_code=500, detail=f"Failed to retrieve historical data: {error['description']}")

        # Check for valid results
        if not chart.get('result') or not chart['result'][0]:
            raise HTTPException(status_code=404, detail="No data returned for symbol")

        # Process chart data
        chart_data = chart['result'][0]
        timestamps = pd.to_datetime(chart_data['timestamp'], unit='s')
        quote = chart_data['indicators']['quote'][0]

        # Create DataFrame
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

        # Clean and format data
        df.dropna(inplace=True)
        df.sort_index(ascending=False, inplace=True)

        # Determine date format based on interval
        is_intraday = interval in [
            Interval.ONE_MINUTE, Interval.FIVE_MINUTES,
            Interval.FIFTEEN_MINUTES, Interval.THIRTY_MINUTES,
            Interval.ONE_HOUR
        ]
        date_format = '%Y-%m-%d %H:%M:%S' if is_intraday else '%Y-%m-%d'

        # Convert to expected output format
        history_dict = {}
        for timestamp, row in df.iterrows():
            date_key = int(timestamp.timestamp()) if epoch else timestamp.strftime(date_format)

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
        raise HTTPException(status_code=500, detail="Invalid JSON response from Yahoo Finance API")
