from fastapi import APIRouter, Query, Security
from fastapi.security import APIKeyHeader

from src.models.analysis import AnalysisData, AnalysisType
from src.services.analysis.get_analysis import get_analysis_data
from src.utils.dependencies import FinanceClient

router = APIRouter()


@router.get(
    path="/analysis/{symbol}",
    summary="Get analysis data for a stock",
    description="Returns analyst analysis data for a stock symbol.",
    response_model=AnalysisData,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": AnalysisData, "description": "Successfully retrieved analysis data"},
        404: {
            "description": "Symbol not found or no analysis data available",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
    },
)
async def analysis(
    symbol: str,
    finance_client: FinanceClient,
    analysis_type: AnalysisType = Query(..., description="The type of analysis data to retrieve."),
):
    return await get_analysis_data(finance_client, symbol, analysis_type)
