from unittest.mock import MagicMock, patch

import pytest
from fastapi.testclient import TestClient

from src.main import app


@pytest.fixture
def client():
    return TestClient(app)


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_recommendations(mock_ticker, client):
    """Test analysis endpoint with recommendations type"""
    # Mock the yfinance Ticker object and its methods
    mock_ticker_instance = MagicMock()
    mock_ticker_instance.recommendations = MagicMock()
    mock_ticker_instance.recommendations.empty = False
    mock_ticker_instance.recommendations.iterrows.return_value = [
        (0, {"period": "3m", "strongBuy": 5, "buy": 10, "hold": 3, "sell": 1, "strongSell": 0}),
        (1, {"period": "1m", "strongBuy": 3, "buy": 8, "hold": 5, "sell": 2, "strongSell": 1}),
    ]
    mock_ticker.return_value = mock_ticker_instance

    response = client.get("/v1/analysis/AAPL?analysis_type=recommendations")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["analysis_type"] == "recommendations"
    assert data["recommendations"] is not None
    assert len(data["recommendations"]) == 2
    assert data["recommendations"][0]["period"] == "3m"
    assert data["recommendations"][0]["strong_buy"] == 5


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_price_targets(mock_ticker, client):
    """Test analysis endpoint with price targets type"""
    # Mock the yfinance Ticker object and its methods
    mock_ticker_instance = MagicMock()
    mock_ticker_instance.analyst_price_targets = {"current": 150.0, "mean": 160.0, "median": 155.0, "low": 140.0, "high": 180.0}
    mock_ticker.return_value = mock_ticker_instance

    response = client.get("/v1/analysis/AAPL?analysis_type=price_targets")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["analysis_type"] == "price_targets"
    assert data["price_targets"] is not None
    assert data["price_targets"]["current"] == 150.0
    assert data["price_targets"]["mean"] == 160.0
    assert data["price_targets"]["high"] == 180.0


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_earnings_estimate(mock_ticker, client):
    """Test analysis endpoint with earnings estimate type"""
    import pandas as pd

    # Mock the yfinance Ticker object and its methods
    mock_ticker_instance = MagicMock()
    mock_df = pd.DataFrame({"2024-12-31": {"avg": 6.5, "low": 6.0, "high": 7.0}, "2025-12-31": {"avg": 7.2, "low": 6.8, "high": 7.6}})
    mock_ticker_instance.earnings_estimate = mock_df
    mock_ticker.return_value = mock_ticker_instance

    response = client.get("/v1/analysis/AAPL?analysis_type=earnings_estimate")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["analysis_type"] == "earnings_estimate"
    assert data["earnings_estimate"] is not None
    assert "estimates" in data["earnings_estimate"]


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_sustainability(mock_ticker, client):
    """Test analysis endpoint with sustainability type"""
    # Mock the yfinance Ticker object and its methods
    mock_ticker_instance = MagicMock()
    mock_ticker_instance.sustainability = MagicMock()
    mock_ticker_instance.sustainability.empty = False
    mock_ticker_instance.sustainability.columns = ["environmentScore", "socialScore", "governanceScore"]
    mock_ticker_instance.sustainability.__getitem__.side_effect = lambda x: {"environmentScore": 75, "socialScore": 80, "governanceScore": 85}[x]
    mock_ticker_instance.sustainability.iloc = [75, 80, 85]
    mock_ticker.return_value = mock_ticker_instance

    response = client.get("/v1/analysis/AAPL?analysis_type=sustainability")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["analysis_type"] == "sustainability"
    assert data["sustainability"] is not None
    assert "scores" in data["sustainability"]


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_upgrades_downgrades(mock_ticker, client):
    """Test analysis endpoint with upgrades/downgrades type"""
    # Mock the yfinance Ticker object and its methods
    mock_ticker_instance = MagicMock()
    mock_ticker_instance.upgrades_downgrades = MagicMock()
    mock_ticker_instance.upgrades_downgrades.empty = False
    mock_ticker_instance.upgrades_downgrades.iterrows.return_value = [
        (0, {"firm": "Goldman Sachs", "toGrade": "Buy", "fromGrade": "Hold", "action": "upgrade", "date": "2024-01-15"}),
        (1, {"firm": "Morgan Stanley", "toGrade": "Hold", "fromGrade": "Buy", "action": "downgrade", "date": "2024-01-10"}),
    ]
    mock_ticker.return_value = mock_ticker_instance

    response = client.get("/v1/analysis/AAPL?analysis_type=upgrades_downgrades")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["analysis_type"] == "upgrades_downgrades"
    assert data["upgrades_downgrades"] is not None
    assert len(data["upgrades_downgrades"]) == 2
    assert data["upgrades_downgrades"][0]["firm"] == "Goldman Sachs"
    assert data["upgrades_downgrades"][0]["action"] == "upgrade"


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_earnings_history(mock_ticker, client):
    """Test analysis endpoint with earnings history type"""
    # Mock the yfinance Ticker object and its methods
    mock_ticker_instance = MagicMock()
    mock_ticker_instance.earnings_history = MagicMock()
    mock_ticker_instance.earnings_history.empty = False
    mock_ticker_instance.earnings_history.iterrows.return_value = [
        (0, {"date": "2024-01-15", "eps_actual": 2.18, "eps_estimate": 2.10, "surprise": 0.08, "surprise_percent": 3.8}),
        (1, {"date": "2023-10-15", "eps_actual": 1.46, "eps_estimate": 1.39, "surprise": 0.07, "surprise_percent": 5.0}),
    ]
    mock_ticker.return_value = mock_ticker_instance

    response = client.get("/v1/analysis/AAPL?analysis_type=earnings_history")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["analysis_type"] == "earnings_history"
    assert data["earnings_history"] is not None
    assert len(data["earnings_history"]) == 2
    assert data["earnings_history"][0]["eps_actual"] == 2.18
    assert data["earnings_history"][0]["surprise_percent"] == 3.8


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_revenue_estimate(mock_ticker, client):
    """Test analysis endpoint with revenue estimate type"""
    import pandas as pd

    # Mock the yfinance Ticker object and its methods
    mock_ticker_instance = MagicMock()
    mock_df = pd.DataFrame(
        {
            "2024-12-31": {"avg": 400000000000, "low": 380000000000, "high": 420000000000},
            "2025-12-31": {"avg": 420000000000, "low": 400000000000, "high": 440000000000},
        }
    )
    mock_ticker_instance.revenue_estimate = mock_df
    mock_ticker.return_value = mock_ticker_instance

    response = client.get("/v1/analysis/AAPL?analysis_type=revenue_estimate")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["analysis_type"] == "revenue_estimate"
    assert data["revenue_estimate"] is not None
    assert "estimates" in data["revenue_estimate"]


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_empty_data(mock_ticker, client):
    """Test analysis endpoint with empty data"""
    # Mock the yfinance Ticker object with empty data
    mock_ticker_instance = MagicMock()
    mock_ticker_instance.recommendations = MagicMock()
    mock_ticker_instance.recommendations.empty = True
    mock_ticker.return_value = mock_ticker_instance

    response = client.get("/v1/analysis/AAPL?analysis_type=recommendations")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["analysis_type"] == "recommendations"
    assert data["recommendations"] == []


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_yfinance_error(mock_ticker, client):
    """Test analysis endpoint with yfinance error"""
    # Mock the yfinance Ticker object to raise an exception
    mock_ticker_instance = MagicMock()
    mock_ticker_instance.recommendations = MagicMock()
    mock_ticker_instance.recommendations.empty = False
    mock_ticker_instance.recommendations.iterrows.side_effect = Exception("Yahoo Finance API error")
    mock_ticker.return_value = mock_ticker_instance

    response = client.get("/v1/analysis/AAPL?analysis_type=recommendations")

    assert response.status_code == 500
    data = response.json()
    assert "detail" in data


