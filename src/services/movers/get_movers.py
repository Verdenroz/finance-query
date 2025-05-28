from src.models.marketmover import MarketMover, MoverCount
from src.services.movers.fetchers import fetch_movers, scrape_movers
from src.utils.cache import cache
from src.utils.retry import retry


@cache(expire=15, market_closed_expire=3600)
@retry(lambda: scrape_movers("https://finance.yahoo.com/markets/stocks/most-active/?start=0&count=50"))
async def get_actives(count: MoverCount = MoverCount.FIFTY) -> list[MarketMover]:
    """
    Scrape the most active stocks from Yahoo Finance
    """
    api_url = f"https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?count={count.value}&formatted=true&scrIds=MOST_ACTIVES"
    return await fetch_movers(api_url)


@cache(expire=15, market_closed_expire=3600)
@retry(lambda: scrape_movers("https://finance.yahoo.com/markets/stocks/gainers/?start=0&count=50"))
async def get_gainers(count: MoverCount = MoverCount.FIFTY) -> list[MarketMover]:
    """
    Scrape the top gaining stocks from Yahoo Finance
    """
    api_url = f"https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?count={count.value}&formatted=true&scrIds=DAY_GAINERS"
    return await fetch_movers(api_url)


@cache(expire=15, market_closed_expire=3600)
@retry(lambda: scrape_movers("https://finance.yahoo.com/markets/stocks/losers/?start=0&count=50"))
async def get_losers(count: MoverCount = MoverCount.FIFTY) -> list[MarketMover]:
    """
    Scrape the top losing stocks from Yahoo Finance
    """
    api_url = f"https://query1.finance.yahoo.com/v1/finance/screener/predefined/saved?count={count.value}&formatted=true&scrIds=DAY_LOSERS"
    return await fetch_movers(api_url)
