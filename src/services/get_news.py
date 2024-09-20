from aiohttp import ClientSession
from bs4 import BeautifulSoup, SoupStrainer
from fastapi import HTTPException
from typing_extensions import List

from src.constants import headers
from src.proxies import aio_proxy, proxy_auth
from src.redis import cache
from src.schemas import News


async def fetch_with_aiohttp(url: str, client: ClientSession):
    async with client.get(url, headers=headers) as response:
        return await response.text()


async def parse_news(html: str) -> List[News]:
    soup = BeautifulSoup(html, 'lxml', parse_only=SoupStrainer('div'))
    news = soup.find_all('div', class_='gap-4 border-gray-300 bg-white p-4 shadow last:pb-1 last:shadow-none '
                                       'dark:border-dark-600 dark:bg-dark-800 sm:border-b sm:px-0 sm:shadow-none '
                                       'sm:last:border-b-0 lg:gap-5 sm:grid sm:grid-cols-news sm:py-6')

    news_list = []
    for new in news:
        img_element = new.find("img", class_="rounded")
        img = img_element["src"]
        if not img:
            continue

        title_element = new.find("h3",
                                 class_="mb-2 mt-3 text-xl font-bold leading-snug sm:order-2 sm:mt-0 sm:leading-tight")
        title_link_element = title_element.find("a")
        title = title_link_element.text
        link = title_link_element["href"]

        source_date_element = new.find("div", class_="mt-1 text-sm text-faded sm:order-1 sm:mt-0")
        source_date = source_date_element.text
        time = source_date.split(" - ")[0]
        source = source_date.split(" - ")[1]

        news_item = News(title=title, link=link, source=source, img=img, time=time)
        news_list.append(news_item)

    return news_list


@cache(300)
async def scrape_news_for_quote(symbol: str) -> List[News]:
    urls = [
        'https://stockanalysis.com/stocks/' + symbol,
        'https://stockanalysis.com/etf/' + symbol
    ]

    async with ClientSession() as session:
        # Try to fetch news from the stocks url, if it fails, try etf
        for url in urls:
            html = await fetch_with_aiohttp(url, session)
            news_list = await parse_news(html)
            if news_list:
                break
        # If no news was found, raise an error
        else:
            raise HTTPException(status_code=404, detail="Error fetching news")

    return news_list


@cache(900)
async def scrape_general_news():
    url = 'https://stockanalysis.com/news/'
    async with ClientSession() as session:
        html = await fetch_with_aiohttp(url, session)
        news_list = await parse_news(html)
        # If no news was found, raise an error
        if not news_list:
            raise HTTPException(status_code=404, detail="Error fetching news")

    return news_list
