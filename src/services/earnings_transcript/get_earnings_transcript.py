from typing import Optional, Dict, Any
import logging
from datetime import datetime

from fastapi import HTTPException

from src.clients.defeatbeta_client import DefeatBetaClient
from src.models.earnings_transcript import EarningsTranscript

logger = logging.getLogger(__name__)


async def get_earnings_transcript(
    symbol: str,
    quarter: Optional[str] = None,
    year: Optional[int] = None
) -> Dict[str, Any]:
    """
    Get earnings call transcript for a symbol
    
    Args:
        symbol: Stock symbol (e.g., 'AAPL', 'TSLA')
        quarter: Optional quarter filter (e.g., 'Q1', 'Q2')
        year: Optional year filter (e.g., 2024, 2023)
        
    Returns:
        Dictionary containing transcript data in JSON format
    """
    try:
        # Initialize DefeatBeta client
        defeatbeta_client = DefeatBetaClient()
        
        # Fetch raw transcript data
        logger.info(f"Fetching earnings transcript for {symbol}")
        transcript_data = await defeatbeta_client.get_earnings_transcript(
            symbol=symbol,
            quarter=quarter,
            year=year
        )
        
        if not transcript_data.get("transcripts"):
            raise HTTPException(
                status_code=404,
                detail=f"No earnings transcripts found for {symbol}"
            )
        
        # Process each transcript
        processed_transcripts = []
        
        for transcript_item in transcript_data["transcripts"]:
            # Create transcript model
            transcript = EarningsTranscript(
                symbol=transcript_item["symbol"],
                quarter=transcript_item["quarter"],
                year=transcript_item["year"],
                date=transcript_item["date"],
                transcript=transcript_item["transcript"],
                participants=transcript_item.get("participants", []),
                metadata=transcript_item.get("metadata", {})
            )
            
            processed_transcripts.append(transcript.model_dump())
        
        return {
            "symbol": symbol.upper(),
            "transcripts": processed_transcripts,
            "metadata": {
                "total_transcripts": len(processed_transcripts),
                "filters_applied": {
                    "quarter": quarter,
                    "year": year
                },
                "retrieved_at": datetime.now().isoformat()
            }
        }
        
    except HTTPException:
        # Re-raise HTTP exceptions as-is
        raise
    except Exception as e:
        logger.error(f"Unexpected error in get_earnings_transcript: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Internal server error: {str(e)}"
        )
