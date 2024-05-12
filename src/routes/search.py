from fastapi import APIRouter
from typing_extensions import Optional

from src.services import get_search
from src.services.get_search import Type

router = APIRouter()


@router.get("/search",
            summary="Search for a stock",
            description="Search for a stock by name or symbol.",
            )
async def search(query: str, type: Optional[Type] = None):
    return await get_search(query, type)
