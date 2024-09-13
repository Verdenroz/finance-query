import asyncio

from fastapi import APIRouter, Query, HTTPException, Security
from fastapi.responses import StreamingResponse
from fastapi.security import APIKeyHeader
from orjson import orjson

from src.services import scrape_simple_quotes

router = APIRouter()


async def quotes_generator(symbols: list[str]):
    while True:
        quotes = await scrape_simple_quotes(symbols)
        quotes = [quote if isinstance(quote, dict) else quote.dict() for quote in quotes]
        data = orjson.dumps(quotes).decode('utf-8')
        yield f"quote: {data}\n\n"
        await asyncio.sleep(10)


@router.get(
    "/stream/quotes",
    summary="Stream stock quotes by SSE",
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    tags=["Server Sent Events (SSE)"]
)
async def stream_quotes(
        symbols: str = Query(..., title="Symbols", description="Comma-separated list of stock symbols")):
    if not symbols:
        raise HTTPException(status_code=400, detail="Symbols parameter is required")

    symbols = list(set(symbols.upper().replace(' ', '').split(',')))
    return StreamingResponse(quotes_generator(symbols), media_type="text/event-stream")
