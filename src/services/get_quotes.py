import asyncio
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime

import psutil
from fastapi import HTTPException
from lxml import etree, html
from yahooquery import Ticker

from src.cache import cache
from src.dependencies import fetch, get_logo
from src.schemas import Quote, SimpleQuote

# Calculate adaptive max_workers based on available CPU cores
thread_pool = ThreadPoolExecutor(max_workers=psutil.cpu_count(logical=True) * 2)


def get_adaptive_chunk_size() -> int:
    """
    Calculate an adaptive chunk size based on available CPU cores and memory.
    :return: The calculated chunk size as an integer
    """
    cpu_count = psutil.cpu_count()
    memory_info = psutil.virtual_memory()
    available_memory = memory_info.available

    base_chunk_size = 5

    # Adjust chunk size based on available CPU and memory
    chunk_size = base_chunk_size * cpu_count * (available_memory // (512 * 1024 * 1024))

    # Ensure chunk size is within reasonable limits
    chunk_size = max(base_chunk_size, min(chunk_size, 100))

    return chunk_size


async def scrape_quotes(symbols: list[str]) -> list[Quote]:
    """
    Asynchronously scrapes multiple quotes from a list of symbols and returns a list of Quote objects.
    Duplicate symbols should be removed before calling this function.
    Chunked scraping is used to avoid blocking the event loop.

    :param symbols: List of symbols
    """
    chunk_size = get_adaptive_chunk_size()
    chunks = [symbols[i:i + chunk_size] for i in range(0, len(symbols), chunk_size)]

    all_quotes = await asyncio.gather(*(
        asyncio.gather(*(_scrape_quote(symbol) for symbol in chunk)) for chunk in chunks
    ))

    return [quote for quotes in all_quotes for quote in quotes if not isinstance(quote, Exception)]


async def scrape_simple_quotes(symbols: list[str]) -> list[SimpleQuote]:
    """
    Asynchronously scrapes multiple simple quotes from a list of symbols and returns a list of SimpleQuote objects.
    Duplicate symbols should be removed before calling this function.
    Chunks the symbols into groups of 10 to avoid rate limiting.

    :param symbols: List of symbols
    """
    chunk_size = get_adaptive_chunk_size()
    chunks = [symbols[i:i + chunk_size] for i in range(0, len(symbols), chunk_size)]

    all_quotes = await asyncio.gather(*(
        asyncio.gather(*(_scrape_simple_quote(symbol) for symbol in chunk)) for chunk in chunks
    ))

    return [quote for quotes in all_quotes for quote in quotes if not isinstance(quote, Exception)]


def parse_tree(html_content: str) -> etree.ElementTree:
    """
    Parse HTML content in a separate thread to avoid blocking the event loop.
    """
    return html.fromstring(html_content)


async def get_company_name(tree: etree.ElementTree):
    name_path = './/h1/text()'
    name_container = tree.xpath(name_path)
    if not name_container:
        raise HTTPException(status_code=500, detail="Failed to extract company name")

    name_container = name_container[1]
    company_name = name_container.split('(')[0].strip()

    return company_name


async def _scrape_price_data(tree: etree.ElementTree) -> tuple:
    """
    Scrape the price data from the HTML content using XPath and format the data.

    :param tree: The parsed HTML tree
    :return: Regular price, change, percent change, and post price as a tuple
    """
    try:
        # XPath expressions
        price_xpath = "//span[@data-testid='qsp-price']/text()"
        change_xpath = "//span[@data-testid='qsp-price-change']/text()"
        percent_change_xpath = "//span[@data-testid='qsp-price-change-percent']/text()"
        post_price_xpath = "//fin-streamer[@data-testid='qsp-post-price']/@data-value"
        pre_price_xpath = "//fin-streamer[@data-testid='qsp-pre-price']/@data-value"

        # Extract values
        regular_price = tree.xpath(price_xpath)
        regular_change = tree.xpath(change_xpath)
        regular_percent_change = tree.xpath(percent_change_xpath)
        post_price = tree.xpath(post_price_xpath)
        pre_price = tree.xpath(pre_price_xpath)

        # Format values
        regular_price = regular_price[0].strip() if regular_price else None
        regular_change = regular_change[0].strip() if regular_change else None
        regular_percent_change = regular_percent_change[0].strip().replace('(', '').replace(')',
                                                                                            '') if regular_percent_change else None
        post_price = post_price[0].strip() if post_price else None
        pre_price = pre_price[0].strip() if pre_price else None

        return regular_price, regular_change, regular_percent_change, pre_price, post_price

    except Exception as e:
        print(f"Failed to scrape prices: {e}")
        return None, None, None, None, None


async def _scrape_general_info(tree: etree.ElementTree):
    """
    Scrape misc. info from the tree object and formats the data

    :param tree: The parsed HTML tree
    :return: A tuple of the scraped data
    """
    try:
        ul_xpath = './/div[@data-testid="quote-statistics"]/ul'
        list_items_xpath = './/li'
        label_xpath = './/span[contains(@class, "label")]/text()'
        value_xpath = './/span[contains(@class, "value")]/fin-streamer/@data-value | .//span[contains(@class, "value")]/text()'

        ul_element = tree.xpath(ul_xpath)
        if not ul_element:
            return {}

        ul_element = ul_element[0]
        list_items = ul_element.xpath(list_items_xpath)

        # Extract all data in one pass
        data = {}
        for item in list_items:
            label = item.xpath(label_xpath)[0].strip()
            value_elements = item.xpath(value_xpath)
            value = value_elements[0].strip() if value_elements else None
            data[label] = value

        # Process the extracted data
        days_range = data.get("Day's Range", '')
        low, high = days_range.split(' - ') if ' - ' in days_range else (None, None)

        fifty_two_week_range = data.get("52 Week Range", '')
        year_low, year_high = fifty_two_week_range.split(' - ') if ' - ' in fifty_two_week_range else (None, None)

        volume_str = data.get("Volume", '')
        avg_volume_str = data.get("Avg. Volume", '')

        volume = int(volume_str.replace(',', '')) if volume_str and volume_str.replace(',', '').isdigit() else None
        avg_volume = int(avg_volume_str.replace(',', '')) if avg_volume_str and avg_volume_str.replace(',',
                                                                                                       '').isdigit() else None

        forward_dividend_yield = data.get("Forward Dividend & Yield", '')
        if forward_dividend_yield and any(char.isdigit() for char in forward_dividend_yield):
            dividend, yield_percent = forward_dividend_yield.replace("(", "").replace(")", "").split()
        else:
            dividend, yield_percent = None, data.get("Yield")

        return {
            'open': data.get("Open"),
            'high': high,
            'low': low,
            'year_high': year_high,
            'year_low': year_low,
            'volume': volume,
            'avg_volume': avg_volume,
            'market_cap': data.get("Market Cap (intraday)"),
            'beta': data.get("Beta (5Y Monthly)"),
            'pe': data.get("PE Ratio (TTM)"),
            'eps': data.get("EPS (TTM)"),
            'earnings_date': data.get("Earnings Date"),
            'dividend': dividend,
            'dividend_yield': yield_percent,
            'ex_dividend': data.get("Ex-Dividend Date") if data.get("Ex-Dividend Date") != "--" else None,
            'net_assets': data.get("Net Assets"),
            'nav': data.get("NAV"),
            'expense_ratio': data.get("Expense Ratio (net)"),
            'category': data.get("Category"),
            'last_capital_gain': data.get("Last Cap Gain"),
            'morningstar_rating': data.get("Morningstar Rating", "").split()[0] if data.get(
                "Morningstar Rating") else None,
            'morningstar_risk_rating': data.get("Morningstar Risk Rating"),
            'holdings_turnover': data.get("Holdings Turnover"),
            'last_dividend': data.get("Last Dividend"),
            'inception_date': data.get("Inception Date")
        }

    except Exception as e:
        print(f"Failed to scrape general info: {e}")
        return {}


async def _scrape_logo(tree: etree.ElementTree) -> str:
    """
    Scrape only the logo from the HTML tree

    :param tree: The parsed HTML tree
    :return: URL to logo as a string
    """
    website_xpath = '/html/body/div[2]/main/section/section/section/article/section[2]/div/div/div[2]/div/div[1]/div[1]/a/@href'
    website_elements = tree.xpath(website_xpath)
    website = website_elements[0].strip() if website_elements else None

    return await get_logo(url=website) if website else None


async def _scrape_company_info(tree: etree.ElementTree):
    """
    Scrape the sector and industry data from the tree object

    :return: sector, industry, about, employees, logo as a tuple
    """
    try:
        container_xpath = '/html/body/div[2]/main/section/section/section/article/section[2]/div/div/div[2]/div'
        xpaths = {
            'about': './/div[contains(@class, "description")]/p/text()',
            'website': './/div[contains(@class, "description")]/a[contains(@data-ylk, "business-url")]/@href',
            'sector': './/div[contains(@class, "infoSection")][h3[text()="Sector"]]/p/a/text()',
            'industry': './/div[contains(@class, "infoSection")][h3[text()="Industry"]]/p/a/text()',
            'employees': './/div[contains(@class, "infoSection")][h3[text()="Full Time Employees"]]/p/text()'
        }

        container_element = tree.xpath(container_xpath)
        if not container_element:
            return {}

        container_element = container_element[0]
        results = {}

        for key, xpath in xpaths.items():
            elements = container_element.xpath(xpath)
            results[key] = elements[0].strip() if elements else None

        # Get logo asynchronously if website exists
        logo = await get_logo(url=results['website']) if results.get('website') else None

        return {
            'sector': results['sector'],
            'industry': results['industry'],
            'about': results['about'],
            'employees': results['employees'],
            'logo': logo
        }

    except Exception as e:
        print(f"Failed to scrape company info: {e}")
        return {}


async def _scrape_performance(tree: etree.ElementTree):
    """
    Scrape the performance data from the parsed HTML tree using XPath.

    :param tree: Parsed HTML tree
    :return: YTD, 1 year, 3 year, and 5 year returns as a tuple
    """
    try:
        container_xpath = '/html/body/div[2]/main/section/section/section/article/section[5]'
        return_xpaths = {
            'ytd_return': './/section[1]//div[contains(@class, "perf")]/text()',
            'year_return': './/section[2]//div[contains(@class, "perf")]/text()',
            'three_year_return': './/section[3]//div[contains(@class, "perf")]/text()',
            'five_year_return': './/section[4]//div[contains(@class, "perf")]/text()'
        }

        container_element = tree.xpath(container_xpath)
        if not container_element:
            return {}

        container_element = container_element[0]
        results = {}

        for key, xpath in return_xpaths.items():
            elements = container_element.xpath(xpath)
            results[key] = elements[0].strip() if elements else None

        return results

    except Exception as e:
        print(f"Failed to scrape performance: {e}")
        return {}


@cache(expire=10, market_closed_expire=60)
async def _scrape_quote(symbol: str) -> Quote:
    """
    Asynchronously scrapes a quote from a given symbol and returns a Quote object.
    Uses yahooquery as a fallback if scraping fails.
    :param symbol: Quote symbol

    :raises HTTPException: if information is not found, even from yahooquery
    """
    try:
        url = f'https://finance.yahoo.com/quote/{symbol}/'
        html_content = await fetch(url=url)

        # Parse HTML in a separate thread
        loop = asyncio.get_event_loop()
        tree = await loop.run_in_executor(thread_pool, parse_tree, html_content)

        # Get company name
        name_elements = tree.xpath('.//h1/text()')
        if not name_elements or len(name_elements) < 2:
            return await _get_quote_from_yahooquery(symbol)

        name = name_elements[1].split('(')[0].strip()

        # Execute all scraping tasks in parallel
        prices_task = asyncio.create_task(_scrape_price_data(tree))
        general_info_task = asyncio.create_task(_scrape_general_info(tree))
        company_info_task = asyncio.create_task(_scrape_company_info(tree))
        performance_task = asyncio.create_task(_scrape_performance(tree))

        prices, general_info, company_info, performance = await asyncio.gather(
            prices_task, general_info_task, company_info_task, performance_task
        )

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
            **performance
        )

    except Exception as e:
        print(f"Scraping failed for {symbol}: {e}")
        return await _get_quote_from_yahooquery(symbol)


