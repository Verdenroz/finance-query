from unittest.mock import AsyncMock, patch

import pytest
from orjson import orjson

from src.models import INDEX_REGIONS, Index, MarketIndex, Region
from src.services import get_indices
from src.services.indices.fetchers import fetch_index
from tests.conftest import VERSION

# Mock indices response data
MOCK_INDICES_RESPONSE = [
    {
        "name": "S&P 500",
        "value": 5234.32,
        "change": "+15.29",
        "percentChange": "+0.29%",
        "fiveDaysReturn": None,
        "oneMonthReturn": None,
        "threeMonthReturn": None,
        "sixMonthReturn": None,
        "ytdReturn": None,
        "yearReturn": None,
        "threeYearReturn": None,
        "fiveYearReturn": None,
        "tenYearReturn": None,
        "maxReturn": None,
    },
    {
        "name": "Dow Jones Industrial Average",
        "value": 39127.14,
        "change": "+62.39",
        "percentChange": "+0.16%",
        "fiveDaysReturn": None,
        "oneMonthReturn": None,
        "threeMonthReturn": None,
        "sixMonthReturn": None,
        "ytdReturn": None,
        "yearReturn": None,
        "threeYearReturn": None,
        "fiveYearReturn": None,
        "tenYearReturn": None,
        "maxReturn": None,
    },
    {
        "name": "NASDAQ Composite",
        "value": 16439.22,
        "change": "+83.95",
        "percentChange": "+0.51%",
        "fiveDaysReturn": None,
        "oneMonthReturn": None,
        "threeMonthReturn": None,
        "sixMonthReturn": None,
        "ytdReturn": None,
        "yearReturn": None,
        "threeYearReturn": None,
        "fiveYearReturn": None,
        "tenYearReturn": None,
        "maxReturn": None,
    },
    {
        "name": "Russell 2000",
        "value": 2042.60,
        "change": "-3.71",
        "percentChange": "-0.18%",
        "fiveDaysReturn": None,
        "oneMonthReturn": None,
        "threeMonthReturn": None,
        "sixMonthReturn": None,
        "ytdReturn": None,
        "yearReturn": None,
        "threeYearReturn": None,
        "fiveYearReturn": None,
        "tenYearReturn": None,
        "maxReturn": None,
    },
]


