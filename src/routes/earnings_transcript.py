from typing import Any, Optional

from fastapi import APIRouter, HTTPException, Query, Security
from fastapi.security import APIKeyHeader

from src.models.earnings_transcript import EarningsTranscriptRequest
from src.services.earnings_transcript.get_earnings_transcript import get_earnings_transcript

router = APIRouter()


@router.get(
    path="/earnings-transcript/{symbol}",
    summary="Get earnings call transcript for a stock",
    description="Returns earnings call transcript data for a stock symbol in JSON format.",
    response_model=dict[str, Any],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved earnings transcript",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "AAPL",
                        "transcripts": [
                            {
                                "symbol": "AAPL",
                                "quarter": "Q3",
                                "year": 2024,
                                "date": "2024-10-15T00:00:00",
                                "transcript": "Full transcript text...",
                                "participants": ["CEO", "CFO", "Analysts"],
                                "metadata": {"source": "defeatbeta-api"},
                            }
                        ],
                        "metadata": {"total_transcripts": 1, "filters_applied": {"quarter": "Q3", "year": 2024}, "retrieved_at": "2024-10-15T12:00:00"},
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found or no transcripts available",
            "content": {"application/json": {"example": {"detail": "No earnings transcripts found for INVALID"}}},
        },
        500: {"description": "Internal server error", "content": {"application/json": {"example": {"detail": "Failed to fetch earnings transcript"}}}},
    },
)
async def get_earnings_transcript_endpoint(
    symbol: str,
    quarter: Optional[str] = Query(None, description="Specific quarter (e.g., 'Q1', 'Q2', 'Q3', 'Q4')"),
    year: Optional[int] = Query(None, description="Specific year (e.g., 2024, 2023)"),
):
    """
    Get earnings call transcript for a stock symbol in JSON format.

    This endpoint fetches earnings call transcripts using the defeatbeta-api and returns
    the raw transcript data in structured JSON format.

    **Parameters:**
    - **symbol**: Stock ticker symbol (e.g., AAPL, TSLA, MSFT)
    - **quarter**: Optional filter for specific quarter (Q1, Q2, Q3, Q4)
    - **year**: Optional filter for specific year

    **Returns:**
    - Raw transcript data from earnings calls in JSON format
    - Participant information
    - Metadata about the transcript
    """
    return await get_earnings_transcript(symbol=symbol, quarter=quarter, year=year)


@router.post(
    path="/earnings-transcript/analyze",
    summary="Get earnings transcript with custom parameters",
    description="Get earnings call transcript with custom parameters via POST request.",
    response_model=dict[str, Any],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"description": "Successfully retrieved earnings transcript"},
        400: {"description": "Invalid request parameters"},
        500: {"description": "Internal server error"},
    },
)
async def analyze_earnings_transcript_endpoint(request: EarningsTranscriptRequest):
    """
    Get earnings call transcript with custom parameters via POST request.

    This endpoint allows for more detailed control over the transcript retrieval process
    with specific filtering criteria provided in the request body.
    """
    return await get_earnings_transcript(symbol=request.symbol, quarter=request.quarter, year=request.year)


@router.get(
    path="/earnings-transcript/{symbol}/latest",
    summary="Get latest earnings transcript",
    description="Returns the most recent earnings call transcript for a stock symbol.",
    response_model=dict[str, Any],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={200: {"description": "Successfully retrieved latest earnings transcript"}, 404: {"description": "No transcripts found for symbol"}},
)
async def get_latest_earnings_transcript_endpoint(symbol: str):
    """
    Get the most recent earnings call transcript for a stock symbol.

    This is a convenience endpoint that automatically fetches the latest available
    earnings transcript without requiring specific quarter/year parameters.
    """
    # Get all transcripts and return the most recent one
    result = await get_earnings_transcript(symbol=symbol, quarter=None, year=None)

    if not result.get("transcripts"):
        raise HTTPException(status_code=404, detail=f"No earnings transcripts found for {symbol}")

    # Sort by date and return the most recent
    transcripts = result["transcripts"]
    if len(transcripts) > 1:
        # Sort by year and quarter to get the most recent
        transcripts.sort(key=lambda x: (x["year"], {"Q1": 1, "Q2": 2, "Q3": 3, "Q4": 4}.get(x["quarter"], 0)), reverse=True)

    # Return only the latest transcript
    result["transcripts"] = [transcripts[0]]
    result["metadata"]["total_transcripts"] = 1
    result["metadata"]["note"] = "Latest transcript only"

    return result
