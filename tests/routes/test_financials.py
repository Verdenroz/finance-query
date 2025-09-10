import pandas as pd
import pytest
from unittest.mock import AsyncMock, patch

from fastapi.testclient import TestClient

from src.main import app
from src.models.financials import Frequency, StatementType


@pytest.fixture
def client():
    return TestClient(app)


@pytest.mark.asyncio
@patch("src.services.financials.get_financials.Ticker")
async def test_get_financials_income_annual(mock_ticker, client):
    # Mock the yfinance Ticker object and its methods
    mock_df = pd.DataFrame({
        '2023-12-31': {'Total Revenue': 1000, 'Net Income': 100},
        '2022-12-31': {'Total Revenue': 900, 'Net Income': 90}
    })
    mock_instance = mock_ticker.return_value
    mock_instance.get_income_stmt.return_value = mock_df

    # Make the request
    response = client.get("/v1/financials/AAPL?statement=income&frequency=annual")

    # Assertions
    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["statement_type"] == "income"
    assert data["frequency"] == "annual"
    assert "2023-12-31" in data["statement"]["Total Revenue"]
    assert data["statement"]["Total Revenue"]["2023-12-31"] == 1000


@pytest.mark.asyncio
@patch("src.services.financials.get_financials.Ticker")
async def test_get_financials_balance_sheet_quarterly(mock_ticker, client):
    mock_df = pd.DataFrame({
        '2024-03-31': {'Total Assets': 2000, 'Total Liabilities': 1000},
        '2023-12-31': {'Total Assets': 1900, 'Total Liabilities': 950}
    })
    mock_instance = mock_ticker.return_value
    mock_instance.get_balance_sheet.return_value = mock_df

    response = client.get("/v1/financials/MSFT?statement=balance&frequency=quarterly")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "MSFT"
    assert data["statement_type"] == "balance"
    assert data["frequency"] == "quarterly"
    assert data["statement"]["Total Assets"]["2024-03-31"] == 2000


@pytest.mark.asyncio
@patch("src.services.financials.get_financials.Ticker")
async def test_get_financials_cash_flow(mock_ticker, client):
    mock_df = pd.DataFrame({
        '2023-12-31': {'Operating Cash Flow': 500, 'Capital Expenditure': -50}
    })
    mock_instance = mock_ticker.return_value
    mock_instance.get_cash_flow.return_value = mock_df

    response = client.get("/v1/financials/GOOG?statement=cashflow&frequency=annual")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "GOOG"
    assert data["statement_type"] == "cashflow"
    assert data["statement"]["Operating Cash Flow"]["2023-12-31"] == 500


@pytest.mark.asyncio
@patch("src.services.financials.get_financials.Ticker")
async def test_get_financials_empty_data(mock_ticker, client):
    mock_instance = mock_ticker.return_value
    mock_instance.get_income_stmt.return_value = pd.DataFrame()  # Empty DataFrame

    response = client.get("/v1/financials/EMPTY?statement=income&frequency=annual")

    assert response.status_code == 404
    assert response.json()["detail"] == "No data found for EMPTY"


@pytest.mark.asyncio
@patch("src.services.financials.get_financials.Ticker")
async def test_get_financials_yfinance_error(mock_ticker, client):
    mock_instance = mock_ticker.return_value
    mock_instance.get_income_stmt.side_effect = Exception("Yahoo Finance error")

    response = client.get("/v1/financials/ERROR?statement=income&frequency=annual")

    assert response.status_code == 500
    assert "Yahoo Finance error" in response.json()["detail"]
