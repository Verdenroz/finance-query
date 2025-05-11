from fastapi import APIRouter, Security
from fastapi.security import APIKeyHeader

from utils.dependencies import YahooCookies, YahooCrumb
from src.models import ValidationErrorResponse
from src.models.sector import MarketSector, MarketSectorDetails, Sector
from src.services import get_sector_details, get_sector_for_symbol, get_sectors

router = APIRouter()


@router.get(
    path="/sectors",
    summary="Get summary performance for all sectors",
    description="Returns a summary of all sectors, or detailed information for a specific sector or symbol, depending on the query parameters provided.",
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": list[MarketSector],
            "description": "Successfully retrieved sector information",
            "content": {
                "application/json": {
                    "example": [
                        {
                            "sector": "Technology",
                            "dayReturn": "-0.69%",
                            "ytdReturn": "-2.36%",
                            "yearReturn": "+24.00%",
                            "threeYearReturn": "+50.20%",
                            "fiveYearReturn": "+158.41%",
                        },
                        {
                            "sector": "Healthcare",
                            "dayReturn": "+0.87%",
                            "ytdReturn": "+7.45%",
                            "yearReturn": "+4.04%",
                            "threeYearReturn": "+7.59%",
                            "fiveYearReturn": "+44.74%",
                        },
                        {
                            "sector": "Financial Services",
                            "dayReturn": "+0.81%",
                            "ytdReturn": "+5.94%",
                            "yearReturn": "+30.86%",
                            "threeYearReturn": "+26.28%",
                            "fiveYearReturn": "+63.57%",
                        },
                        {
                            "sector": "Consumer Cyclical",
                            "dayReturn": "-2.59%",
                            "ytdReturn": "+1.55%",
                            "yearReturn": "+27.74%",
                            "threeYearReturn": "+19.39%",
                            "fiveYearReturn": "+102.42%",
                        },
                        {
                            "sector": "Industrials",
                            "dayReturn": "+0.08%",
                            "ytdReturn": "+3.06%",
                            "yearReturn": "+12.32%",
                            "threeYearReturn": "+24.85%",
                            "fiveYearReturn": "+57.96%",
                        },
                        {
                            "sector": "Consumer Defensive",
                            "dayReturn": "+0.74%",
                            "ytdReturn": "+3.47%",
                            "yearReturn": "+15.60%",
                            "threeYearReturn": "+15.16%",
                            "fiveYearReturn": "+39.80%",
                        },
                        {
                            "sector": "Energy",
                            "dayReturn": "-1.13%",
                            "ytdReturn": "+4.96%",
                            "yearReturn": "+10.88%",
                            "threeYearReturn": "+25.30%",
                            "fiveYearReturn": "+61.17%",
                        },
                        {
                            "sector": "Real Estate",
                            "dayReturn": "+1.26%",
                            "ytdReturn": "+2.33%",
                            "yearReturn": "+14.11%",
                            "threeYearReturn": "-3.16%",
                            "fiveYearReturn": "+14.27%",
                        },
                        {
                            "sector": "Utilities",
                            "dayReturn": "+2.06%",
                            "ytdReturn": "+4.73%",
                            "yearReturn": "+37.61%",
                            "threeYearReturn": "+23.87%",
                            "fiveYearReturn": "+31.59%",
                        },
                        {
                            "sector": "Basic Materials",
                            "dayReturn": "-0.47%",
                            "ytdReturn": "+6.16%",
                            "yearReturn": "+9.99%",
                            "threeYearReturn": "+8.52%",
                            "fiveYearReturn": "+53.48%",
                        },
                        {
                            "sector": "Communication Services",
                            "dayReturn": "-4.61%",
                            "ytdReturn": "+5.72%",
                            "yearReturn": "+30.47%",
                            "threeYearReturn": "+29.44%",
                            "fiveYearReturn": "+73.06%",
                        },
                    ]
                }
            },
        }
    },
)
async def sectors():
    return await get_sectors()


@router.get(
    path="/sectors/symbol/{symbol}",
    summary="Get summary sector performance of a quote",
    description="Returns the quote's sector performance summary, including its returns over a 1d, ytd, year, 3y, and 5y period",
    response_model=MarketSector,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": MarketSector,
            "description": "Successfully retrieved sector information",
            "content": {
                "application/json": {
                    "example": {
                        "sector": "Technology",
                        "dayReturn": "-0.46%",
                        "ytdReturn": "-2.13%",
                        "yearReturn": "+24.28%",
                        "threeYearReturn": "+50.55%",
                        "fiveYearReturn": "+159.00%",
                    }
                }
            },
        },
        404: {
            "description": "Sector not found when querying sector by symbol",
            "content": {"application/json": {"example": {"detail": "Sector for {symbol} not found"}}},
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {"application/json": {"example": {"detail": "Invalid request"}}},
        },
    },
)
async def sector_by_symbol(
    cookies: YahooCookies,
    crumb: YahooCrumb,
    symbol: str,
):
    return await get_sector_for_symbol(symbol, cookies, crumb)


@router.get(
    path="/sectors/details/{sector}",
    summary="Get a more comprehensive summary of an individual sector",
    description="Returns the quote's sector performance details, including its returns, market cap, market weight, "
    "number of industries, number of companies, top industries, and top companies",
    response_model=MarketSectorDetails,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": MarketSectorDetails,
            "description": "Successfully retrieved sector information",
            "content": {
                "application/json": {
                    "example": {
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
                            "Electronics & Computer Distribution: 0.16%",
                        ],
                        "topCompanies": ["NVDA", "AAPL", "MSFT", "AVGO", "ORCL", "CRM", "CSCO", "NOW", "ACN", "IBM"],
                    }
                }
            },
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {
                            "sector": [
                                "Input should be 'Basic Materials', 'Communication Services', 'Consumer Cyclical', "
                                "'Consumer Defensive', 'Energy', 'Financial Services', 'Healthcare', 'Industrials', "
                                "'Real Estate', 'Technology' or 'Utilities'"
                            ]
                        },
                    }
                }
            },
        },
    },
)
async def sector_details(sector: Sector):
    return await get_sector_details(sector)
