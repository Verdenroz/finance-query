import asyncio

from fastapi import APIRouter, Query, Security
from fastapi.responses import StreamingResponse
from fastapi.security import APIKeyHeader
from orjson import orjson

from src.schemas import ValidationErrorResponse
from src.services import scrape_simple_quotes

router = APIRouter()


async def quotes_generator(symbols: list[str]):
    """
    Stream simplified quotes by SSE (Server Sent Events) for the given symbols every 10 seconds
    Data is sent in the format of "quote: {json_data}\n\n"
    :param symbols: the list of stock symbols
    :return:
    """
    while True:
        quotes = await scrape_simple_quotes(symbols)
        quotes = [quote if isinstance(quote, dict) else quote.dict() for quote in quotes]
        data = orjson.dumps(quotes).decode('utf-8')
        yield f"quote: {data}\n\n"
        await asyncio.sleep(10)


@router.get(
    path="/stream/quotes",
    summary="Stream stock quotes by SSE",
    description="Stream stock quotes via SSE for the given symbols every 10 seconds. Response format: 'quote: {"
                "json_data}\\n\\n' with text/event-stream content type.",
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    tags=["SSE"],
    responses={
        200: {
            "description": "Stream stock quotes by SSE",
            "content": {
                "text/event-stream": {
                    "example": 'quote: [{"symbol":"NVDA","name":"NVIDIA Corporation","price":"142.62",'
                               '"change":"-4.60","percentChange":"-3.12%",'
                               '"logo":"https://logo.clearbit.com/https://www.nvidia.com"}]\n\n'
                }
            }
        },
        404: {"description": "Symbol not found"},
        422: {
            "model": ValidationErrorResponse,
            "description": "Invalid symbol"
        }
    }
)
async def stream_quotes(symbols: str = Query(..., title="Symbols", description="Comma-separated list of stock symbols")):
    symbols = list(set(symbols.upper().replace(' ', '').split(',')))
    return StreamingResponse(quotes_generator(symbols), media_type="text/event-stream")
