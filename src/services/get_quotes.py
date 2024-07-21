import asyncio
from datetime import datetime
from decimal import Decimal
from typing import List

import requests
from bs4 import BeautifulSoup, SoupStrainer
from fastapi import HTTPException
from httpx import AsyncClient
from yahooquery import Ticker

from src.schemas import Quote, SimpleQuote
from ..constants import headers
from ..utils import cache


async def fetch(url: str, client: AsyncClient):
    """
    Fetches the HTML content of a given URL
    """
    response = await client.get(url, headers=headers)
    return response.text


async def get_logo(url: str):
    response = requests.get(f"https://logo.clearbit.com/{url}")
    if response.status_code == 200:
        return response.url
    else:
        return None


async def get_quote_from_yahooquery(symbol: str) -> Quote:
    """
    Get quote data from Yahoo Finance using yahooquery in case the scraping fails
    :param symbol: Stock symbol

    :raises: HTTPException if ticker is not found
    """
    ticker = Ticker(symbol)
    quote = ticker.quotes
    profile = ticker.asset_profile
    ticker_calendar = ticker.calendar_events
    if not quote or symbol not in quote:
        raise HTTPException(status_code=404, detail="Symbol not found")
    name = quote[symbol]['longName']
    regular_price = quote[symbol]['regularMarketPrice']
    regular_change = quote[symbol]['regularMarketChange']
    regular_percent_change = quote[symbol]['regularMarketChangePercent']
    post_price = quote[symbol]['postMarketPrice'] if 'postMarketPrice' in quote[symbol] else None
    open_price = quote[symbol]['regularMarketOpen']
    high = quote[symbol]['regularMarketDayHigh']
    low = quote[symbol]['regularMarketDayLow']
    year_high = quote[symbol]['fiftyTwoWeekHigh']
    year_low = quote[symbol]['fiftyTwoWeekLow']
    volume = quote[symbol]['regularMarketVolume']
    avg_volume = quote[symbol]['averageDailyVolume10Day']
    market_cap = quote[symbol]['marketCap'] if 'marketCap' in quote[symbol] else None
    pe = quote[symbol]['trailingPE'] if 'trailingPE' in quote[symbol] else None
    eps = quote[symbol]['trailingEps'] if 'trailingEps' in quote[symbol] else None
    earnings_date = ticker_calendar[symbol]['earnings']['earningsDate'] if 'earnings' in ticker_calendar[
        symbol] and 'earningsDate' in ticker_calendar[symbol]['earnings'] else None
    dividend = quote[symbol]['dividendRate'] if 'dividendRate' in quote[symbol] else None
    yield_percent = quote[symbol]['dividendYield'] if 'dividendYield' in quote[symbol] else None
    ex_dividend = ticker_calendar[symbol]['exDividendDate'] if 'exDividendDate' in ticker_calendar[symbol] else None
    net_assets = quote[symbol]['netAssets'] if 'netAssets' in quote[symbol] else None
    expense_ratio = quote[symbol]['annualReportExpenseRatio'] if 'annualReportExpenseRatio' in quote[symbol] else None
    sector = profile[symbol]['sector'] if 'sector' in profile[symbol] else None
    industry = profile[symbol]['industry'] if 'industry' in profile[symbol] else None
    about = profile[symbol]['longBusinessSummary'] if 'longBusinessSummary' in profile[symbol] else None
    website = profile[symbol]['website'] if 'website' in profile[symbol] else None
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

    # Convert float values to string
    regular_change = str(round(regular_change, 2)) if regular_change else None
    regular_percent_change = str(round(regular_percent_change, 2)) + "%" if regular_percent_change else None
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


