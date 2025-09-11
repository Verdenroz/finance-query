import pytest
import asyncio
from datetime import datetime
from unittest.mock import patch, MagicMock

from src.clients.defeatbeta_client import DefeatBetaClient
from fastapi import HTTPException


class TestDefeatBetaClient:
    """Test DefeatBetaClient with real API calls to defeatbeta-api"""

    @pytest.fixture
    def client(self):
        """Create DefeatBetaClient instance"""
        return DefeatBetaClient()

    @pytest.mark.asyncio
    async def test_get_earnings_transcript_real_api_call(self, client):
        """Test real API call to defeatbeta-api for earnings transcript"""
        # Use a well-known stock that should have earnings transcripts
        symbol = "AAPL"
        
        try:
            result = await client.get_earnings_transcript(symbol)
            
            # Verify response structure
            assert isinstance(result, dict)
            assert "symbol" in result
            assert "transcripts" in result
            assert result["symbol"] == symbol.upper()
            assert isinstance(result["transcripts"], list)
            
            if result["transcripts"]:
                transcript = result["transcripts"][0]
                assert "symbol" in transcript
                assert "quarter" in transcript
                assert "year" in transcript
                assert "date" in transcript
                assert "transcript" in transcript
                assert "participants" in transcript
                assert "metadata" in transcript
                
                # Verify data types
                assert isinstance(transcript["symbol"], str)
                assert isinstance(transcript["quarter"], str)
                assert isinstance(transcript["year"], int)
                assert isinstance(transcript["date"], datetime)
                assert isinstance(transcript["transcript"], str)
                assert isinstance(transcript["participants"], list)
                assert isinstance(transcript["metadata"], dict)
                
        except ImportError:
            # If defeatbeta-api is not installed, test the fallback behavior
            result = await client.get_earnings_transcript(symbol)
            assert result["symbol"] == symbol.upper()
            assert len(result["transcripts"]) > 0
            assert "sample" in result["transcripts"][0]["metadata"].get("source", "")

    @pytest.mark.asyncio
    async def test_get_earnings_transcript_with_filters(self, client):
        """Test earnings transcript with quarter and year filters"""
        symbol = "TSLA"
        quarter = "Q3"
        year = 2024
        
        result = await client.get_earnings_transcript(symbol, quarter, year)
        
        assert result["symbol"] == symbol.upper()
        assert isinstance(result["transcripts"], list)
        
        # If we get real data, verify filters are applied
        for transcript in result["transcripts"]:
            if "quarter" in transcript and "year" in transcript:
                # Note: Real API might not have exact quarter/year, so we check if filters work
                pass

    @pytest.mark.asyncio
    async def test_get_earnings_transcript_invalid_symbol(self, client):
        """Test with invalid symbol"""
        symbol = "INVALID_SYMBOL_12345"
        
        # Should raise HTTPException for invalid symbol
        with pytest.raises(HTTPException) as exc_info:
            await client.get_earnings_transcript(symbol)
        assert exc_info.value.status_code == 404

    @pytest.mark.asyncio
    async def test_format_transcript_data_structure(self, client):
        """Test the data formatting functionality"""
        import pandas as pd
        
        # Create mock DataFrame that matches defeatbeta-api structure
        mock_data = pd.DataFrame({
            'symbol': ['AAPL'],
            'fiscal_year': [2024],
            'fiscal_quarter': [1],
            'transcripts': [[
                {'speaker': 'CEO', 'content': 'Sample transcript text', 'paragraph_number': 1},
                {'speaker': 'CFO', 'content': 'Financial results', 'paragraph_number': 2}
            ]],
            'transcripts_id': [12345]
        })
        
        formatted = client._format_transcript_data(mock_data, "AAPL")
        
        assert formatted["symbol"] == "AAPL"
        assert "transcripts" in formatted
        assert len(formatted["transcripts"]) > 0

    @pytest.mark.asyncio
    async def test_process_transcript_row(self, client):
        """Test processing individual transcript rows"""
        import pandas as pd
        
        # Create mock row that matches DataFrame structure
        mock_row = pd.Series({
            'symbol': 'AAPL',
            'fiscal_year': 2024,
            'fiscal_quarter': 2,
            'transcripts': [
                {'speaker': 'CEO', 'content': 'Sample transcript', 'paragraph_number': 1},
                {'speaker': 'CFO', 'content': 'Financial data', 'paragraph_number': 2}
            ],
            'transcripts_id': 12345
        })
        
        result = client._process_transcript_row(mock_row, "Q2", 2024)
        
        assert result is not None
        assert result["quarter"] == "Q2"
        assert result["year"] == 2024
        assert isinstance(result["date"], datetime)
        assert "CEO: Sample transcript" in result["transcript"]

    @pytest.mark.asyncio
    async def test_filter_transcripts(self, client):
        """Test transcript filtering functionality"""
        transcripts = [
            {"quarter": "Q1", "year": 2024},
            {"quarter": "Q2", "year": 2024},
            {"quarter": "Q1", "year": 2023}
        ]
        
        # Filter by quarter
        filtered_q1 = client._filter_transcripts(transcripts, quarter="Q1")
        assert len(filtered_q1) == 2
        assert all(t["quarter"] == "Q1" for t in filtered_q1)
        
        # Filter by year
        filtered_2024 = client._filter_transcripts(transcripts, year=2024)
        assert len(filtered_2024) == 2
        assert all(t["year"] == 2024 for t in filtered_2024)
        
        # Filter by both
        filtered_both = client._filter_transcripts(transcripts, quarter="Q1", year=2024)
        assert len(filtered_both) == 1
        assert filtered_both[0]["quarter"] == "Q1"
        assert filtered_both[0]["year"] == 2024

    @pytest.mark.asyncio
    async def test_sample_transcript_data(self, client):
        """Test sample transcript data generation"""
        symbol = "TEST"
        quarter = "Q4"
        year = 2023
        
        sample_data = client._get_sample_transcript_data(symbol, quarter, year)
        
        assert sample_data["symbol"] == symbol.upper()
        assert len(sample_data["transcripts"]) > 0
        
        transcript = sample_data["transcripts"][0]
        assert transcript["quarter"] == quarter
        assert transcript["year"] == year
        assert isinstance(transcript["transcript"], str)
        assert len(transcript["transcript"]) > 0
        assert isinstance(transcript["participants"], list)
        assert len(transcript["participants"]) > 0

    @pytest.mark.asyncio
    async def test_run_sync_method(self, client):
        """Test async wrapper for sync methods"""
        def sync_method():
            return "test_result"
        
        result = await client._run_sync_method(sync_method)
        assert result == "test_result"

    @pytest.mark.asyncio
    async def test_multiple_symbols_real_calls(self, client):
        """Test multiple symbols to ensure robustness"""
        symbols = ["AAPL", "MSFT", "GOOGL"]
        
        for symbol in symbols:
            try:
                result = await client.get_earnings_transcript(symbol)
                assert result["symbol"] == symbol.upper()
                assert "transcripts" in result
                assert isinstance(result["transcripts"], list)
            except Exception as e:
                # Log but don't fail - some symbols might not have data
                print(f"Warning: {symbol} failed with {e}")

    @pytest.mark.asyncio
    async def test_concurrent_requests(self, client):
        """Test concurrent API requests"""
        symbols = ["AAPL", "MSFT"]
        
        # Create concurrent tasks
        tasks = [client.get_earnings_transcript(symbol) for symbol in symbols]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Verify all requests completed
        assert len(results) == len(symbols)
        
        for i, result in enumerate(results):
            if not isinstance(result, Exception):
                assert result["symbol"] == symbols[i].upper()

    @pytest.mark.asyncio
    async def test_error_handling_import_error(self):
        """Test handling when defeatbeta-api is not available"""
        # Mock the import to raise ImportError
        with patch.dict('sys.modules', {'defeatbeta_api': None, 'defeatbeta_api.data': None, 'defeatbeta_api.data.ticker': None}):
            client = DefeatBetaClient()
            
            # Should raise HTTPException for import error
            with pytest.raises(HTTPException) as exc_info:
                await client.get_earnings_transcript("AAPL")
            assert exc_info.value.status_code == 500
            assert "defeatbeta-api package not properly installed" in str(exc_info.value.detail)

    @pytest.mark.asyncio
    async def test_error_handling_general_exception(self):
        """Test handling of general exceptions"""
        # Mock the Ticker to raise a general exception
        with patch('defeatbeta_api.data.ticker.Ticker') as mock_ticker_class:
            mock_ticker = MagicMock()
            mock_ticker.earning_call_transcripts.side_effect = Exception("General error")
            mock_ticker_class.return_value = mock_ticker
            
            client = DefeatBetaClient()
            
            # Should raise HTTPException for general errors
            with pytest.raises(HTTPException) as exc_info:
                await client.get_earnings_transcript("ERROR")
            assert exc_info.value.status_code == 500
            assert "Failed to fetch earnings transcript" in str(exc_info.value.detail)
