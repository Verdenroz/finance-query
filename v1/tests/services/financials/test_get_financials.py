from unittest.mock import AsyncMock

import pytest
from fastapi import HTTPException

from src.models.financials import Frequency, StatementType
from src.services.financials.get_financials import (
    _map_statement_type_to_key,
    _parse_timeseries_data,
    get_financial_statement,
)


@pytest.fixture
def mock_finance_client():
    """Create a mock YahooFinanceClient"""
    return AsyncMock()


@pytest.fixture
def sample_timeseries_response():
    """Sample Yahoo Finance timeseries API response"""
    return {
        "timeseries": {
            "result": [
                {
                    "meta": {"symbol": ["AAPL"], "type": ["annualTotalRevenue"]},
                    "timestamp": [1640995200, 1672531200, 1704067200],
                    "annualTotalRevenue": [
                        {
                            "dataId": 1,
                            "asOfDate": "2022-09-30",
                            "periodType": "12M",
                            "currencyCode": "USD",
                            "reportedValue": {"raw": {"source": "394.328E9", "parsedValue": 394328000000}, "fmt": "394.33B"},
                        },
                        {
                            "dataId": 2,
                            "asOfDate": "2023-09-30",
                            "periodType": "12M",
                            "currencyCode": "USD",
                            "reportedValue": {"raw": {"source": "383.285E9", "parsedValue": 383285000000}, "fmt": "383.29B"},
                        },
                        {
                            "dataId": 3,
                            "asOfDate": "2024-09-30",
                            "periodType": "12M",
                            "currencyCode": "USD",
                            "reportedValue": {"raw": {"source": "391.035E9", "parsedValue": 391035000000}, "fmt": "391.04B"},
                        },
                    ],
                },
                {
                    "meta": {"symbol": ["AAPL"], "type": ["annualNetIncome"]},
                    "timestamp": [1640995200, 1672531200, 1704067200],
                    "annualNetIncome": [
                        {
                            "dataId": 4,
                            "asOfDate": "2022-09-30",
                            "periodType": "12M",
                            "currencyCode": "USD",
                            "reportedValue": {"raw": {"source": "99.803E9", "parsedValue": 99803000000}, "fmt": "99.80B"},
                        },
                        {
                            "dataId": 5,
                            "asOfDate": "2023-09-30",
                            "periodType": "12M",
                            "currencyCode": "USD",
                            "reportedValue": {"raw": {"source": "96.995E9", "parsedValue": 96995000000}, "fmt": "96.99B"},
                        },
                        {
                            "dataId": 6,
                            "asOfDate": "2024-09-30",
                            "periodType": "12M",
                            "currencyCode": "USD",
                            "reportedValue": {"raw": {"source": "93.736E9", "parsedValue": 93736000000}, "fmt": "93.74B"},
                        },
                    ],
                },
            ],
            "error": None,
        }
    }


# Test mapping function
def test_map_statement_type_to_key():
    """Test statement type to key mapping"""
    assert _map_statement_type_to_key(StatementType.INCOME_STATEMENT) == "income"
    assert _map_statement_type_to_key(StatementType.BALANCE_SHEET) == "balance"
    assert _map_statement_type_to_key(StatementType.CASH_FLOW) == "cashflow"


# Test parser function
def test_parse_timeseries_data():
    """Test parsing timeseries data from Yahoo Finance API"""
    timeseries_result = [
        {
            "meta": {"type": ["annualTotalRevenue"]},
            "annualTotalRevenue": [
                {"asOfDate": "2023-09-30", "reportedValue": {"raw": {"parsedValue": 383285000000}}},
                {"asOfDate": "2024-09-30", "reportedValue": {"raw": {"parsedValue": 391035000000}}},
            ],
        },
        {
            "meta": {"type": ["annualNetIncome"]},
            "annualNetIncome": [
                {"asOfDate": "2023-09-30", "reportedValue": {"raw": {"parsedValue": 96995000000}}},
                {"asOfDate": "2024-09-30", "reportedValue": {"raw": {"parsedValue": 93736000000}}},
            ],
        },
    ]

    result = _parse_timeseries_data(timeseries_result)

    assert "TotalRevenue" in result
    assert "NetIncome" in result
    assert result["TotalRevenue"]["2023-09-30"] == 383285000000
    assert result["TotalRevenue"]["2024-09-30"] == 391035000000
    assert result["NetIncome"]["2023-09-30"] == 96995000000
    assert result["NetIncome"]["2024-09-30"] == 93736000000


