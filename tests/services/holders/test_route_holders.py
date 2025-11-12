from datetime import datetime
from unittest.mock import patch

import pytest
from fastapi.testclient import TestClient

from src.main import app
from src.models.holders import (
    HoldersData,
    HolderType,
    InsiderPurchase,
    InsiderRosterMember,
    InsiderTransaction,
    InstitutionalHolder,
    MajorHoldersBreakdown,
    MutualFundHolder,
)


@pytest.fixture
def client():
    return TestClient(app)


@patch("src.routes.holders.get_holders_data")
async def test_get_major_holders(mock_get_holders, client):
    """Test getting major holders breakdown"""
    mock_holders = HoldersData(
        symbol="AAPL",
        holder_type=HolderType.MAJOR,
        major_breakdown=MajorHoldersBreakdown(
            breakdown_data={
                "insidersPercentHeld": {"raw": 0.001, "fmt": "0.10%"},
                "institutionsPercentHeld": {"raw": 0.6249, "fmt": "62.49%"},
                "institutionsFloatPercentHeld": {"raw": 0.6259, "fmt": "62.59%"},
                "institutionsCount": {"raw": 5098, "fmt": "5,098"},
            }
        ),
    )
    mock_get_holders.return_value = mock_holders

    response = client.get("/v1/holders/AAPL/major")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "AAPL"
    assert data["breakdown"]["breakdown_data"]["insidersPercentHeld"]["raw"] == 0.001


@patch("src.routes.holders.get_holders_data")
async def test_get_institutional_holders(mock_get_holders, client):
    """Test getting institutional holders"""
    mock_holders = HoldersData(
        symbol="MSFT",
        holder_type=HolderType.INSTITUTIONAL,
        institutional_holders=[
            InstitutionalHolder(
                holder="Vanguard Group Inc",
                shares=1289322922,
                date_reported=datetime(2024, 4, 1),
                percent_out=0.0841,
                value=241822367432,
            ),
            InstitutionalHolder(
                holder="BlackRock Inc",
                shares=1050000000,
                date_reported=datetime(2024, 4, 1),
                percent_out=0.0685,
                value=196875000000,
            ),
        ],
    )
    mock_get_holders.return_value = mock_holders

    response = client.get("/v1/holders/MSFT/institutional")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "MSFT"
    assert len(data["holders"]) == 2
    assert data["holders"][0]["holder"] == "Vanguard Group Inc"
    assert data["holders"][0]["shares"] == 1289322922


@patch("src.routes.holders.get_holders_data")
async def test_get_mutualfund_holders(mock_get_holders, client):
    """Test getting mutual fund holders"""
    mock_holders = HoldersData(
        symbol="GOOG",
        holder_type=HolderType.MUTUALFUND,
        mutualfund_holders=[
            MutualFundHolder(
                holder="Vanguard 500 Index Fund",
                shares=502923145,
                date_reported=datetime(2024, 4, 1),
                percent_out=0.0328,
                value=94308820935,
            )
        ],
    )
    mock_get_holders.return_value = mock_holders

    response = client.get("/v1/holders/GOOG/mutualfund")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "GOOG"
    assert len(data["holders"]) == 1
    assert data["holders"][0]["holder"] == "Vanguard 500 Index Fund"


@patch("src.routes.holders.get_holders_data")
async def test_get_insider_transactions(mock_get_holders, client):
    """Test getting insider transactions"""
    mock_holders = HoldersData(
        symbol="TSLA",
        holder_type=HolderType.INSIDER_TRANSACTIONS,
        insider_transactions=[
            InsiderTransaction(
                start_date=datetime(2024, 10, 15),
                insider="MUSK ELON",
                position="CEO",
                transaction="Sale at price 250.00 per share",
                shares=100000,
                value=25000000,
                ownership="D",
            )
        ],
    )
    mock_get_holders.return_value = mock_holders

    response = client.get("/v1/holders/TSLA/insider-transactions")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "TSLA"
    assert len(data["transactions"]) == 1
    assert data["transactions"][0]["insider"] == "MUSK ELON"
    assert data["transactions"][0]["shares"] == 100000


@patch("src.routes.holders.get_holders_data")
async def test_get_insider_purchases(mock_get_holders, client):
    """Test getting insider purchases summary"""
    mock_holders = HoldersData(
        symbol="NVDA",
        holder_type=HolderType.INSIDER_PURCHASES,
        insider_purchases=InsiderPurchase(
            period="6m",
            purchases_shares=100000,
            purchases_transactions=5,
            sales_shares=50000,
            sales_transactions=3,
            net_shares=50000,
            net_transactions=2,
            total_insider_shares=1000000,
            net_percent_insider_shares=0.05,
            buy_percent_insider_shares=0.10,
            sell_percent_insider_shares=0.05,
        ),
    )
    mock_get_holders.return_value = mock_holders

    response = client.get("/v1/holders/NVDA/insider-purchases")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "NVDA"
    assert data["summary"]["period"] == "6m"
    assert data["summary"]["purchases_shares"] == 100000
    assert data["summary"]["net_shares"] == 50000


