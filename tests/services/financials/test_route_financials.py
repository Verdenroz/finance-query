from unittest.mock import patch

import pytest
from fastapi.testclient import TestClient

from src.main import app
from src.models.financials import FinancialStatement, Frequency, StatementType


@pytest.fixture
def client():
    return TestClient(app)


@patch("src.routes.financials.get_financial_statement")
async def test_get_financials_income_annual(mock_get_statement, client):
    """Test getting annual income statement"""
    # Mock the service layer response
    mock_statement = FinancialStatement(
        symbol="AAPL",
        statement_type=StatementType.INCOME_STATEMENT,
        frequency=Frequency.ANNUAL,
        statement={
            "TotalRevenue": {"2023-09-30": 383285000000, "2024-09-30": 391035000000},
            "NetIncome": {"2023-09-30": 96995000000, "2024-09-30": 93736000000},
        },
    )
    mock_get_statement.return_value = mock_statement

    # Make the request
    response = client.get("/v1/financials/AAPL?statement=income&frequency=annual")

    # Assertions
    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["statement_type"] == "income"
    assert data["frequency"] == "annual"
    assert "TotalRevenue" in data["statement"]
    assert data["statement"]["TotalRevenue"]["2024-09-30"] == 391035000000


@patch("src.routes.financials.get_financial_statement")
async def test_get_financials_balance_quarterly(mock_get_statement, client):
    """Test getting quarterly balance sheet"""
    mock_statement = FinancialStatement(
        symbol="MSFT",
        statement_type=StatementType.BALANCE_SHEET,
        frequency=Frequency.QUARTERLY,
        statement={
            "TotalAssets": {"2024-Q2": 350000000000, "2024-Q3": 360000000000},
            "TotalLiabilities": {"2024-Q2": 200000000000, "2024-Q3": 205000000000},
        },
    )
    mock_get_statement.return_value = mock_statement

    response = client.get("/v1/financials/MSFT?statement=balance&frequency=quarterly")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "MSFT"
    assert data["statement_type"] == "balance"
    assert data["frequency"] == "quarterly"
    assert data["statement"]["TotalAssets"]["2024-Q3"] == 360000000000


@patch("src.routes.financials.get_financial_statement")
async def test_get_financials_cashflow_annual(mock_get_statement, client):
    """Test getting annual cash flow statement"""
    mock_statement = FinancialStatement(
        symbol="GOOG",
        statement_type=StatementType.CASH_FLOW,
        frequency=Frequency.ANNUAL,
        statement={
            "OperatingCashFlow": {"2023-12-31": 77434000000, "2024-12-31": 80000000000},
            "CapitalExpenditure": {"2023-12-31": -31688000000, "2024-12-31": -35000000000},
        },
    )
    mock_get_statement.return_value = mock_statement

    response = client.get("/v1/financials/GOOG?statement=cashflow&frequency=annual")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "GOOG"
    assert data["statement_type"] == "cashflow"
    assert data["statement"]["OperatingCashFlow"]["2024-12-31"] == 80000000000


@patch("src.routes.financials.get_financial_statement")
async def test_get_financials_default_frequency(mock_get_statement, client):
    """Test that frequency defaults to annual if not specified"""
    mock_statement = FinancialStatement(
        symbol="NVDA",
        statement_type=StatementType.INCOME_STATEMENT,
        frequency=Frequency.ANNUAL,
        statement={"TotalRevenue": {"2024-01-31": 60922000000}},
    )
    mock_get_statement.return_value = mock_statement

    response = client.get("/v1/financials/NVDA?statement=income")

    assert response.status_code == 200
    data = response.json()
    assert data["frequency"] == "annual"  # Default


async def test_get_financials_invalid_symbol_pattern(client):
    """Test validation error for invalid symbol pattern"""
    response = client.get("/v1/financials/invalid-symbol?statement=income&frequency=annual")

    assert response.status_code == 422
    assert "detail" in response.json()


async def test_get_financials_invalid_statement_type(client):
    """Test validation error for invalid statement type"""
    response = client.get("/v1/financials/AAPL?statement=invalid&frequency=annual")

    assert response.status_code == 422
    assert "detail" in response.json()


async def test_get_financials_invalid_frequency(client):
    """Test validation error for invalid frequency"""
    response = client.get("/v1/financials/AAPL?statement=income&frequency=invalid")

    assert response.status_code == 422
    assert "detail" in response.json()


async def test_get_financials_missing_statement_param(client):
    """Test validation error when statement parameter is missing"""
    response = client.get("/v1/financials/AAPL?frequency=annual")

    assert response.status_code == 422
    assert "detail" in response.json()


@patch("src.routes.financials.get_financial_statement")
async def test_get_financials_all_statement_types(mock_get_statement, client):
    """Test all valid statement type values"""
    statement_types = ["income", "balance", "cashflow"]

    for stmt_type in statement_types:
        mock_statement = FinancialStatement(
            symbol="TEST",
            statement_type=StatementType(stmt_type),
            frequency=Frequency.ANNUAL,
            statement={"TestMetric": {"2024": 100}},
        )
        mock_get_statement.return_value = mock_statement

        response = client.get(f"/v1/financials/TEST?statement={stmt_type}&frequency=annual")
        assert response.status_code == 200
        assert response.json()["statement_type"] == stmt_type


@patch("src.routes.financials.get_financial_statement")
async def test_get_financials_all_frequencies(mock_get_statement, client):
    """Test all valid frequency values"""
    frequencies = ["annual", "quarterly"]

    for freq in frequencies:
        mock_statement = FinancialStatement(
            symbol="TEST",
            statement_type=StatementType.INCOME_STATEMENT,
            frequency=Frequency(freq),
            statement={"TestMetric": {"2024": 100}},
        )
        mock_get_statement.return_value = mock_statement

        response = client.get(f"/v1/financials/TEST?statement=income&frequency={freq}")
        assert response.status_code == 200
        assert response.json()["frequency"] == freq


@patch("src.routes.financials.get_financial_statement")
async def test_get_financials_symbol_case_insensitivity(mock_get_statement, client):
    """Test that symbol is converted to uppercase"""
    mock_statement = FinancialStatement(
        symbol="AAPL",
        statement_type=StatementType.INCOME_STATEMENT,
        frequency=Frequency.ANNUAL,
        statement={"TotalRevenue": {"2024": 100}},
    )
    mock_get_statement.return_value = mock_statement

    # Test with lowercase
    response = client.get("/v1/financials/AAPL?statement=income&frequency=annual")
    assert response.status_code == 200
    assert response.json()["symbol"] == "AAPL"
