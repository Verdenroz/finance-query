from fastapi import APIRouter, Security, Query
from fastapi.security import APIKeyHeader
from typing_extensions import Union, Optional

from src.schemas.analysis import (
    Indicator, SMAData, EMAData, WMAData, VWAPData, RSIData, SRSIData, MACDData, STOCHData,
    ADXData, CCIData, AROONData, BBANDSData, OBVData, SuperTrendData, IchimokuData, Analysis
)
from src.services import get_indicators
from src.services.indicators import get_sma, get_ema, get_wma

router = APIRouter()

IndicatorResponse = Union[
    SMAData, EMAData, WMAData, VWAPData, RSIData, SRSIData, MACDData, STOCHData, ADXData,
    CCIData, AROONData, BBANDSData, OBVData, SuperTrendData, IchimokuData
]


# @router.get("/v1/indicators",
#             summary="Returns technical indicators for a stock",
#             description="Get requested technical indicators for a stock. Invalid API keys are limited to 5 requests "
#                         "per minute.",
#             dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
# async def get_technical_analysis(
#         symbol: str = Query(..., description="The symbol of the stock to get technical indicators for."),
#         function: Indicator = Query(..., description="The technical indicator to get."),
#         period: int = Query(..., description="The look-back period for the technical indicators.")
# ):
#     analysis = await get_indicators(symbol, function, period)
#     return analysis.dict(exclude_none=True)


@router.get("/sma",
            summary="Returns the Simple Moving Average for a stock",
            response_model=Analysis,
            description="Get the Simple Moving Average for a stock. Invalid API keys are limited to 5 requests per "
                        "minute.",
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))]
            )
async def get_sma_analysis(
        symbol: str = Query(..., description="The symbol of the stock to get the Simple Moving Average for."),
        period: Optional[int] = Query(14, description="The look-back period for the Simple Moving Average.")
):
    sma = await get_sma(symbol, period)
    return sma.dict(exclude_none=True)


@router.get("/ema",
            summary="Returns the Exponential Moving Average for a stock",
            response_model=Analysis,
            description="Get the Exponential Moving Average for a stock. Invalid API keys are limited to 5 requests per"
                        "minute.",
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))]
            )
async def get_ema_analysis(
        symbol: str = Query(..., description="The symbol of the stock to get the Exponential Moving Average for."),
        period: Optional[int] = Query(14, description="The look-back period for the Exponential Moving Average.")
):
    ema = await get_ema(symbol, period)
    return ema.dict(exclude_none=True)



@router.get("/wma",
            summary="Returns the Weighted Moving Average for a stock",
            response_model=Analysis,
            description="Get the Weighted Moving Average for a stock. Invalid API keys are limited to 5 requests per"
                        "minute.",
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))]
            )
async def get_wma_analysis(
        symbol: str = Query(..., description="The symbol of the stock to get the Weighted Moving Average for."),
        period: Optional[int] = Query(14, description="The look-back period for the Weighted Moving Average.")
):
    wma = await get_wma(symbol, period)
    return wma.dict(exclude_none=True)
