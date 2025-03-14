from fastapi import HTTPException
from lxml import etree

from src.dependencies import fetch
from src.models import MarketMover


async def scrape_movers(url: str) -> list[MarketMover]:
    """
    Scrape the most active, gainers, or losers from Yahoo Finance
    :param url: the Yahoo Finance URL to scrape

    :raises HTTPException: with status code 500 if an error occurs while scraping or no movers are found
    """
    html = await fetch(url=url)
    tree = etree.HTML(html)

    tbody_xpath = '/html/body/div[2]/main/section/section/section/article/section[1]/div/div[2]/div/table/tbody'
    row_xpath = './/tr'
    symbol_xpath = './/td[1]/span/div/a/div/span/text()'
    name_xpath = './/td[2]//div/text()'
    price_xpath = './/td[4]//fin-streamer[@data-field="regularMarketPrice"]/text()'
    change_xpath = './/td[5]/span/fin-streamer/span/text()'
    percent_change_xpath = './/td[6]/span/fin-streamer/span/text()'

    tbody_element = tree.xpath(tbody_xpath)[0]
    rows = tbody_element.xpath(row_xpath)
    movers = []

    for row in rows:
        symbol_elements = row.xpath(symbol_xpath)
        name_elements = row.xpath(name_xpath)
        price_elements = row.xpath(price_xpath)
        change_elements = row.xpath(change_xpath)
        percent_change_elements = row.xpath(percent_change_xpath)

        if symbol_elements and name_elements and price_elements and change_elements and percent_change_elements:
            symbol = symbol_elements[0].strip()
            name = name_elements[0].strip()
            price = price_elements[0].strip()
            change = change_elements[0].strip()
            percent_change = percent_change_elements[0].strip('()')

            mover = MarketMover(
                symbol=symbol,
                name=name,
                price=price,
                change=change,
                percent_change=percent_change
            )
            movers.append(mover)

    # If no movers are found, raise an HTTPException
    if not movers:
        raise HTTPException(status_code=500, detail='Failed to parse market movers')

    return movers
