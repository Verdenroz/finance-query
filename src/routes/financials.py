from fastapi import APIRouter, Query, Security
from fastapi.security import APIKeyHeader

from src.models.financials import FinancialStatement, Frequency, StatementType
from src.services.financials.get_financials import get_financial_statement

router = APIRouter()


@router.get(
    path="/financials/{symbol}",
    summary="Get financial statements for a stock",
    description="Returns the requested financial statement for a stock symbol.",
    response_model=FinancialStatement,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": FinancialStatement, "description": "Successfully retrieved financial statement"},
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
    },
)
async def financials(
    symbol: str,
    statement: StatementType = Query(..., description="The type of financial statement to retrieve."),
    frequency: Frequency = Query(Frequency.ANNUAL, description="The frequency of the financial statement."),
):
    return await get_financial_statement(symbol, statement, frequency)
