from datetime import datetime
from unittest.mock import AsyncMock

import pytest
from fastapi import HTTPException

from src.models.holders import HolderType
from src.services.holders.get_holders import (
    _parse_insider_purchases,
    _parse_insider_roster,
    _parse_insider_transactions,
    _parse_institutional_holders,
    _parse_major_breakdown,
    _parse_mutualfund_holders,
    get_holders_data,
)


@pytest.fixture
def mock_finance_client():
    """Create a mock YahooFinanceClient"""
    return AsyncMock()


@pytest.fixture
def sample_quote_summary_response():
    """Sample Yahoo Finance quoteSummary API response"""
    return {
        "quoteSummary": {
            "result": [
                {
                    "majorHoldersBreakdown": {
                        "insidersPercentHeld": {"raw": 0.001, "fmt": "0.10%"},
                        "institutionsPercentHeld": {"raw": 0.6249, "fmt": "62.49%"},
                        "institutionsFloatPercentHeld": {"raw": 0.6259, "fmt": "62.59%"},
                        "institutionsCount": {"raw": 5098, "fmt": "5,098"},
                    },
                    "institutionOwnership": {
                        "ownershipList": [
                            {
                                "organization": "Vanguard Group Inc",
                                "reportDate": {"raw": 1711929600, "fmt": "2024-04-01"},
                                "pctHeld": {"raw": 0.0841, "fmt": "8.41%"},
                                "position": {"raw": 1289322922, "fmt": "1.29B"},
                                "value": {"raw": 241822367432, "fmt": "241.82B"},
                            },
                            {
                                "organization": "BlackRock Inc",
                                "reportDate": {"raw": 1711929600, "fmt": "2024-04-01"},
                                "pctHeld": {"raw": 0.0664, "fmt": "6.64%"},
                                "position": {"raw": 1017985988, "fmt": "1.02B"},
                                "value": {"raw": 190880143240, "fmt": "190.88B"},
                            },
                        ]
                    },
                    "fundOwnership": {
                        "ownershipList": [
                            {
                                "organization": "Vanguard 500 Index Fund",
                                "reportDate": {"raw": 1711929600, "fmt": "2024-04-01"},
                                "pctHeld": {"raw": 0.0328, "fmt": "3.28%"},
                                "position": {"raw": 502923145, "fmt": "502.92M"},
                                "value": {"raw": 94308820935, "fmt": "94.31B"},
                            }
                        ]
                    },
                    "insiderTransactions": {
                        "transactions": [
                            {
                                "filerName": "COOK TIMOTHY D",
                                "transactionText": "Sale at price 175.00 - 180.00 per share.",
                                "moneyText": "",
                                "ownership": "D",
                                "startDate": {"raw": 1711929600, "fmt": "2024-04-01"},
                                "value": {"raw": 88563900, "fmt": "88.56M"},
                                "filerRelation": "Chief Executive Officer",
                                "shares": {"raw": 511000, "fmt": "511k"},
                                "filerUrl": "",
                                "maxShares": {"raw": 511000, "fmt": "511k"},
                            }
                        ]
                    },
                    "netSharePurchaseActivity": {
                        "period": "6m",
                        "buyInfoCount": {"raw": 15, "fmt": "15"},
                        "buyInfoShares": {"raw": 125000, "fmt": "125k"},
                        "sellInfoCount": {"raw": 45, "fmt": "45"},
                        "sellInfoShares": {"raw": 2500000, "fmt": "2.5M"},
                        "netInfoCount": {"raw": -30, "fmt": "-30"},
                        "netInfoShares": {"raw": -2375000, "fmt": "-2.38M"},
                        "totalInsiderShares": {"raw": 125000000, "fmt": "125M"},
                        "buyPercentInsiderShares": {"raw": 0.001, "fmt": "0.10%"},
                        "sellPercentInsiderShares": {"raw": 0.02, "fmt": "2.00%"},
                    },
                    "insiderHolders": {
                        "holders": [
                            {
                                "name": "COOK TIMOTHY D",
                                "relation": "Chief Executive Officer",
                                "url": "",
                                "transactionDescription": "Sale",
                                "latestTransDate": {"raw": 1711929600, "fmt": "2024-04-01"},
                                "positionDirect": {"raw": 3279726, "fmt": "3.28M"},
                                "positionDirectDate": {"raw": 1711929600, "fmt": "2024-04-01"},
                            }
                        ]
                    },
                }
            ],
            "error": None,
        }
    }


