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
            from defeatbeta_api.data.ticker import Ticker
            
            # Create ticker instance
            ticker = Ticker(symbol.upper())
            
            # Get earnings transcript data using the correct method name
            transcripts_obj = await self._run_sync_method(ticker.earning_call_transcripts)
            
            # Get the transcripts list as DataFrame
            transcripts_df = await self._run_sync_method(transcripts_obj.get_transcripts_list)
            
            if transcripts_df.empty:
                raise HTTPException(
                    status_code=404, 
                    detail=f"No earnings transcripts found for symbol {symbol}"
                )
            
            # Process and format the data
            formatted_data = self._format_transcript_data(transcripts_df, symbol, quarter, year)
            
            return formatted_data
                
        except ImportError as e:
            logger.error(f"defeatbeta-api import error: {e}")
            raise HTTPException(
                status_code=500,
                detail="defeatbeta-api package not properly installed"
            )
        except HTTPException:
            raise
        except Exception as e:
            logger.error(f"Error fetching earnings transcript for {symbol}: {e}")
            raise HTTPException(
                status_code=500,
                detail=f"Failed to fetch earnings transcript: {str(e)}"
            )

    async def get_financial_statement(self, symbol: str, statement_type: str, frequency: str) -> Dict[str, Any]:
        """
        Fetch financial statement data for a given symbol
        
        Args:
            symbol: Stock symbol (e.g., 'AAPL', 'TSLA')
            statement_type: Type of statement ('income_statement', 'balance_sheet', 'cash_flow')
            frequency: Frequency ('quarterly', 'annual')
            
        Returns:
            Dictionary containing financial statement data
        """
        try:
            from defeatbeta_api.data.ticker import Ticker
            
            # Create ticker instance
            ticker = Ticker(symbol.upper())
            
            # Map statement types and frequencies to appropriate method calls
            method_name = self._get_financial_method_name(statement_type, frequency)
            
            # Get the method from the ticker object
            if not hasattr(ticker, method_name):
                raise HTTPException(
                    status_code=400,
                    detail=f"Method {method_name} not available for {symbol}"
                )
            
            method = getattr(ticker, method_name)
            
            # Execute the method synchronously in async context
            financial_data = await self._run_sync_method(method)
            
            if financial_data is None:
                raise HTTPException(
                    status_code=404,
                    detail=f"No {statement_type} data found for symbol {symbol}"
                )
            
            # Format the data for our API response
            formatted_data = self._format_financial_data(financial_data, symbol, statement_type, frequency)
            
            return formatted_data
                
        except ImportError as e:
            logger.error(f"defeatbeta-api import error: {e}")
            raise HTTPException(
                status_code=500,
                detail="defeatbeta-api package not properly installed"
            )
        except HTTPException:
            raise
        except Exception as e:
            logger.error(f"Error fetching {statement_type} for {symbol}: {e}")
            raise HTTPException(
                status_code=500,
                detail=f"Failed to fetch {statement_type}: {str(e)}"
            )

    def _get_financial_method_name(self, statement_type: str, frequency: str) -> str:
        """
        Map statement type and frequency to defeatbeta-api method name
        """
        frequency_prefix = "quarterly" if frequency == "quarterly" else "annual"
        
        if statement_type == "income_statement":
            return f"{frequency_prefix}_income_statement"
        elif statement_type == "balance_sheet":
            return f"{frequency_prefix}_balance_sheet"
        elif statement_type == "cash_flow":
            return f"{frequency_prefix}_cash_flow"
        else:
            raise HTTPException(
                status_code=400,
                detail=f"Unsupported statement type: {statement_type}"
            )

    def _format_financial_data(self, financial_data: Any, symbol: str, statement_type: str, frequency: str) -> Dict[str, Any]:
        """
        Format defeatbeta-api financial data into standardized structure
        """
        try:
            import pandas as pd
            import numpy as np
            
            logger.info(f"Processing financial data type: {type(financial_data)}")
            
            # Handle case where financial_data might have a method to get the DataFrame
            if hasattr(financial_data, 'get_data'):
                df = financial_data.get_data()
            elif hasattr(financial_data, 'to_dataframe'):
                df = financial_data.to_dataframe()
            elif isinstance(financial_data, pd.DataFrame):
                df = financial_data
            elif hasattr(financial_data, 'data') and isinstance(financial_data.data, pd.DataFrame):
                # Handle case where data is wrapped in an object
                df = financial_data.data
            else:
                # Log the actual data structure for debugging
                logger.info(f"Financial data attributes: {dir(financial_data) if hasattr(financial_data, '__dict__') else 'No attributes'}")
                logger.info(f"Financial data content: {str(financial_data)[:500]}")
                
                # Try to convert to DataFrame if it's not already
                try:
                    if isinstance(financial_data, dict):
                        df = pd.DataFrame([financial_data])
                    elif isinstance(financial_data, list):
                        df = pd.DataFrame(financial_data)
                    else:
                        df = pd.DataFrame(financial_data)
                except Exception as e:
                    logger.error(f"DataFrame conversion failed: {e}")
                    raise HTTPException(
                        status_code=500,
                        detail=f"Unable to convert financial data to DataFrame: {str(e)}"
                    )
            
            if df is None or df.empty:
                raise HTTPException(
                    status_code=404,
                    detail=f"No {statement_type} data found for {symbol}"
                )
            
            # Clean NaN values and convert to serializable format
            def clean_value(value):
                if pd.isna(value) or value is None or (isinstance(value, float) and np.isnan(value)):
                    return None
                return value
            
            # Convert DataFrame to dict format, handling timestamp columns
            data_dict = {}
            for index, row in df.iterrows():
                cleaned_row = {}
                for col in df.columns:
                    value = row[col]
                    if isinstance(value, pd.Timestamp):
                        cleaned_row[str(col)] = value.isoformat()
                    else:
                        cleaned_row[str(col)] = clean_value(value)
                data_dict[str(index)] = cleaned_row
            
            formatted_data = {
                "symbol": symbol.upper(),
                "statement_type": statement_type,
                "frequency": frequency,
                "statement": data_dict,
                "metadata": {
                    "source": "defeatbeta-api",
                    "retrieved_at": datetime.now().isoformat(),
                    "rows_count": len(df),
                    "columns_count": len(df.columns)
                }
            }
            
            return formatted_data
            
        except HTTPException:
            raise
        except Exception as e:
            logger.error(f"Error formatting financial data: {e}")
            raise HTTPException(
                status_code=500,
                detail=f"Failed to format financial data: {str(e)}"
            )

    async def _run_sync_method(self, sync_method):
        """
        Run synchronous defeatbeta-api methods in async context
        """
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, sync_method)

    def _format_transcript_data(self, transcripts_df: Any, symbol: str, quarter: Optional[str] = None, year: Optional[int] = None) -> Dict[str, Any]:
        """
        Format DataFrame transcript data into standardized structure
        """
        try:
            import pandas as pd
            
            # Ensure we have a DataFrame
            if not isinstance(transcripts_df, pd.DataFrame):
                raise ValueError("Expected pandas DataFrame from defeatbeta-api")

            formatted_data = {
                "symbol": symbol.upper(),
                "transcripts": []
            }

            # Filter DataFrame by quarter and year if specified
            filtered_df = transcripts_df.copy()
            
            if year:
                filtered_df = filtered_df[filtered_df['fiscal_year'] == year]
            
            if quarter:
                # Extract quarter number from string like "Q1" -> 1
                quarter_num = int(quarter[1:]) if quarter.startswith('Q') and len(quarter) == 2 else None
                if quarter_num:
                    filtered_df = filtered_df[filtered_df['fiscal_quarter'] == quarter_num]

            # Process each row in the filtered DataFrame
            for _, row in filtered_df.iterrows():
                transcript_item = self._process_transcript_row(row, quarter, year)
                if transcript_item:
                    formatted_data["transcripts"].append(transcript_item)

            if not formatted_data["transcripts"]:
                raise HTTPException(
                    status_code=404,
                    detail=f"No transcripts found for {symbol} with the specified filters"
                )

            return formatted_data

        except HTTPException:
            # Re-raise HTTP exceptions as-is (like 404 for no data found)
            raise
        except Exception as e:
            logger.error(f"Error formatting transcript data: {e}")
            raise HTTPException(
                status_code=500,
                detail=f"Failed to format transcript data: {str(e)}"
            )

    def _process_transcript_row(self, row: Any, quarter: Optional[str] = None, year: Optional[int] = None) -> Optional[Dict[str, Any]]:
        """
        Process individual transcript row from DataFrame
        """
        try:
            import pandas as pd
            import numpy as np
            
            # Helper function to handle NaN/NA values
            def clean_value(value):
                if pd.isna(value) or value is None or (isinstance(value, float) and np.isnan(value)):
                    return None
                return value
            
            # Extract transcript content from the transcripts column
            transcript_content = ""
            participants = []
            
            if 'transcripts' in row and row['transcripts'] is not None and len(row['transcripts']) > 0:
                # The transcripts column contains a numpy array of dictionaries with paragraph data
                transcript_paragraphs = row['transcripts']
                
                # Build full transcript text and extract participants
                transcript_lines = []
                seen_speakers = set()
                
                for paragraph in transcript_paragraphs:
                    speaker = paragraph.get('speaker', 'Unknown')
                    content = paragraph.get('content', '')
                    
                    # Add speaker to participants list
                    if speaker and speaker != 'Unknown' and speaker not in seen_speakers:
                        participants.append(speaker)
                        seen_speakers.add(speaker)
                    
                    # Format transcript line
                    if speaker and content:
                        transcript_lines.append(f"{speaker}: {content}")
                
                transcript_content = "\n\n".join(transcript_lines)
            
            # Determine quarter from fiscal_quarter or use provided quarter
            row_quarter = quarter
            fiscal_quarter = clean_value(row.get('fiscal_quarter'))
            if fiscal_quarter is not None:
                row_quarter = f"Q{fiscal_quarter}"
            
            # Get fiscal year
            row_year = year
            fiscal_year = clean_value(row.get('fiscal_year'))
            if fiscal_year is not None:
                row_year = int(fiscal_year)
            
            # Create date from fiscal year and quarter
            transcript_date = datetime.now()
            if row_year:
                # Estimate date based on fiscal quarter
                quarter_months = {'Q1': 3, 'Q2': 6, 'Q3': 9, 'Q4': 12}
                month = quarter_months.get(row_quarter, 12)
                try:
                    transcript_date = datetime(row_year, month, 15)  # Mid-month estimate
                except:
                    transcript_date = datetime.now()
            
            # Clean all values to prevent serialization errors
            transcript = {
                "symbol": clean_value(row.get('symbol', '')).upper() if clean_value(row.get('symbol', '')) else '',
                "quarter": row_quarter or 'Q1',
                "year": row_year or datetime.now().year,
                "date": transcript_date,
                "transcript": transcript_content,
                "participants": participants,
                "metadata": {
                    "source": "defeatbeta-api",
                    "retrieved_at": datetime.now().isoformat(),
                    "transcripts_id": clean_value(row.get('transcripts_id'))
                }
            }

            return transcript

        except Exception as e:
            logger.error(f"Error processing transcript row: {e}")
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
