import asyncio

from fastapi import HTTPException

from src.models import Quote, SimpleQuote
from src.services.quotes.utils import (
    _scrape_company_info,
    _scrape_general_info,
    _scrape_performance,
    _scrape_price_data,
    get_adaptive_chunk_size,
    parse_tree,
    thread_pool,
)
from src.utils.cache import cache
from src.utils.dependencies import fetch, get_logo


async def scrape_quotes(symbols: list[str]):
    """
    Asynchronously scrapes multiple quotes from a list of symbols and returns a list of Quote objects.
    :param symbols: the list of symbols to scrape

    :raises HTTPException: with code 500 if scraping fails
    """
    chunk_size = get_adaptive_chunk_size()
    chunks = [symbols[i : i + chunk_size] for i in range(0, len(symbols), chunk_size)]

    all_quotes = await asyncio.gather(*(asyncio.gather(*(_scrape_quote(symbol) for symbol in chunk)) for chunk in chunks))

    return [quote for quotes in all_quotes for quote in quotes if not isinstance(quote, Exception)]


async def scrape_simple_quotes(symbols: list[str]):
    """
    Asynchronously scrapes multiple simple quotes from a list of symbols and returns a list of SimpleQuote objects.
    :param symbols: the list of symbols to scrape
    """
    chunk_size = get_adaptive_chunk_size()
    chunks = [symbols[i : i + chunk_size] for i in range(0, len(symbols), chunk_size)]

    all_quotes = await asyncio.gather(*(asyncio.gather(*(_scrape_simple_quote(symbol) for symbol in chunk)) for chunk in chunks))

    return [quote for quotes in all_quotes for quote in quotes if not isinstance(quote, Exception)]


@cache(expire=10, market_closed_expire=60)
async def _scrape_quote(symbol: str) -> Quote:
    """
    Asynchronously scrapes a quote from a given symbol and returns a Quote object.
    :param symbol: Quote symbol
    """
    try:
        url = f"https://finance.yahoo.com/quote/{symbol}/"
        html_content = await fetch(url=url)

        # Parse HTML in a separate thread
        loop = asyncio.get_event_loop()
        tree = await loop.run_in_executor(thread_pool, parse_tree, html_content)

        # Get company name
        name_elements = tree.xpath(".//h1/text()")
        name = name_elements[1].split("(")[0].strip()

        # Execute all scraping tasks in parallel
        prices_task = asyncio.create_task(_scrape_price_data(tree))
        general_info_task = asyncio.create_task(_scrape_general_info(tree))
        company_info_task = asyncio.create_task(_scrape_company_info(tree, symbol))
        performance_task = asyncio.create_task(_scrape_performance(tree))

        prices, general_info, company_info, performance = await asyncio.gather(prices_task, general_info_task, company_info_task, performance_task)

        regular_price, regular_change, regular_percent_change, pre_price, post_price = prices

        return Quote(
            symbol=symbol.upper(),
            name=name,
            price=regular_price,
            pre_market_price=pre_price,
            after_hours_price=post_price,
            change=regular_change,
            percent_change=regular_percent_change,
            **general_info,
            **company_info,
            **performance,
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error scraping quote for {symbol}: {e}") from e


@cache(expire=10, market_closed_expire=60)
async def _scrape_simple_quote(symbol: str) -> SimpleQuote:
    """
    Asynchronously scrapes a simple quote from a given symbol and returns a SimpleQuote object.
    :param symbol: Quote symbol
    """
    try:
        url = f"https://finance.yahoo.com/quote/{symbol}/"
        html_content = await fetch(url=url)

        # Parse HTML in a separate thread
        loop = asyncio.get_event_loop()
        tree = await loop.run_in_executor(thread_pool, parse_tree, html_content)

        # Get company name
        name_elements = tree.xpath(".//h1/text()")
        name = name_elements[1].split("(")[0].strip()

        # Execute price scraping and logo fetching concurrently
        prices_task = asyncio.create_task(_scrape_price_data(tree))

        # Get logo asynchronously - extract website first, then fetch logo
        website_elements = tree.xpath("/html/body/div[2]/main/section/section/section/article/section[2]/div/div/div[2]/div/div[1]/div[1]/a/@href")
        website = website_elements[0].strip() if website_elements else None

        async def get_logo_or_none():
            return await get_logo(symbol=symbol, url=website) if website else None

        logo_task = asyncio.create_task(get_logo_or_none())

        prices, logo = await asyncio.gather(prices_task, logo_task)
        regular_price, regular_change, regular_percent_change, pre_price, post_price = prices

        return SimpleQuote(
            symbol=symbol.upper(),
            name=name,
            price=regular_price,
            pre_market_price=pre_price,
            after_hours_price=post_price,
            change=regular_change,
            percent_change=regular_percent_change,
            logo=logo,
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error scraping simple quote for {symbol}: {e}") from e