# Test parser functions
def test_parse_major_breakdown():
    """Test parsing major holders breakdown"""
    data = {
        "insidersPercentHeld": {"raw": 0.001, "fmt": "0.10%"},
        "institutionsPercentHeld": {"raw": 0.6249, "fmt": "62.49%"},
        "institutionsFloatPercentHeld": {"raw": 0.6259, "fmt": "62.59%"},
        "institutionsCount": {"raw": 5098, "fmt": "5,098"},
    }

    result = _parse_major_breakdown(data)

    # The parser stores the entire Yahoo Finance structure (with raw and fmt)
    assert result.breakdown_data["insidersPercentHeld"] == {"raw": 0.001, "fmt": "0.10%"}
    assert result.breakdown_data["institutionsPercentHeld"] == {"raw": 0.6249, "fmt": "62.49%"}
    assert result.breakdown_data["institutionsFloatPercentHeld"] == {"raw": 0.6259, "fmt": "62.59%"}
    assert result.breakdown_data["institutionsCount"] == {"raw": 5098, "fmt": "5,098"}


def test_parse_major_breakdown_empty():
    """Test parsing empty major holders data raises 404"""
    with pytest.raises(HTTPException) as exc_info:
        _parse_major_breakdown({})

    assert exc_info.value.status_code == 404


def test_parse_institutional_holders():
    """Test parsing institutional holders"""
    data = [
        {
            "organization": "Vanguard Group Inc",
            "reportDate": {"raw": 1711929600, "fmt": "2024-04-01"},
            "pctHeld": {"raw": 0.0841, "fmt": "8.41%"},
            "position": {"raw": 1289322922, "fmt": "1.29B"},
            "value": {"raw": 241822367432, "fmt": "241.82B"},
        },
        {
            "organization": "BlackRock Inc",
            "reportDate": {"raw": 1711929600, "fmt": "2024-04-01"},
            "pctHeld": {"raw": 0.0664, "fmt": "6.64%"},
            "position": {"raw": 1017985988, "fmt": "1.02B"},
            "value": {"raw": 190880143240, "fmt": "190.88B"},
        },
    ]

    result = _parse_institutional_holders(data)

    assert len(result) == 2
    assert result[0].holder == "Vanguard Group Inc"
    assert result[0].shares == 1289322922
    assert result[0].percent_out == 0.0841
    assert result[0].value == 241822367432
    assert isinstance(result[0].date_reported, datetime)


def test_parse_institutional_holders_empty():
    """Test parsing empty institutional holders returns empty list"""
    result = _parse_institutional_holders([])
    assert result == []


def test_parse_mutualfund_holders():
    """Test parsing mutual fund holders"""
    data = [
        {
            "organization": "Vanguard 500 Index Fund",
            "reportDate": {"raw": 1711929600, "fmt": "2024-04-01"},
            "pctHeld": {"raw": 0.0328, "fmt": "3.28%"},
            "position": {"raw": 502923145, "fmt": "502.92M"},
            "value": {"raw": 94308820935, "fmt": "94.31B"},
        }
    ]

    result = _parse_mutualfund_holders(data)

    assert len(result) == 1
    assert result[0].holder == "Vanguard 500 Index Fund"
    assert result[0].shares == 502923145
    assert result[0].percent_out == 0.0328
    assert result[0].value == 94308820935


def test_parse_insider_transactions():
    """Test parsing insider transactions"""
    data = [
        {
            "filerName": "COOK TIMOTHY D",
            "transactionText": "Sale at price 175.00 - 180.00 per share.",
            "ownership": "D",
            "startDate": {"raw": 1711929600, "fmt": "2024-04-01"},
            "value": {"raw": 88563900, "fmt": "88.56M"},
            "filerRelation": "Chief Executive Officer",
            "shares": {"raw": 511000, "fmt": "511k"},
        }
    ]

    result = _parse_insider_transactions(data)

    assert len(result) == 1
    assert result[0].insider == "COOK TIMOTHY D"
    assert result[0].position == "Chief Executive Officer"
    assert result[0].transaction == "Sale at price 175.00 - 180.00 per share."
    assert result[0].shares == 511000
    assert result[0].value == 88563900
    assert result[0].ownership == "D"
    assert isinstance(result[0].start_date, datetime)


