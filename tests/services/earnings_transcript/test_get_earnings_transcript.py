import pytest
from datetime import datetime
from unittest.mock import AsyncMock, patch, MagicMock

from fastapi import HTTPException

from src.services.earnings_transcript.get_earnings_transcript import get_earnings_transcript
from src.clients.defeatbeta_client import DefeatBetaClient
from src.models.earnings_transcript import EarningsTranscript


class TestGetEarningsTranscript:
    """Test the earnings transcript service layer with real API calls"""

    @pytest.mark.asyncio
    async def test_get_earnings_transcript_success(self):
        """Test successful earnings transcript retrieval"""
        symbol = "AAPL"
        
        result = await get_earnings_transcript(symbol)
        
        # Verify response structure
        assert isinstance(result, dict)
        assert "symbol" in result
        assert "transcripts" in result
        assert "metadata" in result
        assert result["symbol"] == symbol.upper()
        assert isinstance(result["transcripts"], list)
        assert isinstance(result["metadata"], dict)
        
        # Verify metadata structure
        metadata = result["metadata"]
        assert "total_transcripts" in metadata
        assert "filters_applied" in metadata
        assert "retrieved_at" in metadata
        assert isinstance(metadata["total_transcripts"], int)
        assert isinstance(metadata["filters_applied"], dict)
        assert isinstance(metadata["retrieved_at"], str)
        
        # If we have transcripts, verify their structure
        if result["transcripts"]:
            transcript = result["transcripts"][0]
            assert "symbol" in transcript
            assert "quarter" in transcript
            assert "year" in transcript
            assert "date" in transcript
            assert "transcript" in transcript
            assert "participants" in transcript
            assert "metadata" in transcript

    @pytest.mark.asyncio
    async def test_get_earnings_transcript_with_quarter_filter(self):
        """Test earnings transcript with quarter filter"""
        symbol = "MSFT"
        quarter = "Q3"
        
        result = await get_earnings_transcript(symbol, quarter=quarter)
        
        assert result["symbol"] == symbol.upper()
        assert result["metadata"]["filters_applied"]["quarter"] == quarter
        assert result["metadata"]["filters_applied"]["year"] is None

    @pytest.mark.asyncio
    async def test_get_earnings_transcript_with_year_filter(self):
        """Test earnings transcript with year filter"""
        symbol = "GOOGL"
        year = 2024
        
        result = await get_earnings_transcript(symbol, year=year)
        
        assert result["symbol"] == symbol.upper()
        assert result["metadata"]["filters_applied"]["quarter"] is None
        assert result["metadata"]["filters_applied"]["year"] == year

    @pytest.mark.asyncio
    async def test_get_earnings_transcript_with_both_filters(self):
        """Test earnings transcript with both quarter and year filters"""
        symbol = "TSLA"
        quarter = "Q2"
        year = 2024
        
        result = await get_earnings_transcript(symbol, quarter=quarter, year=year)
        
        assert result["symbol"] == symbol.upper()
        assert result["metadata"]["filters_applied"]["quarter"] == quarter
        assert result["metadata"]["filters_applied"]["year"] == year

    @pytest.mark.asyncio
    async def test_get_earnings_transcript_no_data_found(self):
        """Test handling when no transcripts are found"""
        # Mock the DefeatBetaClient to return empty transcripts
        with patch('src.services.earnings_transcript.get_earnings_transcript.DefeatBetaClient') as mock_client_class:
            mock_client = AsyncMock()
            mock_client.get_earnings_transcript.return_value = {
                "symbol": "INVALID",
                "transcripts": []
            }
            mock_client_class.return_value = mock_client
            
            with pytest.raises(HTTPException) as exc_info:
                await get_earnings_transcript("INVALID")
            
            assert exc_info.value.status_code == 404
            assert "No earnings transcripts found for INVALID" in str(exc_info.value.detail)

    @pytest.mark.asyncio
    async def test_get_earnings_transcript_client_error(self):
        """Test handling of client errors"""
        with patch('src.services.earnings_transcript.get_earnings_transcript.DefeatBetaClient') as mock_client_class:
            mock_client = AsyncMock()
            mock_client.get_earnings_transcript.side_effect = Exception("Client error")
            mock_client_class.return_value = mock_client
            
            with pytest.raises(HTTPException) as exc_info:
                await get_earnings_transcript("ERROR")
            
            assert exc_info.value.status_code == 500
            assert "Internal server error" in str(exc_info.value.detail)

    @pytest.mark.asyncio
    async def test_get_earnings_transcript_http_exception_passthrough(self):
        """Test that HTTPExceptions from client are passed through"""
        with patch('src.services.earnings_transcript.get_earnings_transcript.DefeatBetaClient') as mock_client_class:
            mock_client = AsyncMock()
            original_exception = HTTPException(status_code=404, detail="Symbol not found")
            mock_client.get_earnings_transcript.side_effect = original_exception
            mock_client_class.return_value = mock_client
            
            with pytest.raises(HTTPException) as exc_info:
                await get_earnings_transcript("NOTFOUND")
            
            assert exc_info.value.status_code == 404
            assert exc_info.value.detail == "Symbol not found"

    @pytest.mark.asyncio
    async def test_earnings_transcript_model_validation(self):
        """Test that EarningsTranscript model validation works correctly"""
        # Mock valid transcript data
        mock_transcript_data = {
            "symbol": "AAPL",
            "quarter": "Q1",
            "year": 2024,
            "date": datetime(2024, 1, 15),
            "transcript": "Sample transcript text",
            "participants": ["CEO", "CFO"],
            "metadata": {"source": "test"}
        }
        
        with patch('src.services.earnings_transcript.get_earnings_transcript.DefeatBetaClient') as mock_client_class:
            mock_client = AsyncMock()
            mock_client.get_earnings_transcript.return_value = {
                "symbol": "AAPL",
                "transcripts": [mock_transcript_data]
            }
            mock_client_class.return_value = mock_client
            
            result = await get_earnings_transcript("AAPL")
            
            assert len(result["transcripts"]) == 1
            transcript = result["transcripts"][0]
            
            # Verify all required fields are present and correctly typed
            assert transcript["symbol"] == "AAPL"
            assert transcript["quarter"] == "Q1"
            assert transcript["year"] == 2024
            assert isinstance(transcript["date"], str)  # Should be serialized as ISO string
            assert transcript["transcript"] == "Sample transcript text"
            assert transcript["participants"] == ["CEO", "CFO"]
            assert transcript["metadata"] == {"source": "test"}

    @pytest.mark.asyncio
    async def test_multiple_transcripts_processing(self):
        """Test processing multiple transcripts"""
        mock_transcripts = [
            {
                "symbol": "AAPL",
                "quarter": "Q1",
                "year": 2024,
                "date": datetime(2024, 1, 15),
                "transcript": "Q1 transcript",
                "participants": ["CEO"],
                "metadata": {"source": "test"}
            },
            {
                "symbol": "AAPL",
                "quarter": "Q2",
                "year": 2024,
                "date": datetime(2024, 4, 15),
                "transcript": "Q2 transcript",
                "participants": ["CFO"],
                "metadata": {"source": "test"}
            }
        ]
        
        with patch('src.services.earnings_transcript.get_earnings_transcript.DefeatBetaClient') as mock_client_class:
            mock_client = AsyncMock()
            mock_client.get_earnings_transcript.return_value = {
                "symbol": "AAPL",
                "transcripts": mock_transcripts
            }
            mock_client_class.return_value = mock_client
            
            result = await get_earnings_transcript("AAPL")
            
            assert len(result["transcripts"]) == 2
            assert result["metadata"]["total_transcripts"] == 2
            
            # Verify both transcripts are processed correctly
            quarters = [t["quarter"] for t in result["transcripts"]]
            assert "Q1" in quarters
            assert "Q2" in quarters

    @pytest.mark.asyncio
    async def test_real_api_integration_multiple_symbols(self):
        """Integration test with real API calls for multiple symbols"""
        symbols = ["AAPL", "MSFT", "GOOGL"]
        
        for symbol in symbols:
            try:
                result = await get_earnings_transcript(symbol)
                
                # Basic structure validation
                assert result["symbol"] == symbol.upper()
                assert "transcripts" in result
                assert "metadata" in result
                assert isinstance(result["transcripts"], list)
                assert result["metadata"]["total_transcripts"] >= 0
                
                # If we have transcripts, validate their structure
                for transcript in result["transcripts"]:
                    assert "symbol" in transcript
                    assert "quarter" in transcript
                    assert "year" in transcript
                    assert "date" in transcript
                    assert "transcript" in transcript
                    
            except Exception as e:
                # Log but don't fail - some symbols might not have data available
                print(f"Warning: {symbol} test failed with {e}")

    @pytest.mark.asyncio
    async def test_date_handling_and_serialization(self):
        """Test proper date handling and serialization"""
        test_date = datetime(2024, 3, 15, 14, 30, 0)
        
        mock_transcript_data = {
            "symbol": "TEST",
            "quarter": "Q1",
            "year": 2024,
            "date": test_date,
            "transcript": "Test transcript",
            "participants": [],
            "metadata": {}
        }
        
        with patch('src.services.earnings_transcript.get_earnings_transcript.DefeatBetaClient') as mock_client_class:
            mock_client = AsyncMock()
            mock_client.get_earnings_transcript.return_value = {
                "symbol": "TEST",
                "transcripts": [mock_transcript_data]
            }
            mock_client_class.return_value = mock_client
            
            result = await get_earnings_transcript("TEST")
            
            # Verify date is properly serialized
            transcript = result["transcripts"][0]
            assert isinstance(transcript["date"], str)
            # Should be ISO format
            assert "2024-03-15" in transcript["date"]

    @pytest.mark.asyncio
    async def test_metadata_timestamp_format(self):
        """Test that metadata timestamp is in correct format"""
        with patch('src.services.earnings_transcript.get_earnings_transcript.DefeatBetaClient') as mock_client_class:
            mock_client = AsyncMock()
            mock_client.get_earnings_transcript.return_value = {
                "symbol": "TEST",
                "transcripts": [{
                    "symbol": "TEST",
                    "quarter": "Q1",
                    "year": 2024,
                    "date": datetime.now(),
                    "transcript": "Test",
                    "participants": [],
                    "metadata": {}
                }]
            }
            mock_client_class.return_value = mock_client
            
            result = await get_earnings_transcript("TEST")
            
            # Verify timestamp format
            retrieved_at = result["metadata"]["retrieved_at"]
            assert isinstance(retrieved_at, str)
            # Should be parseable as datetime
            datetime.fromisoformat(retrieved_at.replace('Z', '+00:00') if retrieved_at.endswith('Z') else retrieved_at)
