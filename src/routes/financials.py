from fastapi import APIRouter, Path, Query, Security
from fastapi.security import APIKeyHeader

from src.models import ValidationErrorResponse
from src.models.financials import FinancialStatement, Frequency, StatementType
from src.services.financials.get_financials import get_financial_statement
from src.utils.dependencies import FinanceClient

router = APIRouter()


@router.get(
    path="/financials/{symbol}",
    summary="Get financial statements for a stock",
    description="Returns the requested financial statement (income statement, balance sheet, or cash flow) for a stock symbol.",
    response_model=FinancialStatement,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "Successfully retrieved financial statement",
            "content": {
                "application/json": {
                    "example": {
                        "symbol": "AAPL",
                        "statement_type": "income",
                        "frequency": "annual",
                        "data": [
                            {
                                "asOfDate": "2024-09-30",
                                "periodType": "12M",
                                "currencyCode": "USD",
                                "TotalRevenue": 391035000000,
                                "CostOfRevenue": 210352000000,
                                "GrossProfit": 180683000000,
                                "OperatingExpense": 61021000000,
                                "OperatingIncome": 119662000000,
                                "NetIncome": 100913000000,
                            },
                            {
                                "asOfDate": "2023-09-30",
                                "periodType": "12M",
                                "currencyCode": "USD",
                                "TotalRevenue": 383285000000,
                                "CostOfRevenue": 214137000000,
                                "GrossProfit": 169148000000,
                                "OperatingExpense": 55013000000,
                                "OperatingIncome": 114301000000,
                                "NetIncome": 96995000000,
                            },
                        ],
                    }
                }
            },
        },
        404: {
            "description": "Symbol not found or no financial data available",
            "content": {"application/json": {"example": {"detail": "No income data found for INVALID"}}},
        },
        422: {"model": ValidationErrorResponse, "description": "Validation error"},
    },
)
async def financials(
    finance_client: FinanceClient,
    symbol: str = Path(..., description="Stock ticker symbol", pattern="^[A-Za-z]{1,10}$"),
    statement: StatementType = Query(..., description="The type of financial statement to retrieve."),
    frequency: Frequency = Query(Frequency.ANNUAL, description="The frequency of the financial statement."),
):
    """
    Get financial statements for a stock symbol.

    Returns income statement, balance sheet, or cash flow statement data with historical periods.
    """
    return await get_financial_statement(finance_client, symbol.upper(), statement, frequency)
