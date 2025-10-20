from fastapi import HTTPException

from src.models.holders import HolderType
from src.services.holders.get_holders import get_holders_data


class TestGetHolders:
    """Test suite for the get_holders_data service."""

    async def test_get_holders_data_institutional(self):
        """
        Tests the get_holders_data service to ensure it retrieves institutional holders data correctly.
        This test makes a real API call to the holders endpoint.
        """
        ticker = "AAPL"
        holder_type = HolderType.INSTITUTIONAL

        holders_data = await get_holders_data(ticker, holder_type)

        assert holders_data is not None
        assert holders_data.symbol == ticker
        assert holders_data.holder_type == holder_type
        assert holders_data.institutional_holders is not None

    async def test_get_holders_data_insider_transactions(self):
        """
        Tests the get_holders_data service for insider transactions data.
        """
        ticker = "AAPL"
        holder_type = HolderType.INSIDER_TRANSACTIONS

        holders_data = await get_holders_data(ticker, holder_type)

        assert holders_data is not None
        assert holders_data.symbol == ticker
        assert holders_data.holder_type == holder_type
        assert holders_data.insider_transactions is not None
        # Insider transactions can be empty list if no data available
        assert isinstance(holders_data.insider_transactions, list)

    async def test_get_holders_data_insider_purchases(self):
        """
        Tests the get_holders_data service for insider purchases data.
        """
        ticker = "AAPL"
        holder_type = HolderType.INSIDER_PURCHASES

        holders_data = await get_holders_data(ticker, holder_type)

        assert holders_data is not None
        assert holders_data.symbol == ticker
        assert holders_data.holder_type == holder_type
        assert holders_data.insider_purchases is not None
        # Check that the insider_purchases object has the expected period field
        assert hasattr(holders_data.insider_purchases, "period")

    async def test_get_holders_data_insider_roster(self):
        """
        Tests the get_holders_data service for insider roster data.
        """
        ticker = "AAPL"
        holder_type = HolderType.INSIDER_ROSTER

        holders_data = await get_holders_data(ticker, holder_type)

        assert holders_data is not None
        assert holders_data.symbol == ticker
        assert holders_data.holder_type == holder_type
        assert holders_data.insider_roster is not None
        # Insider roster can be empty list if no data available
        assert isinstance(holders_data.insider_roster, list)

    async def test_get_holders_data_major(self):
        """
        Tests the get_holders_data service for major holders breakdown data.
        """
        ticker = "AAPL"
        holder_type = HolderType.MAJOR

        holders_data = await get_holders_data(ticker, holder_type)

        assert holders_data is not None
        assert holders_data.symbol == ticker
        assert holders_data.holder_type == holder_type
        assert holders_data.major_breakdown is not None

    async def test_get_holders_data_mutualfund(self):
        """
        Tests the get_holders_data service for mutual fund holders data.
        """
        ticker = "AAPL"
        holder_type = HolderType.MUTUALFUND

        holders_data = await get_holders_data(ticker, holder_type)

        assert holders_data is not None
        assert holders_data.symbol == ticker
        assert holders_data.holder_type == holder_type
        assert holders_data.mutualfund_holders is not None
        assert isinstance(holders_data.mutualfund_holders, list)

    async def test_get_holders_data_invalid_symbol(self):
        """
        Tests the get_holders_data service with an invalid symbol.
        """
        ticker = "INVALID_SYMBOL_THAT_DOES_NOT_EXIST"
        holder_type = HolderType.INSTITUTIONAL

        # This should either succeed with empty data or raise an HTTPException
        # depending on how yfinance handles invalid symbols
        try:
            holders_data = await get_holders_data(ticker, holder_type)
            # If it succeeds, verify the structure is still correct
            assert holders_data.symbol == ticker
            assert holders_data.holder_type == holder_type
        except HTTPException as e:
            # If it raises an exception, it should be 404 or 500
            assert e.status_code in [404, 500]