async def scrape_quote(symbol: str, client: AsyncClient) -> Quote:
    """
    Asynchronously scrapes a quote from a given symbol and returns a Quote object.
    :param symbol: Stock symbol
    :param client: HTTP client

    :raises: HTTPException if there is an error scraping and unable to get quote from yahooquery
    """

    def get_decimal(number, key):
        num = number.get(key)
        dec_value = Decimal(num) if num and num.replace('.', '', 1).isdigit() else None
        if dec_value is None:
            return None
        return dec_value

    async def extract_sector_and_industry(sector_soup: BeautifulSoup):
        info_sections = sector_soup.find_all("div", class_="infoSection yf-1xu2f9r")

        curr_sector = None
        curr_industry = None

        for section in info_sections:
            h3_text = section.find("h3").text
            a_element = section.find("a")
            a_text = a_element.text if a_element else None
            if h3_text == "Sector":
                curr_sector = a_text.strip()
            elif h3_text == "Industry":
                curr_industry = a_text.strip()

        return curr_sector, curr_industry

    url = 'https://finance.yahoo.com/quote/' + symbol + "/"
    html = await fetch(url, client)

    parse_only = SoupStrainer(['h1', 'div', 'card'])
    soup = BeautifulSoup(html, 'lxml', parse_only=parse_only)

    symbol_name_element = soup.select_one('h1.yf-3a2v0c')
    if not symbol_name_element:
        return await get_quote_from_yahooquery(symbol)

    name = symbol_name_element.text.split('(')[0].strip()

    regular_price = round(Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price"})["data-value"]), 2)
    regular_change_value = round(Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price-change"})["data-value"]),
                                 2)
    regular_percent_change_value = round(
        Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price-change-percent"})["data-value"]), 2)

    # Add + or - sign and % for percent_change
    regular_change = '+' + str(regular_change_value) if regular_change_value > 0 else str(regular_change_value)
    regular_percent_change = '+' + str(
        regular_percent_change_value) + '%' if regular_percent_change_value > 0 else str(
        regular_percent_change_value) + '%'

    # After hours price
    post_price_element = soup.find("fin-streamer", {"data-testid": "qsp-post-price"})
    if not post_price_element:
        post_price = None
    else:
        post_price = round(Decimal(post_price_element["data-value"]), 2)

    list_items = soup.select('li.yf-tx3nkj')

    data = {}

    for item in list_items:
        label = item.find("span", class_="label").text.strip()
        value = item.find("span", class_="value").text.strip()
        data[label] = value

    open_price = Decimal(data.get("Open").replace(',', '')) if data.get("Open") else None
    market_cap = data.get("Market Cap (intraday)")
    beta = get_decimal(data, "Beta (5Y Monthly)")
    pe = get_decimal(data, "PE Ratio (TTM)")
    eps = get_decimal(data, "EPS (TTM)")
    earnings_date = data.get("Earnings Date")
    forward_dividend_yield = data.get("Forward Dividend & Yield")
    dividend, yield_percent = (None, data.get("Yield")) if not forward_dividend_yield \
        else (None, None) if not any(char.isdigit() for char in forward_dividend_yield) \
        else forward_dividend_yield.replace("(", "").replace(")", "").split()
    ex_dividend = None if data.get("Ex-Dividend Date") == "--" else data.get("Ex-Dividend Date")
    net_assets = data.get("Net Assets")
    nav = data.get("NAV")
    expense_ratio = data.get("Expense Ratio (net)")

    # Day's range
    days_range = data.get("Day's Range")
    low, high = None, None
    if days_range:
        low, high = [Decimal(x.replace(',', '')) for x in days_range.split(' - ')]

    # 52-week range
    fifty_two_week_range = data.get("52 Week Range")
    year_low, year_high = None, None
    if fifty_two_week_range:
        year_low, year_high = [Decimal(x.replace(',', '')) for x in fifty_two_week_range.split(' - ')]

    # Volume
    volume = data.get("Volume")
    avg_volume = data.get("Avg. Volume")

    # About the company
    about = soup.find('p', class_='yf-1xu2f9r').text
    # Logo
    logo_element = soup.find('a', class_='subtle-link fin-size-medium yf-13p9sh2')
    logo_url = logo_element['href'] if logo_element else None

    # Funds
    category = data.get("Category")
    last_cap = data.get("Last Cap Gain")
    morningstar_rating = data.get("Morningstar Rating").split()[0] if data.get("Morningstar Rating") else None
    morningstar_risk = data.get("Morningstar Risk Rating")
    holdings_turnover = data.get("Holdings Turnover")
    last_dividend = data.get("Last Dividend")
    inception_date = data.get("Inception Date")

    # Scrape sector, industry, and logo concurrently
    sector_and_industry_future = asyncio.create_task(extract_sector_and_industry(soup))
    logo_future = asyncio.create_task(get_logo(logo_url))

    (sector, industry), logo = await asyncio.gather(sector_and_industry_future, logo_future)

    # Scrape performance:
    returns = soup.find_all('section', 'card small yf-13ievhf bdr sticky')
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


@cache(30, after_market_expire=600)
async def scrape_quotes(symbols: List[str]):
    async with AsyncClient(http2=True, max_redirects=5) as client:
        tasks = [scrape_quote(symbol, client) for symbol in symbols]
        quotes = await asyncio.gather(*tasks)
    return quotes


async def get_simple_quote_from_yahooquery(symbol: str) -> SimpleQuote:
    """
    Get simple quote data from Yahoo Finance using yahooquery in case the scraping fails
    :param symbol: The stock symbol

    :raises: HTTPException if ticker is not found
    """
    ticker = Ticker(symbol)
    quote = ticker.quotes
    if not quote or symbol not in quote:
        raise HTTPException(status_code=404, detail="Symbol not found")
    name = quote[symbol]['longName']
    regular_price = quote[symbol]['regularMarketPrice']
    regular_change = quote[symbol]['regularMarketChange']
    regular_percent_change = quote[symbol]['regularMarketChangePercent']

    return SimpleQuote(
        symbol=symbol.upper(),
        name=name,
        price=regular_price,
        change=regular_change,
        percent_change=regular_percent_change
    )


async def scrape_simple_quote(symbol: str, client: AsyncClient) -> SimpleQuote:
    """
    Asynchronously scrapes a simple quote from a given symbol and returns a SimpleQuote object.
    :param symbol: The stock symbol
    :param client: The HTTP client
    """
    url = 'https://finance.yahoo.com/quote/' + symbol + "/"
    html = await fetch(url, client)

    parse_only = SoupStrainer(['h1', 'fin-streamer'])
    soup = BeautifulSoup(html, 'lxml', parse_only=parse_only)

    symbol_name_element = soup.select_one('h1.yf-3a2v0c')
    if not symbol_name_element:
        return await get_simple_quote_from_yahooquery(symbol)

    name = symbol_name_element.text.split('(')[0].strip()

    regular_price = round(Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price"})["data-value"]), 2)
    regular_change_value = round(Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price-change"})["data-value"]),
                                 2)
    regular_percent_change_value = round(
        Decimal(soup.find("fin-streamer", {"data-testid": "qsp-price-change-percent"})["data-value"]), 2)

    # Add + or - sign and % for percent_change
    regular_change = '+' + str(regular_change_value) if regular_change_value >= 0 else str(regular_change_value)
    regular_percent_change = '+' + str(
        regular_percent_change_value) + '%' if regular_percent_change_value >= 0 else str(
        regular_percent_change_value) + '%'

    return SimpleQuote(
        symbol=symbol.upper(),
        name=name,
        price=regular_price,
        change=regular_change,
        percent_change=regular_percent_change
    )


@cache(30, after_market_expire=600)
async def scrape_simple_quotes(symbols: List[str]):
    async with AsyncClient(http2=True, max_redirects=5) as client:
        quotes = await asyncio.gather(*(scrape_simple_quote(symbol, client) for symbol in symbols))
        return [quote for quote in quotes if not isinstance(quote, Exception)]
