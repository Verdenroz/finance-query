import asyncio

from fastapi import HTTPException
from orjson import orjson

from src.dependencies import fetch, get_logo
from src.models import Quote, SimpleQuote
from src.services.quotes.utils import (
    format_change,
    format_date,
    format_percent,
    get_adaptive_chunk_size,
    get_fmt,
    get_morningstar_risk_rating,
    get_raw,
    is_within_post_market_time,
    is_within_pre_market_time,
)


async def fetch_quotes(symbols: list[str], cookies: str, crumb: str) -> list[Quote]:
    """Fetch quotes using Yahoo Finance API"""
    if not cookies or not crumb:
        raise ValueError("Cookies and crumb are required for Yahoo Finance API")

    chunk_size = get_adaptive_chunk_size()
    chunks = [symbols[i : i + chunk_size] for i in range(0, len(symbols), chunk_size)]

    all_quotes = await asyncio.gather(*(asyncio.gather(*(_get_quote_from_yahoo(symbol, cookies, crumb) for symbol in chunk)) for chunk in chunks))

    return [quote for quotes in all_quotes for quote in quotes if not isinstance(quote, Exception)]


async def fetch_simple_quotes(symbols: list[str], cookies: str, crumb: str) -> list[SimpleQuote]:
    """Fetch quotes using Yahoo Finance API"""
    if not cookies or not crumb:
        raise ValueError("Cookies and crumb are required for Yahoo Finance API")

    chunk_size = get_adaptive_chunk_size()
    chunks = [symbols[i : i + chunk_size] for i in range(0, len(symbols), chunk_size)]

    all_quotes = await asyncio.gather(*(asyncio.gather(*(_get_simple_quote_from_yahoo(symbol, cookies, crumb) for symbol in chunk)) for chunk in chunks))

    return [quote for quotes in all_quotes for quote in quotes if not isinstance(quote, Exception)]


async def _get_quote_from_yahoo(symbol: str, cookies: str, crumb: str) -> Quote:
    """Get individual quote data from Yahoo Finance API."""
    summary_data = await _fetch_yahoo_data(symbol, cookies, crumb)
    return await _parse_yahoo_quote_data(summary_data)


async def _get_simple_quote_from_yahoo(symbol: str, cookies: str, crumb: str) -> SimpleQuote:
    """Get individual simplified quote data from Yahoo Finance API."""
    summary_data = await _fetch_yahoo_data(symbol, cookies, crumb)
    return await _parse_yahoo_simple_quote_data(summary_data)


async def _fetch_yahoo_data(symbol: str, cookies: str, crumb: str) -> dict:
    """
    Fetch raw data from Yahoo Finance API using cookies and crumb.

    :raises HTTPException: with code 404 if symbol is not found
    """
    summary_url = f"https://query2.finance.yahoo.com/v10/finance/quoteSummary/{symbol}"
    summary_params = {
        "modules": "assetProfile,price,summaryDetail,defaultKeyStatistics,calendarEvents,quoteUnadjustedPerformanceOverview",
        "crumb": crumb,
    }
    headers = {
        "Cookie": cookies,
        "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
        "Accept": "application/json",
    }

    summary_response = await fetch(url=summary_url, params=summary_params, headers=headers, return_response=True)

    if summary_response.status == 404:
        raise HTTPException(status_code=404, detail=f"Symbol not found: {symbol}")

    response_text = await summary_response.text()
    summary_data = orjson.loads(response_text)
    return summary_data


