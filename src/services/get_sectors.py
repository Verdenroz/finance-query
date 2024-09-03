import asyncio
from typing import List

from bs4 import BeautifulSoup, SoupStrainer
from fastapi import HTTPException
from httpx import AsyncClient
from yahooquery import Ticker

from src.constants import headers
from src.schemas import MarketSector
from src.schemas.sector import Sector, MarketSectorDetails
from src.services import scrape_simple_quotes
from src.utils import cache

urls = {
    Sector.TECHNOLOGY: 'https://finance.yahoo.com/sectors/technology/',
    Sector.HEALTHCARE: 'https://finance.yahoo.com/sectors/healthcare/',
    Sector.FINANCIAL_SERVICES: 'https://finance.yahoo.com/sectors/financial-services/',
    Sector.CONSUMER_CYCLICAL: 'https://finance.yahoo.com/sectors/consumer-cyclical/',
    Sector.INDUSTRIALS: 'https://finance.yahoo.com/sectors/industrials/',
    Sector.CONSUMER_DEFENSIVE: 'https://finance.yahoo.com/sectors/consumer-defensive/',
    Sector.ENERGY: 'https://finance.yahoo.com/sectors/energy/',
    Sector.REAL_ESTATE: 'https://finance.yahoo.com/sectors/real-estate/',
    Sector.UTILITIES: 'https://finance.yahoo.com/sectors/utilities/',
    Sector.BASIC_MATERIALS: 'https://finance.yahoo.com/sectors/basic-materials/',
    Sector.COMMUNICATION: 'https://finance.yahoo.com/sectors/communication-services/'
}


async def parse_sector(html: str, sector: str) -> MarketSector:
    soup = BeautifulSoup(html, 'lxml', parse_only=SoupStrainer(['section']))
    returns = soup.find_all('section', 'card small yf-13ievhf bdr sticky')
    data = []
    for changes in returns:
        perf_div = changes.find('div', class_=['perf positive yf-12wncuy', 'perf negative yf-12wncuy'])
        sign = '+' if 'positive' in perf_div['class'] else '-'
        data.append(sign + perf_div.text)
    return MarketSector(
        sector=sector,
        day_return=data[0].strip(),
        ytd_return=data[1].strip(),
        year_return=data[2].strip(),
        three_year_return=data[3].strip(),
        five_year_return=data[4].strip()
    )


async def parse_sector_details(html: str, sector_name: str) -> MarketSectorDetails:
    async def parse_info(info_soup):
        return [div.text for div in info_soup.find_all('div', 'value yf-e2k9sg')]

    async def parse_returns(returns_soup):
        data = []
        returns = returns_soup.find_all('section', 'card small yf-13ievhf bdr sticky')
        for changes in returns:
            perf_div = changes.find('div', class_=['perf positive yf-12wncuy', 'perf negative yf-12wncuy'])
            sign = '+' if 'positive' in perf_div['class'] else '-'
            data.append(sign + perf_div.text)
        return data

    async def parse_industries(industries_soup):
        data = []
        industries = industries_soup.find_all('tr', 'yf-152j1g3')
        for industry in industries[1:]:  # Skip the first row
            industry_name_tag = industry.find('td', class_='name yf-152j1g3')
            market_weight_tag = industry.find('span', class_='yf-152j1g3')
            if industry_name_tag and market_weight_tag:
                industry_name = industry_name_tag.text
                if industry_name.startswith(f"{sector_name} - "):
                    industry_name = industry_name[len(f"{sector_name} - "):]
                market_weight = market_weight_tag.text
                market_weight_rounded = f"{float(market_weight.strip('%')):.2f}%"
                data.append(f"{industry_name}:{market_weight_rounded}")
        return data

    async def parse_companies(companies_soup):
        company_symbols = companies_soup.find_all('span', 'symbol yf-ravs5v', limit=10)
        return [symbol.text for symbol in company_symbols]

    soup = BeautifulSoup(html, 'lxml', parse_only=SoupStrainer(['div', 'section', 'tr', 'span']))

    info_task = parse_info(soup)
    returns_task = parse_returns(soup)
    industries_task = parse_industries(soup)
    companies_task = parse_companies(soup)

    info, returns, industries, symbols = await asyncio.gather(info_task, returns_task, industries_task, companies_task)

    data = returns + info + industries

    day_return = data[0].strip()
    ytd_return = data[1].strip()
    year_return = data[2].strip()
    three_year_return = data[3].strip()
    five_year_return = data[4].strip()
    market_cap = data[5]
    market_weight = data[6]
    num_industries = int(data[7])
    num_companies = int(data[8])
    industries = data[10:]

    quotes = await scrape_simple_quotes(symbols)

    return MarketSectorDetails(
        sector=sector_name,
        day_return=day_return,
        ytd_return=ytd_return,
        year_return=year_return,
        three_year_return=three_year_return,
        five_year_return=five_year_return,
        market_cap=market_cap,
        market_weight=market_weight,
        industries=num_industries,
        companies=num_companies,
        top_industries=industries,
        top_companies=quotes
    )


@cache(expire=300, after_market_expire=3600)
async def get_sectors() -> List[MarketSector]:
    async with AsyncClient(http2=True, max_redirects=5) as client:
        tasks = []
        for sector, url in urls.items():
            tasks.append((sector.value, client.get(url, headers=headers)))
        responses = await asyncio.gather(*[task for _, task in tasks])

    sectors = []
    for (sector, _), response in zip(tasks, responses):
        html = response.text
        sector_data = await parse_sector(html, sector)
        sectors.append(sector_data)
    return sectors


@cache(expire=60, after_market_expire=600)
async def get_sector_for_symbol(symbol: str) -> MarketSector:
    ticker = Ticker(symbol)
    profile = ticker.asset_profile
    sector = profile[symbol]['sector'] if 'sector' in profile[symbol] else None
    if not sector:
        raise HTTPException(status_code=404, detail=f"Sector for {symbol} not found.")

    url = urls[Sector(sector)]
    async with AsyncClient(http2=True, max_redirects=5) as client:
        response = await client.get(url, headers=headers)
        html = response.text

    sector = await parse_sector(html, sector)
    return sector


@cache(expire=300, after_market_expire=3600)
async def get_sector_details(sector: Sector) -> MarketSectorDetails:
    url = urls[sector]
    async with AsyncClient(http2=True, max_redirects=5) as client:
        response = await client.get(url, headers=headers)
        html = response.text
    sector = await parse_sector_details(html, sector.value)

    return sector
