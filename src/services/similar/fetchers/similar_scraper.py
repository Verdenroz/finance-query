from fastapi import HTTPException
from lxml import etree

from src.dependencies import fetch
from src.schemas import SimpleQuote


async def scrape_similar_quotes(symbol: str, limit: int = 10) -> list[SimpleQuote]:
    """
    Parse similar stocks from Yahoo Finance HTML
    :param symbol: the symbol of the stock to find similar stocks around
    :param limit: the number of similar stocks to return
    :return: a list of SimpleQuote objects
    """
    try:
        url = 'https://finance.yahoo.com/quote/' + symbol
        html = await fetch(url=url)
        tree = etree.HTML(html)

        quotes = await parse_stocks(tree, symbol, limit)
        if len(quotes) < limit:
            quotes.extend(await parse_etfs(tree, symbol, limit - len(quotes)))

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"No similar stocks found or invalid symbol: {e}")

    return quotes


async def parse_stocks(tree: etree.ElementTree, symbol: str, limit: int) -> list[SimpleQuote]:
    container_xpath = '//*[@data-testid="compare-to"]//section'
    stock_sections = tree.xpath(container_xpath)
    quotes = []

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

    return quotes


async def parse_etfs(tree: etree.ElementTree, symbol: str, limit: int) -> list[SimpleQuote]:
    container_xpath = '//*[@data-testid="people-also-watch"]//section'
    etf_sections = tree.xpath(container_xpath)
    quotes = []

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

        parsed_symbol = symbol_elements[0].strip()
        if parsed_symbol == symbol:
            continue

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
            symbol=parsed_symbol,
            name=name,
            price=price,
            change=change_str,
            percent_change=percent_change,
        )
        quotes.append(etf)

        if len(quotes) >= limit:
            break

    return quotes
