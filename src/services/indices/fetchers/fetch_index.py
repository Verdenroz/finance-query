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


def _get_yahoo_index_symbol(index: Index) -> str:
    """Get the Yahoo Finance symbol for a given index"""
    # Special case mapping from Index enum to actual Yahoo Finance symbols
    special_cases = {
        Index.MOEX_ME: "MOEX.ME",
        Index.DX_Y_NYB: "DX-Y.NYB",
        Index.USD_STRD: "^125904-USD-STRD",
        Index.MSCI_WORLD: "^990100-USD-STRD",
        Index.SHANGHAI: "000001.SS",
        Index.SZSE: "399001.SZ",
        Index.PSI: "PSI20.LS",
        Index.BUX: "^BUX.BD",
        Index.BIST100: "XU100.IS",
        Index.TA35: "TA35.TA",
        Index.TASI: "^TASI.SR",
        Index.SET: "^SET.BK",
        Index.PSEI: "PSEI.PS",
        Index.IMOEX: "IMOEX.ME",
        Index.RTSI: "RTSI.ME",
        Index.CHINA_A50: "XIN9.FGI",
        Index.WIG20: "WIG20.WA",
        Index.FTSEMIB: "FTSEMIB.MI",
        Index.FTSEJSE: "^J580.JO",
        Index.AFR40: "^JA0R.JO",
        Index.SA40: "^J200.JO",
        Index.RAF40: "^J260.JO",
        Index.ALT15: "^J233.JO",
        Index.TAMAYUZ: "^TAMAYUZ.CA",
        Index.IVBX: "^IVBX",
        Index.IBRX_50: "^IBX50",
    }

    # Return special case or default format with caret prefix
    return special_cases.get(index, f"^{index.name.replace('_', '.')}")


def _get_formatted_index_name(index: Index, default_name: str) -> str:
    """
    Get the properly formatted name for an index if it is not already formatted.
    """
    formatted_names = {
        Index.GDAXI: 'DAX Performance Index',
        Index.STOXX50E: 'EURO STOXX 50',
        Index.NZ50: 'S&P/NZX 50 Index',
        Index.SET: 'Thailand SET Index',
        Index.JN0U_JO: 'FTSE JSE Top 40- USD Net TRI',
        Index.SA40: 'South Africa Top 40'
    }

    return formatted_names.get(index, default_name)


async def _fetch_yahoo_index(index: Index, cookies: str, crumb: str) -> dict:
    """ Fetch raw index data from Yahoo Finance API using cookies and crumb. """
    symbol = _get_yahoo_index_symbol(index)
    summary_url = f"https://query2.finance.yahoo.com/v10/finance/quoteSummary/{symbol}"

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
    default_name = price_data.get('longName') or price_data.get('shortName') or index.value
    formatted_name = _get_formatted_index_name(index, default_name)

    return MarketIndex(
        name=formatted_name,
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
