from fastapi import APIRouter, Security, Query, HTTPException
from fastapi.security import APIKeyHeader
from typing_extensions import Optional

from src.models import Analysis, Indicator, Interval, ValidationErrorResponse, SummaryAnalysis, TimeRange
from src.services.indicators import (
    get_sma, get_ema, get_wma, get_vwma, get_rsi, get_srsi, get_stoch, get_cci, get_macd, get_adx, get_aroon,
    get_bbands, get_obv, get_super_trend, get_ichimoku, get_summary_analysis
)

router = APIRouter()

IndicatorFunctions = {
    Indicator.SMA: get_sma,
    Indicator.EMA: get_ema,
    Indicator.WMA: get_wma,
    Indicator.VWMA: get_vwma,
    Indicator.RSI: get_rsi,
    Indicator.SRSI: get_srsi,
    Indicator.STOCH: get_stoch,
    Indicator.CCI: get_cci,
    Indicator.MACD: get_macd,
    Indicator.ADX: get_adx,
    Indicator.AROON: get_aroon,
    Indicator.BBANDS: get_bbands,
    Indicator.OBV: get_obv,
    Indicator.SUPER_TREND: get_super_trend,
    Indicator.ICHIMOKU: get_ichimoku
}


@router.get(
    path="/indicators",
    summary="Get technical indicators for a stock",
    description="Returns the history of the requested technical indicator for a stock.",
    response_model=Analysis,
    tags=["Technical Indicators"],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": Analysis,
            "description": "The technical indicator data for the stock."
        },
        400: {
            "description": "Invalid parameter for the technical indicator.",
            "content": {
                "application/json": {
                    "examples": {
                        "invalid_parameter": {
                            "summary": "Invalid parameter",
                            "value": {
                                "detail": "Invalid parameter: {parameter} for the {function} function."
                            }
                        },
                        "invalid_range_interval": {
                            "summary": "Invalid range and interval combination",
                            "value": {
                                "detail": "If interval is 1m, range must be 1d, 5d, or 1mo."
                            }
                        }
                    }
                }
            }
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error when function or interval has invalid value.",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {
                            "function": [
                                "Field required",
                                "Input should be 'SMA', 'EMA', 'WMA', 'VWMA', 'RSI', 'SRSI', 'STOCH', 'CCI', 'OBV', "
                                "'BBANDS', 'AROON', 'ADX', 'MACD', 'SUPERTREND' or 'ICHIMOKU'"
                            ],
                            "symbol": ["Field required"],
                            "interval": [
                                "Input should be '1m', '5m', '15m', '30m', '1h', '1d', '1wk', '1mo' or '3mo'"
                            ],
                            "period": [
                                "Input should be a valid integer, unable to parse string as an integer"
                            ],
                            "stoch_period": [
                                "Input should be a valid integer, unable to parse string as an integer"
                            ],
                            "signal_period": [
                                "Input should be a valid integer, unable to parse string as an integer"
                            ],
                            "smooth": [
                                "Input should be a valid integer, unable to parse string as an integer"
                            ],
                            "fast_period": [
                                "Input should be a valid integer, unable to parse string as an integer"
                            ],
                            "slow_period": [
                                "Input should be a valid integer, unable to parse string as an integer"
                            ],
                            "std_dev": [
                                "Input should be a valid integer, unable to parse string as an integer"
                            ],
                            "sma_periods": [
                                "Input should be a valid integer, unable to parse string as an integer"
                            ],
                            "multiplier": [
                                "Input should be a valid integer, unable to parse string as an integer"
                            ],
                            "tenkan_period": [
                                "Input should be a valid integer, unable to parse string as an integer"
                            ],
                            "kijun_period": [
                                "Input should be a valid integer, unable to parse string as an integer"
                            ],
                            "senkou_period": [
                                "Input should be a valid integer, unable to parse string as an integer"
                            ]
                        }
                    }
                }
            }
        },
        500: {
            "description": "Failed to retrieve technical indicators.",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Failed to retrieve technical indicators"
                    }
                }
            }
        }
    }
)
async def get_technical_indicators(
        function: Indicator = Query(
            ...,
            description="The technical indicator to get.",
        ),
        symbol: str = Query(
            ...,
            description="The symbol of the stock to get technical indicators for.",
        ),
        time_range: Optional[TimeRange] = Query(
            TimeRange.TWO_YEARS,
            description="The time range for the historical data.",
            alias="range"
        ),
        interval: Optional[Interval] = Query(
            default=Interval.DAILY,
            description="The interval between data points. Available values: 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo."
        ),
        period: Optional[int] = Query(
            None,
            description="The look-back period for the technical indicators.",
            alias="lookBackPeriod"
        ),
        stoch_period: Optional[int] = Query(
            None,
            description="The stochastic look-back period for STOCH and SRSI.",
            alias="stochPeriod"
        ),
        signal_period: Optional[int] = Query(
            None,
            description="The signal period for MACD, STOCH, and SRSI.",
            alias="signalPeriod"
        ),
        smooth: Optional[int] = Query(
            None,
            description="The smoothing period for STOCH and SRSI.",
            alias="smooth"
        ),
        fast_period: Optional[int] = Query(
            None,
            description="The fast period for MACD.",
            alias="fastPeriod"
        ),
        slow_period: Optional[int] = Query(
            None,
            description="The slow period for MACD.",
            alias="slowPeriod"
        ),
        std_dev: Optional[int] = Query(
            None,
            description="The standard deviation for Bollinger Bands.",
            alias="stdDev"
        ),
        sma_periods: Optional[int] = Query(
            None,
            description="The look-back period for the SMA in OBV.",
            alias="smaPeriods"
        ),
        multiplier: Optional[int] = Query(
            None,
            description="The multiplier for SuperTrend.",
            alias="multiplier"
        ),
        tenkan_period: Optional[int] = Query(
            None,
            description="The look-back period for the Tenkan line in Ichimoku.",
            alias="tenkanPeriod"
        ),
        kijun_period: Optional[int] = Query(
            None,
            description="The look-back period for the Kijun line in Ichimoku.",
            alias="kijunPeriod"
        ),
        senkou_period: Optional[int] = Query(
            None,
            description="The look-back period for the Senkou span in Ichimoku.",
            alias="senkouPeriod"
        ),
):
    params = {
        "symbol": symbol,
        "time_range": time_range,
        "interval": interval,
        "period": period,
        "stoch_period": stoch_period,
        "signal_period": signal_period,
        "smooth": smooth,
        "fast_period": fast_period,
        "slow_period": slow_period,
        "std_dev": std_dev,
        "sma_periods": sma_periods,
        "multiplier": multiplier,
        "tenkan_period": tenkan_period,
        "kijun_period": kijun_period,
        "senkou_period": senkou_period,
    }
    # Filter out None values
    params = {k: v for k, v in params.items() if v is not None}

    try:
        return await IndicatorFunctions[function](**params)

    except TypeError as e:
        param_name = str(e).split("'")[1]
        raise HTTPException(status_code=400,
                            detail=f"Invalid parameter: {param_name} for the {function.name} function.")
    except HTTPException as e:
        # Re-raise HTTPException from get_historical_quotes
        raise e
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to retrieve technical indicators: {str(e)}")


@router.get(
    path="/analysis",
    summary="Get an aggregated summary of technical indicators for a stock",
    description="Returns all available technical indicators for a stock with popular default periods and settings.",
    tags=["Technical Indicators"],
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "model": SummaryAnalysis,
            "description": "The technical analysis summary for the stock."
        },
        404: {
            "description": "Symbol not found",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Symbol not found"
                    }
                }
            }
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {
                            "symbol": [
                                "Field required"
                            ],
                            "interval": [
                                "Input should be '1m', '5m', '15m', '30m', '1h', '1d', '1wk', '1mo' or '3mo'"
                            ]
                        }
                    }
                }
            }
        },
        500: {
            "description": "Failed to retrieve technical analysis",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Failed to retrieve technical analysis"
                    }
                }
            }
        }
    }
)
async def get_technical_analysis(
        symbol: str = Query(..., description="The symbol of the stock to get technical indicators for."),
        interval: Interval = Query(Interval.DAILY, description="The interval to get historical data for."),
):
    try:
        return await get_summary_analysis(symbol, interval)
    except HTTPException as e:
        # Re-raise HTTPException from get_summary_analysis
        raise e
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to retrieve technical analysis: {str(e)}")