@cache(expire=10, market_closed_expire=60)
async def _scrape_simple_quote(symbol: str) -> SimpleQuote:
    """
    Asynchronously scrapes a simple quote from a given symbol and returns a SimpleQuote object.
    Uses yahooquery as a fallback if scraping fails.
    :param symbol: Quote symbol

    :raises HTTPException: if information is not found, even from yahooquery
    """
    try:
        url = f'https://finance.yahoo.com/quote/{symbol}/'
        html_content = await fetch(url=url)

        # Parse HTML in a separate thread
        loop = asyncio.get_event_loop()
        tree = await loop.run_in_executor(thread_pool, parse_tree, html_content)

        # Get company name
        name_elements = tree.xpath('.//h1/text()')
        if not name_elements or len(name_elements) < 2:
            return await _get_simple_quote_from_yahooquery(symbol)

        name = name_elements[1].split('(')[0].strip()

        # Get price data
        prices = await _scrape_price_data(tree)
        regular_price, regular_change, regular_percent_change, pre_price, post_price = prices

        # Get logo asynchronously
        website_elements = tree.xpath(
            '/html/body/div[2]/main/section/section/section/article/section[2]/div/div/div[2]/div/div[1]/div[1]/a/@href')
        website = website_elements[0].strip() if website_elements else None
        logo = await get_logo(url=website) if website else None

        return SimpleQuote(
            symbol=symbol.upper(),
            name=name,
            price=regular_price,
            pre_market_price=pre_price,
            after_hours_price=post_price,
            change=regular_change,
            percent_change=regular_percent_change,
            logo=logo
        )

    except Exception as e:
        print(f"Simple scraping failed for {symbol}: {e}")
        return await _get_simple_quote_from_yahooquery(symbol)


