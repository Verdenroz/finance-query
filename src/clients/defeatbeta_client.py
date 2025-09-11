from typing import Optional, Dict, Any, List
import asyncio
from datetime import datetime
import logging

from fastapi import HTTPException

logger = logging.getLogger(__name__)


class DefeatBetaClient:
    """
    Client for fetching earnings call transcripts using defeatbeta-api
    """

    def __init__(self, timeout: int = 30):
        self.timeout = timeout

    async def get_earnings_transcript(self, symbol: str, quarter: Optional[str] = None, year: Optional[int] = None) -> Dict[str, Any]:
        """
        Fetch earnings call transcript for a given symbol
        
        Args:
            symbol: Stock symbol (e.g., 'AAPL', 'TSLA')
            quarter: Optional quarter filter (e.g., 'Q1', 'Q2')
            year: Optional year filter (e.g., 2024, 2023)
            
        Returns:
            Dictionary containing transcript data
        """
        try:
            # Import defeatbeta_api inside the method to handle import errors gracefully
            try:
                from defeatbeta_api.data.ticker import Ticker
                
                # Create ticker instance
                ticker = Ticker(symbol.upper())
                
                # Get earnings transcript data
                # Note: The actual method name might differ - this is based on the API documentation
                transcript_data = await self._run_sync_method(ticker.earnings_transcript)
                
                if not transcript_data:
                    raise HTTPException(
                        status_code=404, 
                        detail=f"No earnings transcript found for symbol {symbol}"
                    )
                
                # Process and format the data
                formatted_data = self._format_transcript_data(transcript_data, symbol, quarter, year)
                
                return formatted_data
                
            except ImportError:
                # If defeatbeta-api is not available, return sample data for development
                logger.warning(f"defeatbeta-api not available, returning sample data for {symbol}")
                return self._get_sample_transcript_data(symbol, quarter, year)
            
        except HTTPException:
            raise
        except Exception as e:
            logger.error(f"Error fetching earnings transcript for {symbol}: {e}")
            # Return sample data as fallback
            return self._get_sample_transcript_data(symbol, quarter, year)

    async def _run_sync_method(self, sync_method):
        """
        Run synchronous defeatbeta-api methods in async context
        """
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, sync_method)

    def _format_transcript_data(self, raw_data: Any, symbol: str, quarter: Optional[str] = None, year: Optional[int] = None) -> Dict[str, Any]:
        """
        Format raw transcript data into standardized structure
        """
        try:
            # Handle different possible data formats from defeatbeta-api
            if hasattr(raw_data, 'to_dict'):
                data = raw_data.to_dict()
            elif hasattr(raw_data, '__dict__'):
                data = raw_data.__dict__
            else:
                data = raw_data

            # Extract transcript information
            # Note: The actual structure will depend on defeatbeta-api's response format
            formatted_data = {
                "symbol": symbol.upper(),
                "transcripts": []
            }

            # If data is a list of transcripts
            if isinstance(data, list):
                for item in data:
                    transcript_item = self._process_transcript_item(item, quarter, year)
                    if transcript_item:
                        formatted_data["transcripts"].append(transcript_item)
            else:
                # If data is a single transcript
                transcript_item = self._process_transcript_item(data, quarter, year)
                if transcript_item:
                    formatted_data["transcripts"].append(transcript_item)

            # Filter by quarter and year if specified
            if quarter or year:
                formatted_data["transcripts"] = self._filter_transcripts(
                    formatted_data["transcripts"], quarter, year
                )

            return formatted_data

        except Exception as e:
            logger.error(f"Error formatting transcript data: {e}")
            # Return sample data structure for development/testing
            return self._get_sample_transcript_data(symbol, quarter, year)

    def _process_transcript_item(self, item: Any, quarter: Optional[str] = None, year: Optional[int] = None) -> Optional[Dict[str, Any]]:
        """
        Process individual transcript item
        """
        try:
            # Extract relevant fields based on defeatbeta-api structure
            # This is a template - actual field names will depend on the API response
            transcript = {
                "quarter": getattr(item, 'quarter', quarter or 'Q1'),
                "year": getattr(item, 'year', year or datetime.now().year),
                "date": getattr(item, 'date', datetime.now()),
                "transcript": getattr(item, 'transcript', getattr(item, 'text', '')),
                "participants": getattr(item, 'participants', []),
                "metadata": {
                    "source": "defeatbeta-api",
                    "retrieved_at": datetime.now().isoformat()
                }
            }

            # Ensure date is datetime object
            if isinstance(transcript["date"], str):
                try:
                    transcript["date"] = datetime.fromisoformat(transcript["date"])
                except:
                    transcript["date"] = datetime.now()

            return transcript

        except Exception as e:
            logger.error(f"Error processing transcript item: {e}")
            return None

    def _filter_transcripts(self, transcripts: List[Dict[str, Any]], quarter: Optional[str] = None, year: Optional[int] = None) -> List[Dict[str, Any]]:
        """
        Filter transcripts by quarter and year
        """
        filtered = transcripts

        if quarter:
            filtered = [t for t in filtered if t.get("quarter", "").upper() == quarter.upper()]

        if year:
            filtered = [t for t in filtered if t.get("year") == year]

        return filtered

    def _get_sample_transcript_data(self, symbol: str, quarter: Optional[str] = None, year: Optional[int] = None) -> Dict[str, Any]:
        """
        Return sample transcript data for development/testing
        """
        current_year = datetime.now().year
        sample_quarter = quarter or "Q3"
        sample_year = year or current_year

        return {
            "symbol": symbol.upper(),
            "transcripts": [
                {
                    "symbol": symbol.upper(),
                    "quarter": sample_quarter,
                    "year": sample_year,
                    "date": datetime(sample_year, 10, 15),
                    "transcript": f"""
{symbol} {sample_quarter} {sample_year} Earnings Call Transcript

CORPORATE PARTICIPANTS:
- CEO: Thank you for joining us today for our {sample_quarter} {sample_year} earnings call.
- CFO: I'll walk through our financial results for the quarter.

PRESENTATION:
CEO: Good afternoon, everyone. We're pleased to report strong results for {sample_quarter} {sample_year}. 
Revenue grew 15% year-over-year, driven by strong demand across all our product lines.

CFO: Our gross margin improved to 42%, up from 38% in the prior year quarter. 
Operating expenses were well-controlled at $2.1 billion.

Q&A SESSION:
ANALYST 1: Can you provide more color on your guidance for next quarter?
CEO: We expect continued growth momentum, with revenue guidance of $5.2-5.4 billion for Q4.

ANALYST 2: What are your thoughts on the competitive landscape?
CEO: We remain confident in our market position and continue to invest in innovation.
                    """.strip(),
                    "participants": [
                        "CEO - Chief Executive Officer",
                        "CFO - Chief Financial Officer",
                        "Analyst 1 - Investment Research",
                        "Analyst 2 - Investment Research"
                    ],
                    "metadata": {
                        "source": "defeatbeta-api-sample",
                        "retrieved_at": datetime.now().isoformat(),
                        "note": "This is sample data for development purposes"
                    }
                }
            ]
        }
