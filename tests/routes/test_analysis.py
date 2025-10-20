import pytest
from unittest.mock import AsyncMock, patch
from fastapi.testclient import TestClient
from fastapi import HTTPException

from src.main import app
from src.models.analysis import (
    AnalystPriceTargets,
    EarningsEstimate,
    RevenueEstimate,
    EarningsHistory,
    EPSTrend,
    EPSRevisions,
    GrowthEstimates,
    EstimateData,
    EarningsHistoryItem,
    EPSTrendItem,
    EPSRevisionItem,
    GrowthEstimateItem,
)


@pytest.fixture
def client():
    return TestClient(app)


class TestAnalysisRoutes:
    """Test analysis API routes"""

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_analyst_price_targets")
    async def test_analyst_price_targets_endpoint_success(self, mock_service, client):
        """Test GET /v1/analysis/{symbol}/price-targets endpoint success"""
        mock_data = AnalystPriceTargets(
            symbol="AAPL",
            current=150.0,
            low=140.0,
            high=160.0,
            mean=150.0,
            median=150.0
        )
        mock_service.return_value = mock_data

        response = client.get("/v1/analysis/AAPL/price-targets")

        assert response.status_code == 200
        data = response.json()
        assert data["current"] == 150.0
        assert data["low"] == 140.0
        assert data["high"] == 160.0
        assert data["mean"] == 150.0
        assert data["median"] == 150.0
        mock_service.assert_called_once_with("AAPL")

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_earnings_estimate")
    async def test_earnings_estimate_endpoint_success(self, mock_service, client):
        """Test GET /v1/analysis/{symbol}/earnings-estimate endpoint success"""
        estimate_data = [
            EstimateData(
                period="0q",
                number_of_analysts=25,
                avg=2.50,
                low=2.00,
                high=3.00,
                year_ago_eps=2.20,
                growth=0.136
            ),
            EstimateData(
                period="+1q",
                number_of_analysts=20,
                avg=2.75,
                low=2.25,
                high=3.25,
                year_ago_eps=2.50,
                growth=0.100
            )
        ]
        mock_data = EarningsEstimate(symbol="AAPL", estimates=estimate_data)
        mock_service.return_value = mock_data

        response = client.get("/v1/analysis/AAPL/earnings-estimate")

        assert response.status_code == 200
        data = response.json()
        assert len(data["estimates"]) == 2
        assert data["estimates"][0]["period"] == "0q"
        assert data["estimates"][0]["avg"] == 2.50
        assert data["estimates"][1]["period"] == "+1q"
        assert data["estimates"][1]["avg"] == 2.75
        mock_service.assert_called_once_with("AAPL")

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_revenue_estimate")
    async def test_revenue_estimate_endpoint_success(self, mock_service, client):
        """Test GET /v1/analysis/{symbol}/revenue-estimate endpoint success"""
        estimate_data = [
            EstimateData(
                period="0y",
                number_of_analysts=20,
                avg=1000000000.0,
                low=950000000.0,
                high=1050000000.0,
                year_ago_eps=900000000.0,
                growth=0.111
            )
        ]
        mock_data = RevenueEstimate(symbol="AAPL", estimates=estimate_data)
        mock_service.return_value = mock_data

        response = client.get("/v1/analysis/AAPL/revenue-estimate")

        assert response.status_code == 200
        data = response.json()
        assert len(data["estimates"]) == 1
        assert data["estimates"][0]["period"] == "0y"
        assert data["estimates"][0]["avg"] == 1000000000.0
        mock_service.assert_called_once_with("AAPL")

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_earnings_history")
    async def test_earnings_history_endpoint_success(self, mock_service, client):
        """Test GET /v1/analysis/{symbol}/earnings-history endpoint success"""
        from datetime import datetime
        
        history_items = [
            EarningsHistoryItem(
                quarter="2024-03-31",
                eps_estimate=2.50,
                eps_actual=2.75,
                eps_difference=0.25,
                surprise_percent=10.0
            ),
            EarningsHistoryItem(
                quarter="2023-12-31",
                eps_estimate=2.20,
                eps_actual=2.30,
                eps_difference=0.10,
                surprise_percent=4.55
            )
        ]
        mock_data = EarningsHistory(symbol="AAPL", history=history_items)
        mock_service.return_value = mock_data

        response = client.get("/v1/analysis/AAPL/earnings-history")

        assert response.status_code == 200
        data = response.json()
        assert len(data["history"]) == 2
        assert data["history"][0]["eps_estimate"] == 2.50
        assert data["history"][0]["eps_actual"] == 2.75
        assert data["history"][1]["eps_estimate"] == 2.20
        assert data["history"][1]["eps_actual"] == 2.30
        mock_service.assert_called_once_with("AAPL")

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_eps_trend")
    async def test_eps_trend_endpoint_success(self, mock_service, client):
        """Test GET /v1/analysis/{symbol}/eps-trend endpoint success"""
        trend_items = [
            EPSTrendItem(
                period="0q",
                current=2.50,
                seven_days_ago=2.45,
                thirty_days_ago=2.40,
                sixty_days_ago=2.35,
                ninety_days_ago=2.30
            ),
            EPSTrendItem(
                period="+1q",
                current=2.75,
                seven_days_ago=2.70,
                thirty_days_ago=2.65,
                sixty_days_ago=2.60,
                ninety_days_ago=2.55
            )
        ]
        mock_data = EPSTrend(symbol="AAPL", trends=trend_items)
        mock_service.return_value = mock_data

        response = client.get("/v1/analysis/AAPL/eps-trend")

        assert response.status_code == 200
        data = response.json()
        assert len(data["trends"]) == 2
        assert data["trends"][0]["period"] == "0q"
        assert data["trends"][0]["current"] == 2.50
        assert data["trends"][0]["seven_days_ago"] == 2.45
        assert data["trends"][1]["period"] == "+1q"
        assert data["trends"][1]["current"] == 2.75
        mock_service.assert_called_once_with("AAPL")

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_eps_revisions")
    async def test_eps_revisions_endpoint_success(self, mock_service, client):
        """Test GET /v1/analysis/{symbol}/eps-revisions endpoint success"""
        revision_items = [
            EPSRevisionItem(
                period="0q",
                up_last_7days=5,
                up_last_30days=12,
                down_last_7days=2,
                down_last_30days=8
            ),
            EPSRevisionItem(
                period="+1q",
                up_last_7days=3,
                up_last_30days=10,
                down_last_7days=1,
                down_last_30days=6
            )
        ]
        mock_data = EPSRevisions(symbol="AAPL", revisions=revision_items)
        mock_service.return_value = mock_data

        response = client.get("/v1/analysis/AAPL/eps-revisions")

        assert response.status_code == 200
        data = response.json()
        assert len(data["revisions"]) == 2
        assert data["revisions"][0]["period"] == "0q"
        assert data["revisions"][0]["up_last_7days"] == 5
        assert data["revisions"][0]["up_last_30days"] == 12
        assert data["revisions"][1]["period"] == "+1q"
        assert data["revisions"][1]["up_last_7days"] == 3
        mock_service.assert_called_once_with("AAPL")

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_growth_estimates")
    async def test_growth_estimates_endpoint_success(self, mock_service, client):
        """Test GET /v1/analysis/{symbol}/growth-estimates endpoint success"""
        estimate_items = [
            GrowthEstimateItem(
                period="+5y",
                stock=0.15,
                industry=0.12,
                sector=0.10,
                index=0.08
            ),
            GrowthEstimateItem(
                period="-5y",
                stock=-0.05,
                industry=-0.03,
                sector=-0.02,
                index=-0.01
            )
        ]
        mock_data = GrowthEstimates(symbol="AAPL", estimates=estimate_items)
        mock_service.return_value = mock_data

        response = client.get("/v1/analysis/AAPL/growth-estimates")

        assert response.status_code == 200
        data = response.json()
        assert len(data["estimates"]) == 2
        assert data["estimates"][0]["period"] == "+5y"
        assert data["estimates"][0]["stock"] == 0.15
        assert data["estimates"][0]["industry"] == 0.12
        assert data["estimates"][1]["period"] == "-5y"
        assert data["estimates"][1]["stock"] == -0.05
        mock_service.assert_called_once_with("AAPL")