async def _parse_yahoo_quote_data(summary_data: dict) -> Quote:
    """Parse Yahoo Finance API response into Quote object."""
    summary_result = summary_data.get("quoteSummary", {}).get("result", [{}])[0]
    price_data = summary_result.get("price", {})
    summary_detail = summary_result.get("summaryDetail", {})
    stats = summary_result.get("defaultKeyStatistics", {})
    profile = summary_result.get("assetProfile", {})
    calendar = summary_result.get("calendarEvents", {})
    performance_overview = summary_result.get("quoteUnadjustedPerformanceOverview", {}).get("performanceOverview", {})

    # Get pre- and post-market prices if within timeframe
    pre_market_price = get_fmt(price_data, "preMarketPrice") if is_within_pre_market_time(price_data.get("preMarketTime", 0)) else None
    post_market_price = get_fmt(price_data, "postMarketPrice") if is_within_post_market_time(price_data.get("postMarketTime", 0)) else None

    # Parse earnings dates
    earnings_dates = calendar.get("earnings", {}).get("earningsDate", [])
    earnings_date = None
    if earnings_dates:
        formatted_dates = [format_date(date.get("fmt")) for date in earnings_dates if date.get("fmt")]
        earnings_date = " - ".join(formatted_dates) if formatted_dates else None

    quote_data = {
        "symbol": price_data.get("symbol"),
        "name": price_data.get("longName"),
        "price": get_fmt(price_data, "regularMarketPrice"),
        "pre_market_price": pre_market_price,
        "after_hours_price": post_market_price,
        "change": format_change(get_fmt(price_data, "regularMarketChange")),
        "percent_change": format_change(format_percent(price_data.get("regularMarketChangePercent"))),
        "open": get_fmt(summary_detail, "open"),
        "high": get_fmt(summary_detail, "dayHigh"),
        "low": get_fmt(summary_detail, "dayLow"),
        "year_high": get_fmt(summary_detail, "fiftyTwoWeekHigh"),
        "year_low": get_fmt(summary_detail, "fiftyTwoWeekLow"),
        "volume": get_raw(summary_detail, "volume"),
        "avg_volume": get_raw(summary_detail, "averageVolume"),
        "market_cap": get_fmt(summary_detail, "marketCap"),
        "beta": get_fmt(summary_detail, "beta"),
        "pe": get_fmt(summary_detail, "trailingPE"),
        "eps": get_fmt(summary_detail, "trailingEps"),
        "dividend": get_fmt(summary_detail, "dividendRate"),
        "dividend_yield": get_fmt(summary_detail, "dividendYield"),
        "ex_dividend": format_date(calendar.get("exDividendDate", {}).get("fmt")),
        "net_assets": get_fmt(summary_detail, "totalAssets"),
        "nav": get_fmt(summary_detail, "navPrice"),
        "expense_ratio": format_percent(stats.get("annualReportExpenseRatio")),
        "category": stats.get("category"),
        "last_capital_gain": get_fmt(stats, "lastCapGain"),
        "morningstar_rating": f"★{'★' * (stats.get('morningStarOverallRating', {}).get('raw', 0) - 1)}" if stats.get("morningStarOverallRating") else None,
        "morningstar_risk_rating": get_morningstar_risk_rating(stats.get("morningStarRiskRating", {}).get("raw", -1)),
        "holdings_turnover": format_percent(stats.get("annualHoldingsTurnover")),
        "earnings_date": earnings_date,
        "last_dividend": get_fmt(stats, "lastDividendValue"),
        "inception_date": format_date(stats.get("fundInceptionDate", {}).get("raw")),
        "sector": profile.get("sector"),
        "industry": profile.get("industry"),
        "about": profile.get("longBusinessSummary"),
        "employees": str(profile.get("fullTimeEmployees")) if profile.get("fullTimeEmployees") is not None else None,
        "five_days_return": performance_overview.get("fiveDaysReturn", {}).get("fmt"),
        "one_month_return": performance_overview.get("oneMonthReturn", {}).get("fmt"),
        "three_month_return": performance_overview.get("threeMonthReturn", {}).get("fmt"),
        "six_month_return": performance_overview.get("sixMonthReturn", {}).get("fmt"),
        "ytd_return": performance_overview.get("ytdReturnPct", {}).get("fmt"),
        "year_return": performance_overview.get("oneYearTotalReturn", {}).get("fmt"),
        "three_year_return": performance_overview.get("threeYearTotalReturn", {}).get("fmt"),
        "five_year_return": performance_overview.get("fiveYearTotalReturn", {}).get("fmt"),
        "ten_year_return": performance_overview.get("tenYearTotalReturn", {}).get("fmt"),
        "max_return": performance_overview.get("maxReturn", {}).get("fmt"),
    }

    return Quote(**quote_data, logo=await get_logo(symbol=price_data.get("symbol"), url=profile.get("website")))


async def _parse_yahoo_simple_quote_data(summary_data: dict) -> SimpleQuote:
    """Parse Yahoo Finance API response into SimpleQuote object."""
    summary_result = summary_data.get("quoteSummary", {}).get("result", [{}])[0]
    price_data = summary_result.get("price", {})
    profile = summary_result.get("assetProfile", {})

    # Get pre- and post-market prices if within timeframe
    pre_market_price = get_fmt(price_data, "preMarketPrice") if is_within_pre_market_time(price_data.get("preMarketTime", 0)) else None
    post_market_price = get_fmt(price_data, "postMarketPrice") if is_within_post_market_time(price_data.get("postMarketTime", 0)) else None

    quote_data = {
        "symbol": price_data.get("symbol"),
        "name": price_data.get("longName"),
        "price": get_fmt(price_data, "regularMarketPrice"),
        "pre_market_price": pre_market_price,
        "after_hours_price": post_market_price,
        "change": format_change(get_fmt(price_data, "regularMarketChange")),
        "percent_change": format_change(format_percent(price_data.get("regularMarketChangePercent"))),
    }
    return SimpleQuote(**quote_data, logo=await get_logo(symbol=price_data.get("symbol"), url=profile.get("website")))