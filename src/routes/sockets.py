import asyncio

from fastapi import APIRouter
from starlette.websockets import WebSocket, WebSocketDisconnect

from src.connections import RedisConnectionManager
from src.schemas import SimpleQuote
from src.security import validate_websocket
from src.services import scrape_quotes, scrape_similar_stocks, scrape_actives, \
    scrape_news_for_quote, scrape_losers, scrape_gainers, scrape_simple_quotes, scrape_indices, scrape_general_news
from src.services.get_sectors import get_sector_for_symbol, get_sectors

router = APIRouter()
connection_manager = RedisConnectionManager()


@router.websocket("/profile/{symbol}")
async def websocket_profile(websocket: WebSocket, symbol: str):
    is_valid, metadata = await validate_websocket(websocket)
    if not is_valid:
        return

    await websocket.accept()
    channel = f"profile:{symbol}"

    async def get_profile():
        """
        Fetches the profile data for a symbol.
        """
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

        return {
            "quote": quotes[0],
            "similar": similar_stocks,
            "performance": sector_performance,
            "news": news
        }

    async def fetch_data():
        """
        Fetches the profile data every 10 seconds.
        """
        while True:
            result = await get_profile()
            await connection_manager.publish(result, channel)
            await asyncio.sleep(10)

    # Starts the connection and fetches the initial data
    if websocket not in connection_manager.active_connections.get(channel, []):
        initial_result = await get_profile()
        if metadata:
            metadata.update(initial_result)
            initial_result = metadata
        try:
            await websocket.send_json(initial_result)
        except WebSocketDisconnect:
            # If the client disconnects before the initial data is sent, return
            return
        await connection_manager.connect(websocket, channel, fetch_data)

    # Keep the connection alive
    try:
        while True:
            await websocket.receive_text()
    except WebSocketDisconnect:
        await connection_manager.disconnect(websocket, channel)


@router.websocket("/quotes")
async def websocket_quotes(websocket: WebSocket):
    is_valid, metadata = await validate_websocket(websocket)
    if not is_valid:
        return
    await websocket.accept()
    try:
        channel = await websocket.receive_text()
        symbols = list(set(channel.split(",")))

        async def get_quotes():
            """
            Fetches quotes for a list of symbols.
            """
            result = await scrape_simple_quotes(symbols)
            return [
                {
                    "symbol": quote.symbol,
                    "name": quote.name,
                    "price": str(quote.price),
                    "afterHoursPrice": str(quote.after_hours_price),
                    "change": quote.change,
                    "percentChange": quote.percent_change,
                    "logo": quote.logo
                } if isinstance(quote, SimpleQuote) else quote for quote in result]

        async def fetch_data():
            """
            Fetches quotes every 10 seconds.
            """
            while True:
                result = await get_quotes()
                await connection_manager.publish(result, channel)
                await asyncio.sleep(10)

        # Starts the connection and fetches the initial data
        if websocket not in connection_manager.active_connections.get(channel, []):
            initial_result = await get_quotes()
            if metadata:
                initial_result.insert(0, metadata)
            try:
                await websocket.send_json(initial_result)
            except WebSocketDisconnect:
                # If the client disconnects before the initial data is sent, return
                return
            await connection_manager.connect(websocket, channel, fetch_data)

        # Keep the connection alive
        try:
            while True:
                await websocket.receive_text()
        except WebSocketDisconnect:
            await connection_manager.disconnect(websocket, channel)

    except WebSocketDisconnect:
        # If the client disconnects before the channel is received return,
        return


@router.websocket("/market")
async def websocket_market(websocket: WebSocket):
    is_valid, metadata = await validate_websocket(websocket)
    if not is_valid:
        return

    await websocket.accept()
    channel = "market"

    async def get_market_info():
        """
        Fetches market information.
        """
        actives_task = scrape_actives()
        gainers_task = scrape_gainers()
        losers_task = scrape_losers()
        indices_task = scrape_indices()
        news_task = scrape_general_news()
        sectors_task = get_sectors()

        actives, gainers, losers, indices, news, sectors = await asyncio.gather(
            actives_task, gainers_task, losers_task, indices_task, news_task, sectors_task
        )

        actives = [active if isinstance(active, dict) else active.dict() for active in actives]
        gainers = [gainer if isinstance(gainer, dict) else gainer.dict() for gainer in gainers]
        losers = [loser if isinstance(loser, dict) else loser.dict() for loser in losers]
        indices = [index if isinstance(index, dict) else index.dict() for index in indices]
        headlines = [headline if isinstance(headline, dict) else headline.dict() for headline in news]
        sectors = [sector if isinstance(sector, dict) else sector.dict() for sector in sectors]

        return {
            "actives": actives,
            "gainers": gainers,
            "losers": losers,
            "indices": indices,
            "headlines": headlines,
            "sectors": sectors
        }

    async def fetch_data():
        """
        Fetches market information every 10 seconds.
        """
        while True:
            try:
                result = await get_market_info()
                await connection_manager.publish(result, channel)
                await asyncio.sleep(10)
            except WebSocketDisconnect:
                await connection_manager.disconnect(websocket, channel)
                break

    # Starts the connection and fetches the initial data
    if websocket not in connection_manager.active_connections.get(channel, []):
        initial_result = await get_market_info()
        if metadata:
            metadata.update(initial_result)
            initial_result = metadata
        try:
            await websocket.send_json(initial_result)
        except WebSocketDisconnect:
            # If the client disconnects before the initial data is sent, return
            return

        await connection_manager.connect(websocket, channel, fetch_data)

    # Keep the connection alive
    try:
        while True:
            await websocket.receive_text()
    except WebSocketDisconnect:
        await connection_manager.disconnect(websocket, channel)