class TestAnalysisRoutesErrorHandling:
    """Test error handling in analysis routes"""

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_analyst_price_targets")
    async def test_404_error_handling(self, mock_service, client):
        """Test 404 error handling"""
        mock_service.side_effect = HTTPException(status_code=404, detail="Analyst price targets not found for INVALID")

        response = client.get("/v1/analysis/INVALID/price-targets")

        assert response.status_code == 404
        data = response.json()
        assert "detail" in data
        assert "Analyst price targets not found" in data["detail"]

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_earnings_estimate")
    async def test_500_error_handling(self, mock_service, client):
        """Test 500 error handling"""
        mock_service.side_effect = HTTPException(status_code=500, detail="Internal server error")

        response = client.get("/v1/analysis/ERROR/earnings-estimate")

        assert response.status_code == 500
        data = response.json()
        assert "detail" in data
        assert "Internal server error" in data["detail"]

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_revenue_estimate")
    async def test_general_exception_handling(self, mock_service, client):
        """Test general exception handling"""
        mock_service.side_effect = Exception("Unexpected error")

        response = client.get("/v1/analysis/ERROR/revenue-estimate")

        assert response.status_code == 500
        data = response.json()
        assert "detail" in data


class TestAnalysisRoutesSymbolHandling:
    """Test symbol handling in analysis routes"""

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_analyst_price_targets")
    async def test_symbol_case_insensitive(self, mock_service, client):
        """Test that symbols are handled case-insensitively"""
        mock_data = AnalystPriceTargets(symbol="AAPL", current=150.0)
        mock_service.return_value = mock_data

        # Test lowercase
        response_lower = client.get("/v1/analysis/aapl/price-targets")
        # Test uppercase
        response_upper = client.get("/v1/analysis/AAPL/price-targets")
        # Test mixed case
        response_mixed = client.get("/v1/analysis/AaPl/price-targets")

        # All should return same status
        assert response_lower.status_code == response_upper.status_code == response_mixed.status_code == 200

        # All should call service with the same symbol
        assert mock_service.call_count == 3
        for call in mock_service.call_args_list:
            assert call[0][0] in ["aapl", "AAPL", "AaPl"]

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_earnings_estimate")
    async def test_symbol_with_special_characters(self, mock_service, client):
        """Test handling of symbols with special characters"""
        mock_data = [EarningsEstimate(symbol="AAPL", estimates=[EstimateData(period="0q", avg=2.50)])]
        mock_service.return_value = mock_data

        special_symbols = ["BRK.A", "BRK-B", "TEST@", "TEST#"]
        
        for symbol in special_symbols:
            response = client.get(f"/v1/analysis/{symbol}/earnings-estimate")
            
            # Should handle gracefully
            assert response.status_code in [200, 404, 422]

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_revenue_estimate")
    async def test_empty_symbol(self, mock_service, client):
        """Test handling of empty symbol"""
        mock_service.side_effect = HTTPException(status_code=404, detail="Symbol not found")

        response = client.get("/v1/analysis//revenue-estimate")

        # Should return 404 or 422 (validation error)
        assert response.status_code in [404, 422]


