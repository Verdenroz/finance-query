import pytest
from unittest.mock import AsyncMock, patch

from fastapi.testclient import TestClient

from src.main import app
from src.models.financials import Frequency, StatementType


@pytest.fixture
def client():
    return TestClient(app)


@pytest.mark.asyncio
@patch("src.services.financials.get_financials.DefeatBetaClient")
async def test_get_financials_income_annual(mock_client_class, client):
    # Mock the DefeatBetaClient instance and its methods
    mock_client = AsyncMock()
    mock_client_class.return_value = mock_client
    
    mock_response = {
        "symbol": "AAPL",
        "statement_type": "income_statement",
        "frequency": "annual",
        "statement": {
            "Total Revenue": {"2023-12-31": 1000, "2022-12-31": 900},
            "Net Income": {"2023-12-31": 100, "2022-12-31": 90}
        },
        "metadata": {
            "source": "defeatbeta-api",
            "retrieved_at": "2024-09-14T11:53:14",
            "rows_count": 2,
            "columns_count": 2
        }
    }
    
    mock_client.get_financial_statement.return_value = mock_response

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
@patch("src.services.financials.get_financials.DefeatBetaClient")
async def test_get_financials_balance_sheet_quarterly(mock_client_class, client):
    mock_client = AsyncMock()
    mock_client_class.return_value = mock_client
    
    mock_response = {
        "symbol": "MSFT",
        "statement_type": "balance_sheet",
        "frequency": "quarterly",
        "statement": {
            "Total Assets": {"2024-03-31": 2000, "2023-12-31": 1900},
            "Total Liabilities": {"2024-03-31": 1000, "2023-12-31": 950}
        },
        "metadata": {
            "source": "defeatbeta-api",
            "retrieved_at": "2024-09-14T11:53:14",
            "rows_count": 2,
            "columns_count": 2
        }
    }
    
    mock_client.get_financial_statement.return_value = mock_response

    response = client.get("/v1/financials/MSFT?statement=balance&frequency=quarterly")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "MSFT"
    assert data["statement_type"] == "balance"
    assert data["frequency"] == "quarterly"
    assert data["statement"]["Total Assets"]["2024-03-31"] == 2000


@pytest.mark.asyncio
@patch("src.services.financials.get_financials.DefeatBetaClient")
async def test_get_financials_cash_flow(mock_client_class, client):
    mock_client = AsyncMock()
    mock_client_class.return_value = mock_client
    
    mock_response = {
        "symbol": "GOOG",
        "statement_type": "cash_flow",
        "frequency": "annual",
        "statement": {
            "Operating Cash Flow": {"2023-12-31": 500},
            "Capital Expenditure": {"2023-12-31": -50}
        },
        "metadata": {
            "source": "defeatbeta-api",
            "retrieved_at": "2024-09-14T11:53:14",
            "rows_count": 2,
            "columns_count": 1
        }
    }
    
    mock_client.get_financial_statement.return_value = mock_response

    response = client.get("/v1/financials/GOOG?statement=cashflow&frequency=annual")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "GOOG"
    assert data["statement_type"] == "cashflow"
    assert data["statement"]["Operating Cash Flow"]["2023-12-31"] == 500


@pytest.mark.asyncio
@patch("src.services.financials.get_financials.DefeatBetaClient")
async def test_get_financials_empty_data(mock_client_class, client):
    mock_client = AsyncMock()
    mock_client_class.return_value = mock_client
    
    # Mock response with empty statement data
    mock_response = {
        "symbol": "EMPTY",
        "statement_type": "income_statement",
        "frequency": "annual",
        "statement": {},  # Empty statement
        "metadata": {
            "source": "defeatbeta-api",
            "retrieved_at": "2024-09-14T11:53:14",
            "rows_count": 0,
            "columns_count": 0
        }
    }
    
    mock_client.get_financial_statement.return_value = mock_response

    response = client.get("/v1/financials/EMPTY?statement=income&frequency=annual")

    assert response.status_code == 404
    assert response.json()["detail"] == "No data found for EMPTY"


@pytest.mark.asyncio
@patch("src.services.financials.get_financials.DefeatBetaClient")
async def test_get_financials_defeatbeta_error(mock_client_class, client):
    mock_client = AsyncMock()
    mock_client_class.return_value = mock_client
    
    # Mock the client to raise an exception
    mock_client.get_financial_statement.side_effect = Exception("DefeatBeta API error")

    response = client.get("/v1/financials/ERROR?statement=income&frequency=annual")

    assert response.status_code == 500
    assert "DefeatBeta API error" in response.json()["detail"]