def test_get_analysis_invalid_analysis_type(client):
    """Test analysis endpoint with invalid analysis type"""
    response = client.get("/v1/analysis/AAPL?analysis_type=invalid_type")

    assert response.status_code == 422  # Validation error


def test_get_analysis_missing_analysis_type(client):
    """Test analysis endpoint with missing analysis type parameter"""
    response = client.get("/v1/analysis/AAPL")

    assert response.status_code == 422  # Validation error


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_price_targets_series_format(mock_ticker, client):
    """Test analysis endpoint with price targets in Series format"""
    import pandas as pd

    # Mock the yfinance Ticker object with Series format
    mock_ticker_instance = MagicMock()
    mock_series = pd.Series({"current": 150.0, "mean": 160.0, "median": 155.0, "low": 140.0, "high": 180.0})
    mock_ticker_instance.analyst_price_targets = mock_series
    mock_ticker.return_value = mock_ticker_instance

    response = client.get("/v1/analysis/AAPL?analysis_type=price_targets")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["analysis_type"] == "price_targets"
    assert data["price_targets"] is not None
    assert data["price_targets"]["current"] == 150.0


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_price_targets_none_data(mock_ticker, client):
    """Test analysis endpoint with None price targets data"""
    # Mock the yfinance Ticker object with None data
    mock_ticker_instance = MagicMock()
    mock_ticker_instance.analyst_price_targets = None
    mock_ticker.return_value = mock_ticker_instance

    response = client.get("/v1/analysis/AAPL?analysis_type=price_targets")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["analysis_type"] == "price_targets"
    assert data["price_targets"] is not None
    # All fields should be None
    assert data["price_targets"]["current"] is None
    assert data["price_targets"]["mean"] is None


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_with_nan_values(mock_ticker, client):
    """Test analysis endpoint handling NaN values"""
    import pandas as pd

    # Mock the yfinance Ticker object with NaN values
    mock_ticker_instance = MagicMock()
    mock_ticker_instance.recommendations = MagicMock()
    mock_ticker_instance.recommendations.empty = False
    mock_ticker_instance.recommendations.iterrows.return_value = [(0, {"period": "3m", "strongBuy": 5, "buy": pd.NA, "hold": 3, "sell": 1, "strongSell": 0})]
    mock_ticker.return_value = mock_ticker_instance

    response = client.get("/v1/analysis/AAPL?analysis_type=recommendations")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["analysis_type"] == "recommendations"
    assert data["recommendations"] is not None
    assert len(data["recommendations"]) == 1
    assert data["recommendations"][0]["buy"] is None  # NaN should be converted to None


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_multiple_symbols(mock_ticker, client):
    """Test analysis endpoint with different symbols"""
    # Mock the yfinance Ticker object
    mock_ticker_instance = MagicMock()
    mock_ticker_instance.recommendations = MagicMock()
    mock_ticker_instance.recommendations.empty = False
    mock_ticker_instance.recommendations.iterrows.return_value = [(0, {"period": "3m", "strongBuy": 5, "buy": 10, "hold": 3, "sell": 1, "strongSell": 0})]
    mock_ticker.return_value = mock_ticker_instance

    # Test with different symbols
    symbols = ["AAPL", "MSFT", "GOOGL", "TSLA"]
    for symbol in symbols:
        response = client.get(f"/v1/analysis/{symbol}?analysis_type=recommendations")
        assert response.status_code == 200
        data = response.json()
        assert data["symbol"] == symbol
        assert data["analysis_type"] == "recommendations"


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_concurrent_requests(mock_ticker, client):
    """Test analysis endpoint with concurrent requests"""
    # Mock the yfinance Ticker object
    mock_ticker_instance = MagicMock()
    mock_ticker_instance.recommendations = MagicMock()
    mock_ticker_instance.recommendations.empty = False
    mock_ticker_instance.recommendations.iterrows.return_value = [(0, {"period": "3m", "strongBuy": 5, "buy": 10, "hold": 3, "sell": 1, "strongSell": 0})]
    mock_ticker.return_value = mock_ticker_instance

    # Test concurrent requests
    import asyncio

    async def make_request(symbol):
        response = client.get(f"/v1/analysis/{symbol}?analysis_type=recommendations")
        return response.status_code == 200

    symbols = ["AAPL", "MSFT", "GOOGL", "TSLA"]
    tasks = [make_request(symbol) for symbol in symbols]
    results = await asyncio.gather(*tasks)

    assert all(results)


