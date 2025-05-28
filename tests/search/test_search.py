from unittest.mock import AsyncMock, MagicMock, patch, ANY

import pytest
from orjson import orjson

from src.models import SearchResult, Type
from src.services import get_search
from src.services.search.fetchers import fetch_algolia_search_results, fetch_yahoo_search_results
from tests.conftest import VERSION

# Mock search response data for all three supported types
MOCK_SEARCH_RESPONSE = [
    {"name": "Amazon.com, Inc.", "symbol": "AMZN", "exchange": "NASDAQ", "type": "stock"},
    {"name": "Amazon ETF", "symbol": "AMZN-ETF", "exchange": "NYSE", "type": "etf"},
    {"name": "Amazon Trust", "symbol": "AMZN-TRUST", "exchange": "NYSE", "type": "trust"},
]


class TestSearch:
    @pytest.fixture
    def mock_api_response(self):
        """
        Fixture that provides a function to get a mock API response for search queries.
        """

        def get_mock_response(url, params=None):
            # Define mock response data based on the query
            if "query1.finance.yahoo.com/v1/finance/search" in url:
                mock_data = {
                    "quotes": [
                        {
                            "shortname": "Amazon.com, Inc.",
                            "symbol": "AMZN",
                            "exchange": "NASDAQ",
                            "quoteType": "EQUITY",
                        },
                        {"shortname": "Amazon ETF", "symbol": "AMZN-ETF", "exchange": "NYSE", "quoteType": "ETF"},
                        {
                            "shortname": "Amazon Trust",
                            "symbol": "AMZN-TRUST",
                            "exchange": "NYSE",
                            "quoteType": "MUTUALFUND",
                        },
                        {"shortname": "Amazon Futures", "symbol": "AMZN-FUT", "exchange": "CME", "quoteType": "FUTURE"},
                    ]
                }
                return orjson.dumps(mock_data).decode("utf-8")
            return orjson.dumps({}).decode("utf-8")

        return get_mock_response

    def test_search_success(self, test_client, monkeypatch):
        """Test successful search retrieval"""
        # Mock the search service function
        mock_fetch_search_results = AsyncMock(return_value=MOCK_SEARCH_RESPONSE)
        monkeypatch.setattr("src.routes.search.get_search", mock_fetch_search_results)

        # Make the request
        response = test_client.get(f"{VERSION}/search?query=AMZN&hits=10")
        data = response.json()

        # Assertions
        assert response.status_code == 200
        assert len(data) == len(MOCK_SEARCH_RESPONSE)
        assert data == MOCK_SEARCH_RESPONSE

        # Verify mock was called
        mock_fetch_search_results.assert_awaited_once_with(ANY, "AMZN", 10, None)

    @pytest.mark.parametrize("hits", [101, 0])
    def test_search_invalid_hits(self, test_client, hits):
        """Test search retrieval with invalid hits parameter"""
        response = test_client.get(f"{VERSION}/search?query=AMZN&hits={hits}")
        # Should return a 422 Unprocessable Entity
        assert response.status_code == 422
        # Response should contain validation error
        error_detail = response.json()["errors"]
        assert "hits" in error_detail

    def test_search_invalid_type(self, test_client):
        """Test search with invalid type"""
        response = test_client.get(f"{VERSION}/search?query=AMZN&type=invalid")
        # Should return a 422 Unprocessable Entity
        assert response.status_code == 422
        # Response should contain validation error
        error_detail = response.json()["errors"]
        assert "type" in error_detail

    @pytest.mark.parametrize("type_value, expected_type", [("stock", Type.STOCK), ("etf", Type.ETF), ("trust", Type.TRUST)])
    def test_search_with_type_filter(self, test_client, monkeypatch, type_value, expected_type):
        """Test search with different type filters"""
        # Mock the search service function with filtering
        filtered_response = [item for item in MOCK_SEARCH_RESPONSE if item["type"] == type_value]
        mock_fetch_search_results = AsyncMock(return_value=filtered_response)
        monkeypatch.setattr("src.routes.search.get_search", mock_fetch_search_results)

        # Make the request with type filter
        response = test_client.get(f"{VERSION}/search?query=AMZN&type={type_value}")
        data = response.json()

        # Assertions
        assert response.status_code == 200
        assert len(data) == 1
        assert data[0]["type"] == type_value

        # Verify mock was called with correct type
        mock_fetch_search_results.assert_awaited_once_with(ANY, "AMZN", 50, expected_type)

    @pytest.mark.parametrize(
        "query, hits, type_filter, expected_count",
        [
            ("AMZN", 10, None, 3),  # All types
            ("AMZN", 10, Type.STOCK, 1),
            ("AMZN", 10, Type.ETF, 1),
            ("AMZN", 10, Type.TRUST, 1),
        ],
    )
    async def test_get_search_algolia(self, mock_finance_client, query, hits, type_filter, expected_count):
        """Test get_search function with mocked Algolia API response"""
        # Create filtered responses based on type filter
        filtered_response = [item for item in MOCK_SEARCH_RESPONSE if type_filter is None or item["type"] == type_filter.value]

        # Mock the Algolia search to succeed
        with patch("src.services.search.get_search.fetch_algolia_search_results", new_callable=AsyncMock) as mock_algolia:
            mock_algolia.return_value = filtered_response

            # Mock the Yahoo search as fallback (should not be called)
            mock_finance_client.search = AsyncMock()

            # Call the function
            result = await get_search(mock_finance_client, query, hits, type_filter)

            # Verify results
            assert len(result) == expected_count
            if type_filter:
                assert all(item["type"] == type_filter.value for item in result)

            assert mock_algolia.called
            mock_finance_client.search.assert_not_called()
            mock_algolia.assert_called_once_with(query, hits, type_filter)

    @pytest.mark.parametrize(
        "type_filter, facet_filter",
        [(None, None), (Type.STOCK, ["type:stock"]), (Type.ETF, ["type:etf"]), (Type.TRUST, ["type:trust"])],
    )
    async def test_fetch_algolia_search_results(self, type_filter, facet_filter):
        """Test fetch_algolia_search_results function with appropriate facet filters"""
        query = "AMZN"
        hits = 10

        # Create appropriate mock response based on type filter
        mock_hits = [
            {"name": "Amazon.com, Inc.", "symbol": "AMZN", "exchangeShortName": "NASDAQ", "type": "stock"},
            {"name": "Amazon ETF", "symbol": "AMZN-ETF", "exchangeShortName": "NYSE", "type": "etf"},
            {"name": "Amazon Trust", "symbol": "AMZN-TRUST", "exchangeShortName": "NYSE", "type": "trust"},
        ]

        # Filter hits based on type_filter for realistic return values
        if type_filter:
            filtered_hits = [hit for hit in mock_hits if hit["type"] == type_filter.value]
        else:
            filtered_hits = mock_hits

        # Create mock Algolia search client
        mock_index = MagicMock()
        mock_index.search.return_value = {"hits": filtered_hits}

        mock_client = MagicMock()
        mock_client.init_index.return_value = mock_index

        with patch("algoliasearch.search_client.SearchClient.create", return_value=mock_client):
            # Call the function with the specified filter
            results = await fetch_algolia_search_results(query, hits, type_filter)

            # Verify search was called once with the correct query
            mock_index.search.assert_called_once()
            args, kwargs = mock_index.search.call_args

            # First argument should be the query
            assert query in args

            # Verify expected number of results based on filter
            expected_count = 1 if type_filter else 3
            assert len(results) == expected_count

            # If type filter provided, verify all results have that type
            if type_filter:
                assert all(r.type == type_filter.value for r in results)

            # Check that params were passed correctly to search
            if "params" in kwargs:
                params = kwargs["params"]

                # Verify hits parameter
                assert params["hitsPerPage"] == hits

                # Verify facet filters if type_filter is provided
                if type_filter:
                    assert params["facetFilters"] == facet_filter
                else:
                    assert "facetFilters" not in params
