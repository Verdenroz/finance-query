from utils.dependencies import FinanceClient

from src.models.index import Index, MarketIndex
from src.services.quotes.get_quotes import get_quotes


async def fetch_index(
    finance_client: FinanceClient,
    index: Index,
) -> MarketIndex:
    """
    Try the rich quote-summary endpoint first; on any failure fall back to
    the lightweight batch-quote endpoint so we still return the core data.

    :param finance_client: The Yahoo Finance client for API requests
    :param index: The index to fetch data for
    :return: MarketIndex object with the index data
    """
    symbol = _get_yahoo_index_symbol(index)

    # Try the full quote-summary
    quote_data = await finance_client.get_quote(symbol)
    if quote_data:
        return await _parse_yahoo_index(quote_data, index)

    # If that fails, try the simple quotes endpoint
    batch = await finance_client.get_simple_quotes([symbol])
    result = batch.get("quoteResponse", {}).get("result", [])
    if result:
        return await _parse_yahoo_index(result[0], index)

    # As last resort, try get_quotes which might use scraping
    quotes_data = await get_quotes(finance_client, [symbol])
    if quotes_data:
        quote = quotes_data[0]
        return MarketIndex(
            name=quote.name or index.value,
            value=float(quote.price),
            change=quote.change,
            percent_change=quote.change_percent,
            five_days_return=quote.five_days_return,
            one_month_return=quote.one_month_return,
            three_month_return=quote.three_month_return,
            six_month_return=quote.six_month_return,
            ytd_return=quote.ytd_return,
            year_return=quote.year_return,
            three_year_return=quote.three_year_return,
            five_year_return=quote.five_year_return,
            ten_year_return=quote.ten_year_return,
            max_return=quote.max_return,
        )

    # If all else fails, create a minimal MarketIndex with just the name
    return MarketIndex(name=index.value, value=0.0, change="", percent_change="")


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
        value=round(price_data["regularMarketPrice"]["raw"], 2) if "regularMarketPrice" in price_data and "raw" in price_data["regularMarketPrice"] else None,
        change=price_data.get("regularMarketChange", {}).get("fmt"),
        percent_change=price_data.get("regularMarketChangePercent", {}).get("fmt"),
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
