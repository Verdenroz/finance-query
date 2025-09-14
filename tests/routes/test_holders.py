import pandas as pd
import pytest
from unittest.mock import AsyncMock, patch, MagicMock, PropertyMock

from fastapi.testclient import TestClient

from src.main import app
from src.models.holders import HolderType


@pytest.fixture
def client():
    return TestClient(app)


@pytest.mark.asyncio
@patch("src.services.holders.get_holders.yf.Ticker")
async def test_get_holders_institutional(mock_ticker, client):
    # Mock the yfinance Ticker object and its methods
    mock_df = pd.DataFrame({
        'Holder': ['Vanguard Group Inc', 'BlackRock Inc'],
        'Shares': [1000000, 800000],
        'Date Reported': [pd.Timestamp('2024-01-01'), pd.Timestamp('2024-01-01')],
        'Value': [150000000, 120000000]
    })
    
    mock_instance = mock_ticker.return_value
    mock_instance.institutional_holders = mock_df

    # Make the request
    response = client.get("/v1/holders/AAPL?holder_type=institutional")

    # Assertions
    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["holder_type"] == "institutional"
    assert data["institutional_holders"] is not None
    assert len(data["institutional_holders"]) == 2
    assert data["institutional_holders"][0]["holder"] == "Vanguard Group Inc"


@pytest.mark.asyncio
@patch("src.services.holders.get_holders.yf.Ticker")
async def test_get_holders_major(mock_ticker, client):
    mock_df = pd.DataFrame({
        'Value': [0.85, 0.10, 0.05]
    }, index=['institutionsPercentHeld', 'insidersPercentHeld', 'floatHeld'])
    
    mock_instance = mock_ticker.return_value
    mock_instance.major_holders = mock_df

    response = client.get("/v1/holders/MSFT?holder_type=major")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "MSFT"
    assert data["holder_type"] == "major"
    assert data["major_breakdown"] is not None
    assert data["major_breakdown"]["breakdown_data"]["institutionsPercentHeld"] == 0.85


@pytest.mark.asyncio
@patch("src.services.holders.get_holders.yf.Ticker")
async def test_get_holders_mutualfund(mock_ticker, client):
    mock_df = pd.DataFrame({
        'Holder': ['Vanguard 500 Index Fund', 'SPDR S&P 500 ETF'],
        'Shares': [500000, 300000],
        'Date Reported': [pd.Timestamp('2024-01-01'), pd.Timestamp('2024-01-01')],
        'Value': [75000000, 45000000]
    })
    
    mock_instance = mock_ticker.return_value
    mock_instance.mutualfund_holders = mock_df

    response = client.get("/v1/holders/GOOG?holder_type=mutualfund")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "GOOG"
    assert data["holder_type"] == "mutualfund"
    assert data["mutualfund_holders"] is not None
    assert len(data["mutualfund_holders"]) == 2


@pytest.mark.asyncio
@patch("src.services.holders.get_holders.yf.Ticker")
async def test_get_holders_empty_data(mock_ticker, client):
    mock_instance = mock_ticker.return_value
    mock_instance.institutional_holders = pd.DataFrame()  # Empty DataFrame

    response = client.get("/v1/holders/EMPTY?holder_type=institutional")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "EMPTY"
    assert data["holder_type"] == "institutional"
    assert data["institutional_holders"] == []


@pytest.mark.asyncio
@patch("src.services.holders.get_holders.yf.Ticker")
async def test_get_holders_yfinance_error(mock_ticker, client):
    mock_instance = mock_ticker.return_value
    # Mock the institutional_holders property to raise an exception when accessed
    type(mock_instance).institutional_holders = PropertyMock(side_effect=Exception("Yahoo Finance error"))

    response = client.get("/v1/holders/ERROR?holder_type=institutional")

    assert response.status_code == 500
    assert "Yahoo Finance error" in response.json()["detail"]
