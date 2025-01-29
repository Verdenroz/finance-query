from fastapi import HTTPException
from lxml import etree

from src.redis import cache
from src.schemas import News
from src.utils import fetch


def parse_symbol_exchange(yahoo_symbol: str):
    """
    Parses a Yahoo Finance symbol into a base symbol and exchange code.
    :param yahoo_symbol: the Yahoo Finance symbol to parse where the exchange code is separated by a dot
    :return: the base symbol and exchange code if present (base_symbol, exchange_code)
    """
    # Mapping of Yahoo Finance exchange codes to StockAnalysis codes because ffs they are different!
    exchange_mapping = {
        # Americas
        'OTC': 'OTC',  # US OTC
        'BA': 'BCBA',  # Buenos Aires Stock Exchange
        'MX': 'BMV',  # Mexican Stock Exchange
        'TO': 'TSX',  # Toronto Stock Exchange
        'V': 'TSXV',  # TSX Venture Exchange
        'CN': 'CSE',  # Canadian Securities Exchange
        'SA': 'BVMF',  # Brazil Stock Exchange
        'CR': 'BVC',  # Colombia Stock Exchange

        # Asia Pacific
        'BO': 'BOM',  # Bombay Stock Exchange
        'NS': 'NSE',  # National Stock Exchange of India
        'T': 'TYO',  # Tokyo Stock Exchange
        'HK': 'HKG',  # Hong Kong Stock Exchange
        'SZ': 'SHE',  # Shenzhen Stock Exchange
        'SS': 'SHA',  # Shanghai Stock Exchange
        'KS': 'KRX',  # Korea Stock Exchange
        'KQ': 'KOSDAQ',  # KOSDAQ
        'TW': 'TPE',  # Taiwan Stock Exchange
        'TWO': 'TPEX',  # Taipei Exchange
        'KL': 'KLSE',  # Bursa Malaysia
        'BK': 'BKK',  # Stock Exchange of Thailand
        'JK': 'IDX',  # Indonesia Stock Exchange
        'AX': 'ASX',  # Australian Securities Exchange
        'NZ': 'NZE',  # New Zealand Stock Exchange
        'SI': 'SGX',  # Singapore Exchange

        # Europe
        'L': 'LON',  # London Stock Exchange
        'PA': 'EPA',  # Euronext Paris
        'F': 'FRA',  # Frankfurt Stock Exchange
        'DE': 'ETR',  # Deutsche Börse Xetra
        'MI': 'BIT',  # Borsa Italiana
        'MC': 'BME',  # Madrid Stock Exchange
        'AS': 'AMS',  # Euronext Amsterdam
        'BR': 'EBR',  # Euronext Brussels
        'ST': 'STO',  # Nasdaq Stockholm
        'CO': 'CPH',  # Copenhagen Stock Exchange
        'HE': 'HEL',  # Nasdaq Helsinki
        'OL': 'OSL',  # Oslo Børs
        'SW': 'SWX',  # SIX Swiss Exchange
        'LS': 'ELI',  # Euronext Lisbon
        'AT': 'ATH',  # Athens Stock Exchange
        'VI': 'VIE',  # Vienna Stock Exchange
        'BE': 'BELEX',  # Belgrade Stock Exchange
        'PR': 'PRA',  # Prague Stock Exchange
        'WA': 'WSE',  # Warsaw Stock Exchange

        # Middle East & Africa
        'TA': 'TLV',  # Tel Aviv Stock Exchange
        'KW': 'KWSE',  # Kuwait Stock Exchange
        'QA': 'QSE',  # Qatar Stock Exchange
        'SR': 'TADAWUL',  # Saudi Stock Exchange
        'JO': 'ASE',  # Amman Stock Exchange
        'CA': 'CBSE',  # Casablanca Stock Exchange
        'J': 'JSE',  # Johannesburg Stock Exchange
    }

    try:
        # Split the symbol into base symbol and exchange code
        if '.' in yahoo_symbol:
            base_symbol, yahoo_exchange = yahoo_symbol.split('.')
        else:
            # If no exchange code is present, return the symbol as is with None for exchange
            return yahoo_symbol, None

        # Convert the exchange code if it exists in our mapping
        stockanalysis_exchange = exchange_mapping.get(yahoo_exchange, None)

        return base_symbol, stockanalysis_exchange

    except Exception:
        return yahoo_symbol, None


@cache(300)
async def scrape_news_for_quote(symbol: str) -> list[News]:
    """
    Fetches news for a specific stock or ETF symbol from StockAnalysis.
    :param symbol: the stock or ETF symbol to fetch news for
    :return: a list of news items if found

    :raises HTTPException: if no news was found for the symbol
    """
    # First convert the symbol if it has an exchange code
    base_symbol, exchange = parse_symbol_exchange(symbol)
    # Build URLs based on whether we have an exchange code
    if exchange:
        urls = [f'https://stockanalysis.com/quote/{exchange.lower()}/{base_symbol}']
    else:
        # If no exchange code, try all possible U.S. URLs for the symbol
        urls = [
            f'https://stockanalysis.com/stocks/{base_symbol}',
            f'https://stockanalysis.com/etf/{base_symbol}',
            f"https://stockanalysis.com/quote/otc/{base_symbol}"
        ]

    container_xpath = '/html/body/div/div[1]/div[2]/main/div[3]/div[2]/div/div[2]'

    # Try each URL until we find news
    for url in urls:
        try:
            html = await fetch(url)
            news_list = await _parse_news(html, container_xpath)

            if news_list:
                return news_list

        except Exception:
            continue  # Try next URL if current one fails

    # If we get here, no news was found on any URL
    raise HTTPException(
        status_code=404,
        detail="Could not find news for the provided symbol"
    )


@cache(900)
async def scrape_general_news():
    url = 'https://stockanalysis.com/news/'
    html = await fetch(url)
    container_xpath = '/html/body/div/div[1]/div[2]/main/div[2]/div/div'
    news_list = await _parse_news(html, container_xpath)
    # If no news was found, raise an error
    if not news_list:
        raise HTTPException(status_code=404, detail="Error fetching news")

    return news_list


async def _parse_news(html: str, container_xpath: str) -> list[News]:
    """
    Parses news from the HTML content of a page.
    :param html: the HTML content of the page
    :param container_xpath: the XPath expression to select the container element containing the news
    :return: a list of News objects
    """
    tree = etree.HTML(html)

    news_xpath = './/div[div/h3/a and div/p and div/div[@title]]'
    img_xpath = './/a/img/@src'
    title_xpath = './/h3/a/text()'
    link_xpath = './/h3/a/@href'
    source_date_xpath = './/div[@title]/text()'

    container_elements = tree.xpath(container_xpath)
    # If no container elements were found, return an empty list to try etfs
    if not container_elements:
        return []

    container_element = container_elements[0]
    news_elements = container_element.xpath(news_xpath)
    news_list = []

    for news in news_elements:
        img_elements = news.xpath(img_xpath)
        title_elements = news.xpath(title_xpath)
        link_elements = news.xpath(link_xpath)
        source_date_elements = news.xpath(source_date_xpath)

        if img_elements and title_elements and link_elements and source_date_elements:
            img = img_elements[0].strip()
            title = title_elements[0].strip()
            link = link_elements[0].strip()
            source_date = source_date_elements[0].strip()
            time, source = source_date.split(" - ")

            news_item = News(title=title, link=link, source=source, img=img, time=time)
            news_list.append(news_item)

    return news_list