def test_analysis_endpoint_documentation(client):
    """Test that the analysis endpoint has proper documentation"""
    response = client.get("/docs")
    assert response.status_code == 200

    # Check that the analysis endpoint appears in the OpenAPI spec
    openapi_spec = client.get("/openapi.json").json()
    assert "/v1/analysis/{symbol}" in openapi_spec["paths"]

    # Check that the endpoint has proper tags
    analysis_path = openapi_spec["paths"]["/v1/analysis/{symbol}"]
    assert "get" in analysis_path
    assert "tags" in analysis_path["get"]
    assert "Analysis" in analysis_path["get"]["tags"]


@pytest.mark.asyncio
@patch("src.services.analysis.get_analysis.yf.Ticker")
async def test_get_analysis_response_structure(mock_ticker, client):
    """Test that the analysis response has the correct structure"""
    # Mock the yfinance Ticker object
    mock_ticker_instance = MagicMock()
    mock_ticker_instance.recommendations = MagicMock()
    mock_ticker_instance.recommendations.empty = False
    mock_ticker_instance.recommendations.iterrows.return_value = [(0, {"period": "3m", "strongBuy": 5, "buy": 10, "hold": 3, "sell": 1, "strongSell": 0})]
    mock_ticker.return_value = mock_ticker_instance

    response = client.get("/v1/analysis/AAPL?analysis_type=recommendations")

    assert response.status_code == 200
    data = response.json()

    # Check required fields
    assert "symbol" in data
    assert "analysis_type" in data
    assert data["symbol"] == "AAPL"
    assert data["analysis_type"] == "recommendations"

    # Check that only the relevant field is populated
    assert data["recommendations"] is not None
    assert data["price_targets"] is None
    assert data["earnings_estimate"] is None
    assert data["revenue_estimate"] is None
    assert data["earnings_history"] is None
    assert data["sustainability"] is None
    assert data["upgrades_downgrades"] is None
