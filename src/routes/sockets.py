import asyncio

from fastapi import APIRouter
from starlette.websockets import WebSocket, WebSocketDisconnect

from src.connections import RedisConnectionManager
from src.schemas import SimpleQuote
from src.services import scrape_quotes, scrape_similar_stocks, scrape_actives, \
    scrape_news_for_quote, scrape_losers, scrape_gainers, scrape_simple_quotes, scrape_indices
from src.services.get_sectors import get_sector_for_symbol, get_sectors

router = APIRouter()


@router.websocket("/profile/{symbol}")
async def websocket_profile(websocket: WebSocket, symbol: str):
    connection_manager = RedisConnectionManager()
    await websocket.accept()
    channel = f"profile:{symbol}"

    async def fetch_data():
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
            sector_performance = sector_performance if isinstance(sector_performance, dict) else sector_performance.dict()
            news = [headline if isinstance(headline, dict) else headline.dict() for headline in news]

            result = {
                "quote": quotes[0],
                "similar": similar_stocks,
                "performance": sector_performance,
                "news": news
            }
            await connection_manager.publish(result, channel)
            await asyncio.sleep(10)

    if websocket not in connection_manager.active_connections.get(channel, []):
        await connection_manager.connect(websocket, channel, fetch_data)

    # Keep the connection alive
    try:
        while True:
            await websocket.receive_text()
    except WebSocketDisconnect:
        await connection_manager.disconnect(websocket, channel)


@router.websocket("/quotes")
async def websocket_quotes(websocket: WebSocket):
    connection_manager = RedisConnectionManager()
    await websocket.accept()

    channel = await websocket.receive_text()

    # Channel is a comma-separated string of symbols
    symbols = list(set(channel.split(",")))

    async def fetch_data():
        while True:
            result = await scrape_simple_quotes(symbols)
            formatted_result = [
                {
                    "symbol": quote.symbol,
                    "name": quote.name,
                    "price": str(quote.price),
                    "change": quote.change,
                    "percentChange": quote.percent_change,
                    "logo": quote.logo
                } if isinstance(quote, SimpleQuote) else quote
                for quote in result
            ]
            await connection_manager.publish(formatted_result, channel)
            await asyncio.sleep(10)

    if websocket not in connection_manager.active_connections.get(channel, []):
        await connection_manager.connect(websocket, channel, fetch_data)

    # Keep the connection alive
    try:
        while True:
            await websocket.receive_text()
    except WebSocketDisconnect:
        await connection_manager.disconnect(websocket, channel)


@router.websocket("/market")
async def websocket_market(websocket: WebSocket):
    connection_manager: RedisConnectionManager = RedisConnectionManager()

    await websocket.accept()
    channel = "market"

    async def fetch_data():
        while True:
            actives_task = scrape_actives()
            gainers_task = scrape_gainers()
            losers_task = scrape_losers()
            indices_task = scrape_indices()
            sectors_task = get_sectors()

            actives, gainers, losers, indices, sectors = await asyncio.gather(
                actives_task, gainers_task, losers_task, indices_task, sectors_task
            )

            actives = [active if isinstance(active, dict) else active.dict() for active in actives]
            gainers = [gainer if isinstance(gainer, dict) else gainer.dict() for gainer in gainers]
            losers = [loser if isinstance(loser, dict) else loser.dict() for loser in losers]
            indices = [index if isinstance(index, dict) else index.dict() for index in indices]
            sectors = [sector if isinstance(sector, dict) else sector.dict() for sector in sectors]

            result = {
                "actives": actives,
                "gainers": gainers,
                "losers": losers,
                "indices": indices,
                "sectors": sectors
            }

            await connection_manager.publish(result, channel)
            await asyncio.sleep(10)

    if websocket not in connection_manager.active_connections.get(channel, []):
        await connection_manager.connect(websocket, channel, fetch_data)

    # Keep the connection alive
    try:
        while True:
            await websocket.receive_text()
    except WebSocketDisconnect:
        await connection_manager.disconnect(websocket, channel)
