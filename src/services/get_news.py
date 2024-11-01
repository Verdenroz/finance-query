from typing import Optional

from fastapi import HTTPException
from lxml import etree
from typing_extensions import List

from src.redis import cache
from src.schemas import News
from src.utils import fetch


@cache(300)
async def scrape_news_for_quote(symbol: str, is_etf: Optional[bool]) -> List[News]:
    urls = [
        'https://stockanalysis.com/stocks/' + symbol,
        'https://stockanalysis.com/etf/' + symbol
    ]
    container_xpath = '/html/body/div/div[1]/div[2]/main/div[3]/div[2]/div/div[2]'

    if is_etf:
        html = await fetch(urls[1])
        news_list = await _parse_news(html, container_xpath)
        if not news_list:
            raise HTTPException(status_code=404, detail="Are you sure this is an ETF?")
        return news_list

    else:
        # Try to fetch news from the stocks url, if it fails, try etf
        # Yes, I know this is slower for ETFs, but this is how I can save money on proxies
        # If you care about this, you can change this to fetch both at the same time with asyncio.gather
        # Best way is to specify the type of symbol in the request
        for url in urls:
            html = await fetch(url)
            news_list = await _parse_news(html, container_xpath)

            # If news was found, break the loop because the symbol is a stock
            if news_list:
                break

        # If no news was found, raise an error
        else:
            raise HTTPException(status_code=404, detail="Error fetching news")

        return news_list


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


async def _parse_news(html: str, container_xpath: str) -> List[News]:
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
