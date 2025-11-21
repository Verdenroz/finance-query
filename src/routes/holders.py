from fastapi import APIRouter, Path, Security
from fastapi.security import APIKeyHeader

from src.models import ValidationErrorResponse
from src.models.holders import (
    HolderType,
    InsiderPurchasesResponse,
    InsiderRosterResponse,
    InsiderTransactionsResponse,
    InstitutionalHoldersResponse,
    MajorHoldersResponse,
    MutualFundHoldersResponse,
)
from src.services.holders.get_holders import get_holders_data
from src.utils.dependencies import FinanceClient

router = APIRouter()


@router.get(
    path="/holders/{symbol}/major",
    summary="Get major holders breakdown",
    description="Returns major holders breakdown including insider and institutional ownership percentages.",
    response_model=MajorHoldersResponse,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved major holders breakdown",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "NVDA",
                        "breakdown": {
                            "breakdown_data": {
                                "insidersPercentHeld": 0.04321,
                                "institutionsPercentHeld": 0.69011,
                                "institutionsFloatPercentHeld": 0.72127,
                                "institutionsCount": 6848,
                            }
                        },
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found or no data available",
            "content": {"application/json": {"example": {"detail": "No major data found for INVALID"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_major_holders(finance_client: FinanceClient, symbol: str = Path(..., description="Stock ticker symbol", pattern="^[A-Za-z]{1,10}$")):
    """
    Get major holders breakdown for a stock symbol.

    Returns ownership percentages for insiders and institutions.
    """
    data = await get_holders_data(finance_client, symbol.upper(), HolderType.MAJOR)
    return MajorHoldersResponse(symbol=symbol.upper(), breakdown=data.major_breakdown)


@router.get(
    path="/holders/{symbol}/institutional",
    summary="Get institutional holders",
    description="Returns list of institutional holders with share counts and values.",
    response_model=InstitutionalHoldersResponse,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved institutional holders",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "NVDA",
                        "holders": [
                            {
                                "holder": "Vanguard Group Inc",
                                "shares": 2223533800,
                                "date_reported": "2025-09-29T20:00:00",
                                "percent_out": 0.0915,
                                "value": 431154318774,
                            }
                        ],
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found or no data available",
            "content": {"application/json": {"example": {"detail": "No institutional data found for INVALID"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_institutional_holders(finance_client: FinanceClient, symbol: str = Path(..., description="Stock ticker symbol", pattern="^[A-Za-z]{1,10}$")):
    """
    Get institutional holders for a stock symbol.

    Returns top institutional shareholders with their holdings.
    """
    data = await get_holders_data(finance_client, symbol.upper(), HolderType.INSTITUTIONAL)
    return InstitutionalHoldersResponse(symbol=symbol.upper(), holders=data.institutional_holders)


@router.get(
    path="/holders/{symbol}/mutualfund",
    summary="Get mutual fund holders",
    description="Returns list of mutual fund holders with share counts and values.",
    response_model=MutualFundHoldersResponse,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved mutual fund holders",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "MSFT",
                        "holders": [
                            {
                                "holder": "VANGUARD INDEX FUNDS-Vanguard Total Stock Market Index Fund",
                                "shares": 239000816,
                                "date_reported": "2025-06-29T20:00:00",
                                "percent_out": 0.0322,
                                "value": 121670532200,
                            }
                        ],
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found or no data available",
            "content": {"application/json": {"example": {"detail": "No mutualfund data found for INVALID"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_mutualfund_holders(finance_client: FinanceClient, symbol: str = Path(..., description="Stock ticker symbol", pattern="^[A-Za-z]{1,10}$")):
    """
    Get mutual fund holders for a stock symbol.

    Returns top mutual fund shareholders with their holdings.
    """
    data = await get_holders_data(finance_client, symbol.upper(), HolderType.MUTUALFUND)
    return MutualFundHoldersResponse(symbol=symbol.upper(), holders=data.mutualfund_holders)


@router.get(
    path="/holders/{symbol}/insider-transactions",
    summary="Get insider transactions",
    description="Returns list of recent insider buy/sell transactions.",
    response_model=InsiderTransactionsResponse,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved insider transactions",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "AAPL",
                        "transactions": [
                            {
                                "start_date": "2025-10-15T20:00:00",
                                "insider": "COOK TIMOTHY D",
                                "position": "Chief Executive Officer",
                                "transaction": "Sale at price 254.83 - 257.57 per share.",
                                "shares": 129963,
                                "value": 33375723,
                                "ownership": "D",
                            }
                        ],
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found or no data available",
            "content": {"application/json": {"example": {"detail": "No insider_transactions data found for INVALID"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_insider_transactions(finance_client: FinanceClient, symbol: str = Path(..., description="Stock ticker symbol", pattern="^[A-Za-z]{1,10}$")):
    """
    Get insider transactions for a stock symbol.

    Returns recent buy/sell transactions by company insiders.
    """
    data = await get_holders_data(finance_client, symbol.upper(), HolderType.INSIDER_TRANSACTIONS)
    return InsiderTransactionsResponse(symbol=symbol.upper(), transactions=data.insider_transactions)


@router.get(
    path="/holders/{symbol}/insider-purchases",
    summary="Get insider purchases summary",
    description="Returns summary of insider purchase activity over different time periods.",
    response_model=InsiderPurchasesResponse,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved insider purchases summary",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "TSLA",
                        "summary": {
                            "period": "6m",
                            "purchases_shares": 100000,
                            "purchases_transactions": 5,
                            "sales_shares": 50000,
                            "sales_transactions": 3,
                            "net_shares": 50000,
                            "net_transactions": 2,
                        },
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found or no data available",
            "content": {"application/json": {"example": {"detail": "No insider_purchases data found for INVALID"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_insider_purchases(finance_client: FinanceClient, symbol: str = Path(..., description="Stock ticker symbol", pattern="^[A-Za-z]{1,10}$")):
    """
    Get insider purchases summary for a stock symbol.

    Returns aggregated purchase/sale activity by insiders.
    """
    data = await get_holders_data(finance_client, symbol.upper(), HolderType.INSIDER_PURCHASES)
    return InsiderPurchasesResponse(symbol=symbol.upper(), summary=data.insider_purchases)


@router.get(
    path="/holders/{symbol}/insider-roster",
    summary="Get insider roster",
    description="Returns list of company insiders with their positions and holdings.",
    response_model=InsiderRosterResponse,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved insider roster",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "GOOGL",
                        "roster": [
                            {
                                "name": "PICHAI SUNDAR",
                                "position": "Chief Executive Officer",
                                "most_recent_transaction": "Sale",
                                "latest_transaction_date": "2025-10-01T20:00:00",
                                "shares_owned_directly": 500000,
                                "shares_owned_indirectly": 100000,
                            }
                        ],
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found or no data available",
            "content": {"application/json": {"example": {"detail": "No insider_roster data found for INVALID"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def get_insider_roster(finance_client: FinanceClient, symbol: str = Path(..., description="Stock ticker symbol", pattern="^[A-Za-z]{1,10}$")):
    """
    Get insider roster for a stock symbol.

    Returns list of company insiders with their current positions and shareholdings.
    """
    data = await get_holders_data(finance_client, symbol.upper(), HolderType.INSIDER_ROSTER)
    return InsiderRosterResponse(symbol=symbol.upper(), roster=data.insider_roster)
