import pytest
from unittest.mock import AsyncMock, patch
from src.services.financials.get_financials import get_financial_statement
from src.models.financials import StatementType, Frequency

@pytest.mark.asyncio
@patch("src.services.financials.get_financials.DefeatBetaClient")
async def test_get_financial_statement(mock_client_class):
    """
    Tests the get_financial_statement service to ensure it retrieves financial data correctly.
    This test mocks the DefeatBetaClient to avoid making real API calls.
    """
    # Mock the DefeatBetaClient instance and its method
    mock_client = AsyncMock()
    mock_client_class.return_value = mock_client
    
    # Mock response data from DefeatBetaClient
    mock_response = {
        "symbol": "AAPL",
        "statement_type": "income_statement", 
        "frequency": "annual",
        "statement": {
            "Total Revenue": {"2023-12-31": 383285000000, "2022-12-31": 394328000000},
            "Net Income": {"2023-12-31": 96995000000, "2022-12-31": 99803000000}
        },
        "metadata": {
            "source": "defeatbeta-api",
            "retrieved_at": "2024-09-14T11:53:14",
            "rows_count": 2,
            "columns_count": 2
        }
    }
    
    mock_client.get_financial_statement.return_value = mock_response
    
    ticker = "AAPL"
    statement_type = StatementType.INCOME_STATEMENT
    freq = Frequency.ANNUAL
    
    financials = await get_financial_statement(ticker, statement_type, freq)
    
    # Verify the DefeatBetaClient was called with correct parameters
    mock_client.get_financial_statement.assert_called_once_with("AAPL", "income_statement", "annual")
    
    # Verify the returned FinancialStatement object
    assert financials is not None
    assert financials.symbol == ticker
    assert financials.statement_type == statement_type
    assert financials.frequency == freq
    assert financials.statement is not None
    assert len(financials.statement) > 0
    assert "Total Revenue" in financials.statement
    assert "Net Income" in financials.statement

@pytest.mark.asyncio
@patch("src.services.financials.get_financials.DefeatBetaClient")
async def test_get_financial_statement_balance_sheet(mock_client_class):
    """Test getting balance sheet data"""
    mock_client = AsyncMock()
    mock_client_class.return_value = mock_client
    
    mock_response = {
        "symbol": "TSLA",
        "statement_type": "balance_sheet",
        "frequency": "quarterly", 
        "statement": {
            "Total Assets": {"2024-03-31": 106618000000, "2023-12-31": 106618000000},
            "Total Liabilities": {"2024-03-31": 43009000000, "2023-12-31": 43009000000}
        },
        "metadata": {
            "source": "defeatbeta-api",
            "retrieved_at": "2024-09-14T11:53:14",
            "rows_count": 2,
            "columns_count": 2
        }
    }
    
    mock_client.get_financial_statement.return_value = mock_response
    
    financials = await get_financial_statement("TSLA", StatementType.BALANCE_SHEET, Frequency.QUARTERLY)
    
    mock_client.get_financial_statement.assert_called_once_with("TSLA", "balance_sheet", "quarterly")
    assert financials.symbol == "TSLA"
    assert financials.statement_type == StatementType.BALANCE_SHEET
    assert financials.frequency == Frequency.QUARTERLY

@pytest.mark.asyncio
@patch("src.services.financials.get_financials.DefeatBetaClient")
async def test_get_financial_statement_cash_flow(mock_client_class):
    """Test getting cash flow data"""
    mock_client = AsyncMock()
    mock_client_class.return_value = mock_client
    
    mock_response = {
        "symbol": "GOOG",
        "statement_type": "cash_flow",
        "frequency": "annual",
        "statement": {
            "Operating Cash Flow": {"2023-12-31": 77434000000, "2022-12-31": 91495000000},
            "Free Cash Flow": {"2023-12-31": 45688000000, "2022-12-31": 60010000000}
        },
        "metadata": {
            "source": "defeatbeta-api",
            "retrieved_at": "2024-09-14T11:53:14",
            "rows_count": 2,
            "columns_count": 2
        }
    }
    
    mock_client.get_financial_statement.return_value = mock_response
    
    financials = await get_financial_statement("GOOG", StatementType.CASH_FLOW, Frequency.ANNUAL)
    
    mock_client.get_financial_statement.assert_called_once_with("GOOG", "cash_flow", "annual")
    assert financials.symbol == "GOOG"
    assert financials.statement_type == StatementType.CASH_FLOW
    assert financials.frequency == Frequency.ANNUAL


