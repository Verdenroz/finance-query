from orjson import orjson

from src.dependencies import fetch
from src.models.index import Index, MarketIndex


async def fetch_index(index: Index, cookies: str, crumb: str) -> MarketIndex | None:
    """
    Fetches the index data from the Yahoo Finance and returns a MarketIndex object or None if an error occurs.
    :param index: the index to retrieve data for
    :param cookies: the cookies required for Yahoo Finance API
    :param crumb: the crumb required for Yahoo Finance API
    """
    if not cookies or not crumb:
        return None

    try:
        summary_data = await _fetch_yahoo_index(index, cookies, crumb)
        if not summary_data:
            return None

        return await _parse_yahoo_index(summary_data, index)
    except Exception as e:
        print(f"Error processing {index.name}: {str(e)}")
        return None


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
        Index.GDAXI: "DAX Performance Index",
        Index.STOXX50E: "EURO STOXX 50",
        Index.NZ50: "S&P/NZX 50 Index",
        Index.SET: "Thailand SET Index",
        Index.JN0U_JO: "FTSE JSE Top 40- USD Net TRI",
        Index.SA40: "South Africa Top 40",
    }

    return formatted_names.get(index, default_name)


async def _fetch_yahoo_index(index: Index, cookies: str, crumb: str) -> dict | None:
    """Fetch raw index data from Yahoo Finance API using cookies and crumb."""
    try:
        symbol = _get_yahoo_index_symbol(index)
        summary_url = f"https://query2.finance.yahoo.com/v10/finance/quoteSummary/{symbol}"

        summary_params = {"modules": "price,quoteUnadjustedPerformanceOverview", "crumb": crumb}
        headers = {
            "Cookie": cookies,
            "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
            "Accept": "application/json",
        }

        summary_response = await fetch(url=summary_url, params=summary_params, headers=headers, return_response=True)
        response_text = await summary_response.text()
        summary_data = orjson.loads(response_text)
        return summary_data

    except Exception as e:
        print(f"Error fetching {index.name}: {str(e)}")
        return None


async def _parse_yahoo_index(summary_data: dict, index: Index) -> MarketIndex:
    """Parse Yahoo Finance API response into MarketIndex object."""
    summary_result = summary_data.get("quoteSummary", {}).get("result", [{}])[0]
    price_data = summary_result.get("price", {})
    performance_data = summary_result.get("quoteUnadjustedPerformanceOverview", {}).get("performanceOverview", {})

    # Cleans unformatted names for some indices
    default_name = price_data.get("longName") or price_data.get("shortName") or index.value
    formatted_name = _get_formatted_index_name(index, default_name)

    # Helper function to format return values with plus sign for positives
    def format_return(value):
        if not value:
            return None
        if isinstance(value, dict):
            fmt = value.get("fmt")
            if fmt and not fmt.startswith("-") and not fmt == "0.00%":
                return f"+{fmt}"
            return fmt
        return value

    return MarketIndex(
        name=formatted_name,
        value=round(price_data["regularMarketPrice"]["raw"], 2),
        change=price_data["regularMarketChange"]["fmt"],
        percent_change=price_data["regularMarketChangePercent"]["fmt"],
        five_days_return=format_return(performance_data.get("fiveDaysReturn")),
        one_month_return=format_return(performance_data.get("oneMonthReturn")),
        three_month_return=format_return(performance_data.get("threeMonthReturn")),
        six_month_return=format_return(performance_data.get("sixMonthReturn")),
        ytd_return=format_return(performance_data.get("ytdReturnPct")),
        year_return=format_return(performance_data.get("oneYearTotalReturn")),
        three_year_return=format_return(performance_data.get("threeYearTotalReturn")),
        five_year_return=format_return(performance_data.get("fiveYearTotalReturn")),
        ten_year_return=format_return(performance_data.get("tenYearTotalReturn")),
        max_return=format_return(performance_data.get("maxReturn")),
    )
