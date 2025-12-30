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
        container_xpath = '//*[@data-testid="performance-overview"]'
        card_xpath = './/section[@data-testid="card-container"]'
        sector_perf_xpath = './/div[contains(@class, "perf") and not(contains(@class, "perfInfo"))][1]/text()'
        perf_class_xpath = './/div[contains(@class, "perf") and not(contains(@class, "perfInfo"))][1]/@class'

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

    async def parse_info(tree: etree._Element) -> list[str]:
        """
        Parses the market cap, market weight, num. industries, and num. companies from the HTML tree.
        :param tree: the lxml tree
        :return: a list of the parsed data
        """
        # Use data-testid for sector header info
        container_xpath = '//*[@data-testid="sector-header"]'
        container = tree.xpath(container_xpath)[0]

        # Get values by label text
        market_cap = container.xpath(
            './/div[contains(@class, "label") and contains(text(), "Market Cap")]/following-sibling::div[contains(@class, "value")]/text()'
        )
        market_weight = container.xpath(
            './/div[contains(@class, "label") and contains(text(), "Market Weight")]/following-sibling::div[contains(@class, "value")]/text()'
        )
        industries = container.xpath(
            './/div[contains(@class, "label") and contains(text(), "Industries")]/following-sibling::div[contains(@class, "value")]/text()'
        )
        companies = container.xpath(
            './/div[contains(@class, "label") and contains(text(), "Companies")]/following-sibling::div[contains(@class, "value")]/text()'
        )

        return [
            market_cap[0].strip() if market_cap else "",
            market_weight[0].strip() if market_weight else "",
            industries[0].strip() if industries else "",
            companies[0].strip() if companies else "",
        ]

    async def parse_returns(tree: etree._Element) -> list[str]:
        """
        Parses the returns data from the HTML tree.
        :param tree: the lxml tree
        :return: the returns data as a list
        """
        # Use data-testid for performance overview
        container_xpath = '//*[@data-testid="performance-overview"]'
        card_xpath = './/section[@data-testid="card-container"]'
        sector_perf_xpath = './/div[contains(@class, "perf") and not(contains(@class, "perfInfo"))][1]/text()'
        perf_class_xpath = './/div[contains(@class, "perf") and not(contains(@class, "perfInfo"))][1]/@class'

        container = tree.xpath(container_xpath)[0]
        cards = container.xpath(card_xpath)
        performance_data = []
        for card in cards:
            sector_perf_elements = card.xpath(sector_perf_xpath)
            perf_class_elements = card.xpath(perf_class_xpath)

            if sector_perf_elements and perf_class_elements:
                sector_perf = sector_perf_elements[0].strip()
                perf_class = perf_class_elements[0].strip()

                if "positive" in perf_class:
                    sector_perf = f"+{sector_perf}"
                elif "negative" in perf_class:
                    sector_perf = f"-{sector_perf}"
                performance_data.append(sector_perf)

        return performance_data

    async def parse_industries(tree: etree._Element) -> list[str]:
        """
        Parses the top industries from the HTML tree.
        :param tree: the lxml tree
        :return: the top industries as a list
        """
        # Use data-testid for sector listing table
        container_xpath = '//*[@data-testid="sector-listing"]//table/tbody/tr'
        industry_name_xpath = "./td[1]//text()"
        market_weight_xpath = "./td[2]//text()"

        rows = tree.xpath(container_xpath)
        parsed_industries = []

        for row in rows:
            industry_names = row.xpath(industry_name_xpath)
            market_weights = row.xpath(market_weight_xpath)

            if industry_names and market_weights:
                industry_name = "".join(industry_names).strip()
                market_weight = "".join(market_weights).strip()
                if industry_name and market_weight:
                    parsed_industries.append(f"{industry_name}: {market_weight}")

        return parsed_industries

    async def parse_companies(tree: etree._Element) -> list[str]:
        """
        Parses the top companies from the HTML tree.
        :param tree: the lxml tree
        :return: the top companies as a list
        """
        # Use data-testid for largest companies table
        container_xpath = '//*[@data-testid="largest-companies"]//table/tbody/tr'
        symbol_xpath = './/a[@data-testid="table-cell-ticker"]//span[contains(@class, "symbol")]/text()'

        rows = tree.xpath(container_xpath)
        companies = []

        for row in rows:
            symbols = row.xpath(symbol_xpath)
            if symbols:
                symbol = symbols[0].strip()
                companies.append(symbol)

        return companies

    try:
        tree = etree.HTML(html)
        info_task = parse_info(tree)
        returns_task = parse_returns(tree)
        industries_task = parse_industries(tree)
        companies_task = parse_companies(tree)

        info, returns, industries, companies = await asyncio.gather(info_task, returns_task, industries_task, companies_task)

        day_return = returns[0].strip() if len(returns) > 0 else ""
        ytd_return = returns[1].strip() if len(returns) > 1 else ""
        year_return = returns[2].strip() if len(returns) > 2 else ""
        three_year_return = returns[3].strip() if len(returns) > 3 else ""
        five_year_return = returns[4].strip() if len(returns) > 4 else ""
        market_cap = info[0] if len(info) > 0 else ""
        market_weight = info[1] if len(info) > 1 else ""
        num_industries = int(info[2]) if len(info) > 2 and info[2].isdigit() else 0
        num_companies = int(info[3]) if len(info) > 3 and info[3].isdigit() else 0

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