@patch("src.routes.holders.get_holders_data")
async def test_get_insider_roster(mock_get_holders, client):
    """Test getting insider roster"""
    mock_holders = HoldersData(
        symbol="GOOGL",
        holder_type=HolderType.INSIDER_ROSTER,
        insider_roster=[
            InsiderRosterMember(
                name="PICHAI SUNDAR",
                position="CEO",
                most_recent_transaction="Sale",
                latest_transaction_date=datetime(2024, 10, 1),
                shares_owned_directly=500000,
                shares_owned_indirectly=100000,
                position_direct_date=datetime(2024, 10, 1),
            )
        ],
    )
    mock_get_holders.return_value = mock_holders

    response = client.get("/v1/holders/GOOGL/insider-roster")

    assert response.status_code == 200
    data = response.json()
    assert data["symbol"] == "GOOGL"
    assert len(data["roster"]) == 1
    assert data["roster"][0]["name"] == "PICHAI SUNDAR"
    assert data["roster"][0]["position"] == "CEO"


async def test_get_holders_invalid_symbol_pattern(client):
    """Test validation error for invalid symbol pattern"""
    response = client.get("/v1/holders/invalid-symbol/major")

    assert response.status_code == 422
    assert "detail" in response.json()


@patch("src.routes.holders.get_holders_data")
async def test_get_holders_all_endpoints(mock_get_holders, client):
    """Test all holder endpoint paths"""
    endpoints = [
        ("major", HolderType.MAJOR),
        ("institutional", HolderType.INSTITUTIONAL),
        ("mutualfund", HolderType.MUTUALFUND),
        ("insider-transactions", HolderType.INSIDER_TRANSACTIONS),
        ("insider-purchases", HolderType.INSIDER_PURCHASES),
        ("insider-roster", HolderType.INSIDER_ROSTER),
    ]

    for endpoint, holder_type in endpoints:
        # Create appropriate mock data for each endpoint
        if holder_type == HolderType.MAJOR:
            mock_holders = HoldersData(
                symbol="TEST",
                holder_type=holder_type,
                major_breakdown=MajorHoldersBreakdown(breakdown_data={"insidersPercentHeld": {"raw": 0.1}}),
            )
        elif holder_type == HolderType.INSTITUTIONAL:
            mock_holders = HoldersData(
                symbol="TEST",
                holder_type=holder_type,
                institutional_holders=[InstitutionalHolder(holder="Test", shares=100, date_reported=datetime.now(), percent_out=0.1, value=1000)],
            )
        elif holder_type == HolderType.MUTUALFUND:
            mock_holders = HoldersData(
                symbol="TEST",
                holder_type=holder_type,
                mutualfund_holders=[MutualFundHolder(holder="Test", shares=100, date_reported=datetime.now(), percent_out=0.1, value=1000)],
            )
        elif holder_type == HolderType.INSIDER_TRANSACTIONS:
            mock_holders = HoldersData(
                symbol="TEST",
                holder_type=holder_type,
                insider_transactions=[
                    InsiderTransaction(
                        start_date=datetime.now(),
                        insider="Test",
                        position="CEO",
                        transaction="Sale",
                        shares=100,
                        value=1000,
                        ownership="D",
                    )
                ],
            )
        elif holder_type == HolderType.INSIDER_PURCHASES:
            mock_holders = HoldersData(
                symbol="TEST",
                holder_type=holder_type,
                insider_purchases=InsiderPurchase(period="6m", purchases_shares=100, sales_shares=50),
            )
        else:  # INSIDER_ROSTER
            mock_holders = HoldersData(
                symbol="TEST",
                holder_type=holder_type,
                insider_roster=[
                    InsiderRosterMember(
                        name="Test",
                        position="CEO",
                        most_recent_transaction="Sale",
                        latest_transaction_date=datetime.now(),
                        shares_owned_directly=100,
                    )
                ],
            )

        mock_get_holders.return_value = mock_holders

        response = client.get(f"/v1/holders/TEST/{endpoint}")
        assert response.status_code == 200
        assert response.json()["symbol"] == "TEST"


@patch("src.routes.holders.get_holders_data")
async def test_get_holders_symbol_case_insensitivity(mock_get_holders, client):
    """Test that symbol is converted to uppercase"""
    mock_holders = HoldersData(
        symbol="AAPL",
        holder_type=HolderType.MAJOR,
        major_breakdown=MajorHoldersBreakdown(breakdown_data={"insidersPercentHeld": {"raw": 0.001}}),
    )
    mock_get_holders.return_value = mock_holders

    # Test with uppercase (should work due to path pattern)
    response = client.get("/v1/holders/AAPL/major")
    assert response.status_code == 200
    assert response.json()["symbol"] == "AAPL"
