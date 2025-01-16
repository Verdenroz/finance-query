import asyncio
from datetime import datetime
from typing import List

from fastapi import HTTPException
from lxml import etree
from yahooquery import Ticker

from src.schemas import Quote, SimpleQuote
from ..redis import cache
from ..utils import fetch, get_logo


async def scrape_quotes(symbols: List[str]) -> List[Quote]:
    """
    Asynchronously scrapes multiple quotes from a list of symbols and returns a list of Quote objects.
    :param symbols: List of symbols
    :return: List of Quote objects
    """
    quotes = await asyncio.gather(*(_scrape_quote(symbol) for symbol in symbols))
    return [quote for quote in quotes if not isinstance(quote, Exception)]


async def scrape_simple_quotes(symbols: List[str]) -> List[SimpleQuote]:
    """
    Asynchronously scrapes multiple simple quotes from a list of symbols and returns a list of SimpleQuote objects.
    :param symbols: List of symbols
    :return: List of SimpleQuote objects
    """
    quotes = await asyncio.gather(*(_scrape_simple_quote(symbol) for symbol in symbols))
    return [quote for quote in quotes if not isinstance(quote, Exception)]


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


async def _scrape_general_info(tree: etree.ElementTree) -> tuple:
    """
    Scrape misc. info from the soup object and formats the data

    :param tree: The parsed HTML tree
    :return: A tuple of the scraped data
    """
    # XPath expressions
    ul_xpath = './/div[@data-testid="quote-statistics"]/ul'
    list_items_xpath = './/li'
    label_xpath = './/span[contains(@class, "label")]/text()'
    value_xpath = './/span[contains(@class, "value")]/fin-streamer/@data-value | .//span[contains(@class, "value")]/text()'

    # Container ul element
    ul_element = tree.xpath(ul_xpath)

    if not ul_element:
        raise HTTPException(status_code=500, detail="Failed to extract general info")

    # Extraction from the ul element
    try:
        ul_element = tree.xpath(ul_xpath)[0]
        list_items = ul_element.xpath(list_items_xpath)
        data = {}
        for item in list_items:
            label = item.xpath(label_xpath)[0].strip()
            value_elements = item.xpath(value_xpath)
            value = value_elements[0].strip() if value_elements else None
            data[label] = value

        # Formatting
        open_price = data.get("Open")
        market_cap = data.get("Market Cap (intraday)")
        beta = data.get("Beta (5Y Monthly)")
        pe = data.get("PE Ratio (TTM)")
        eps = data.get("EPS (TTM)")
        earnings_date = data.get("Earnings Date")
        forward_dividend_yield = data.get("Forward Dividend & Yield")
        dividend, yield_percent = (None, data.get("Yield")) if not forward_dividend_yield else (None, None) if not (
            any(char.isdigit() for char in forward_dividend_yield)) \
            else forward_dividend_yield.replace("(", "").replace(")", "").split()
        ex_dividend = data.get("Ex-Dividend Date") if data.get("Ex-Dividend Date") != "--" else None
        net_assets = data.get("Net Assets")
        nav = data.get("NAV")
        expense_ratio = data.get("Expense Ratio (net)")

        days_range = data.get("Day's Range")
        low, high = days_range.split(' - ') if days_range else (None, None)

        fifty_two_week_range = data.get("52 Week Range")
        year_low, year_high = fifty_two_week_range.split(' - ') if fifty_two_week_range else (None, None)

        volume_str = data.get("Volume")
        avg_volume_str = data.get("Avg. Volume")

        volume = int(volume_str.replace(',', '')) if volume_str and volume_str.isdigit() else None
        avg_volume = int(avg_volume_str.replace(',', '')) if avg_volume_str and avg_volume_str.isdigit() else None

        category = data.get("Category")
        last_cap = data.get("Last Cap Gain")
        morningstar_rating = data.get("Morningstar Rating").split()[0] if data.get("Morningstar Rating") else None
        morningstar_risk = data.get("Morningstar Risk Rating")
        holdings_turnover = data.get("Holdings Turnover")
        last_dividend = data.get("Last Dividend")
        inception_date = data.get("Inception Date")

        return (open_price, high, low, year_high, year_low, volume, avg_volume, market_cap, beta, pe, eps, earnings_date,
                dividend, yield_percent, ex_dividend, net_assets, nav, expense_ratio, category, last_cap,
                morningstar_rating, morningstar_risk,
                holdings_turnover, last_dividend, inception_date)
    except Exception as e:
        print("Failed to scrape general info", e)
        return tuple(None for _ in range(24))


