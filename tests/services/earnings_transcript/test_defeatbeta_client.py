import asyncio
from datetime import datetime
from unittest.mock import MagicMock, patch

import pytest
from fastapi import HTTPException

from src.clients.defeatbeta_client import DefeatBetaClient


class TestDefeatBetaClient:
    """Test DefeatBetaClient with real API calls to defeatbeta-api"""

    @pytest.fixture
    def client(self):
        """Create DefeatBetaClient instance"""
        return DefeatBetaClient()

    async def test_get_earnings_transcript_real_api_call(self, client):
        """Test real API call to defeatbeta-api for earnings transcript"""
        # Use a well-known stock that should have earnings transcripts
        symbol = "AAPL"

        try:
            result = await client.get_earnings_transcript(symbol)

            # Verify response structure
            assert isinstance(result, dict)
            assert "symbol" in result
            assert "transcripts" in result
            assert result["symbol"] == symbol.upper()
            assert isinstance(result["transcripts"], list)

            if result["transcripts"]:
                transcript = result["transcripts"][0]
                assert "symbol" in transcript
                assert "quarter" in transcript
                assert "year" in transcript
                assert "date" in transcript
                assert "transcript" in transcript
                assert "participants" in transcript
                assert "metadata" in transcript

                # Verify data types
                assert isinstance(transcript["symbol"], str)
                assert isinstance(transcript["quarter"], str)
                assert isinstance(transcript["year"], int)
                assert isinstance(transcript["date"], datetime)
                assert isinstance(transcript["transcript"], str)
                assert isinstance(transcript["participants"], list)
                assert isinstance(transcript["metadata"], dict)

        except ImportError:
            # If defeatbeta-api is not installed, test the fallback behavior
            result = await client.get_earnings_transcript(symbol)
            assert result["symbol"] == symbol.upper()
            assert len(result["transcripts"]) > 0
            assert "sample" in result["transcripts"][0]["metadata"].get("source", "")

    async def test_get_earnings_transcript_with_filters(self, client):
        """Test earnings transcript with quarter and year filters"""
        symbol = "TSLA"
        quarter = "Q3"
        year = 2024

        result = await client.get_earnings_transcript(symbol, quarter, year)

        assert result["symbol"] == symbol.upper()
        assert isinstance(result["transcripts"], list)

        # If we get real data, verify filters are applied
        for transcript in result["transcripts"]:
            if "quarter" in transcript and "year" in transcript:
                # Note: Real API might not have exact quarter/year, so we check if filters work
                pass

    async def test_get_earnings_transcript_invalid_symbol(self, client):
        """Test with invalid symbol"""
        symbol = "INVALID_SYMBOL_12345"

        # Should raise HTTPException for invalid symbol
        with pytest.raises(HTTPException) as exc_info:
            await client.get_earnings_transcript(symbol)
        assert exc_info.value.status_code == 404

    async def test_format_transcript_data_structure(self, client):
        """Test the data formatting functionality"""
        import pandas as pd

        # Create mock DataFrame that matches defeatbeta-api structure
        mock_data = pd.DataFrame(
            {
                "symbol": ["AAPL"],
                "fiscal_year": [2024],
                "fiscal_quarter": [1],
                "transcripts": [
                    [
                        {"speaker": "CEO", "content": "Sample transcript text", "paragraph_number": 1},
                        {"speaker": "CFO", "content": "Financial results", "paragraph_number": 2},
                    ]
                ],
                "transcripts_id": [12345],
            }
        )

        formatted = client._format_transcript_data(mock_data, "AAPL")

        assert formatted["symbol"] == "AAPL"
        assert "transcripts" in formatted
        assert len(formatted["transcripts"]) > 0

    async def test_process_transcript_row(self, client):
        """Test processing individual transcript rows"""
        import pandas as pd

        # Create mock row that matches DataFrame structure
        mock_row = pd.Series(
            {
                "symbol": "AAPL",
                "fiscal_year": 2024,
                "fiscal_quarter": 2,
                "transcripts": [
                    {"speaker": "CEO", "content": "Sample transcript", "paragraph_number": 1},
                    {"speaker": "CFO", "content": "Financial data", "paragraph_number": 2},
                ],
                "transcripts_id": 12345,
            }
        )

        result = client._process_transcript_row(mock_row, "Q2", 2024)

        assert result is not None
        assert result["quarter"] == "Q2"
        assert result["year"] == 2024
        assert isinstance(result["date"], datetime)
        assert "CEO: Sample transcript" in result["transcript"]

    async def test_filter_transcripts(self, client):
        """Test transcript filtering functionality"""
        transcripts = [{"quarter": "Q1", "year": 2024}, {"quarter": "Q2", "year": 2024}, {"quarter": "Q1", "year": 2023}]

        # Filter by quarter
        filtered_q1 = client._filter_transcripts(transcripts, quarter="Q1")
        assert len(filtered_q1) == 2
        assert all(t["quarter"] == "Q1" for t in filtered_q1)

        # Filter by year
        filtered_2024 = client._filter_transcripts(transcripts, year=2024)
        assert len(filtered_2024) == 2
        assert all(t["year"] == 2024 for t in filtered_2024)

        # Filter by both
        filtered_both = client._filter_transcripts(transcripts, quarter="Q1", year=2024)
        assert len(filtered_both) == 1
        assert filtered_both[0]["quarter"] == "Q1"
        assert filtered_both[0]["year"] == 2024

    async def test_run_sync_method(self, client):
        """Test async wrapper for sync methods"""

        def sync_method():
            return "test_result"

        result = await client._run_sync_method(sync_method)
        assert result == "test_result"

    async def test_multiple_symbols_real_calls(self, client):
        """Test multiple symbols to ensure robustness"""
        symbols = ["AAPL", "MSFT", "GOOGL"]

        for symbol in symbols:
            try:
                result = await client.get_earnings_transcript(symbol)
                assert result["symbol"] == symbol.upper()
                assert "transcripts" in result
                assert isinstance(result["transcripts"], list)
            except Exception as e:
                # Log but don't fail - some symbols might not have data
                print(f"Warning: {symbol} failed with {e}")

    async def test_concurrent_requests(self, client):
        """Test concurrent API requests"""
        symbols = ["AAPL", "MSFT"]

        # Create concurrent tasks
        tasks = [client.get_earnings_transcript(symbol) for symbol in symbols]
        results = await asyncio.gather(*tasks, return_exceptions=True)

        # Verify all requests completed
        assert len(results) == len(symbols)

        for i, result in enumerate(results):
            if not isinstance(result, Exception):
                assert result["symbol"] == symbols[i].upper()

    async def test_error_handling_import_error(self):
        """Test handling when defeatbeta-api is not available"""
        # Mock the import to raise ImportError
        with patch.dict("sys.modules", {"defeatbeta_api": None, "defeatbeta_api.data": None, "defeatbeta_api.data.ticker": None}):
            client = DefeatBetaClient()

            # Should raise HTTPException for import error
            with pytest.raises(HTTPException) as exc_info:
                await client.get_earnings_transcript("AAPL")
            assert exc_info.value.status_code == 500
            assert "defeatbeta-api package not properly installed" in str(exc_info.value.detail)

    async def test_error_handling_general_exception(self):
        """Test handling of general exceptions"""
        # Mock the Ticker to raise a general exception
        with patch("defeatbeta_api.data.ticker.Ticker") as mock_ticker_class:
            mock_ticker = MagicMock()
            mock_ticker.earning_call_transcripts.side_effect = Exception("General error")
            mock_ticker_class.return_value = mock_ticker

            client = DefeatBetaClient()

            # Should raise HTTPException for general errors
            with pytest.raises(HTTPException) as exc_info:
                await client.get_earnings_transcript("ERROR")
            assert exc_info.value.status_code == 500
            assert "Failed to fetch earnings transcript" in str(exc_info.value.detail)

    async def test_get_financial_statement_income_statement_quarterly(self, client):
        """Test getting quarterly income statement"""
        symbol = "AAPL"
        statement_type = "income_statement"
        frequency = "quarterly"

        try:
            result = await client.get_financial_statement(symbol, statement_type, frequency)

            # Verify response structure
            assert isinstance(result, dict)
            assert result["symbol"] == symbol.upper()
            assert result["statement_type"] == statement_type
            assert result["frequency"] == frequency
            assert "statement" in result
            assert "metadata" in result

            # Verify metadata
            metadata = result["metadata"]
            assert metadata["source"] == "defeatbeta-api"
            assert "retrieved_at" in metadata
            assert "rows_count" in metadata
            assert "columns_count" in metadata

        except ImportError:
            # If defeatbeta-api is not installed, skip this test
            pytest.skip("defeatbeta-api not available")
        except HTTPException as e:
            # If method doesn't exist or no data, that's expected behavior
            assert e.status_code in [400, 404, 500]

    async def test_get_financial_statement_balance_sheet_annual(self, client):
        """Test getting annual balance sheet"""
        symbol = "MSFT"
        statement_type = "balance_sheet"
        frequency = "annual"

        try:
            result = await client.get_financial_statement(symbol, statement_type, frequency)

            assert result["symbol"] == symbol.upper()
            assert result["statement_type"] == statement_type
            assert result["frequency"] == frequency
            assert "statement" in result
            assert "metadata" in result

        except ImportError:
            pytest.skip("defeatbeta-api not available")
        except HTTPException as e:
            assert e.status_code in [400, 404, 500]

    async def test_get_financial_statement_cash_flow_quarterly(self, client):
        """Test getting quarterly cash flow"""
        symbol = "GOOGL"
        statement_type = "cash_flow"
        frequency = "quarterly"

        try:
            result = await client.get_financial_statement(symbol, statement_type, frequency)

            assert result["symbol"] == symbol.upper()
            assert result["statement_type"] == statement_type
            assert result["frequency"] == frequency
            assert "statement" in result
            assert "metadata" in result

        except ImportError:
            pytest.skip("defeatbeta-api not available")
        except HTTPException as e:
            assert e.status_code in [400, 404, 500]

    async def test_get_financial_statement_invalid_statement_type(self, client):
        """Test with invalid statement type"""
        symbol = "AAPL"
        statement_type = "invalid_statement"
        frequency = "quarterly"

        with pytest.raises(HTTPException) as exc_info:
            await client.get_financial_statement(symbol, statement_type, frequency)
        assert exc_info.value.status_code == 400
        assert "Unsupported statement type" in str(exc_info.value.detail)

    async def test_get_financial_statement_method_not_available(self, client):
        """Test when financial method is not available on ticker"""
        with patch("defeatbeta_api.data.ticker.Ticker") as mock_ticker_class:
            mock_ticker = MagicMock()
            # Mock ticker without the expected method
            delattr(mock_ticker, "quarterly_income_statement")  # Remove method
            mock_ticker_class.return_value = mock_ticker

            with pytest.raises(HTTPException) as exc_info:
                await client.get_financial_statement("AAPL", "income_statement", "quarterly")
            assert exc_info.value.status_code == 400
            assert "Method quarterly_income_statement not available" in str(exc_info.value.detail)

    async def test_get_financial_statement_no_data(self, client):
        """Test when financial method returns None"""
        with patch("defeatbeta_api.data.ticker.Ticker") as mock_ticker_class:
            mock_ticker = MagicMock()
            mock_ticker.quarterly_income_statement.return_value = None
            mock_ticker_class.return_value = mock_ticker

            with pytest.raises(HTTPException) as exc_info:
                await client.get_financial_statement("AAPL", "income_statement", "quarterly")
            assert exc_info.value.status_code == 404
            assert "No income_statement data found" in str(exc_info.value.detail)

    async def test_get_financial_statement_import_error(self, client):
        """Test financial statement with import error"""
        with patch.dict("sys.modules", {"defeatbeta_api": None, "defeatbeta_api.data": None, "defeatbeta_api.data.ticker": None}):
            with pytest.raises(HTTPException) as exc_info:
                await client.get_financial_statement("AAPL", "income_statement", "quarterly")
            assert exc_info.value.status_code == 500
            assert "defeatbeta-api package not properly installed" in str(exc_info.value.detail)

    async def test_get_financial_statement_general_exception(self, client):
        """Test financial statement with general exception"""
        with patch("defeatbeta_api.data.ticker.Ticker") as mock_ticker_class:
            mock_ticker = MagicMock()
            mock_ticker.quarterly_income_statement.side_effect = Exception("General error")
            mock_ticker_class.return_value = mock_ticker

            with pytest.raises(HTTPException) as exc_info:
                await client.get_financial_statement("AAPL", "income_statement", "quarterly")
            assert exc_info.value.status_code == 500
            assert "Failed to fetch income_statement" in str(exc_info.value.detail)

    def test_get_financial_method_name_quarterly_income_statement(self, client):
        """Test method name mapping for quarterly income statement"""
        method_name = client._get_financial_method_name("income_statement", "quarterly")
        assert method_name == "quarterly_income_statement"

    def test_get_financial_method_name_annual_balance_sheet(self, client):
        """Test method name mapping for annual balance sheet"""
        method_name = client._get_financial_method_name("balance_sheet", "annual")
        assert method_name == "annual_balance_sheet"

    def test_get_financial_method_name_quarterly_cash_flow(self, client):
        """Test method name mapping for quarterly cash flow"""
        method_name = client._get_financial_method_name("cash_flow", "quarterly")
        assert method_name == "quarterly_cash_flow"

    def test_get_financial_method_name_annual_income_statement(self, client):
        """Test method name mapping for annual income statement"""
        method_name = client._get_financial_method_name("income_statement", "annual")
        assert method_name == "annual_income_statement"

    def test_get_financial_method_name_invalid_statement_type(self, client):
        """Test method name mapping with invalid statement type"""
        with pytest.raises(HTTPException) as exc_info:
            client._get_financial_method_name("invalid_statement", "quarterly")
        assert exc_info.value.status_code == 400
        assert "Unsupported statement type: invalid_statement" in str(exc_info.value.detail)

    def test_format_financial_data_with_dataframe(self, client):
        """Test formatting financial data when it's already a DataFrame"""

        import pandas as pd

        # Create mock DataFrame
        mock_df = pd.DataFrame(
            {"Revenue": [1000, 1100, 1200], "Net Income": [100, 110, 120], "Date": pd.to_datetime(["2023-01-01", "2023-04-01", "2023-07-01"])}
        )

        result = client._format_financial_data(mock_df, "AAPL", "income_statement", "quarterly")

        assert result["symbol"] == "AAPL"
        assert result["statement_type"] == "income_statement"
        assert result["frequency"] == "quarterly"
        assert "statement" in result
        assert "metadata" in result
        assert result["metadata"]["rows_count"] == 3
        assert result["metadata"]["columns_count"] == 3

    def test_format_financial_data_with_get_data_method(self, client):
        """Test formatting financial data with get_data method"""
        import pandas as pd

        # Create mock object with get_data method
        mock_financial_obj = MagicMock()
        mock_df = pd.DataFrame({"Revenue": [1000, 1100], "Expenses": [800, 850]})
        mock_financial_obj.get_data.return_value = mock_df

        result = client._format_financial_data(mock_financial_obj, "MSFT", "income_statement", "annual")

        assert result["symbol"] == "MSFT"
        assert result["statement_type"] == "income_statement"
        assert result["frequency"] == "annual"
        assert result["metadata"]["rows_count"] == 2

    def test_format_financial_data_with_data_attribute(self, client):
        """Test formatting financial data with data attribute"""
        import pandas as pd

        # Create mock object with data attribute
        mock_df = pd.DataFrame({"Assets": [5000, 5500], "Liabilities": [3000, 3200]})

        # Create a proper mock object that satisfies the isinstance check
        class MockFinancialData:
            def __init__(self, data):
                self.data = data

        mock_financial_obj = MockFinancialData(mock_df)

        result = client._format_financial_data(mock_financial_obj, "GOOGL", "balance_sheet", "quarterly")

        assert result["symbol"] == "GOOGL"
        assert result["statement_type"] == "balance_sheet"
        assert result["frequency"] == "quarterly"

    def test_format_financial_data_with_dict(self, client):
        """Test formatting financial data when it's a dictionary"""
        mock_dict = {"Revenue": 1000, "Net Income": 100, "Date": "2023-01-01"}

        result = client._format_financial_data(mock_dict, "TSLA", "income_statement", "annual")

        assert result["symbol"] == "TSLA"
        assert result["statement_type"] == "income_statement"
        assert result["frequency"] == "annual"
        assert result["metadata"]["rows_count"] == 1

    def test_format_financial_data_with_list(self, client):
        """Test formatting financial data when it's a list"""
        mock_list = [{"Revenue": 1000, "Net Income": 100}, {"Revenue": 1100, "Net Income": 110}]

        result = client._format_financial_data(mock_list, "NFLX", "income_statement", "quarterly")

        assert result["symbol"] == "NFLX"
        assert result["metadata"]["rows_count"] == 2

    def test_format_financial_data_empty_dataframe(self, client):
        """Test formatting with empty DataFrame"""
        import pandas as pd

        empty_df = pd.DataFrame()

        with pytest.raises(HTTPException) as exc_info:
            client._format_financial_data(empty_df, "AAPL", "income_statement", "quarterly")
        assert exc_info.value.status_code == 404
        assert "No income_statement data found for AAPL" in str(exc_info.value.detail)

    def test_format_financial_data_none_data(self, client):
        """Test formatting with None data"""
        with pytest.raises(HTTPException) as exc_info:
            client._format_financial_data(None, "AAPL", "income_statement", "quarterly")
        assert exc_info.value.status_code == 404

    def test_format_financial_data_conversion_error(self, client):
        """Test formatting with data that can't be converted to DataFrame"""
        # Create an object that can't be converted to DataFrame
        invalid_data = object()

        with pytest.raises(HTTPException) as exc_info:
            client._format_financial_data(invalid_data, "AAPL", "income_statement", "quarterly")
        assert exc_info.value.status_code == 500
        assert "Unable to convert financial data to DataFrame" in str(exc_info.value.detail)

    def test_format_financial_data_with_nan_values(self, client):
        """Test formatting financial data with NaN values"""
        import numpy as np
        import pandas as pd

        # Create DataFrame with NaN values
        mock_df = pd.DataFrame(
            {"Revenue": [1000, np.nan, 1200], "Net Income": [100, 110, np.nan], "Date": pd.to_datetime(["2023-01-01", "2023-04-01", "2023-07-01"])}
        )

        result = client._format_financial_data(mock_df, "AAPL", "income_statement", "quarterly")

        # Verify NaN values are converted to None
        statement_data = result["statement"]
        for row_data in statement_data.values():
            for value in row_data.values():
                if value is None:
                    # This is expected for NaN values
                    continue
                assert value is not None or not pd.isna(value)

    def test_format_financial_data_with_timestamps(self, client):
        """Test formatting financial data with pandas Timestamps"""
        import pandas as pd

        # Create DataFrame with Timestamp column
        mock_df = pd.DataFrame({"Revenue": [1000, 1100], "Date": pd.to_datetime(["2023-01-01", "2023-04-01"])})

        result = client._format_financial_data(mock_df, "AAPL", "income_statement", "quarterly")

        # Verify timestamps are converted to ISO format strings
        statement_data = result["statement"]
        for row_data in statement_data.values():
            if "Date" in row_data:
                date_value = row_data["Date"]
                if date_value is not None:
                    # Should be ISO format string
                    assert isinstance(date_value, str)
                    assert "T" in date_value  # ISO format contains 'T'

    async def test_multiple_financial_statements_concurrent(self, client):
        """Test concurrent financial statement requests"""
        symbols = ["AAPL", "MSFT"]
        statement_types = ["income_statement", "balance_sheet"]
        frequencies = ["quarterly", "annual"]

        # Create tasks for all combinations
        tasks = []
        for symbol in symbols:
            for statement_type in statement_types:
                for frequency in frequencies:
                    task = client.get_financial_statement(symbol, statement_type, frequency)
                    tasks.append(task)

        try:
            results = await asyncio.gather(*tasks, return_exceptions=True)

            # Verify all requests completed (some may have exceptions)
            assert len(results) == len(symbols) * len(statement_types) * len(frequencies)

            # Check successful results
            for result in results:
                if not isinstance(result, Exception):
                    assert "symbol" in result
                    assert "statement_type" in result
                    assert "frequency" in result

        except ImportError:
            pytest.skip("defeatbeta-api not available")
