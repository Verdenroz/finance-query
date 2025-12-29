from fastapi import APIRouter, Path, Security
from fastapi.security import APIKeyHeader

from src.models import ValidationErrorResponse
from src.models.earnings_transcript import EarningsCallsList, EarningsTranscript, Quarter
from src.services.earnings_transcript import get_earnings_calls_list, get_earnings_transcript
from src.utils.dependencies import FinanceClient

router = APIRouter()


@router.get(
    path="/earnings-transcript/{symbol}",
    summary="Get available earnings calls for a stock",
    description="Returns a list of available earnings call transcripts for a stock symbol.",
    response_model=EarningsCallsList,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved earnings calls list",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "NVDA",
                        "earnings_calls": [{"event_id": "351238", "quarter": "Q2", "year": 2026, "title": "Q2 2026", "url": "https://finance.yahoo.com/..."}],
                        "total": 20,
                    }
                }
            },
        },
        404: {
            "description": "No earnings calls found for symbol",
            "content": {"application/json": {"example": {"detail": "No earnings calls found for INVALID"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_earnings_calls_list_endpoint(
    finance_client: FinanceClient,
    symbol: str = Path(..., description="Stock ticker symbol", pattern=r"^[A-Za-z0-9]{1,10}(\.[A-Za-z]{1,3})?$"),
):
    """
    Get list of available earnings call transcripts for a stock symbol.

    This endpoint returns all available earnings calls with their event IDs, quarters, and years.
    Results are sorted by most recent first.

    **Parameters:**
    - **symbol**: Stock ticker symbol

    **Returns:**
    - List of available earnings calls with metadata
    """
    return await get_earnings_calls_list(finance_client, symbol.upper())


@router.get(
    path="/earnings-transcript/{symbol}/latest",
    summary="Get latest earnings call transcript",
    description="Returns the most recent earnings call transcript for a stock symbol.",
    response_model=EarningsTranscript,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved latest earnings transcript",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "NVDA",
                        "quarter": "Q2",
                        "year": 2026,
                        "date": "2025-08-27T00:00:00",
                        "title": "Q2 2026 Earnings Call",
                        "speakers": [{"name": "Jensen Huang", "role": "CEO", "company": "NVIDIA"}],
                        "paragraphs": [{"speaker": "Jensen Huang", "text": "Good afternoon everyone..."}],
                        "metadata": {"eventId": "351238", "fiscalYear": 2026, "fiscalPeriod": "Q2"},
                    }
                }
            },
        },
        404: {"description": "No earnings calls found for symbol"},
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_latest_earnings_transcript_endpoint(
    finance_client: FinanceClient,
    symbol: str = Path(..., description="Stock ticker symbol", pattern=r"^[A-Za-z0-9]{1,10}(\.[A-Za-z]{1,3})?$"),
):
    """
    Get the most recent earnings call transcript for a stock symbol.

    This is a convenience endpoint that automatically fetches the latest available
    earnings transcript without requiring specific quarter/year parameters.

    **Parameters:**
    - **symbol**: Stock ticker symbol

    **Returns:**
    - Full transcript with speakers and paragraphs
    """
    return await get_earnings_transcript(finance_client, symbol.upper(), quarter=None, year=None)


@router.get(
    path="/earnings-transcript/{symbol}/{quarter}/{year}",
    summary="Get specific earnings call transcript",
    description="Returns earnings call transcript for a specific quarter and year.",
    response_model=EarningsTranscript,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved earnings transcript",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "NVDA",
                        "quarter": "Q2",
                        "year": 2026,
                        "date": "2025-08-27T00:00:00",
                        "title": "Q2 2026 Earnings Call",
                        "speakers": [{"name": "Jensen Huang", "role": "CEO", "company": "NVIDIA"}],
                        "paragraphs": [{"speaker": "Jensen Huang", "text": "Good afternoon everyone..."}],
                        "metadata": {"eventId": "351238", "fiscalYear": 2026, "fiscalPeriod": "Q2"},
                    }
                }
            },
        },
        404: {"description": "Earnings call not found for specified quarter/year"},
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_specific_earnings_transcript_endpoint(
    finance_client: FinanceClient,
    symbol: str = Path(..., description="Stock ticker symbol", pattern=r"^[A-Za-z0-9]{1,10}(\.[A-Za-z]{1,3})?$"),
    quarter: Quarter = Path(..., description="Quarter (Q1, Q2, Q3, or Q4)"),
    year: int = Path(..., description="Year", ge=1990, le=2030),
):
    """
    Get earnings call transcript for a specific quarter and year.

    **Parameters:**
    - **symbol**: Stock ticker symbol
    - **quarter**: Quarter (Q1, Q2, Q3, Q4)
    - **year**: Year (e.g., 2026)

    **Returns:**
    - Full transcript with speakers and paragraphs
    """
    return await get_earnings_transcript(finance_client, symbol.upper(), quarter=quarter, year=year)
