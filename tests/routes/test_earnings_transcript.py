import pytest
from fastapi.testclient import TestClient
from unittest.mock import AsyncMock, patch, MagicMock
from datetime import datetime

from src.main import app


class TestEarningsTranscriptRoutes:
    """Test earnings transcript API endpoints with real API calls"""

    @pytest.fixture
    def client(self):
        """Create test client"""
        return TestClient(app)

    def test_get_earnings_transcript_endpoint_success(self, client):
        """Test GET /v1/earnings-transcript/{symbol} endpoint"""
        response = client.get("/v1/earnings-transcript/AAPL")
        
        assert response.status_code == 200
        data = response.json()
        
        # Verify response structure
        assert "symbol" in data
        assert "transcripts" in data
        assert "metadata" in data
        assert data["symbol"] == "AAPL"
        assert isinstance(data["transcripts"], list)
        assert isinstance(data["metadata"], dict)
        
        # Verify metadata structure
        metadata = data["metadata"]
        assert "total_transcripts" in metadata
        assert "filters_applied" in metadata
        assert "retrieved_at" in metadata

    def test_get_earnings_transcript_with_quarter_filter(self, client):
        """Test endpoint with quarter filter"""
        response = client.get("/v1/earnings-transcript/MSFT?quarter=Q3")
        
        assert response.status_code == 200
        data = response.json()
        assert data["symbol"] == "MSFT"
        assert data["metadata"]["filters_applied"]["quarter"] == "Q3"

    def test_get_earnings_transcript_with_year_filter(self, client):
        """Test endpoint with year filter"""
        response = client.get("/v1/earnings-transcript/GOOGL?year=2024")
        
        assert response.status_code == 200
        data = response.json()
        assert data["symbol"] == "GOOGL"
        assert data["metadata"]["filters_applied"]["year"] == 2024

    def test_get_earnings_transcript_with_both_filters(self, client):
        """Test endpoint with both quarter and year filters"""
        response = client.get("/v1/earnings-transcript/TSLA?quarter=Q2&year=2024")
        
        assert response.status_code == 200
        data = response.json()
        assert data["symbol"] == "TSLA"
        assert data["metadata"]["filters_applied"]["quarter"] == "Q2"
        assert data["metadata"]["filters_applied"]["year"] == 2024

    def test_get_earnings_transcript_invalid_quarter(self, client):
        """Test endpoint with invalid quarter parameter"""
        response = client.get("/v1/earnings-transcript/AAPL?quarter=Q5")
        
        # Should still work but might not find data
        assert response.status_code in [200, 404]

    def test_get_earnings_transcript_invalid_year(self, client):
        """Test endpoint with invalid year parameter"""
        response = client.get("/v1/earnings-transcript/AAPL?year=1900")
        
        # Should still work but might not find data
        assert response.status_code in [200, 404]

    def test_post_earnings_transcript_analyze_endpoint(self, client):
        """Test POST /v1/earnings-transcript/analyze endpoint"""
        request_data = {
            "symbol": "AAPL",
            "quarter": "Q3",
            "year": 2024
        }
        
        response = client.post("/v1/earnings-transcript/analyze", json=request_data)
        
        assert response.status_code == 200
        data = response.json()
        assert data["symbol"] == "AAPL"
        assert data["metadata"]["filters_applied"]["quarter"] == "Q3"
        assert data["metadata"]["filters_applied"]["year"] == 2024

    def test_post_earnings_transcript_analyze_minimal_data(self, client):
        """Test POST endpoint with minimal required data"""
        request_data = {
            "symbol": "MSFT"
        }
        
        response = client.post("/v1/earnings-transcript/analyze", json=request_data)
        
        assert response.status_code == 200
        data = response.json()
        assert data["symbol"] == "MSFT"

    def test_post_earnings_transcript_analyze_invalid_data(self, client):
        """Test POST endpoint with invalid request data"""
        request_data = {
            "invalid_field": "value"
        }
        
        response = client.post("/v1/earnings-transcript/analyze", json=request_data)
        
        assert response.status_code == 422  # Validation error

    def test_get_latest_earnings_transcript_endpoint(self, client):
        """Test GET /v1/earnings-transcript/{symbol}/latest endpoint"""
        response = client.get("/v1/earnings-transcript/AAPL/latest")
        
        assert response.status_code == 200
        data = response.json()
        
        assert data["symbol"] == "AAPL"
        assert "transcripts" in data
        assert "metadata" in data
        
        # Should return only one transcript (the latest)
        if data["transcripts"]:
            assert data["metadata"]["total_transcripts"] == 1
            assert "note" in data["metadata"]
            assert "Latest transcript only" in data["metadata"]["note"]

    def test_multiple_symbols_real_api_calls(self, client):
        """Test multiple symbols with real API calls"""
        symbols = ["AAPL", "MSFT", "GOOGL", "TSLA", "AMZN"]
        
        for symbol in symbols:
            response = client.get(f"/v1/earnings-transcript/{symbol}")
            
            # Should either succeed or return 404 if no data
            assert response.status_code in [200, 404]
            
            if response.status_code == 200:
                data = response.json()
                assert data["symbol"] == symbol
                assert "transcripts" in data
                assert "metadata" in data

    def test_case_insensitive_symbol_handling(self, client):
        """Test that symbols are handled case-insensitively"""
        # Test lowercase
        response_lower = client.get("/v1/earnings-transcript/aapl")
        # Test uppercase
        response_upper = client.get("/v1/earnings-transcript/AAPL")
        # Test mixed case
        response_mixed = client.get("/v1/earnings-transcript/AaPl")
        
        # All should return same status
        assert response_lower.status_code == response_upper.status_code == response_mixed.status_code
        
        if response_lower.status_code == 200:
            data_lower = response_lower.json()
            data_upper = response_upper.json()
            data_mixed = response_mixed.json()
            
            # All should return uppercase symbol
            assert data_lower["symbol"] == "AAPL"
            assert data_upper["symbol"] == "AAPL"
            assert data_mixed["symbol"] == "AAPL"

    def test_response_headers_and_content_type(self, client):
        """Test response headers and content type"""
        response = client.get("/v1/earnings-transcript/AAPL")
        
        assert response.headers["content-type"] == "application/json"
        
        if response.status_code == 200:
            # Verify JSON is valid
            data = response.json()
            assert isinstance(data, dict)

    def test_error_handling_404(self, client):
        """Test 404 error handling"""
        # Mock the service to raise 404
        with patch('src.routes.earnings_transcript.get_earnings_transcript') as mock_service:
            from fastapi import HTTPException
            mock_service.side_effect = HTTPException(status_code=404, detail="No earnings transcripts found for INVALID")
            
            response = client.get("/v1/earnings-transcript/INVALID")
            
            assert response.status_code == 404
            data = response.json()
            assert "detail" in data
            assert "No earnings transcripts found" in data["detail"]

    def test_error_handling_500(self, client):
        """Test 500 error handling"""
        # Mock the service to raise 500
        with patch('src.routes.earnings_transcript.get_earnings_transcript') as mock_service:
            from fastapi import HTTPException
            mock_service.side_effect = HTTPException(status_code=500, detail="Internal server error")
            
            response = client.get("/v1/earnings-transcript/ERROR")
            
            assert response.status_code == 500
            data = response.json()
            assert "detail" in data
            assert "Internal server error" in data["detail"]

    def test_concurrent_requests(self, client):
        """Test handling of concurrent requests"""
        import threading
        import time
        
        results = []
        
        def make_request(symbol):
            response = client.get(f"/v1/earnings-transcript/{symbol}")
            results.append((symbol, response.status_code))
        
        # Create multiple threads for concurrent requests
        threads = []
        symbols = ["AAPL", "MSFT", "GOOGL"]
        
        for symbol in symbols:
            thread = threading.Thread(target=make_request, args=(symbol,))
            threads.append(thread)
            thread.start()
        
        # Wait for all threads to complete
        for thread in threads:
            thread.join()
        
        # Verify all requests completed
        assert len(results) == len(symbols)
        
        # All should return valid status codes
        for symbol, status_code in results:
            assert status_code in [200, 404, 500]

    def test_large_symbol_name(self, client):
        """Test handling of unusually large symbol names"""
        large_symbol = "A" * 100  # Very long symbol
        
        response = client.get(f"/v1/earnings-transcript/{large_symbol}")
        
        # Should handle gracefully
        assert response.status_code in [200, 404, 422]

    def test_special_characters_in_symbol(self, client):
        """Test handling of special characters in symbol"""
        special_symbols = ["BRK.A", "BRK-B", "TEST@", "TEST#"]
        
        for symbol in special_symbols:
            response = client.get(f"/v1/earnings-transcript/{symbol}")
            
            # Should handle gracefully without crashing
            assert response.status_code in [200, 404, 422]

    def test_response_time_performance(self, client):
        """Test response time performance"""
        import time
        
        start_time = time.time()
        response = client.get("/v1/earnings-transcript/AAPL")
        end_time = time.time()
        
        response_time = end_time - start_time
        
        # Response should be reasonably fast (under 30 seconds for real API calls)
        assert response_time < 30.0
        
        # Should return valid response
        assert response.status_code in [200, 404]

    def test_json_response_structure_validation(self, client):
        """Test detailed JSON response structure validation"""
        response = client.get("/v1/earnings-transcript/AAPL")
        
        if response.status_code == 200:
            data = response.json()
            
            # Verify top-level structure
            required_fields = ["symbol", "transcripts", "metadata"]
            for field in required_fields:
                assert field in data, f"Missing required field: {field}"
            
            # Verify metadata structure
            metadata = data["metadata"]
            metadata_fields = ["total_transcripts", "filters_applied", "retrieved_at"]
            for field in metadata_fields:
                assert field in metadata, f"Missing metadata field: {field}"
            
            # Verify filters_applied structure
            filters = metadata["filters_applied"]
            assert "quarter" in filters
            assert "year" in filters
            
            # If transcripts exist, verify their structure
            for transcript in data["transcripts"]:
                transcript_fields = ["symbol", "quarter", "year", "date", "transcript", "participants", "metadata"]
                for field in transcript_fields:
                    assert field in transcript, f"Missing transcript field: {field}"

    def test_endpoint_documentation_compliance(self, client):
        """Test that endpoints comply with OpenAPI documentation"""
        # Test that all documented endpoints exist and return expected status codes
        endpoints = [
            "/v1/earnings-transcript/AAPL",
            "/v1/earnings-transcript/AAPL/latest"
        ]
        
        for endpoint in endpoints:
            response = client.get(endpoint)
            # Should not return 404 for endpoint not found
            assert response.status_code != 404 or "not found" not in response.json().get("detail", "").lower()

    def test_api_key_header_handling(self, client):
        """Test API key header handling (if implemented)"""
        # Test without API key
        response = client.get("/v1/earnings-transcript/AAPL")
        # Should work (API key is optional in current implementation)
        assert response.status_code in [200, 404]
        
        # Test with API key
        headers = {"x-api-key": "test-api-key"}
        response = client.get("/v1/earnings-transcript/AAPL", headers=headers)
        assert response.status_code in [200, 404]