def test_parse_insider_purchases():
    """Test parsing insider purchases summary"""
    data = {
        "period": "6m",
        "buyInfoCount": {"raw": 15, "fmt": "15"},
        "buyInfoShares": {"raw": 125000, "fmt": "125k"},
        "sellInfoCount": {"raw": 45, "fmt": "45"},
        "sellInfoShares": {"raw": 2500000, "fmt": "2.5M"},
        "netInfoCount": {"raw": -30, "fmt": "-30"},
        "netInfoShares": {"raw": -2375000, "fmt": "-2.38M"},
        "totalInsiderShares": {"raw": 125000000, "fmt": "125M"},
        "buyPercentInsiderShares": {"raw": 0.001, "fmt": "0.10%"},
        "sellPercentInsiderShares": {"raw": 0.02, "fmt": "2.00%"},
    }

    result = _parse_insider_purchases(data)

    assert result.period == "6m"
    assert result.purchases_transactions == 15
    assert result.purchases_shares == 125000
    assert result.sales_transactions == 45
    assert result.sales_shares == 2500000
    assert result.net_transactions == -30
    assert result.net_shares == -2375000
    assert result.total_insider_shares == 125000000
    assert result.buy_percent_insider_shares == 0.001
    assert result.sell_percent_insider_shares == 0.02


def test_parse_insider_roster():
    """Test parsing insider roster"""
    data = [
        {
            "name": "COOK TIMOTHY D",
            "relation": "Chief Executive Officer",
            "transactionDescription": "Sale",
            "latestTransDate": {"raw": 1711929600, "fmt": "2024-04-01"},
            "positionDirect": {"raw": 3279726, "fmt": "3.28M"},
            "positionDirectDate": {"raw": 1711929600, "fmt": "2024-04-01"},
        }
    ]

    result = _parse_insider_roster(data)

    assert len(result) == 1
    assert result[0].name == "COOK TIMOTHY D"
    assert result[0].position == "Chief Executive Officer"
    assert result[0].most_recent_transaction == "Sale"
    assert result[0].shares_owned_directly == 3279726
    assert isinstance(result[0].latest_transaction_date, datetime)
    assert isinstance(result[0].position_direct_date, datetime)


async def test_get_holders_data_institutional(mock_finance_client, bypass_cache):
    """Test getting institutional holders"""
    mock_response = {
        "quoteSummary": {
            "result": [
                {
                    "institutionOwnership": {
                        "ownershipList": [
                            {
                                "organization": "Vanguard Group Inc",
                                "reportDate": {"raw": 1711929600},
                                "pctHeld": {"raw": 0.0841},
                                "position": {"raw": 1289322922},
                                "value": {"raw": 241822367432},
                            }
                        ]
                    }
                }
            ]
        }
    }
    mock_finance_client.get_quote_summary.return_value = mock_response

    result = await get_holders_data(mock_finance_client, "AAPL", HolderType.INSTITUTIONAL)

    assert result.symbol == "AAPL"
    assert result.holder_type == HolderType.INSTITUTIONAL
    assert len(result.institutional_holders) == 1
    assert result.institutional_holders[0].holder == "Vanguard Group Inc"


async def test_get_holders_data_major(mock_finance_client, bypass_cache):
    """Test getting major holders breakdown"""
    mock_response = {
        "quoteSummary": {
            "result": [
                {
                    "majorHoldersBreakdown": {
                        "insidersPercentHeld": {"raw": 0.001},
                        "institutionsPercentHeld": {"raw": 0.6249},
                    }
                }
            ]
        }
    }
    mock_finance_client.get_quote_summary.return_value = mock_response

    result = await get_holders_data(mock_finance_client, "MSFT", HolderType.MAJOR)

    assert result.symbol == "MSFT"
    assert result.holder_type == HolderType.MAJOR
    # The breakdown_data stores the full Yahoo Finance structure
    assert result.major_breakdown.breakdown_data["insidersPercentHeld"] == {"raw": 0.001}
    assert result.major_breakdown.breakdown_data["institutionsPercentHeld"] == {"raw": 0.6249}


async def test_get_holders_data_mutualfund(mock_finance_client, bypass_cache):
    """Test getting mutual fund holders"""
    mock_response = {
        "quoteSummary": {
            "result": [
                {
                    "fundOwnership": {
                        "ownershipList": [
                            {
                                "organization": "Vanguard 500 Index Fund",
                                "reportDate": {"raw": 1711929600},
                                "pctHeld": {"raw": 0.0328},
                                "position": {"raw": 502923145},
                                "value": {"raw": 94308820935},
                            }
                        ]
                    }
                }
            ]
        }
    }
    mock_finance_client.get_quote_summary.return_value = mock_response

    result = await get_holders_data(mock_finance_client, "GOOG", HolderType.MUTUALFUND)

    assert result.symbol == "GOOG"
    assert result.holder_type == HolderType.MUTUALFUND
    assert len(result.mutualfund_holders) == 1


