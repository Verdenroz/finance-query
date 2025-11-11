from datetime import datetime

from fastapi import HTTPException

from src.models.earnings_transcript import (
    EarningsCallListing,
    EarningsCallsList,
    EarningsTranscript,
    TranscriptParagraph,
    TranscriptSpeaker,
)
from src.utils.cache import cache
from src.utils.dependencies import FinanceClient
from src.utils.logging import get_logger

logger = get_logger(__name__)


@cache(expire=86400)
async def get_earnings_calls_list(finance_client: FinanceClient, symbol: str) -> EarningsCallsList:
    """
    Get list of available earnings calls for a symbol.

    Args:
        finance_client: Yahoo Finance client
        symbol: Stock symbol

    Returns:
        EarningsCallsList with all available earnings calls

    Raises:
        HTTPException: If no earnings calls found or fetch fails
    """
    try:
        # Scrape earnings calls page to get list
        calls_data = await finance_client.get_earnings_calls_list(symbol)

        if not calls_data:
            raise HTTPException(status_code=404, detail=f"No earnings calls found for {symbol}")

        # Convert to Pydantic models
        earnings_calls = [
            EarningsCallListing(event_id=call["eventId"], quarter=call["quarter"], year=call["year"], title=call["title"], url=call["url"])
            for call in calls_data
        ]

        # Sort by year and quarter (most recent first)
        def sort_key(call: EarningsCallListing):
            # Convert quarter to number for sorting
            quarter_num = int(call.quarter[1]) if call.quarter and len(call.quarter) > 1 else 0
            year = call.year or 0
            return (year, quarter_num)

        earnings_calls.sort(key=sort_key, reverse=True)

        return EarningsCallsList(symbol=symbol.upper(), earnings_calls=earnings_calls, total=len(earnings_calls))

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error fetching earnings calls list for {symbol}: {e}")
        raise HTTPException(status_code=500, detail=f"Failed to fetch earnings calls list: {str(e)}") from e


@cache(expire=604800)
async def get_earnings_transcript(finance_client: FinanceClient, symbol: str, quarter: str | None = None, year: int | None = None) -> EarningsTranscript:
    """
    Get earnings call transcript for a symbol, optionally filtered by quarter/year.

    Args:
        finance_client: Yahoo Finance client
        symbol: Stock symbol
        quarter: Optional quarter filter (e.g., 'Q1', 'Q2')
        year: Optional year filter (e.g., 2026)

    Returns:
        EarningsTranscript with full transcript data

    Raises:
        HTTPException: If no transcript found or fetch fails
    """
    try:
        # Get list of available earnings calls
        calls_list = await get_earnings_calls_list(finance_client, symbol)

        if not calls_list.earnings_calls:
            raise HTTPException(status_code=404, detail=f"No earnings calls found for {symbol}")

        # Filter by quarter/year if specified
        target_call = None

        if quarter or year:
            # Normalize quarter format
            if quarter:
                quarter = quarter.upper()
                if not quarter.startswith("Q"):
                    quarter = f"Q{quarter}"

            # Find matching call
            for call in calls_list.earnings_calls:
                quarter_match = not quarter or call.quarter == quarter
                year_match = not year or call.year == year

                if quarter_match and year_match:
                    target_call = call
                    break

            if not target_call:
                filter_str = f"{quarter} {year}" if quarter and year else quarter or str(year)
                raise HTTPException(status_code=404, detail=f"No earnings call found for {symbol} {filter_str}")
        else:
            # Get the most recent call (first in sorted list)
            target_call = calls_list.earnings_calls[0]

        # Get company ID (quartrId)
        quote_type_data = await finance_client.get_quote_type(symbol)
        quote_type_result = quote_type_data.get("quoteType", {}).get("result", [])

        if not quote_type_result:
            raise HTTPException(status_code=404, detail=f"Could not find company ID for {symbol}")

        company_id = quote_type_result[0].get("quartrId")
        if not company_id:
            raise HTTPException(status_code=404, detail=f"No company ID (quartrId) found for {symbol}")

        # Fetch the transcript
        transcript_data = await finance_client.get_earnings_transcript(target_call.event_id, str(company_id))

        # Parse the transcript
        return _parse_transcript(symbol, transcript_data, target_call)

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error fetching earnings transcript for {symbol}: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Failed to fetch earnings transcript: {str(e)}") from e


def _parse_transcript(symbol: str, transcript_data: dict, call_info: EarningsCallListing) -> EarningsTranscript:
    """
    Parse raw transcript data from Yahoo Finance into simplified format.

    Args:
        symbol: Stock symbol
        transcript_data: Raw transcript data from Yahoo Finance
        call_info: Call information from the earnings calls list

    Returns:
        EarningsTranscript with simplified structure
    """
    content = transcript_data.get("transcriptContent", {})
    metadata = transcript_data.get("transcriptMetadata", {})

    # Parse speaker mapping
    speaker_mapping = {}
    speakers_list = []

    for speaker_data in content.get("speaker_mapping", []):
        speaker_id = speaker_data.get("speaker")
        speaker_info = speaker_data.get("speaker_data", {})

        name = speaker_info.get("name", f"Speaker {speaker_id}")
        role = speaker_info.get("role")
        company = speaker_info.get("company")

        speaker_mapping[speaker_id] = name
        speakers_list.append(TranscriptSpeaker(name=name, role=role, company=company))

    # Parse paragraphs
    transcript_obj = content.get("transcript", {})
    paragraphs_data = transcript_obj.get("paragraphs", [])

    paragraphs = []
    for para in paragraphs_data:
        speaker_id = para.get("speaker")
        speaker_name = speaker_mapping.get(speaker_id, f"Speaker {speaker_id}")
        text = para.get("text", "")

        if text:  # Only include paragraphs with text
            paragraphs.append(TranscriptParagraph(speaker=speaker_name, text=text))

    # Extract metadata
    fiscal_year = metadata.get("fiscalYear", call_info.year)
    fiscal_period = metadata.get("fiscalPeriod", call_info.quarter)
    title = metadata.get("title", call_info.title)

    # Parse date
    date_timestamp = metadata.get("date")
    if date_timestamp:
        date = datetime.fromtimestamp(date_timestamp)
    else:
        date = datetime.now()

    # Build metadata dict
    meta_dict = {
        "eventId": call_info.event_id,
        "fiscalYear": fiscal_year,
        "fiscalPeriod": fiscal_period,
        "transcriptId": metadata.get("transcriptId"),
        "eventType": metadata.get("eventType", "Earnings Call"),
        "isLatest": metadata.get("isLatest", False),
        "retrieved_at": datetime.now().isoformat(),
    }

    return EarningsTranscript(
        symbol=symbol.upper(),
        quarter=fiscal_period,
        year=fiscal_year,
        date=date,
        title=title,
        speakers=speakers_list,
        paragraphs=paragraphs,
        metadata=meta_dict,
    )