async def _get_quote_from_yahooquery(symbol: str) -> Quote:
    """
    Get quote data from Yahoo Finance using yahooquery in case the scraping fails
    :param symbol: Quote symbol

    :raises HTTPException: if ticker is not found
    """
    print(f"Getting quote from yahooquery for symbol {symbol}")
    ticker = Ticker(symbol)
    quote = ticker.quotes
    profile = ticker.asset_profile
    ticker_calendar = ticker.calendar_events
    ticker_calendar = ticker_calendar.get(symbol, {})

    if not ticker or not quote or symbol not in quote or not quote[symbol].get('longName'):
        raise HTTPException(status_code=404, detail=f"{symbol} not found")
    try:
        symbol_data = quote[symbol]

        def format_value(value: float):
            if not value:
                return None
            for div, suffix in [(1e12, 'T'), (1e9, 'B'), (1e6, 'M')]:
                if value >= div:
                    return f"{value / div:.1f}{suffix}"
            return str(value)

        def format_date(date_string: str):
            if not date_string:
                return None
            try:
                date = datetime.strptime(date_string, "%Y-%m-%d %H:%M:%S")
            except ValueError:
                try:
                    date = datetime.strptime(date_string, "%Y-%m-%d")
                except ValueError:
                    return None
            return date.strftime("%b %d, %Y")

        # Format earnings date
        earnings_date = ticker_calendar.get('earnings', {}).get('earningsDate', []) if isinstance(ticker_calendar,
                                                                                                  dict) else None
        if earnings_date:
            formatted_dates = [format_date(date.split(' ')[0]) for date in earnings_date]
            earnings_date = ' - '.join(formatted_dates)

        # Format dividend data
        dividend_rate = symbol_data.get('dividendRate')
        dividend_yield = symbol_data.get('dividendYield')
        return Quote(
            symbol=symbol.upper(),
            name=symbol_data['longName'],
            price=str(symbol_data['regularMarketPrice']),
            pre_market_price=str(symbol_data.get('preMarketPrice')) if symbol_data.get('preMarketPrice') else None,
            after_hours_price=str(symbol_data.get('postMarketPrice')) if symbol_data.get('postMarketPrice') else None,
            change=f"{symbol_data['regularMarketChange']:+.2f}",
            percent_change=f"{symbol_data['regularMarketChangePercent']:+.2f}%",
            open=str(symbol_data.get('regularMarketOpen')) if symbol_data.get('regularMarketOpen') else None,
            high=str(symbol_data.get('regularMarketDayHigh')) if symbol_data.get('regularMarketDayHigh') else None,
            low=str(symbol_data.get('regularMarketDayLow')) if symbol_data.get('regularMarketDayLow') else None,
            year_high=str(symbol_data.get('fiftyTwoWeekHigh')) if symbol_data.get('fiftyTwoWeekHigh') else None,
            year_low=str(symbol_data.get('fiftyTwoWeekLow')) if symbol_data.get('fiftyTwoWeekLow') else None,
            volume=symbol_data.get('regularMarketVolume') if symbol_data.get('regularMarketVolume') else None,
            avg_volume=symbol_data.get('averageDailyVolume10Day') if symbol_data.get(
                'averageDailyVolume10Day') else None,
            market_cap=format_value(symbol_data.get('marketCap')) if symbol_data.get('marketCap') else None,
            pe=f"{symbol_data.get('trailingPE', 0):.2f}" if symbol_data.get('trailingPE') else None,
            eps=str(symbol_data.get('trailingEps')) if symbol_data.get('trailingEps') else None,
            earnings_date=earnings_date,
            dividend=str(dividend_rate) if dividend_rate else None,
            dividend_yield=f"{dividend_yield:.2f}%" if dividend_yield else None,
            ex_dividend=format_date(
                str(ticker_calendar.get('exDividendDate')) if isinstance(ticker_calendar, dict) else None),
            net_assets=format_value(symbol_data.get('netAssets')) if symbol_data.get('netAssets') else None,
            expense_ratio=str(symbol_data.get('annualReportExpenseRatio')) if symbol_data.get(
                'annualReportExpenseRatio') else None,
            sector=profile.get(symbol, {}).get('sector'),
            industry=profile.get(symbol, {}).get('industry'),
            about=profile.get(symbol, {}).get('longBusinessSummary'),
            logo=await get_logo(url=profile.get(symbol, {}).get('website'))
        )

    except Exception as e:
        print(f"Yahooquery quote failed for {symbol}: {e}")
        raise HTTPException(status_code=500, detail=f"Failed to get quote for {symbol}")