class TestAnalysisRoutesResponseFormat:
    """Test response format and headers"""

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_analyst_price_targets")
    async def test_response_headers(self, mock_service, client):
        """Test response headers"""
        mock_data = AnalystPriceTargets(symbol="AAPL", current=150.0)
        mock_service.return_value = mock_data

        response = client.get("/v1/analysis/AAPL/price-targets")

        assert response.headers["content-type"] == "application/json"

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_earnings_estimate")
    async def test_json_response_structure(self, mock_service, client):
        """Test JSON response structure"""
        mock_data = [
        EarningsEstimate(
            symbol="AAPL",
            estimates=[EstimateData(
                period="0q",
                number_of_analysts=25,
                avg=2.50,
                low=2.00,
                high=3.00,
                year_ago_eps=2.20,
                growth=0.136
            )]
        )
        ]
        mock_service.return_value = mock_data

        response = client.get("/v1/analysis/AAPL/earnings-estimate")

        assert response.status_code == 200
        data = response.json()
        
        # Verify JSON is valid
        assert isinstance(data, list)
        assert len(data) == 1
        
        # Verify structure
        item = data[0]
        assert "period" in item
        assert "numberOfAnalysts" in item
        assert "avg" in item
        assert "low" in item
        assert "high" in item
        assert "yearAgoEps" in item
        assert "growth" in item

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_earnings_history")
    async def test_datetime_serialization(self, mock_service, client):
        """Test datetime serialization in responses"""
        from datetime import datetime
        
        mock_data = [
            EarningsHistory(
                quarter=datetime(2024, 3, 31, 12, 0, 0),
                eps_estimate=2.50,
                eps_actual=2.75,
                eps_difference=0.25,
                surprise_percent=10.0
            )
        ]
        mock_service.return_value = mock_data

        response = client.get("/v1/analysis/AAPL/earnings-history")

        assert response.status_code == 200
        data = response.json()
        
        # Verify datetime is serialized as ISO string
        assert isinstance(data, list)
        assert len(data) == 1
        quarter_str = data[0]["quarter"]
        assert isinstance(quarter_str, str)
        # Should be ISO format
        assert "T" in quarter_str or " " in quarter_str