class TestIndices:
    @pytest.fixture
    def mock_api_response(self):
        """
        Fixture that provides a function to get a mock API response for URLs.
        """
        response_cache = {}

        def get_mock_response(url):
            if url in response_cache:
                return response_cache[url]

            # Define mock response data based on the URL
            mock_responses = {
                "https://query2.finance.yahoo.com/v10/finance/quoteSummary/GSPC": {
                    "quoteSummary": {
                        "result": [
                            {
                                "price": {
                                    "regularMarketPrice": {"raw": 5234.32},
                                    "regularMarketChange": {"fmt": "+15.29"},
                                    "regularMarketChangePercent": {"fmt": "+0.29%"},
                                    "longName": "S&P 500",
                                    "shortName": "S&P 500",
                                },
                                "quoteUnadjustedPerformanceOverview": {
                                    "performanceOverview": {
                                        "fiveDaysReturn": {"fmt": "+1.23%"},
                                        "oneMonthReturn": {"fmt": "-2.34%"},
                                        "threeMonthReturn": {"fmt": "+3.45%"},
                                        "sixMonthReturn": {"fmt": "-4.56%"},
                                        "ytdReturnPct": {"fmt": "+5.67%"},
                                        "oneYearTotalReturn": {"fmt": "-6.78%"},
                                        "threeYearTotalReturn": {"fmt": "+7.89%"},
                                        "fiveYearTotalReturn": {"fmt": "-8.90%"},
                                        "tenYearTotalReturn": {"fmt": "+9.01%"},
                                        "maxReturn": {"fmt": "-10.12%"},
                                    }
                                },
                            }
                        ],
                        "error": None,
                    }
                },
                "https://query2.finance.yahoo.com/v10/finance/quoteSummary/DJI": {
                    "quoteSummary": {
                        "result": [
                            {
                                "price": {
                                    "regularMarketPrice": {"raw": 39127.14},
                                    "regularMarketChange": {"fmt": "+62.39"},
                                    "regularMarketChangePercent": {"fmt": "+0.16%"},
                                    "longName": "Dow Jones Industrial Average",
                                    "shortName": "Dow Jones Industrial Average",
                                },
                                "quoteUnadjustedPerformanceOverview": {
                                    "performanceOverview": {
                                        "fiveDaysReturn": {"fmt": "+1.23%"},
                                        "oneMonthReturn": {"fmt": "-2.34%"},
                                        "threeMonthReturn": {"fmt": "+3.45%"},
                                        "sixMonthReturn": {"fmt": "-4.56%"},
                                        "ytdReturnPct": {"fmt": "+5.67%"},
                                        "oneYearTotalReturn": {"fmt": "-6.78%"},
                                        "threeYearTotalReturn": {"fmt": "+7.89%"},
                                        "fiveYearTotalReturn": {"fmt": "-8.90%"},
                                        "tenYearTotalReturn": {"fmt": "+9.01%"},
                                        "maxReturn": {"fmt": "-10.12%"},
                                    }
                                },
                            }
                        ],
                        "error": None,
                    }
                },
            }

            response_content = orjson.dumps(mock_responses[url]).decode("utf-8")
            response_cache[url] = response_content
            return response_content

        return get_mock_response

    def test_get_indices_success(self, test_client, monkeypatch):
        """Test successful indices retrieval"""
        mock_scrape_indices = AsyncMock(return_value=MOCK_INDICES_RESPONSE)
        monkeypatch.setattr("src.routes.indices.get_indices", mock_scrape_indices)

        response = test_client.get(f"{VERSION}/indices")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == len(MOCK_INDICES_RESPONSE)
        assert data == MOCK_INDICES_RESPONSE

        mock_scrape_indices.assert_awaited_once()

    def test_get_indices_by_specific_index(self, test_client, monkeypatch):
        """Test indices retrieval with specific index filter"""
        mock_scrape_indices = AsyncMock(return_value=[MOCK_INDICES_RESPONSE[0]])
        monkeypatch.setattr("src.routes.indices.get_indices", mock_scrape_indices)

        response = test_client.get(f"{VERSION}/indices?index=snp")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == 1
        assert data[0]["name"] == "S&P 500"

        mock_scrape_indices.assert_awaited_once()

    def test_get_indices_by_region(self, test_client, mock_finance_client, monkeypatch):
        """Test indices retrieval filtered by region"""
        mock_scrape_indices = AsyncMock(return_value=MOCK_INDICES_RESPONSE)
        monkeypatch.setattr("src.routes.indices.get_indices", mock_scrape_indices)

        response = test_client.get(f"{VERSION}/indices?region=US")
        data = response.json()

        assert response.status_code == 200
        assert len(data) > 0

        mock_scrape_indices.assert_awaited_once()

        args = mock_scrape_indices.call_args[0]
        us_indices = args[1]
        for idx in us_indices:
            assert INDEX_REGIONS.get(idx) == Region.UNITED_STATES
        assert len(us_indices) == 7
        us_enum_values = {Index.GSPC, Index.DJI, Index.IXIC, Index.NYA, Index.XAX, Index.RUT, Index.VIX}
        assert all(idx in us_enum_values for idx in us_indices)

    def test_get_indices_multiple_filters(self, test_client, mock_finance_client, monkeypatch):
        """Test indices retrieval with multiple filters"""
        mock_scrape_indices = AsyncMock(return_value=[MOCK_INDICES_RESPONSE[0], MOCK_INDICES_RESPONSE[1]])
        monkeypatch.setattr("src.routes.indices.get_indices", mock_scrape_indices)

        response = test_client.get(f"{VERSION}/indices?index=dax&region=US")
        data = response.json()

        assert response.status_code == 200
        assert len(data) == 2
        assert {item["name"] for item in data} == {"S&P 500", "Dow Jones Industrial Average"}

        mock_scrape_indices.assert_awaited_once()

        args = mock_scrape_indices.call_args[0]
        ordered_indices = args[1]
        assert Index.GDAXI in ordered_indices
        us_indices = [idx for idx in ordered_indices if INDEX_REGIONS.get(idx) == Region.UNITED_STATES]
        assert len(us_indices) == 7
        assert len(ordered_indices) == 8

    def test_get_indices_invalid_region(self, test_client, mock_finance_client):
        """Test indices retrieval with invalid region parameter"""
        response = test_client.get(f"{VERSION}/indices?region=INVALID_REGION")
        assert response.status_code == 422
        error_detail = response.json()["errors"]
        assert "region" in error_detail

    def test_get_indices_invalid_index(self, test_client, mock_finance_client):
        """Test indices retrieval with invalid index parameter"""
        response = test_client.get(f"{VERSION}/indices?index=INVALID_INDEX")
        assert response.status_code == 422
        error_detail = response.json()["errors"]
        assert "index.0" in error_detail

    async def test_get_indices(self, bypass_cache, mock_finance_client):
        """Test get_indices function with all indices"""
        mock_index_data = MarketIndex(
            name="Test Index",
            value=1000.0,
            change="+10.0",
            percent_change="+1.0%",
        )

        with patch("src.services.indices.get_indices.fetch_index", new_callable=AsyncMock) as mock_fetch_index:
            mock_fetch_index.return_value = mock_index_data
            indices = list(Index)
            result = await get_indices(mock_finance_client, None)

            assert len(result) == len(indices)
            assert all(isinstance(index, MarketIndex) for index in result)
            assert mock_fetch_index.call_count == len(indices)

    async def test_get_indices_error_handling(self, bypass_cache, mock_finance_client):
        """Test get_indices handles errors correctly"""
        mock_index_data = MarketIndex(name="Test Index", value=1000.0, change="+10.0", percent_change="+1.0%")

        async def mock_fetch_side_effect(client, index, *args, **kwargs):
            if index == Index.GDAXI:
                return None  # Return None instead of Exception to simulate failure
            return mock_index_data

        with patch("src.services.indices.get_indices.fetch_index", new_callable=AsyncMock) as mock_fetch_index:
            mock_fetch_index.side_effect = mock_fetch_side_effect
            test_indices = [Index.GSPC, Index.DJI, Index.GDAXI, Index.IXIC]
            result = await get_indices(mock_finance_client, test_indices)
            assert len(result) == 3  # One index should be filtered out since it returned None

    async def test_get_indices_missing_credentials(self, bypass_cache, mock_finance_client):
        """Test get_indices raises error with missing credentials scenario"""

        async def mock_get_quote_value_error(*args, **kwargs):
            raise ValueError("Simulated credential error from finance_client")

        mock_finance_client.get_quote = AsyncMock(side_effect=mock_get_quote_value_error)

        with pytest.raises(ValueError, match="Simulated credential error from finance_client"):
            await get_indices(mock_finance_client, None)

    @pytest.mark.parametrize("index", [Index.GSPC, Index.DJI])
    async def test_fetch_index(self, mock_finance_client, mock_api_response, index, bypass_cache):
        """Test fetch_index function with mocked API response"""
        # Prepare the Yahoo symbol that should be used
        symbol_map = {Index.GSPC: "^GSPC", Index.DJI: "^DJI"}
        yahoo_symbol = symbol_map.get(index, f"^{index.name}")

        # Prepare mock response data
        test_url = f"https://query2.finance.yahoo.com/v10/finance/quoteSummary/{index.name}"
        raw_response_str = mock_api_response(test_url)
        mock_response_data = orjson.loads(raw_response_str)
        mock_finance_client.get_quote.return_value = mock_response_data

        # Execute the function
        result = await fetch_index(mock_finance_client, index)

        # Verify the results
        assert isinstance(result, MarketIndex)
        assert result.value is not None
        assert result.change is not None
        assert result.percent_change is not None

        # Check that all return fields have values from the mock data
        assert result.five_days_return is not None
        assert result.one_month_return is not None
        assert result.three_month_return is not None
        assert result.six_month_return is not None
        assert result.ytd_return is not None
        assert result.year_return is not None
        assert result.three_year_return is not None
        assert result.five_year_return is not None
        assert result.ten_year_return is not None
        assert result.max_return is not None

        # Verify that the client was called with the correct symbol
        mock_finance_client.get_quote.assert_called_once_with(yahoo_symbol)

    @pytest.mark.parametrize("index", [Index.GSPC, Index.DJI])
    async def test_fetch_index_failure(self, mock_finance_client, mock_api_response, index):
        """Test fetch_index function with mocked API response failure"""
        # Configure mock_finance_client.get_quote to return None to simulate a failed API call
        mock_finance_client.get_quote.return_value = None
        # Configure the simple quotes fallback to return empty results
        mock_finance_client.get_simple_quotes.return_value = {"quoteResponse": {"result": []}}

        # Also mock the get_quotes fallback to ensure it doesn't accidentally succeed
        with patch("src.services.indices.fetchers.fetch_index.get_quotes", new_callable=AsyncMock) as mock_get_quotes_fallback:
            mock_get_quotes_fallback.return_value = []  # Simulate no data from scraping fallback

            result = await fetch_index(mock_finance_client, index)
            # Expecting a minimal MarketIndex when all fetch attempts fail
            assert result is not None
            assert result.name == index.value
            assert result.value == 0.0
            assert result.change == ""
            assert result.percent_change == ""

            # Verify all fallbacks were called correctly
            symbol_map = {Index.GSPC: "^GSPC", Index.DJI: "^DJI"}
            yahoo_symbol = symbol_map.get(index, f"^{index.name}")
            mock_finance_client.get_quote.assert_called_once_with(yahoo_symbol)
            mock_finance_client.get_simple_quotes.assert_called_once_with([yahoo_symbol])
            mock_get_quotes_fallback.assert_called_once_with(mock_finance_client, [yahoo_symbol])

    @pytest.mark.parametrize("index", [Index.GSPC, Index.DJI])
    async def test_fetch_index_simple_quotes_fallback(self, mock_finance_client, index):
        """Test fetch_index fallback to simple_quotes when get_quote fails"""
        # Configure primary method to fail
        mock_finance_client.get_quote.return_value = None

        # Configure the simple quotes fallback to return valid data
        mock_finance_client.get_simple_quotes.return_value = {
            "quoteResponse": {
                "result": [
                    {
                        "shortName": f"{index.value} Index",
                        "longName": f"{index.value} Index",
                        "regularMarketPrice": 1000.0,
                        "regularMarketChange": 10.0,
                        "regularMarketChangePercent": 1.0,
                        "regularMarketPreviousClose": 990.0,
                        "quoteSummary": {
                            "result": [
                                {
                                    "price": {
                                        "regularMarketPrice": {"raw": 1000.0},
                                        "regularMarketChange": {"fmt": "+10.0"},
                                        "regularMarketChangePercent": {"fmt": "+1.0%"},
                                        "longName": f"{index.value} Index",
                                        "shortName": f"{index.value} Index",
                                    },
                                    "quoteUnadjustedPerformanceOverview": {"performanceOverview": {}},
                                }
                            ]
                        },
                    }
                ],
                "error": None,
            }
        }

        # Mock the third fallback to ensure it's not called
        with patch("src.services.indices.fetchers.fetch_index.get_quotes", new_callable=AsyncMock) as mock_get_quotes_fallback:
            # Run the function
            result = await fetch_index(mock_finance_client, index)

            # Verify the result contains data from the simple_quotes fallback
            assert result is not None
            assert isinstance(result, MarketIndex)
            assert result.name == f"{index.value} Index"
            assert result.value == 1000.0
            assert result.change == "+10.0"
            assert result.percent_change == "+1.0%"

            # Verify the first method was called and failed
            symbol_map = {Index.GSPC: "^GSPC", Index.DJI: "^DJI"}
            yahoo_symbol = symbol_map.get(index, f"^{index.name}")
            mock_finance_client.get_quote.assert_called_once_with(yahoo_symbol)

            # Verify the second method (fallback) was called
            mock_finance_client.get_simple_quotes.assert_called_once_with([yahoo_symbol])

            # The third fallback should not be called since the second worked
            mock_get_quotes_fallback.assert_not_called()

    @pytest.mark.parametrize("index", [Index.GSPC, Index.DJI])
    async def test_fetch_index_get_quotes_fallback(self, mock_finance_client, index):
        """Test fetch_index fallback to get_quotes when both get_quote and get_simple_quotes fail"""
        # Configure first two methods to fail
        mock_finance_client.get_quote.return_value = None
        mock_finance_client.get_simple_quotes.return_value = {"quoteResponse": {"result": []}}

        # Setup the quote object that get_quotes will return
        mock_quote = AsyncMock()
        mock_quote.name = f"{index.value} Fallback"
        mock_quote.price = "500.0"
        mock_quote.change = "+5.0"
        mock_quote.change_percent = "+1.0%"
        mock_quote.five_days_return = "+2.5%"
        mock_quote.one_month_return = "-1.5%"
        mock_quote.three_month_return = "+3.2%"
        mock_quote.six_month_return = "-2.1%"
        mock_quote.ytd_return = "+4.3%"
        mock_quote.year_return = "-0.8%"
        mock_quote.three_year_return = "+12.5%"
        mock_quote.five_year_return = "+25.0%"
        mock_quote.ten_year_return = "+50.0%"
        mock_quote.max_return = "+75.0%"

        # Mock the third fallback to return valid data
        with patch("src.services.indices.fetchers.fetch_index.get_quotes", new_callable=AsyncMock) as mock_get_quotes_fallback:
            mock_get_quotes_fallback.return_value = [mock_quote]

            # Run the function
            result = await fetch_index(mock_finance_client, index)

            # Verify the result contains data from the get_quotes fallback
            assert result is not None
            assert isinstance(result, MarketIndex)
            assert result.name == f"{index.value} Fallback"
            assert result.value == 500.0
            assert result.change == "+5.0"
            assert result.percent_change == "+1.0%"
            assert result.five_days_return == "+2.5%"
            assert result.one_month_return == "-1.5%"

            # Verify all methods were called in sequence
            symbol_map = {Index.GSPC: "^GSPC", Index.DJI: "^DJI"}
            yahoo_symbol = symbol_map.get(index, f"^{index.name}")
            mock_finance_client.get_quote.assert_called_once_with(yahoo_symbol)
            mock_finance_client.get_simple_quotes.assert_called_once_with([yahoo_symbol])
            mock_get_quotes_fallback.assert_called_once_with(mock_finance_client, [yahoo_symbol])

    @pytest.mark.parametrize(
        "index, expected_symbol",
        [
            (Index.GSPC, "^GSPC"),
            (Index.DJI, "^DJI"),
            (Index.MOEX_ME, "MOEX.ME"),
            (Index.SHANGHAI, "000001.SS"),
            (Index.PSI, "PSI20.LS"),
        ],
    )
    async def test_get_yahoo_index_symbol(self, index, expected_symbol):
        """Test _get_yahoo_index_symbol function with various indices"""
        from src.services.indices.fetchers.fetch_index import _get_yahoo_index_symbol

        symbol = _get_yahoo_index_symbol(index)
        assert symbol == expected_symbol

    @pytest.mark.parametrize(
        "index, default_name, expected_name",
        [
            (Index.GDAXI, "German DAX", "DAX Performance Index"),
            (Index.STOXX50E, "Euro Stoxx", "EURO STOXX 50"),
            (Index.NZ50, "NZ Index", "S&P/NZX 50 Index"),
            (Index.GSPC, "S&P 500", "S&P 500"),
            (Index.DJI, "Dow Jones", "Dow Jones"),
        ],
    )
    async def test_get_formatted_index_name(self, index, default_name, expected_name):
        """Test _get_formatted_index_name function with various indices"""
        from src.services.indices.fetchers.fetch_index import _get_formatted_index_name

        formatted_name = _get_formatted_index_name(index, default_name)
        assert formatted_name == expected_name
