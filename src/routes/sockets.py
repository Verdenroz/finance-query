import asyncio

from fastapi import APIRouter, WebSocket, WebSocketDisconnect

from src.services import scrape_quotes, scrape_simple_quotes, scrape_similar_stocks, scrape_actives, \
    scrape_news_for_quote, scrape_losers, scrape_gainers
from src.services.get_sectors import get_sector_for_symbol, get_sectors

router = APIRouter()


@router.websocket("/profile/{symbol}/ws")
async def websocket_profile(
        websocket: WebSocket,
        symbol: str
):
    await websocket.accept()
    try:
        while True:
            quotes_task = scrape_quotes([symbol])
            similar_stocks_task = scrape_similar_stocks(symbol)
            sector_performance_task = get_sector_for_symbol(symbol)
            news_task = scrape_news_for_quote(symbol)

            quotes, similar_stocks, sector_performance, news = await asyncio.gather(
                quotes_task, similar_stocks_task, sector_performance_task, news_task
            )

            quotes = [quote if isinstance(quote, dict) else quote.dict() for quote in quotes]
            similar_stocks = [similar if isinstance(similar, dict) else similar.dict() for similar in similar_stocks]
            sector_performance = [sector if isinstance(sector, dict) else sector.dict() for sector in
                                  sector_performance]
            news = [headline if isinstance(headline, dict) else headline.dict() for headline in news]

            result = {
                "quote": quotes,
                "similar": similar_stocks,
                "performance": sector_performance,
                "news": news
            }

            await websocket.send_json(result)
            await asyncio.sleep(5)
    except WebSocketDisconnect:
        await websocket.close()


@router.websocket("/ws/quotes")
async def websocket_quotes(websocket: WebSocket):
    await websocket.accept()
    try:
        data = await websocket.receive_text()
        symbols = list(set(data.upper().replace('"', '').split(',')))
        while True:
            quotes = await scrape_simple_quotes(symbols)
            quotes = [quote if isinstance(quote, dict) else quote.dict() for quote in quotes]
            await websocket.send_json(quotes)
            await asyncio.sleep(5)
    except WebSocketDisconnect:
        await websocket.close()


@router.websocket("/ws/market")
async def websocket_market(websocket: WebSocket):
    await websocket.accept()
    try:
        while True:
            actives_task = scrape_actives()
            gainers_task = scrape_gainers()
            losers_task = scrape_losers()
            sectors_task = get_sectors()

            actives, gainers, losers, sectors = await asyncio.gather(
                actives_task, gainers_task, losers_task, sectors_task)

            actives = [active if isinstance(active, dict) else active.dict() for active in actives]
            gainers = [gainer if isinstance(gainer, dict) else gainer.dict() for gainer in gainers]
            losers = [loser if isinstance(loser, dict) else loser.dict() for loser in losers]
            sectors = [sector if isinstance(sector, dict) else sector.dict() for sector in sectors]

            result = {
                "actives": actives,
                "gainers": gainers,
                "losers": losers,
                "sectors": sectors
            }
            await websocket.send_json(result)
            await asyncio.sleep(10)

    except WebSocketDisconnect:
        await websocket.close()