async def _scrape_logo(tree: etree.ElementTree) -> str:
    """
    Scrape only the logo from the HTML tree

    :param tree: The parsed HTML tree
    :return: URL to logo as a string
    """
    website_xpath = '/html/body/div[2]/main/section/section/section/article/section[2]/div/div/div[2]/div/div[1]/div[1]/a/@href'
    website_elements = tree.xpath(website_xpath)
    website = website_elements[0].strip() if website_elements else None

    return await get_logo(website) if website else None


async def _scrape_company_info(tree: etree.ElementTree) -> tuple:
    """
    Scrape the sector and industry data from the soup object

    :return: sector, industry, about, employees, logo as a tuple
    """
    # XPath expressions
    container_xpath = '/html/body/div[2]/main/section/section/section/article/section[2]/div/div/div[2]/div'
    about_xpath = './/div[contains(@class, "description")]/p/text()'
    website_xpath = './/div[contains(@class, "description")]/a[contains(@data-ylk, "business-url")]/@href'
    sector_xpath = './/div[contains(@class, "infoSection")][h3[text()="Sector"]]/p/a/text()'
    industry_xpath = './/div[contains(@class, "infoSection")][h3[text()="Industry"]]/p/a/text()'
    employees_xpath = './/div[contains(@class, "infoSection")][h3[text()="Full Time Employees"]]/p/text()'

    try:
        # Extract the container element using XPath
        container_element = tree.xpath(container_xpath)[0]

        # Extract the about text using XPath
        about_elements = container_element.xpath(about_xpath)
        about = about_elements[0].strip() if about_elements else None

        # Extract the company website link using XPath
        website_elements = container_element.xpath(website_xpath)
        website = website_elements[0].strip() if website_elements else None
        logo = await get_logo(website) if website else None

        # Extract the sector using XPath
        sector_elements = container_element.xpath(sector_xpath)
        sector = sector_elements[0].strip() if sector_elements else None

        # Extract the industry using XPath
        industry_elements = container_element.xpath(industry_xpath)
        industry = industry_elements[0].strip() if industry_elements else None

        employees_elements = container_element.xpath(employees_xpath)
        employees = employees_elements[0].strip() if employees_elements else None

        return sector, industry, about, employees, logo
    except Exception as e:
        print("Failed to scrape company info", e)
        return tuple(None for _ in range(5))


async def _scrape_performance(tree: etree.ElementTree) -> tuple:
    """
    Scrape the performance data from the parsed HTML tree using XPath.

    :param tree: Parsed HTML tree
    :return: YTD, 1 year, 3 year, and 5 year returns as a tuple
    """
    # XPath expressions
    container_xpath = '/html/body/div[2]/main/section/section/section/article/section[5]'
    ytd_return_xpath = './/section[1]//div[contains(@class, "perf")]/text()'
    one_year_return_xpath = './/section[2]//div[contains(@class, "perf")]/text()'
    three_year_return_xpath = './/section[3]//div[contains(@class, "perf")]/text()'
    five_year_return_xpath = './/section[4]//div[contains(@class, "perf")]/text()'
    try:
        # Container element
        container_element = tree.xpath(container_xpath)[0]

        # Extract the YTD return
        ytd_return_elements = container_element.xpath(ytd_return_xpath)
        ytd_return = ytd_return_elements[0].strip() if ytd_return_elements else None

        # Extract the 1-Year return
        one_year_return_elements = container_element.xpath(one_year_return_xpath)
        one_year_return = one_year_return_elements[0].strip() if one_year_return_elements else None

        # Extract the 3-Year return
        three_year_return_elements = container_element.xpath(three_year_return_xpath)
        three_year_return = three_year_return_elements[0].strip() if three_year_return_elements else None

        # Extract the 5-Year return
        five_year_return_elements = container_element.xpath(five_year_return_xpath)
        five_year_return = five_year_return_elements[0].strip() if five_year_return_elements else None

        return ytd_return, one_year_return, three_year_return, five_year_return
    except Exception as e:
        print("Failed to scrape performance", e)
        return None, None, None, None


