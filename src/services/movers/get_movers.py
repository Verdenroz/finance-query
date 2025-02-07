from src.cache import cache
from src.models.marketmover import MoverCount
from src.services.movers import fetch_movers, scrape_movers


@cache(expire=15, market_closed_expire=3600)
async def scrape_actives(count: MoverCount = MoverCount.FIFTY):
    """
    Scrape the most active stocks from Yahoo Finance
    :return:
    """
    api_url = f'https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?count={count.value}&formatted=true&scrIds=MOST_ACTIVES'
    scrape_url = f'https://finance.yahoo.com/markets/stocks/most-active/?start=0&count={count.value}'
    try:
        return await fetch_movers(api_url)
    except Exception as e:
        print("Error fetching most active stocks:", e)
        return await scrape_movers(scrape_url)


@cache(expire=15, market_closed_expire=3600)
async def scrape_gainers(count: MoverCount = MoverCount.FIFTY):
    """
    Scrape the top gaining stocks from Yahoo Finance
    :return:
    """
    api_url = f'https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?count={count.value}&formatted=true&scrIds=DAY_GAINERS'
    scrape_url = f'https://finance.yahoo.com/markets/stocks/gainers/?start=0&count={count.value}'
    try:
        return await fetch_movers(api_url)
    except Exception as e:
        print("Error fetching gainers:", e)
        return await scrape_movers(scrape_url)


@cache(expire=15, market_closed_expire=3600)
async def scrape_losers(count: MoverCount = MoverCount.FIFTY):
    """
    Scrape the top losing stocks from Yahoo Finance
    """
    api_url = f'https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?count={count.value}&formatted=true&scrIds=DAY_LOSERS'
    scraper_url = f'https://finance.yahoo.com/markets/stocks/losers/?start=0&count={count.value}'
    try:
        return await fetch_movers(api_url)
    except Exception as e:
        print("Error fetching losers:", e)
        return await scrape_movers(scraper_url)
