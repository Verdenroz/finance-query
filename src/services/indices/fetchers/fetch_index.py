from fastapi import HTTPException
from orjson import orjson

from src.dependencies import fetch
from src.models.index import Index, MarketIndex


async def fetch_index(index: Index, cookies: str, crumb: str) -> MarketIndex:
    """
    Fetches the index data from the Yahoo Finance and returns a MarketIndex object.
    :param index: the index to retrieve data for
    :param cookies: the cookies required for Yahoo Finance API
    :param crumb: the crumb required for Yahoo Finance API
    """
    if not cookies or not crumb:
        raise ValueError("Cookies and crumb are required for Yahoo Finance API")

    summary_data = await _fetch_yahoo_index(index, cookies, crumb)
    return await _parse_yahoo_index(summary_data, index)


async def _fetch_yahoo_index(index: Index, cookies: str, crumb: str) -> dict:
    """ Fetch raw index data from Yahoo Finance API using cookies and crumb. """
    if index == Index.MOEX_ME:
        summary_url = f"https://query2.finance.yahoo.com/v10/finance/quoteSummary/MOEX.ME"
    elif index == Index.DX_Y_NYB:
        summary_url = f"https://query2.finance.yahoo.com/v10/finance/quoteSummary/DX-Y.NYB"
    elif index == Index.USD_STRD:
        summary_url = f"https://query2.finance.yahoo.com/v10/finance/quoteSummary/^125904-USD-STRD"
    elif index == Index.SS:
        summary_url = f"https://query2.finance.yahoo.com/v10/finance/quoteSummary/000001.SS"
    else:
        summary_url = f"https://query2.finance.yahoo.com/v10/finance/quoteSummary/^{index.name.replace('_', '.')}"

    summary_params = {
        "modules": "price,quoteUnadjustedPerformanceOverview",
        "crumb": crumb
    }
    headers = {
        'Cookie': cookies,
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
        'Accept': 'application/json'
    }

    summary_response = await fetch(
        url=summary_url,
        params=summary_params,
        headers=headers,
        return_response=True
    )

    if summary_response.status != 200:
        response_text = await summary_response.text()
        response_data = orjson.loads(response_text)
        error_description = response_data.get('quoteSummary', {}).get('error', {}).get('description')
        raise HTTPException(
            status_code=summary_response.status,
            detail=error_description or f"Failed to fetch index data for {index}"
        )

    response_text = await summary_response.text()
    summary_data = orjson.loads(response_text)
    return summary_data


async def _parse_yahoo_index(summary_data: dict, index: Index) -> MarketIndex:
    """ Parse Yahoo Finance API response into MarketIndex object. """
    summary_result = summary_data.get('quoteSummary', {}).get('result', [{}])[0]
    price_data = summary_result.get('price', {})
    performance_data = summary_result.get('quoteUnadjustedPerformanceOverview', {}).get('performanceOverview', {})

    # Cleans unformatted names for some indices
    if index is index.GDAXI:
        price_data['longName'] = 'DAX Performance Index'
    elif index is index.STOXX50E:
        price_data['longName'] = 'EURO STOXX 50'
    elif index is index.NZ50:
        price_data['longName'] = 'S&P/NZX 50 Index'

    return MarketIndex(
        name=price_data.get('longName') or price_data.get('shortName') or index.value,
        value=round(price_data['regularMarketPrice']['raw'], 2),
        change=price_data['regularMarketChange']['fmt'],
        percent_change=price_data['regularMarketChangePercent']['fmt'],
        five_days_return=performance_data.get('fiveDaysReturn', {}).get('fmt'),
        one_month_return=performance_data.get('oneMonthReturn', {}).get('fmt'),
        three_month_return=performance_data.get('threeMonthReturn', {}).get('fmt'),
        six_month_return=performance_data.get('sixMonthReturn', {}).get('fmt'),
        ytd_return=performance_data.get('ytdReturnPct', {}).get('fmt'),
        year_return=performance_data.get('oneYearTotalReturn', {}).get('fmt'),
        two_year_return=performance_data.get('twoYearTotalReturn', {}).get('fmt'),
        three_year_return=performance_data.get('threeYearTotalReturn', {}).get('fmt'),
        five_year_return=performance_data.get('fiveYearTotalReturn', {}).get('fmt'),
        ten_year_return=performance_data.get('tenYearTotalReturn', {}).get('fmt'),
        max_return=performance_data.get('maxReturn', {}).get('fmt')
    )