async def test_get_holders_data_insider_transactions(mock_finance_client, bypass_cache):
    """Test getting insider transactions"""
    mock_response = {
        "quoteSummary": {
            "result": [
                {
                    "insiderTransactions": {
                        "transactions": [
                            {
                                "filerName": "COOK TIMOTHY D",
                                "transactionText": "Sale",
                                "ownership": "D",
                                "startDate": {"raw": 1711929600},
                                "value": {"raw": 88563900},
                                "filerRelation": "CEO",
                                "shares": {"raw": 511000},
                            }
                        ]
                    }
                }
            ]
        }
    }
    mock_finance_client.get_quote_summary.return_value = mock_response

    result = await get_holders_data(mock_finance_client, "TSLA", HolderType.INSIDER_TRANSACTIONS)

    assert result.symbol == "TSLA"
    assert result.holder_type == HolderType.INSIDER_TRANSACTIONS
    assert len(result.insider_transactions) == 1


async def test_get_holders_data_insider_purchases(mock_finance_client, bypass_cache):
    """Test getting insider purchases summary"""
    mock_response = {
        "quoteSummary": {
            "result": [
                {
                    "netSharePurchaseActivity": {
                        "period": "6m",
                        "buyInfoCount": {"raw": 15},
                        "buyInfoShares": {"raw": 125000},
                        "sellInfoCount": {"raw": 45},
                        "sellInfoShares": {"raw": 2500000},
                    }
                }
            ]
        }
    }
    mock_finance_client.get_quote_summary.return_value = mock_response

    result = await get_holders_data(mock_finance_client, "NVDA", HolderType.INSIDER_PURCHASES)

    assert result.symbol == "NVDA"
    assert result.holder_type == HolderType.INSIDER_PURCHASES
    assert result.insider_purchases.period == "6m"


async def test_get_holders_data_insider_roster(mock_finance_client, bypass_cache):
    """Test getting insider roster"""
    mock_response = {
        "quoteSummary": {
            "result": [
                {
                    "insiderHolders": {
                        "holders": [
                            {
                                "name": "COOK TIMOTHY D",
                                "relation": "CEO",
                                "transactionDescription": "Sale",
                                "latestTransDate": {"raw": 1711929600},
                                "positionDirect": {"raw": 3279726},
                                "positionDirectDate": {"raw": 1711929600},
                            }
                        ]
                    }
                }
            ]
        }
    }
    mock_finance_client.get_quote_summary.return_value = mock_response

    result = await get_holders_data(mock_finance_client, "AMZN", HolderType.INSIDER_ROSTER)

    assert result.symbol == "AMZN"
    assert result.holder_type == HolderType.INSIDER_ROSTER
    assert len(result.insider_roster) == 1


async def test_get_holders_data_empty_result(mock_finance_client, bypass_cache):
    """Test error handling when no data is returned"""
    mock_response = {"quoteSummary": {"result": []}}
    mock_finance_client.get_quote_summary.return_value = mock_response

    with pytest.raises(HTTPException) as exc_info:
        await get_holders_data(mock_finance_client, "INVALID", HolderType.INSTITUTIONAL)

    assert exc_info.value.status_code == 404


async def test_get_holders_data_api_error(mock_finance_client, bypass_cache):
    """Test error handling when API call fails"""
    mock_finance_client.get_quote_summary.side_effect = Exception("Yahoo Finance API error")

    with pytest.raises(HTTPException) as exc_info:
        await get_holders_data(mock_finance_client, "ERROR", HolderType.INSTITUTIONAL)

    assert exc_info.value.status_code == 500
    assert "Failed to fetch holders data" in exc_info.value.detail


async def test_get_holders_data_calls_api_with_correct_params(mock_finance_client, bypass_cache):
    """Test that API is called with correct parameters"""
    mock_response = {
        "quoteSummary": {
            "result": [
                {
                    "institutionOwnership": {
                        "ownershipList": [
                            {
                                "organization": "Test",
                                "reportDate": {"raw": 1711929600},
                                "pctHeld": {"raw": 0.05},
                                "position": {"raw": 1000},
                                "value": {"raw": 100000},
                            }
                        ]
                    }
                }
            ]
        }
    }
    mock_finance_client.get_quote_summary.return_value = mock_response

    await get_holders_data(mock_finance_client, "TEST", HolderType.INSTITUTIONAL)

    # Verify API was called with correct params
    mock_finance_client.get_quote_summary.assert_called_once()
    call_args = mock_finance_client.get_quote_summary.call_args

    assert call_args.kwargs["symbol"] == "TEST"
    assert call_args.kwargs["modules"] == ["institutionOwnership"]
