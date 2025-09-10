import pytest
from src.services.financials.get_financials import get_financial_statement
from src.models.financials import StatementType, Frequency

@pytest.mark.asyncio
async def test_get_financial_statement():
    """
    Tests the get_financial_statement service to ensure it retrieves financial data correctly.
    This test makes a real API call to the financials endpoint.
    """
    ticker = "AAPL"
    statement_type = StatementType.INCOME_STATEMENT
    freq = Frequency.ANNUAL
    
    financials = await get_financial_statement(ticker, statement_type, freq)
    
    assert financials is not None
    assert financials.symbol == ticker
    assert financials.statement_type == statement_type
    assert financials.frequency == freq
    assert financials.statement is not None
    assert len(financials.statement) > 0


