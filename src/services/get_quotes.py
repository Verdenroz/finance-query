import asyncio
from datetime import datetime
from decimal import Decimal
from typing import List

from bs4 import BeautifulSoup, SoupStrainer
from fastapi import HTTPException
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


async def _scrape_price_data(soup: BeautifulSoup):
    """
    Scrape the price data from the soup object and formats the data

    :return: Regular price, change, percent change, and post price as a tuple
    """
    regular_price = round(Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price"})["data-value"]), 2)
    regular_change_value = round(
        Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price-change"})["data-value"]), 2)
    regular_percent_change_value = round(
        Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price-change-percent"})["data-value"]), 2)
    regular_change = f"{regular_change_value:+.2f}"
    regular_percent_change = f"{regular_percent_change_value:+.2f}%"
    post_price_element = soup.find("fin-streamer", {"data-testid": "qsp-post-price"})
    post_price = round(Decimal(post_price_element["data-value"]), 2) if post_price_element else None

    return regular_price, regular_change, regular_percent_change, post_price


async def _scrape_general_info(soup: BeautifulSoup):
    """
    Scrape misc. info from the soup object and formats the data

    :return: A tuple of the scraped data
    """
    list_items = soup.select('li.yf-mrt107')
    data = {}
    for item in list_items:
        label = item.find("span", class_="label").text.strip()
        value = item.find("span", class_="value").text.strip()
        data[label] = value

    open_price = Decimal(data.get("Open").replace(',', '')) if data.get("Open") else None
    market_cap = data.get("Market Cap (intraday)", None)
    beta = Decimal(data.get("Beta (5Y Monthly)")) if data.get('Beta (5Y Monthly)') else None
    pe = Decimal(data.get("PE Ratio (TTM)")) if data.get("PE Ratio (TTM)") else None
    eps = Decimal(data.get("EPS (TTM)")) if data.get("EPS (TTM)") else None
    earnings_date = data.get("Earnings Date")
    forward_dividend_yield = data.get("Forward Dividend & Yield", None)
    dividend, yield_percent = (None, data.get("Yield")) if not forward_dividend_yield else (None, None) if not (
        any(char.isdigit() for char in forward_dividend_yield)) \
        else forward_dividend_yield.replace("(", "").replace(")", "").split()
    ex_dividend = data.get("Ex-Dividend Date") if data.get("Ex-Dividend Date") != "--" else None
    net_assets = data.get("Net Assets", None)
    nav = data.get("NAV", None)
    expense_ratio = data.get("Expense Ratio (net)", None)

    days_range = data.get("Day's Range", None)
    low, high = [Decimal(x.replace(',', '')) for x in days_range.split(' - ')] if days_range else (None, None)

    fifty_two_week_range = data.get("52 Week Range", None)
    year_low, year_high = [Decimal(x.replace(',', '')) for x in
                           fifty_two_week_range.split(' - ')] if fifty_two_week_range else (None, None)

    volume = int(data.get("Volume").replace(',', '')) if data.get("Volume") else None
    avg_volume = int(data.get("Avg. Volume").replace(',', '')) if data.get("Avg. Volume") else None

    category = data.get("Category", None)
    last_cap = data.get("Last Cap Gain", None)
    morningstar_rating = data.get("Morningstar Rating").split()[0] if data.get("Morningstar Rating") else None
    morningstar_risk = data.get("Morningstar Risk Rating", None)
    holdings_turnover = data.get("Holdings Turnover", None)
    last_dividend = data.get("Last Dividend", None)
    inception_date = data.get("Inception Date", None)

    about = soup.find('p', class_='yf-6e9c7m').text
    return (open_price, high, low, year_high, year_low, volume, avg_volume, market_cap, beta, pe, eps, earnings_date,
            dividend, yield_percent, ex_dividend, net_assets, nav, expense_ratio, category, last_cap,
            morningstar_rating, morningstar_risk,
            holdings_turnover, last_dividend, inception_date, about)


async def _scrape_logo(soup: BeautifulSoup):
    """
    Scrape the logo from the soup object

    :return: URL as a string
    """
    logo_element = soup.find('a', class_='subtle-link fin-size-medium yf-1e4diqp')
    url = logo_element['href'] if logo_element else None

    return await get_logo(url) if url else None


async def _scrape_sector_industry(soup: BeautifulSoup):
    """
    Scrape the sector and industry data from the soup object

    :return: sector and industry as a tuple
    """
    info_sections = soup.find_all("div", class_="infoSection yf-6e9c7m")
    curr_sector = None
    curr_industry = None
    for section in info_sections:
        h3_text = section.find("h3").text
        if h3_text == "Sector":
            a_element = section.find("a")
            a_text = a_element.text
            curr_sector = a_text.strip()
        elif h3_text == "Industry":
            a_element = section.find("a")
            a_text = a_element.text
            curr_industry = a_text.strip()

    return curr_sector, curr_industry


async def _scrape_performance(soup: BeautifulSoup):
    """
    Scrape the performance data from the soup object

    :return: YTD, 1 year, 3 year, and 5 year returns as a tuple
    """
    returns = soup.find_all('section', 'card small yf-xvi0tx bdr sticky')
    data = []
    for changes in returns:
        perf_div = changes.find('div', class_=['perf positive yf-12wncuy', 'perf negative yf-12wncuy'])
        if perf_div:
            sign = '+' if 'positive' in perf_div['class'] else '-'
            data.append(sign + perf_div.text.strip())

    ytd_return = data[0] if len(data) > 0 else None
    year_return = data[1] if len(data) > 1 else None
    three_year_return = data[2] if len(data) > 2 else None
    five_year_return = data[3] if len(data) > 3 else None
    return ytd_return, year_return, three_year_return, five_year_return


@cache(10, after_market_expire=60)
async def _scrape_quote(symbol: str) -> Quote:
    """
    Asynchronously scrapes a quote from a given symbol and returns a Quote object.
    :param symbol: Quote symbol

    :return: [Quote] object
    """
    try:
        url = 'https://finance.yahoo.com/quote/' + symbol + "/"
        html = await fetch(url)

        parse_only = SoupStrainer(['h1', 'section', 'li'])
        soup = BeautifulSoup(html, 'lxml', parse_only=parse_only)

        symbol_name_element = soup.select_one('h1.yf-xxbei9')
        if not symbol_name_element:
            return await _get_quote_from_yahooquery(symbol)

        name = symbol_name_element.text.split('(')[0].strip()

        prices_future = asyncio.create_task(_scrape_price_data(soup))
        list_items_future = asyncio.create_task(_scrape_general_info(soup))
        logo_future = asyncio.create_task(_scrape_logo(soup))
        sector_industry_future = asyncio.create_task(_scrape_sector_industry(soup))
        performance_future = asyncio.create_task(_scrape_performance(soup))

        (
            (regular_price, regular_change, regular_percent_change, post_price),
            (open_price, high, low, year_high, year_low, volume, avg_volume, market_cap, beta, pe, eps, earnings_date,
             dividend, yield_percent, ex_dividend, net_assets, nav, expense_ratio, category, last_cap,
             morningstar_rating,
             morningstar_risk, holdings_turnover, last_dividend, inception_date, about),
            logo, (sector, industry),
            (ytd_return, year_return, three_year_return, five_year_return)) = await asyncio.gather(
            prices_future, list_items_future, logo_future, sector_industry_future, performance_future
        )
        print("Logo is", logo)
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
            ytd_return=ytd_return,
            year_return=year_return,
            three_year_return=three_year_return,
            five_year_return=five_year_return,
            logo=logo
        )
    except Exception:
        return await _get_quote_from_yahooquery(symbol)


@cache(10, after_market_expire=60)
async def _scrape_simple_quote(symbol: str) -> SimpleQuote:
    """
    Asynchronously scrapes a simple quote from a given symbol and returns a SimpleQuote object.
    :param symbol: Quote symbol
    """
    try:
        url = 'https://finance.yahoo.com/quote/' + symbol + "/"
        html = await fetch(url)

        parse_only = SoupStrainer(['h1', 'fin-streamer', 'a'])
        soup = BeautifulSoup(html, 'lxml', parse_only=parse_only)

        symbol_name_element = soup.select_one('h1.yf-xxbei9')
        if not symbol_name_element:
            return await _get_simple_quote_from_yahooquery(symbol)

        name = symbol_name_element.text.split('(')[0].strip()

        prices_future = asyncio.create_task(_scrape_price_data(soup))
        logo_future = asyncio.create_task(_scrape_logo(soup))

        (regular_price, regular_change, regular_percent_change, post_price), logo = await asyncio.gather(
            prices_future, logo_future
        )

        return SimpleQuote(
            symbol=symbol.upper(),
            name=name,
            price=regular_price,
            after_hours_price=post_price,
            change=regular_change,
            percent_change=regular_percent_change,
            logo=logo
        )
    except Exception as e:
        print(str(e))
        return await _get_simple_quote_from_yahooquery(symbol)


async def _get_quote_from_yahooquery(symbol: str) -> Quote:
    """
    Get quote data from Yahoo Finance using yahooquery in case the scraping fails
    :param symbol: Quote symbol

    :raises: HTTPException if ticker is not found
    """
    ticker = Ticker(symbol)
    quote = ticker.quotes
    profile = ticker.asset_profile
    ticker_calendar = ticker.calendar_events
    if not quote or symbol not in quote or not quote[symbol].get('longName'):
        raise HTTPException(status_code=404, detail="Symbol not found")

    name = quote[symbol]['longName']
    regular_price = quote[symbol]['regularMarketPrice']
    regular_change_value = quote[symbol]['regularMarketChange']
    regular_percent_change_value = quote[symbol]['regularMarketChangePercent']
    post_price = quote[symbol].get('postMarketPrice', None)
    open_price = quote[symbol].get('regularMarketOpen', None)
    high = quote[symbol].get('regularMarketDayHigh', None)
    low = quote[symbol].get('regularMarketDayLow', None)
    year_high = quote[symbol].get('fiftyTwoWeekHigh', None)
    year_low = quote[symbol].get('fiftyTwoWeekLow', None)
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
    pe = round(pe, 2) if pe else None
    yield_percent = str(yield_percent) + "%" if yield_percent else None
    net_assets = format_value(net_assets) if net_assets else None
    market_cap = format_value(market_cap) if market_cap else None
    ex_dividend = format_date(ex_dividend) if ex_dividend else None
    if earnings_date:
        # Split the date and time, format the date, and join them back
        formatted_dates = [format_date(date.split(' ')[0]) for date in earnings_date]
        earnings_date = ' - '.join(formatted_dates)

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
    ticker = Ticker(symbol)
    quote = ticker.quotes
    profile = ticker.asset_profile

    if not quote or symbol not in quote or not quote[symbol].get('longName'):
        raise HTTPException(status_code=404, detail="Symbol not found")
    name = quote[symbol]['longName']
    regular_price = quote[symbol]['regularMarketPrice']
    post_price = quote[symbol].get('postMarketPrice', None)
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