@cache(10, market_closed_expire=60)
async def _scrape_quote(symbol: str) -> Quote:
    """
    Asynchronously scrapes a quote from a given symbol and returns a Quote object.
    :param symbol: Quote symbol

    :return: [Quote] object
    """
    try:
        url = 'https://finance.yahoo.com/quote/' + symbol + "/"
        html = await fetch(url)
        tree = etree.HTML(html)

        # Async tasks
        name_future = asyncio.create_task(get_company_name(tree))
        prices_future = asyncio.create_task(_scrape_price_data(tree))
        general_info_future = asyncio.create_task(_scrape_general_info(tree))
        sector_industry_future = asyncio.create_task(_scrape_company_info(tree))
        performance_future = asyncio.create_task(_scrape_performance(tree))

        # Gather the async tasks in parallel
        (
            name,
            (regular_price, regular_change, regular_percent_change, pre_price, post_price),
            (open_price, high, low, year_high, year_low, volume, avg_volume, market_cap, beta, pe, eps, earnings_date,
             dividend, yield_percent, ex_dividend, net_assets, nav, expense_ratio, category, last_cap,
             morningstar_rating,
             morningstar_risk, holdings_turnover, last_dividend, inception_date),
            (sector, industry, about, employees, logo),
            (ytd_return, year_return, three_year_return, five_year_return)) = await asyncio.gather(
            name_future, prices_future, general_info_future, sector_industry_future, performance_future
        )

        return Quote(
            symbol=symbol.upper(),
            name=name,
            price=regular_price,
            pre_market_price=pre_price,
            after_hours_price=post_price,
            change=regular_change,
            percent_change=regular_percent_change,
            open=open_price,
            high=high,
            low=low,
            year_high=year_high,
            year_low=year_low,
            volume=volume,
            avg_volume=avg_volume,
            market_cap=market_cap,
            beta=beta,
            pe=pe,
            eps=eps,
            category=category,
            morningstar_rating=morningstar_rating,
            morningstar_risk_rating=morningstar_risk,
            last_capital_gain=last_cap,
            last_dividend=last_dividend,
            holdings_turnover=holdings_turnover,
            inception_date=inception_date,
            earnings_date=earnings_date,
            dividend=dividend,
            dividend_yield=yield_percent,
            ex_dividend=ex_dividend,
            net_assets=net_assets,
            nav=nav,
            expense_ratio=expense_ratio,
            sector=sector,
            industry=industry,
            about=about,
            employees=employees,
            ytd_return=ytd_return,
            year_return=year_return,
            three_year_return=three_year_return,
            five_year_return=five_year_return,
            logo=logo
        )
    except Exception as e:
        print("Scraping failed", e)
        return await _get_quote_from_yahooquery(symbol)


@cache(10, market_closed_expire=60)
async def _scrape_simple_quote(symbol: str) -> SimpleQuote:
    """
    Asynchronously scrapes a simple quote from a given symbol and returns a SimpleQuote object.
    :param symbol: Quote symbol
    """
    try:
        url = 'https://finance.yahoo.com/quote/' + symbol + "/"
        html = await fetch(url)
        tree = etree.HTML(html)

        # Async tasks
        name_future = asyncio.create_task(get_company_name(tree))
        prices_future = asyncio.create_task(_scrape_price_data(tree))
        logo_future = asyncio.create_task(_scrape_logo(tree))

        # Gather the async tasks in parallel
        name, (regular_price, regular_change, regular_percent_change, pre_price, post_price), logo = await asyncio.gather(
            name_future, prices_future, logo_future
        )

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
        print("Scraping failed", e)
        return await _get_simple_quote_from_yahooquery(symbol)


