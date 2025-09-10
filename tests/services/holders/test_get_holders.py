import pytest
from src.services.holders.get_holders import get_holders_data
from src.models.holders import HolderType

@pytest.mark.asyncio
async def test_get_holders_data():
    """
    Tests the get_holders_data service to ensure it retrieves holders data correctly.
    This test makes a real API call to the holders endpoint.
    """
    ticker = "AAPL"
    holder_type = HolderType.INSTITUTIONAL
    
    holders_data = await get_holders_data(ticker, holder_type)
    
    assert holders_data is not None
    assert holders_data.symbol == ticker
    assert holders_data.holder_type == holder_type
    assert holders_data.institutional_holders is not None
