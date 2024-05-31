from fastapi import APIRouter, Security, Response
from fastapi.security import APIKeyHeader

from src.services import get_sectors

router = APIRouter()


@router.get("/sectors",
            summary="Get all sectors",
            description="Get all sectors available in the stock market",
            response_description="Summary of all sectors",
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
            )
async def sector(response: Response):
    response.headers["Access-Control-Allow-Origin"] = "*"
    return await get_sectors()
