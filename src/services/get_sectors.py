import asyncio
from typing import List

from fastapi import HTTPException
from lxml import etree
from yahooquery import Ticker

from src.redis import cache
from src.schemas import MarketSector
from src.schemas.sector import Sector, MarketSectorDetails
from src.utils import fetch

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

@cache(expire=300, after_market_expire=3600)
async def get_sectors() -> List[MarketSector]:
    tasks = []
    for sector, url in urls.items():
        tasks.append((sector.value, fetch(url)))
    responses = await asyncio.gather(*[task for _, task in tasks])

    sectors = []
    for (sector, _), html in zip(tasks, responses):
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
    html = await fetch(url)

    sector = await parse_sector(html, sector)
    return sector


@cache(expire=300, after_market_expire=3600)
async def get_sector_details(sector: Sector) -> MarketSectorDetails:
    url = urls[sector]
    html = await fetch(url)
    sector = await parse_sector_details(html, sector.value)

    return sector


async def parse_sector(html: str, sector: str) -> MarketSector:
    tree = etree.HTML(html)
    container_xpath = '/html/body/div[2]/main/section/section/section/article/section[1]/section[2]'
    card_xpath = './/section'
    sector_perf_xpath = './/div[div[text()="Sector"]]/div[2]/text()'

    container = tree.xpath(container_xpath)[0]
    cards = container.xpath(card_xpath)
    performance_data = []
    for card in cards:
        sector_perf = card.xpath(sector_perf_xpath)[0].strip()
        performance_data.append(sector_perf)

    return MarketSector(
        sector=sector,
        day_return=performance_data[0],
        ytd_return=performance_data[1],
        year_return=performance_data[2],
        three_year_return=performance_data[3],
        five_year_return=performance_data[4]
    )


async def parse_sector_details(html: str, sector_name: str) -> MarketSectorDetails:
    async def parse_info(tree: etree.ElementTree):
        container_xpath = '/html/body/div[2]/main/section/section/section/article/section[1]/div/section/div[2]/div[2]'
        market_cap_xpath = './/div[1]/div[2]/text()'
        market_weight_xpath = './/div[2]/div[2]/text()'
        industries_xpath = './/div[3]/div[2]/text()'
        companies_xpath = './/div[4]/div[2]/text()'

        container = tree.xpath(container_xpath)[0]
        market_cap_text = container.xpath(market_cap_xpath)[0].strip()
        market_weight_text = container.xpath(market_weight_xpath)[0].strip()
        industries_text = container.xpath(industries_xpath)[0].strip()
        companies_text = container.xpath(companies_xpath)[0].strip()

        return [market_cap_text, market_weight_text, industries_text, companies_text]

    async def parse_returns(tree: etree.ElementTree):
        container_xpath = '/html/body/div[2]/main/section/section/section/article/section[1]/section[2]'
        card_xpath = './/section'
        sector_perf_xpath = './/div[div[text()="Sector"]]/div[2]/text()'

        container = tree.xpath(container_xpath)[0]
        cards = container.xpath(card_xpath)

        performance_data = []
        for card in cards:
            sector_perf = card.xpath(sector_perf_xpath)[0].strip()
            performance_data.append(sector_perf)

        return performance_data

    async def parse_industries(tree: etree.ElementTree):
        container_xpath = '/html/body/div[2]/main/section/section/section/article/section[2]/div/div/div[1]/div/div[2]/table/tbody/tr'
        industry_name_xpath = './td[1]/text()'
        market_weight_xpath = './td[2]/span/text()'

        rows = tree.xpath(container_xpath)
        parsed_industries = []

        for row in rows:
            industry_name = row.xpath(industry_name_xpath)[0].strip()
            market_weight = row.xpath(market_weight_xpath)[0].strip()
            parsed_industries.append(f"{industry_name}: {market_weight}")

        return parsed_industries

    async def parse_companies(tree: etree.ElementTree):
        container_xpath = '/html/body/div[2]/main/section/section/section/article/section[3]/div[2]/div/table/tbody/tr'
        symbol_xpath = './td[1]//a/div/span[1]/text()'

        rows = tree.xpath(container_xpath)
        companies = []

        for row in rows:
            symbol = row.xpath(symbol_xpath)[0].strip()
            companies.append(symbol)

        return companies

    tree = etree.HTML(html)
    info_task = parse_info(tree)
    returns_task = parse_returns(tree)
    industries_task = parse_industries(tree)
    companies_task = parse_companies(tree)

    info, returns, industries, companies = await asyncio.gather(info_task, returns_task, industries_task, companies_task)

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
        top_companies=companies
    )