async def _get_quote_from_yahooquery(symbol: str) -> Quote:
    """
    Get quote data from Yahoo Finance using yahooquery in case the scraping fails
    :param symbol: Quote symbol

    :raises: HTTPException if ticker is not found
    """
    print("Getting quote from yahooquery for symbol", symbol)
    ticker = Ticker(symbol)
    quote = ticker.quotes
    profile = ticker.asset_profile
    ticker_calendar = ticker.calendar_events
    if not quote or symbol not in quote or not quote[symbol].get('longName'):
        raise HTTPException(status_code=404, detail="Symbol not found")

    name = quote[symbol]['longName']
    regular_price = str(quote[symbol]['regularMarketPrice'])
    regular_change_value = quote[symbol]['regularMarketChange']
    regular_percent_change_value = quote[symbol]['regularMarketChangePercent']
    post_price = str(quote[symbol]['postMarketPrice']) if quote[symbol].get('postMarketPrice') else None
    open_price = str(quote[symbol]['regularMarketOpen']) if quote[symbol].get('regularMarketOpen') else None
    high = str(quote[symbol]['regularMarketDayHigh']) if quote[symbol].get('regularMarketDayHigh') else None
    low = str(quote[symbol]['regularMarketDayLow']) if quote[symbol].get('regularMarketDayLow') else None
    year_high = str(quote[symbol]['fiftyTwoWeekHigh']) if quote[symbol].get('fiftyTwoWeekHigh') else None
    year_low = str(quote[symbol]['fiftyTwoWeekLow']) if quote[symbol].get('fiftyTwoWeekLow') else None
    volume = quote[symbol].get('regularMarketVolume', None)
    avg_volume = quote[symbol].get('averageDailyVolume10Day', None)
    market_cap = quote[symbol].get('marketCap', None)
    pe = quote[symbol].get('trailingPE', None)
    eps = quote[symbol].get('trailingEps', None)
    earnings_date = ticker_calendar[symbol]['earnings']['earningsDate'] if 'earnings' in ticker_calendar[
        symbol] and 'earningsDate' in ticker_calendar[symbol]['earnings'] else None
    dividend = str(quote[symbol].get('dividendRate', None))
    yield_percent = quote[symbol].get('dividendYield', None)
    ex_dividend = ticker_calendar[symbol]['exDividendDate'] if 'exDividendDate' in ticker_calendar[symbol] else None
    net_assets = quote[symbol].get('netAssets', None)
    expense_ratio = quote[symbol].get('annualReportExpenseRatio', None)
    sector = profile[symbol].get('sector', None)
    industry = profile[symbol].get('industry', None)
    about = profile[symbol].get('longBusinessSummary', None)
    website = profile[symbol].get('website', None)
    logo = await get_logo(website) if website else None

    def format_value(value: float) -> str:
        # Convert the value to millions and round it to one decimal place
        value_in_millions = round(value / 1_000_000, 1)
        value_in_billions = round(value / 1_000_000_000, 1)
        value_in_trillions = round(value / 1_000_000_000_000, 1)

        # Check the size of the value and use the appropriate suffix
        if value_in_trillions >= 1:
            return f"{value_in_trillions}T"
        elif value_in_billions >= 1:
            return f"{value_in_billions}B"
        else:
            return f"{value_in_millions}M"

    def format_date(date_string: str) -> str:
        # Try to parse the date string with time
        try:
            date = datetime.strptime(date_string, "%Y-%m-%d %H:%M:%S")
        except ValueError:
            # If it fails, try to parse the date string without time
            date = datetime.strptime(date_string, "%Y-%m-%d")

        # Format the datetime object into the desired string format
        return date.strftime("%b %d, %Y")

    # Add + or - sign and % for percent_change using f-strings
    regular_change = f"{regular_change_value:+.2f}"
    regular_percent_change = f"{regular_percent_change_value:+.2f}%"

    # Convert float values to string
    pe = str(round(pe, 2)) if pe else None
    yield_percent = str(yield_percent) + "%" if yield_percent else None
    net_assets = format_value(net_assets) if net_assets else None
    market_cap = format_value(market_cap) if market_cap else None
    ex_dividend = format_date(ex_dividend) if ex_dividend else None
    if earnings_date:
        # Split the date and time, format the date, and join them back
        formatted_dates = [format_date(date.split(' ')[0]) for date in earnings_date]
        earnings_date = ' - '.join(formatted_dates)
    else:
        earnings_date = None

    return Quote(
        symbol=symbol.upper(),
        name=name,
        price=regular_price,
        after_hours_price=post_price,
        change=regular_change,
        percent_change=regular_percent_change,
        open=open_price,
        high=high,
        low=low,
        year_high=year_high,
        year_low=year_low,
        volume=volume,
        avg_volume=avg_volume,
        market_cap=market_cap,
        pe=pe,
        eps=eps,
        earnings_date=earnings_date,
        dividend=dividend,
        dividend_yield=yield_percent,
        ex_dividend=ex_dividend,
        net_assets=net_assets,
        expense_ratio=expense_ratio,
        sector=sector,
        industry=industry,
        about=about,
        logo=logo
    )


async def _get_simple_quote_from_yahooquery(symbol: str) -> SimpleQuote:
    """
    Get simple quote data from Yahoo Finance using yahooquery in case the scraping fails
    :param symbol: Quote symbol

    :raises: HTTPException if ticker is not found
    """
    print("Getting simple quote from yahooquery for symbol", symbol)
    ticker = Ticker(symbol)
    quote = ticker.quotes
    profile = ticker.asset_profile

    if not quote or symbol not in quote or not quote[symbol].get('longName'):
        raise HTTPException(status_code=404, detail="Symbol not found")
    name = quote[symbol]['longName']
    regular_price = str(quote[symbol]['regularMarketPrice'])
    post_price = str(quote[symbol]['postMarketPrice']) if quote[symbol].get('postMarketPrice') else None
    regular_change = f"{quote[symbol]['regularMarketChange']:.2f}"
    regular_percent_change = f"{quote[symbol]['regularMarketChangePercent']:.2f}%"
    website = profile[symbol].get('website', None)
    logo = await get_logo(website) if website else None

    return SimpleQuote(
        symbol=symbol.upper(),
        name=name,
        price=regular_price,
        after_hours_price=post_price,
        change=regular_change,
        percent_change=regular_percent_change,
        logo=logo
    )
