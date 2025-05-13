import asyncio
from datetime import datetime

import pytz
from fastapi import APIRouter
from starlette.websockets import WebSocket, WebSocketDisconnect

from src.connections import ConnectionManager, RedisConnectionManager
from utils.dependencies import FinanceClient, Schedule, WebsocketConnectionManager
from src.models import MarketSector, SimpleQuote
from src.security import validate_websocket
from src.services import (
    get_actives,
    get_gainers,
    get_indices,
    get_losers,
    get_quotes,
    get_sector_for_symbol,
    get_sectors,
    get_similar_quotes,
    get_simple_quotes,
    scrape_general_news,
    scrape_news_for_quote,
)

router = APIRouter()

# Refresh interval for fetching data
REFRESH_INTERVAL = 5


def safe_convert_to_dict(items: list, default=None):
    """
    Convert items to dictionaries, handling exceptions and type variations.

    :param items: List of items to convert (dicts or Pydantic models)
    :param default: Default value to use if conversion fails
    :return: List of dictionaries or default values
    """
    if default is None:
        default = []

    if not isinstance(items, list | tuple) or items is None:
        return default

    return [item if isinstance(item, dict) else item.model_dump() if hasattr(item, "model_dump") else default for item in items]


async def handle_websocket_connection(
    websocket: WebSocket,
    channel: str,
    data_fetcher: callable,
    connection_manager: RedisConnectionManager | ConnectionManager,
):
    """
    A generalized WebSocket connection handler.

    :param websocket: The WebSocket connection
    :param channel: The channel name for publishing data
    :param data_fetcher: Async function to fetch data
    :param connection_manager: Connection manager instance
    """
    is_valid, metadata = await validate_websocket(websocket=websocket)
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
                if isinstance(connection_manager, RedisConnectionManager):
                    await connection_manager.publish(result, channel)
                else:
                    await connection_manager.broadcast(channel, result)
                await asyncio.sleep(REFRESH_INTERVAL)
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
    connection_manager: WebsocketConnectionManager,
    finance_client: FinanceClient,
):
    async def get_profile():
        """
        Fetches the profile data for a symbol.
        """
        quotes_task = get_quotes(finance_client, [symbol])
        similar_quotes_task = get_similar_quotes(finance_client, symbol)
        sector_performance_task = get_sector_for_symbol(finance_client, symbol)
        news_task = scrape_news_for_quote(symbol)

        quotes, similar_quotes, sector_performance, news = await asyncio.gather(
            quotes_task, similar_quotes_task, sector_performance_task, news_task, return_exceptions=True
        )

        quotes = safe_convert_to_dict(quotes)
        similar_quotes = safe_convert_to_dict(similar_quotes)
        news = safe_convert_to_dict(news)

        return {
            "quote": quotes[0] if quotes else None,
            "similar": similar_quotes,
            "sectorPerformance": sector_performance.dict() if isinstance(sector_performance, MarketSector) else None,
            "news": news,
        }

    channel = f"profile:{symbol}"
    await handle_websocket_connection(websocket, channel, get_profile, connection_manager)


@router.websocket("/quotes")
async def websocket_quotes(
    websocket: WebSocket,
    connection_manager: WebsocketConnectionManager,
    finance_client: FinanceClient,
):
    is_valid, metadata = await validate_websocket(websocket=websocket)
    if not is_valid:
        return
    await websocket.accept()
    try:
        channel = await websocket.receive_text()
        symbols = list({symbol.upper() for symbol in channel.split(",")})

        async def get_request_symbols() -> list[dict]:
            """
            Fetches quotes for a list of symbols.
            """
            result = await get_simple_quotes(finance_client, symbols)
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
                    "percentChange": quote.percent_change,
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
            Fetches quotes every 5 seconds.
            """
            while True:
                result = await get_request_symbols()
                if isinstance(connection_manager, RedisConnectionManager):
                    await connection_manager.publish(result, channel)
                else:
                    await connection_manager.broadcast(channel, result)
                await asyncio.sleep(REFRESH_INTERVAL)

        # Starts the connection and fetches the initial data
        if websocket not in connection_manager.active_connections.get(channel, []):
            initial_result = await get_request_symbols()
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
    connection_manager: WebsocketConnectionManager,
    finance_client: FinanceClient,
):
    async def get_market_info():
        """
        Fetches market information.
        """
        actives_task = get_actives()
        gainers_task = get_gainers()
        losers_task = get_losers()
        indices_task = get_indices(finance_client)
        news_task = scrape_general_news()
        sectors_task = get_sectors()

        actives, gainers, losers, indices, news, sectors = await asyncio.gather(actives_task, gainers_task, losers_task, indices_task, news_task, sectors_task)

        return {
            "actives": safe_convert_to_dict(actives),
            "gainers": safe_convert_to_dict(gainers),
            "losers": safe_convert_to_dict(losers),
            "indices": safe_convert_to_dict(indices),
            "headlines": safe_convert_to_dict(news),
            "sectors": safe_convert_to_dict(sectors),
        }

    channel = "market"
    await handle_websocket_connection(websocket, channel, get_market_info, connection_manager)


@router.websocket("/hours")
async def market_status_websocket(
    websocket: WebSocket,
    connection_manager: WebsocketConnectionManager,
    market_schedule: Schedule,
):
    async def get_market_status_info():
        """
        Fetches the market status information.
        """
        current_status, reason = market_schedule.get_market_status()
        return {"status": current_status, "reason": reason, "timestamp": datetime.now(pytz.UTC).isoformat()}

    channel = "hours"
    await handle_websocket_connection(websocket, channel, get_market_status_info, connection_manager)