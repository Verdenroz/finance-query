import asyncio

from fastapi import HTTPException
from lxml import etree

from src.models import MarketSector, MarketSectorDetails


async def parse_sector(html: str, sector: str) -> MarketSector:
    """
    Parses sector data from the HTML response.
    :param html: the HTML content
    :param sector: the sector name
    :return: a MarketSector object
    """
    try:
        tree = etree.HTML(html)
        container_xpath = "/html/body/div[2]/main/section/section/section/section/section[1]/section[2]/div"
        card_xpath = ".//section"
        sector_perf_xpath = './/div[contains(@class, "perf")]/text()'
        perf_class_xpath = './/div/div[contains(@class, "perf")]/@class'

        container = tree.xpath(container_xpath)[0]
        cards = container.xpath(card_xpath)
        performance_data = []
        for card in cards:
            sector_perf = card.xpath(sector_perf_xpath)[0].strip()
            perf_class = card.xpath(perf_class_xpath)[0].strip()

            # Determine sign based on class containing 'positive' or 'negative'
            sign = "+" if "positive" in perf_class else "-" if "negative" in perf_class else ""
            sector_perf = f"{sign}{sector_perf}"
            performance_data.append(sector_perf)

        return MarketSector(
            sector=sector,
            dayReturn=performance_data[0],
            ytdReturn=performance_data[1],
            yearReturn=performance_data[2],
            threeYearReturn=performance_data[3],
            fiveYearReturn=performance_data[4],
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to parse sector data: {e}") from e


async def parse_sector_details(html: str, sector_name: str) -> MarketSectorDetails:
    """
    Parses detailed sector data from the HTML response.
    :param html: the HTML content
    :param sector_name: the sector name
    :return: the MarketSectorDetails object
    """

    async def parse_info(tree: etree.ElementTree) -> list[str]:
        """
        Parses the market cap, market weight, num. industries, and num. companies from the HTML tree.
        :param tree: the lxml tree
        :return: a list of the parsed data
        """
        container_xpath = "/html/body/div[2]/main/section/section/section/section/section[1]/div/section/div[2]/div[2]"
        market_cap_xpath = ".//div[1]/div[2]/text()"
        market_weight_xpath = ".//div[2]/div[2]/text()"
        industries_xpath = ".//div[3]/div[2]/text()"
        companies_xpath = ".//div[4]/div[2]/text()"

        container = tree.xpath(container_xpath)[0]
        market_cap_text = container.xpath(market_cap_xpath)[0].strip()
        market_weight_text = container.xpath(market_weight_xpath)[0].strip()
        industries_text = container.xpath(industries_xpath)[0].strip()
        companies_text = container.xpath(companies_xpath)[0].strip()

        return [market_cap_text, market_weight_text, industries_text, companies_text]

    async def parse_returns(tree: etree.ElementTree) -> list[str]:
        """
        Parses the returns data from the HTML tree.
        :param tree: the lxml tree
        :return: the returns data as a list
        """
        container_xpath = "/html/body/div[2]/main/section/section/section/section/section[1]/section[2]/div"
        card_xpath = ".//section"
        sector_perf_xpath = './/div[div[text()="Sector"]]/div[2]/text()'
        positive_xpath = './/div[contains(@class, "positive")]/text()'
        negative_xpath = './/div[contains(@class, "negative")]/text()'

        container = tree.xpath(container_xpath)[0]
        cards = container.xpath(card_xpath)
        performance_data = []
        for card in cards:
            sector_perf = card.xpath(sector_perf_xpath)[0].strip()
            is_positive = bool(card.xpath(positive_xpath))
            is_negative = bool(card.xpath(negative_xpath))
            if is_positive:
                sector_perf = f"+{sector_perf}"
            elif is_negative:
                sector_perf = f"-{sector_perf}"
            performance_data.append(sector_perf)

        return performance_data

    async def parse_industries(tree: etree.ElementTree) -> list[str]:
        """
        Parses the top industries from the HTML tree.
        :param tree: the lxml tree
        :return: the top industries as a list
        """
        container_xpath = "/html/body/div[2]/main/section/section/section/section/section[2]/div/div/div[1]/div/div[2]/table/tbody/tr"
        industry_name_xpath = "./td[1]/text()"
        market_weight_xpath = "./td[2]/span/text()"

        rows = tree.xpath(container_xpath)
        parsed_industries = []

        for row in rows:
            industry_name = row.xpath(industry_name_xpath)[0].strip()
            market_weight = row.xpath(market_weight_xpath)[0].strip()
            parsed_industries.append(f"{industry_name}: {market_weight}")

        return parsed_industries

    async def parse_companies(tree: etree.ElementTree) -> list[str]:
        """
        Parses the top companies from the HTML tree.
        :param tree: the lxml tree
        :return: the top companies as a list
        """
        container_xpath = "/html/body/div[2]/main/section/section/section/section/section[3]/div[2]/div/table/tbody/tr"
        symbol_xpath = "./td[1]/span/div/span/a/div/span[1]/text()"
        rows = tree.xpath(container_xpath)
        companies = []

        for row in rows:
            symbol = row.xpath(symbol_xpath)[0].strip()
            companies.append(symbol)

        return companies

    try:
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
            dayReturn=day_return,
            ytdReturn=ytd_return,
            yearReturn=year_return,
            threeYearReturn=three_year_return,
            fiveYearReturn=five_year_return,
            marketCap=market_cap,
            marketWeight=market_weight,
            industries=num_industries,
            companies=num_companies,
            topIndustries=industries,
            topCompanies=companies,
        )
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to parse sector details: {e}") from e
