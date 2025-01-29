from fastapi import HTTPException
from lxml import etree

from src.redis import cache
from src.schemas import SimpleQuote
from src.utils import fetch


@cache(expire=15, market_closed_expire=600)
async def scrape_similar_quotes(symbol: str, limit: int = 10) -> list[SimpleQuote]:
    """
    Scrape similar stocks from Yahoo Finance for a single symbol
    :param symbol: the symbol of the stock to find similar stocks around
    :param limit: the number of similar stocks to return
    :return:
    """
    url = 'https://finance.yahoo.com/quote/' + symbol
    html = await fetch(url)

    similar = await _parse_similar_quotes(html, symbol, limit)

    # If similar is still empty, the symbol is probably invalid
    if not similar:
        raise HTTPException(status_code=404, detail="No similar stocks found or invalid symbol.")

    return similar


async def _parse_similar_quotes(html: str, symbol: str, limit: int) -> list[SimpleQuote]:
    """
    Parse similar stocks from Yahoo Finance HTML
    :param html: the HTML content of the Yahoo Finance page
    :param symbol: the symbol of the stock to find similar stocks around
    :param limit: the number of similar stocks to return
    :return: a list of SimpleQuote objects
    """
    tree = etree.HTML(html)

    # Try to parse stocks first
    container_xpath = '//*[@data-testid="compare-to"]//section'
    stock_sections = tree.xpath(container_xpath)
    quotes = []

    if stock_sections:
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
            if parsed_symbol == symbol:
                continue

            name = name_elements[0].strip()
            price_text = price_elements[0].strip().replace(',', '')
            price = price_text
            percent_change = percent_change_elements[0].strip()

            change = float(price) / (1 + float(percent_change.strip('%')) / 100) - float(price)
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
            quotes.append(stock)

            if len(quotes) >= limit:
                break

    # If no stocks found, try to parse ETFs
    if not quotes:
        container_xpath = '//*[@data-testid="people-also-watch"]//section'
        etf_sections = tree.xpath(container_xpath)
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
            price = price_elements[0].strip().replace(',', '')
            percent_change = percent_change_elements[0].strip()

            change = float(price) / (1 + float(percent_change.strip('%')) / 100) - float(price)
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
            quotes.append(etf)

            if len(quotes) >= limit:
                break

    return quotes