def test_parse_timeseries_data_removes_frequency_prefix():
    """Test that frequency prefixes are removed from metric names"""
    timeseries_result = [
        {
            "meta": {"type": ["quarterlyTotalRevenue"]},
            "quarterlyTotalRevenue": [{"asOfDate": "2024-Q2", "reportedValue": {"raw": {"parsedValue": 100000000}}}],
        }
    ]

    result = _parse_timeseries_data(timeseries_result)
    assert "TotalRevenue" in result  # Prefix "quarterly" removed
    assert "quarterlyTotalRevenue" not in result


def test_parse_timeseries_data_handles_null_datapoints():
    """Test that null datapoints are skipped"""
    timeseries_result = [
        {
            "meta": {"type": ["annualTotalRevenue"]},
            "annualTotalRevenue": [
                {"asOfDate": "2023-09-30", "reportedValue": {"raw": {"parsedValue": 383285000000}}},
                None,  # Null datapoint
                {"asOfDate": "2024-09-30", "reportedValue": {"raw": {"parsedValue": 391035000000}}},
            ],
        }
    ]

    result = _parse_timeseries_data(timeseries_result)
    assert len(result["TotalRevenue"]) == 2  # Only 2 valid datapoints


def test_parse_timeseries_data_nested_raw_value():
    """Test extraction of nested reportedValue.raw.parsedValue"""
    timeseries_result = [
        {
            "meta": {"type": ["annualTotalRevenue"]},
            "annualTotalRevenue": [
                {
                    "asOfDate": "2024-09-30",
                    "reportedValue": {"raw": {"source": "391.035E9", "parsedValue": 391035000000}, "fmt": "391.04B"},
                }
            ],
        }
    ]

    result = _parse_timeseries_data(timeseries_result)
    assert result["TotalRevenue"]["2024-09-30"] == 391035000000


def test_parse_timeseries_data_backward_compatible_raw():
    """Test backward compatibility with old non-nested raw format"""
    timeseries_result = [
        {
            "meta": {"type": ["annualTotalRevenue"]},
            "annualTotalRevenue": [{"asOfDate": "2024-09-30", "reportedValue": {"raw": 391035000000}}],  # Old format
        }
    ]

    result = _parse_timeseries_data(timeseries_result)
    assert result["TotalRevenue"]["2024-09-30"] == 391035000000


async def test_get_financial_statement_income_annual(mock_finance_client, sample_timeseries_response, bypass_cache):
    """Test getting annual income statement"""
    mock_finance_client.get_fundamentals_timeseries.return_value = sample_timeseries_response

    result = await get_financial_statement(mock_finance_client, "AAPL", StatementType.INCOME_STATEMENT, Frequency.ANNUAL)

    assert result.symbol == "AAPL"
    assert result.statement_type == StatementType.INCOME_STATEMENT
    assert result.frequency == Frequency.ANNUAL
    assert "TotalRevenue" in result.statement
    assert "NetIncome" in result.statement
    assert result.statement["TotalRevenue"]["2024-09-30"] == 391035000000
    assert result.statement["NetIncome"]["2024-09-30"] == 93736000000


