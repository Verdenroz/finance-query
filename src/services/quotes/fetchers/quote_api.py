import asyncio
from typing import List

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
from utils.dependencies import get_logo, FinanceClient


async def fetch_quotes(
        finance_client: FinanceClient,
        symbols: List[str]
) -> List[Quote]:
    """Fetch quotes using Yahoo Finance API"""
    chunk_size = get_adaptive_chunk_size()
    chunks = [symbols[i:i + chunk_size] for i in range(0, len(symbols), chunk_size)]

    # Process each chunk in parallel
    all_quotes_tasks = []
    for chunk in chunks:
        # Process each symbol in the chunk in parallel
        tasks = [_get_quote_from_yahoo(finance_client, symbol) for symbol in chunk]
        all_quotes_tasks.append(asyncio.gather(*tasks))

    # Wait for all chunks to complete
    all_quotes = await asyncio.gather(*all_quotes_tasks)

    # Flatten the results
    return [quote for quotes in all_quotes for quote in quotes]


async def fetch_simple_quotes(
        finance_client: FinanceClient,
        symbols: List[str]
) -> List[SimpleQuote]:
    """Fetch simplified quotes for multiple symbols in batch."""
    # Get batch response for all symbols
    batch_data = await finance_client.get_simple_quotes(symbols)
    # Extract the quotes from the response
    quotes_data = batch_data.get("quoteResponse", {}).get("result", [])

    # Parse each quote in parallel
    parsing_tasks = [_parse_simple_quote(quote_data) for quote_data in quotes_data]
    quotes = await asyncio.gather(*parsing_tasks)

    return quotes


async def _get_quote_from_yahoo(finance_client: FinanceClient, symbol: str) -> Quote:
    """Get individual quote data from Yahoo Finance API."""
    summary_data = await finance_client.get_quote(symbol)
    return await _parse_yahoo_quote_data(summary_data)


async def _parse_simple_quote(quote_data: dict) -> SimpleQuote:
    pre_market_price = (
        get_fmt(quote_data, "preMarketPrice")
        if is_within_pre_market_time(quote_data.get("preMarketTime", 0))
        else None
    )
    post_market_price = (
        get_fmt(quote_data, "postMarketPrice")
        if is_within_post_market_time(quote_data.get("postMarketTime", 0))
        else None
    )

    def _to_str(val):
        return f"{val:.2f}" if isinstance(val, (int, float)) else val

    price = _to_str(get_fmt(quote_data, "regularMarketPrice"))
    pre_market_price = _to_str(pre_market_price)
    post_market_price = _to_str(post_market_price)

    # regular change
    raw_change = get_fmt(quote_data, "regularMarketChange")
    if isinstance(raw_change, (int, float)):
        raw_change = f"{raw_change:.2f}"

    percent_change = format_percent(quote_data.get("regularMarketChangePercent"))
    percent_change = format_change(percent_change)

    payload = {
        "symbol": quote_data.get("symbol"),
        "name": quote_data.get("longName") or quote_data.get("shortName"),
        "price": price,
        "pre_market_price": pre_market_price,
        "after_hours_price": post_market_price,
        "change": format_change(raw_change),
        "percent_change": percent_change,
    }

    logo = await get_logo(symbol=payload["symbol"])
    return SimpleQuote(**payload, logo=logo)


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

    return Quote(**quote_data, logo=await get_logo(symbol=price_data.get("symbol")))
