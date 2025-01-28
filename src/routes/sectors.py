from fastapi import APIRouter, Security, Query
from fastapi.security import APIKeyHeader
from typing_extensions import Optional

from src.schemas import ValidationErrorResponse
from src.schemas.sector import Sector, SectorsResponse
from src.services import get_sectors, get_sector_for_symbol, get_sector_details

router = APIRouter()


@router.get(
    path="/sectors",
    summary="Get sector performance and information",
    description="Returns a summary of all sectors, or detailed information for a specific sector or symbol, "
                "depending on the query parameters provided.",
    response_model=SectorsResponse,
    tags=["Sectors"],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": SectorsResponse,
            "description": "Successfully retrieved sector information",
            "content": {
                "application/json": {
                    "examples": {
                        "all_sectors": {
                            "summary": "Summary of all sectors",
                            "value": [
                                {
                                    "sector": "Technology",
                                    "dayReturn": "+0.97%",
                                    "ytdReturn": "+3.35%",
                                    "yearReturn": "+32.59%",
                                    "threeYearReturn": "+66.92%",
                                    "fiveYearReturn": "+179.23%"
                                },
                                {
                                    "sector": "Healthcare",
                                    "dayReturn": "+0.03%",
                                    "ytdReturn": "+4.92%",
                                    "yearReturn": "+3.66%",
                                    "threeYearReturn": "+8.97%",
                                    "fiveYearReturn": "+43.45%"
                                }
                            ]
                        },
                        "sector_by_symbol": {
                            "summary": "Sector performance for a specific symbol",
                            "value": {
                                "sector": "Consumer Cyclical",
                                "dayReturn": "-0.09%",
                                "ytdReturn": "+3.31%",
                                "yearReturn": "+33.17%",
                                "threeYearReturn": "+26.03%",
                                "fiveYearReturn": "+113.61%"
                            }
                        },
                        "detailed_sector": {
                            "summary": "Detailed information for a specific sector",
                            "value": {
                                "sector": "Technology",
                                "dayReturn": "+0.97%",
                                "ytdReturn": "+3.35%",
                                "yearReturn": "+32.59%",
                                "threeYearReturn": "+66.92%",
                                "fiveYearReturn": "+179.23%",
                                "marketCap": "20.196T",
                                "market_weight": "29.28%",
                                "industries": 12,
                                "companies": 815,
                                "topIndustries": [
                                    "Semiconductors: 29.04%",
                                    "Software - Infrastructure: 26.44%",
                                    "Consumer Electronics: 16.60%",
                                    "Software - Application: 13.92%",
                                    "Information Technology Services: 4.53%",
                                    "Communication Equipment: 2.37%",
                                    "Semiconductor Equipment & Materials: 2.20%",
                                    "Computer Hardware: 2.10%",
                                    "Electronic Components: 1.43%",
                                    "Scientific & Technical Instruments: 1.02%",
                                    "Solar: 0.20%",
                                    "Electronics & Computer Distribution: 0.16%"
                                ],
                                "topCompanies": [
                                    "NVDA",
                                    "AAPL",
                                    "MSFT",
                                    "AVGO",
                                    "ORCL",
                                    "CRM",
                                    "CSCO",
                                    "NOW",
                                    "ACN",
                                    "IBM"
                                ]
                            }
                        }
                    }
                }
            }
        },
        404: {
            "description": "Sector not found when querying sector by symbol",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Sector for {symbol} not found"
                    }
                }
            }
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {
                            "name": [
                                "Input should be 'Basic Materials', 'Communication Services', 'Consumer Cyclical', "
                                "'Consumer Defensive', 'Energy', 'Financial Services', 'Healthcare', 'Industrials', "
                                "'Real Estate', 'Technology' or 'Utilities'"]
                        }
                    }
                }
            }
        }
    }
)
async def sector(
        symbol: Optional[str] = Query(
            None,
            description="Optional symbol to get info for. If not provided, all sectors are returned with summary "
                        "information"),
        name: Optional[Sector] = Query(
            None,
            description="Optional sector name to get detailed info for. If not provided, all sectors are returned with "
                        "summary information"
        )
):
    if symbol and not name:
        return await get_sector_for_symbol(symbol)

    if name and not symbol:
        return await get_sector_details(name)

    return await get_sectors()
