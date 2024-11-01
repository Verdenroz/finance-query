from decimal import Decimal

from fastapi import HTTPException
from lxml import etree
from typing_extensions import List

from src.redis import cache
from src.schemas import SimpleQuote
from src.utils import fetch


@cache(expire=15, after_market_expire=600)
async def scrape_similar_quotes(symbol: str, limit: int = 10) -> List[SimpleQuote]:
    url = 'https://finance.yahoo.com/quote/' + symbol
    html = await fetch(url)

    similar = await _parse_stocks(html, symbol, limit)

    # If similar_stocks is empty, try to scrape ETF data
    if not similar:
        similar = await _parse_etfs(html, limit)

    # If similar is still empty, the symbol is probably invalid
    if not similar:
        raise HTTPException(status_code=404, detail="No similar stocks found or invalid symbol.")

    return similar


async def _parse_stocks(html: str, symbol: str, limit: int) -> List[SimpleQuote]:
    tree = etree.HTML(html)
    container_xpath = '/html/body/div[2]/main/section/section/section/article/section[6]/div/div/div/section'
    stock_sections = tree.xpath(container_xpath)
    stocks = []
    for section in stock_sections:
        symbol_xpath = './/span/text()'
        name_xpath = './/div/div[1]/a/div/div/text()'
        price_xpath = './/div/div[2]/div/span/text()'
        percent_change_xpath = './/div/div[2]/div/div/span/text()'

        symbol_elements = section.xpath(symbol_xpath)
        name_elements = section.xpath(name_xpath)
        price_elements = section.xpath(price_xpath)
        percent_change_elements = section.xpath(percent_change_xpath)

        if not (symbol_elements and name_elements and price_elements and percent_change_elements):
            continue

        parsed_symbol = symbol_elements[0].strip()
        # Skip the queried symbol if it appears in the similar stocks list
        if parsed_symbol == symbol:
            continue

        name = name_elements[0].strip()
        price_text = price_elements[0].strip().replace(',', '')
        price = Decimal(price_text)
        percent_change = percent_change_elements[0].strip()

        change = price / (1 + Decimal(percent_change.strip('%')) / 100) - price
        change = round(change, 2)
        if percent_change.startswith('-'):
            change_str = '-' + str(abs(change))
        else:
            change_str = '+' + str(abs(change))

        stock = SimpleQuote(
            symbol=parsed_symbol,
            name=name,
            price=price,
            change=change_str,
            percent_change=percent_change,
        )
        stocks.append(stock)

        if len(stocks) >= limit:
            break

    return stocks


async def _parse_etfs(html: str, limit: int) -> List[SimpleQuote]:
    tree = etree.HTML(html)
    container_xpath = '/html/body/div[2]/main/section/section/section/article/section[4]/div/div/div/section'
    etf_sections = tree.xpath(container_xpath)
    etfs = []
    for section in etf_sections:
        symbol_xpath = './/div/div[1]/div/span[1]/text()'
        name_xpath = './/div/div[1]/div/span[2]/text()'
        price_xpath = './/div/div[2]/span/strong/text()'
        percent_change_xpath = './/div/div[2]/div/span/text()'

        symbol_elements = section.xpath(symbol_xpath)
        name_elements = section.xpath(name_xpath)
        price_elements = section.xpath(price_xpath)
        percent_change_elements = section.xpath(percent_change_xpath)

        if not (symbol_elements and name_elements and price_elements and percent_change_elements):
            continue

        symbol = symbol_elements[0].strip()
        name = name_elements[0].strip()
        price_text = price_elements[0].strip().replace(',', '')
        price = Decimal(price_text)
        percent_change = percent_change_elements[0].strip()

        change = price / (1 + Decimal(percent_change.strip('%')) / 100) - price
        change = round(change, 2)
        if percent_change.startswith('-'):
            change_str = '-' + str(abs(change))
        else:
            change_str = '+' + str(abs(change))

        etf = SimpleQuote(
            symbol=symbol,
            name=name,
            price=price,
            change=change_str,
            percent_change=percent_change,
        )
        etfs.append(etf)

        if len(etfs) >= limit:
            break

    return etfs