async def test_get_financial_statement_balance_quarterly(mock_finance_client, bypass_cache):
    """Test getting quarterly balance sheet"""
    mock_response = {
        "timeseries": {
            "result": [
                {
                    "meta": {"type": ["quarterlyTotalAssets"]},
                    "quarterlyTotalAssets": [
                        {"asOfDate": "2024-Q2", "reportedValue": {"raw": {"parsedValue": 350000000000}}},
                        {"asOfDate": "2024-Q3", "reportedValue": {"raw": {"parsedValue": 360000000000}}},
                    ],
                }
            ]
        }
    }
    mock_finance_client.get_fundamentals_timeseries.return_value = mock_response

    result = await get_financial_statement(mock_finance_client, "TSLA", StatementType.BALANCE_SHEET, Frequency.QUARTERLY)

    assert result.symbol == "TSLA"
    assert result.statement_type == StatementType.BALANCE_SHEET
    assert result.frequency == Frequency.QUARTERLY
    assert "TotalAssets" in result.statement


async def test_get_financial_statement_cashflow_annual(mock_finance_client, bypass_cache):
    """Test getting annual cash flow statement"""
    mock_response = {
        "timeseries": {
            "result": [
                {
                    "meta": {"type": ["annualOperatingCashFlow"]},
                    "annualOperatingCashFlow": [
                        {"asOfDate": "2023-12-31", "reportedValue": {"raw": {"parsedValue": 77434000000}}},
                        {"asOfDate": "2024-12-31", "reportedValue": {"raw": {"parsedValue": 80000000000}}},
                    ],
                }
            ]
        }
    }
    mock_finance_client.get_fundamentals_timeseries.return_value = mock_response

    result = await get_financial_statement(mock_finance_client, "GOOG", StatementType.CASH_FLOW, Frequency.ANNUAL)

    assert result.symbol == "GOOG"
    assert result.statement_type == StatementType.CASH_FLOW
    assert result.frequency == Frequency.ANNUAL
    assert "OperatingCashFlow" in result.statement


async def test_get_financial_statement_empty_data(mock_finance_client, bypass_cache):
    """Test error handling when no data is returned"""
    mock_response = {"timeseries": {"result": []}}
    mock_finance_client.get_fundamentals_timeseries.return_value = mock_response

    with pytest.raises(HTTPException) as exc_info:
        await get_financial_statement(mock_finance_client, "INVALID", StatementType.INCOME_STATEMENT, Frequency.ANNUAL)

    assert exc_info.value.status_code == 404
    assert "No income data found" in exc_info.value.detail


async def test_get_financial_statement_api_error(mock_finance_client, bypass_cache):
    """Test error handling when API call fails"""
    mock_finance_client.get_fundamentals_timeseries.side_effect = Exception("Yahoo Finance API error")

    with pytest.raises(HTTPException) as exc_info:
        await get_financial_statement(mock_finance_client, "ERROR", StatementType.INCOME_STATEMENT, Frequency.ANNUAL)

    assert exc_info.value.status_code == 500
    assert "Failed to fetch financial statement" in exc_info.value.detail


async def test_get_financial_statement_calls_api_with_correct_params(mock_finance_client, bypass_cache):
    """Test that API is called with correct parameters"""
    mock_response = {
        "timeseries": {
            "result": [
                {
                    "meta": {"type": ["annualTotalRevenue"]},
                    "annualTotalRevenue": [{"asOfDate": "2024-09-30", "reportedValue": {"raw": {"parsedValue": 100}}}],
                }
            ]
        }
    }
    mock_finance_client.get_fundamentals_timeseries.return_value = mock_response

    await get_financial_statement(mock_finance_client, "MSFT", StatementType.INCOME_STATEMENT, Frequency.ANNUAL)

    # Verify API was called
    mock_finance_client.get_fundamentals_timeseries.assert_called_once()
    call_args = mock_finance_client.get_fundamentals_timeseries.call_args

    assert call_args.kwargs["symbol"] == "MSFT"
    assert isinstance(call_args.kwargs["period1"], int)  # Timestamp
    assert isinstance(call_args.kwargs["period2"], int)  # Timestamp
    assert isinstance(call_args.kwargs["types"], list)  # Field list
    assert len(call_args.kwargs["types"]) > 0  # At least some fields requested