async def _get_simple_quote_from_yahooquery(symbol: str) -> SimpleQuote:
    """
    Get simple quote data from Yahoo Finance using yahooquery in case the scraping fails
    :param symbol: Quote symbol

    :raises HTTPException: if ticker is not found
    """
    print(f"Getting simple quote from yahooquery for symbol {symbol}")
    ticker = Ticker(symbol)
    quote = ticker.quotes
    profile = ticker.asset_profile
    if not ticker or not quote or symbol not in quote or not quote[symbol].get('longName'):
        raise HTTPException(status_code=404, detail=f"{symbol} not found")

    try:
        symbol_data = quote[symbol]

        return SimpleQuote(
            symbol=symbol.upper(),
            name=symbol_data['longName'],
            price=str(symbol_data['regularMarketPrice']),
            pre_market_price=str(symbol_data.get('preMarketPrice')) if symbol_data.get('preMarketPrice') else None,
            after_hours_price=str(symbol_data.get('postMarketPrice')) if symbol_data.get('postMarketPrice') else None,
            change=f"{symbol_data['regularMarketChange']:+.2f}",
            percent_change=f"{symbol_data['regularMarketChangePercent']:+.2f}%",
            logo=await get_logo(url=profile.get(symbol, {}).get('website'))
        )

    except Exception as e:
        print(f"Yahooquery simple quote failed for {symbol}: {e}")
        raise HTTPException(status_code=500, detail=f"Failed to get simple quote for {symbol}")
