import asyncio
from collections.abc import AsyncGenerator

from fastapi import APIRouter, Query, Security
from fastapi.responses import StreamingResponse
from fastapi.security import APIKeyHeader
from orjson import orjson

from src.models import ValidationErrorResponse
from src.services import get_simple_quotes
from src.utils.dependencies import FinanceClient
from src.utils.logging import get_logger, log_route_error, log_route_request, log_route_success

router = APIRouter()
logger = get_logger(__name__)


async def quotes_generator(finance_client: FinanceClient, symbols: list[str]) -> AsyncGenerator[str, None]:
    """
    Stream simplified quotes by SSE (Server Sent Events) for the given symbols every 10 seconds
    Data is sent in the format of "quote: {json_data}\n\n"
    :param finance_client: The Yahoo Finance client to use for API requests
    :param symbols: the list of stock symbols

    :return: AsyncGenerator yielding the quotes in the format of "quote: {json_data}\n\n"
    """
    while True:
        quotes = await get_simple_quotes(finance_client, symbols)
        quotes = [quote if isinstance(quote, dict) else quote.model_dump(by_alias=True, exclude_none=True) for quote in quotes]
        data = orjson.dumps(quotes).decode("utf-8")
        yield f"quote: {data}\n\n"
        await asyncio.sleep(10)


@router.get(
    path="/stream/quotes",
    summary="Stream stock quotes by SSE",
    description="Stream stock quotes via SSE for the given symbols every 10 seconds. Response format: 'quote: {"
    "json_data}\\n\\n' with text/event-stream content type.",
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Stream stock quotes by SSE",
            "content": {
                "text/event-stream": {
                    "example": 'quote: [{"symbol":"NVDA","name":"NVIDIA Corporation","price":"142.62",'
                    '"change":"-4.60","percentChange":"-3.12%",'
                    '"logo":"https://img.logo.dev/nvidia.com?token=pk_Xd1Cdye3QYmCOXzcvxhxyw&retina=true"}]\n\n'
                }
            },
        },
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {"application/json": {"example": {"detail": "Invalid request", "errors": {"symbols": ["Field required"]}}}},
        },
    },
)
async def stream_quotes(
    finance_client: FinanceClient,
    symbols: str = Query(..., title="Symbols", description="Comma-separated list of stock symbols"),
):
    symbols_list = list(set(symbols.upper().replace(" ", "").split(",")))
    params = {"symbols": symbols_list}
    log_route_request(logger, "stream_quotes", params)

    try:
        log_route_success(logger, "stream_quotes", params, {"symbols_count": len(symbols_list), "streaming": True})
        return StreamingResponse(quotes_generator(finance_client, symbols_list), media_type="text/event-stream")
    except Exception as e:
        log_route_error(logger, "stream_quotes", params, e)
        raise
