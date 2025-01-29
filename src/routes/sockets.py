import asyncio
from datetime import datetime

import pytz
from fastapi import APIRouter, Depends
from starlette.websockets import WebSocket, WebSocketDisconnect

from src.connections import RedisConnectionManager
from src.di import get_global_rate_limit_manager
from src.market import MarketSchedule
from src.schemas import SimpleQuote
from src.services import (
    scrape_quotes, scrape_similar_quotes, scrape_actives,
    scrape_news_for_quote, scrape_losers, scrape_gainers,
    scrape_simple_quotes, scrape_indices, scrape_general_news,
    get_sectors, get_sector_for_symbol
)

router = APIRouter()


async def validate_websocket(websocket: WebSocket) -> tuple[bool, dict]:
    """
    Backwards compatible wrapper for websocket validation
    """
    rate_limit_manager = get_global_rate_limit_manager()
    return await rate_limit_manager.validate_websocket(websocket)


def safe_convert_to_dict(items, default=None):
    """
    Convert items to dictionaries, handling exceptions and type variations.

    :param items: List of items to convert
    :param default: Default value to use if conversion fails
    :return: List of dictionaries or default values
    """
    if default is None:
        default = []

    try:
        return [
            item if isinstance(item, dict) else
            (item.dict() if hasattr(item, 'dict') else default)
            for item in items
        ]
    except Exception:
        return default


async def handle_websocket_connection(
        websocket: WebSocket,
        channel: str,
        data_fetcher: callable,
        connection_manager: RedisConnectionManager
):
    """
    A generalized WebSocket connection handler.

    :param websocket: The WebSocket connection
    :param channel: The channel name for publishing data
    :param data_fetcher: Async function to fetch data
    :param connection_manager: Connection manager instance
    """
    is_valid, metadata = await validate_websocket(websocket)
    print(is_valid, metadata)
    if not is_valid:
        return

    await websocket.accept()

    async def fetch_data():
        """
        Continuously fetches and publishes data.
        """
        while True:
            try:
                result = await data_fetcher()
                await connection_manager.publish(result, channel)
                await asyncio.sleep(5)
            except WebSocketDisconnect:
                await connection_manager.disconnect(websocket, channel)
                break

    # Starts the connection and fetches the initial data
    if websocket not in connection_manager.active_connections.get(channel, []):
        initial_result = await data_fetcher()

        # Add metadata if available
        if metadata:
            metadata.update(initial_result)
            initial_result = metadata

        try:
            await websocket.send_json(initial_result)
        except WebSocketDisconnect:
            return

        await connection_manager.connect(websocket, channel, fetch_data)

    # Keep the connection alive
    try:
        while True:
            await websocket.receive_text()
    except WebSocketDisconnect:
        await connection_manager.disconnect(websocket, channel)


@router.websocket("/profile/{symbol}")
async def websocket_profile(
        websocket: WebSocket,
        symbol: str,
        connection_manager: RedisConnectionManager = Depends(RedisConnectionManager)
):
    async def get_profile():
        """
        Fetches the profile data for a symbol.
        """
        quotes_task = scrape_quotes([symbol])
        similar_quotes_task = scrape_similar_quotes(symbol)
        sector_performance_task = get_sector_for_symbol(symbol)
        news_task = scrape_news_for_quote(symbol)

        quotes, similar_quotes, sector_performance, news = await asyncio.gather(
            quotes_task, similar_quotes_task, sector_performance_task, news_task, return_exceptions=True
        )

        quotes = safe_convert_to_dict(quotes)
        similar_quotes = safe_convert_to_dict(similar_quotes)

        # Handle sector performance conversion
        if isinstance(sector_performance, Exception):
            sector_performance = None
        elif not isinstance(sector_performance, dict):
            sector_performance = sector_performance.dict()

        news = safe_convert_to_dict(news)

        return {
            "quote": quotes[0] if quotes else None,
            "similar": similar_quotes,
            "performance": sector_performance,
            "news": news
        }

    channel = f"profile:{symbol}"
    await handle_websocket_connection(websocket, channel, get_profile, connection_manager)


@router.websocket("/quotes")
async def websocket_quotes(
        websocket: WebSocket,
        connection_manager: RedisConnectionManager = Depends(RedisConnectionManager)
):
    is_valid, metadata = await validate_websocket(websocket)
    if not is_valid:
        return
    await websocket.accept()
    try:
        channel = await websocket.receive_text()
        symbols = list(set(symbol.upper() for symbol in channel.split(",")))

        async def get_quotes(symbols):
            """
            Fetches quotes for a list of symbols.
            """
            result = await scrape_simple_quotes(symbols)
            quotes = []
            for quote in result:
                if not isinstance(quote, SimpleQuote):
                    quotes.append(quote)
                    continue

                quote_dict = {
                    "symbol": quote.symbol,
                    "name": quote.name,
                    "price": str(quote.price),
                    "change": quote.change,
                    "percentChange": quote.percent_change
                }

                # Add optional fields if they exist
                if quote.pre_market_price is not None:
                    quote_dict["preMarketPrice"] = str(quote.pre_market_price)

                if quote.after_hours_price is not None:
                    quote_dict["afterHoursPrice"] = str(quote.after_hours_price)

                if quote.logo is not None:
                    quote_dict["logo"] = quote.logo

                quotes.append(quote_dict)

            return quotes

        async def fetch_data():
            """
            Fetches quotes every 10 seconds.
            """
            while True:
                result = await get_quotes(symbols)
                await connection_manager.publish(result, channel)
                await asyncio.sleep(10)

        # Starts the connection and fetches the initial data
        if websocket not in connection_manager.active_connections.get(channel, []):
            initial_result = await get_quotes(symbols)
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
async def websocket_market(
        websocket: WebSocket,
        connection_manager: RedisConnectionManager = Depends(RedisConnectionManager)
):
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

        return {
            "actives": safe_convert_to_dict(actives),
            "gainers": safe_convert_to_dict(gainers),
            "losers": safe_convert_to_dict(losers),
            "indices": safe_convert_to_dict(indices),
            "headlines": safe_convert_to_dict(news),
            "sectors": safe_convert_to_dict(sectors)
        }

    channel = "market"
    await handle_websocket_connection(websocket, channel, get_market_info, connection_manager)


@router.websocket("/hours")
async def market_status_websocket(
        websocket: WebSocket,
        connection_manager: RedisConnectionManager = Depends(RedisConnectionManager),
        market_schedule: MarketSchedule = Depends(MarketSchedule)
):
    async def get_market_status_info():
        """
        Fetches the market status information.
        """
        current_status, reason = market_schedule.get_market_status()
        return {
            "status": current_status,
            "reason": reason,
            "timestamp": datetime.now(pytz.UTC).isoformat()
        }

    channel = "hours"
    await handle_websocket_connection(websocket, channel, get_market_status_info, connection_manager)