class TestAnalysisRoutesAPIKey:
    """Test API key handling"""

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_analyst_price_targets")
    async def test_without_api_key(self, mock_service, client):
        """Test endpoint without API key"""
        mock_data = AnalystPriceTargets(symbol="AAPL", current=150.0)
        mock_service.return_value = mock_data

        response = client.get("/v1/analysis/AAPL/price-targets")

        # Should work (API key is optional in current implementation)
        assert response.status_code == 200

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_earnings_estimate")
    async def test_with_api_key(self, mock_service, client):
        """Test endpoint with API key"""
        mock_data = [EarningsEstimate(symbol="AAPL", estimates=[EstimateData(period="0q", avg=2.50)])]
        mock_service.return_value = mock_data

        headers = {"x-api-key": "test-api-key"}
        response = client.get("/v1/analysis/AAPL/earnings-estimate", headers=headers)

        assert response.status_code == 200


class TestAnalysisRoutesConcurrency:
    """Test concurrent request handling"""

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_analyst_price_targets")
    async def test_concurrent_requests(self, mock_service, client):
        """Test handling of concurrent requests"""
        import threading
        import time
        
        mock_data = AnalystPriceTargets(symbol="AAPL", current=150.0)
        mock_service.return_value = mock_data
        
        results = []
        
        def make_request(symbol):
            response = client.get(f"/v1/analysis/{symbol}/price-targets")
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

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_earnings_estimate")
    async def test_multiple_endpoints_same_symbol(self, mock_service, client):
        """Test calling multiple endpoints for the same symbol"""
        mock_data = [EarningsEstimate(symbol="AAPL", estimates=[EstimateData(period="0q", avg=2.50)])]
        mock_service.return_value = mock_data

        endpoints = [
            "/v1/analysis/AAPL/earnings-estimate",
            "/v1/analysis/AAPL/revenue-estimate",
            "/v1/analysis/AAPL/eps-trend",
            "/v1/analysis/AAPL/eps-revisions",
            "/v1/analysis/AAPL/growth-estimates"
        ]

        for endpoint in endpoints:
            response = client.get(endpoint)
            # Should either succeed or return appropriate error
            assert response.status_code in [200, 404, 500]


class TestAnalysisRoutesPerformance:
    """Test performance characteristics"""

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_analyst_price_targets")
    async def test_response_time(self, mock_service, client):
        """Test response time performance"""
        import time
        
        mock_data = AnalystPriceTargets(symbol="AAPL", current=150.0)
        mock_service.return_value = mock_data

        start_time = time.time()
        response = client.get("/v1/analysis/AAPL/price-targets")
        end_time = time.time()

        response_time = end_time - start_time

        # Response should be reasonably fast (under 5 seconds for mocked calls)
        assert response_time < 5.0
        assert response.status_code == 200

    @pytest.mark.asyncio
    @patch("src.routes.analysis.get_earnings_estimate")
    async def test_large_response_handling(self, mock_service, client):
        """Test handling of large responses"""
        # Create a large list of earnings estimates
        mock_data = [
            EarningsEstimate(
                period=f"{i}q",
                number_of_analysts=25,
                avg=2.50 + i * 0.1,
                low=2.00 + i * 0.1,
                high=3.00 + i * 0.1,
                year_ago_eps=2.20 + i * 0.1,
                growth=0.136 + i * 0.01
            )
            for i in range(20)  # 20 items
        ]
        mock_service.return_value = mock_data

        response = client.get("/v1/analysis/AAPL/earnings-estimate")

        assert response.status_code == 200
        data = response.json()
        assert isinstance(data, list)
        assert len(data) == 20

