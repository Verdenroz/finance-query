from typing import Optional

from fastapi import APIRouter, HTTPException, Query, Security
from fastapi.security import APIKeyHeader

from src.models import Indicator, Interval, TechnicalIndicator, TimeRange, ValidationErrorResponse
from src.services.indicators import (
    get_adx,
    get_aroon,
    get_bbands,
    get_cci,
    get_ema,
    get_ichimoku,
    get_macd,
    get_obv,
    get_rsi,
    get_sma,
    get_srsi,
    get_stoch,
    get_super_trend,
    get_technical_indicators,
    get_vwma,
    get_wma,
)
from src.utils.dependencies import FinanceClient

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
    Indicator.ICHIMOKU: get_ichimoku,
}


@router.get(
    path="/indicator",
    summary="Get technical indicator data over time for a stock",
    description="Returns the history of the requested technical indicator",
    response_model=TechnicalIndicator,
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {"model": TechnicalIndicator, "description": "The technical indicator data for the stock."},
        400: {
            "description": "Invalid parameter for the technical indicator.",
            "content": {
                "application/json": {
                    "examples": {
                        "invalid_parameter": {
                            "summary": "Invalid parameter",
                            "value": {"detail": "Invalid parameter: {parameter} for the {function} function."},
                        },
                        "invalid_range_interval": {
                            "summary": "Invalid range and interval combination",
                            "value": {"detail": "If interval is 1m, range must be 1d, 5d, or 1mo."},
                        },
                    }
                }
            },
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
                                "'BBANDS', 'AROON', 'ADX', 'MACD', 'SUPERTREND' or 'ICHIMOKU'",
                            ],
                            "symbol": ["Field required"],
                            "interval": ["Input should be '1m', '5m', '15m', '30m', '1h', '1d', '1wk', or '1mo'"],
                            "period": ["Input should be a valid integer, unable to parse string as an integer"],
                            "stoch_period": ["Input should be a valid integer, unable to parse string as an integer"],
                            "signal_period": ["Input should be a valid integer, unable to parse string as an integer"],
                            "smooth": ["Input should be a valid integer, unable to parse string as an integer"],
                            "fast_period": ["Input should be a valid integer, unable to parse string as an integer"],
                            "slow_period": ["Input should be a valid integer, unable to parse string as an integer"],
                            "std_dev": ["Input should be a valid integer, unable to parse string as an integer"],
                            "sma_periods": ["Input should be a valid integer, unable to parse string as an integer"],
                            "multiplier": ["Input should be a valid integer, unable to parse string as an integer"],
                            "tenkan_period": ["Input should be a valid integer, unable to parse string as an integer"],
                            "kijun_period": ["Input should be a valid integer, unable to parse string as an integer"],
                            "senkou_period": ["Input should be a valid integer, unable to parse string as an integer"],
                        },
                    }
                }
            },
        },
    },
)
async def technical_indicator(
    finance_client: FinanceClient,
    function: Indicator = Query(
        ...,
        description="The technical indicator to get.",
    ),
    symbol: str = Query(
        ...,
        description="The symbol of the stock to get technical indicators for.",
    ),
    time_range: Optional[TimeRange] = Query(TimeRange.TWO_YEARS, description="The time range for the historical data.", alias="range"),
    interval: Optional[Interval] = Query(
        default=Interval.DAILY,
        description="The interval between data points. Available values: 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo.",
    ),
    epoch: Optional[bool] = Query(False, description="Whether to return the timestamps as epoch time."),
    period: Optional[int] = Query(None, description="The look-back period for the technical indicators.", alias="lookBackPeriod"),
    stoch_period: Optional[int] = Query(None, description="The stochastic look-back period for STOCH and SRSI.", alias="stochPeriod"),
    signal_period: Optional[int] = Query(None, description="The signal period for MACD, STOCH, and SRSI.", alias="signalPeriod"),
    smooth: Optional[int] = Query(None, description="The smoothing period for STOCH and SRSI.", alias="smooth"),
    fast_period: Optional[int] = Query(None, description="The fast period for MACD.", alias="fastPeriod"),
    slow_period: Optional[int] = Query(None, description="The slow period for MACD.", alias="slowPeriod"),
    std_dev: Optional[int] = Query(None, description="The standard deviation for Bollinger Bands.", alias="stdDev"),
    sma_periods: Optional[int] = Query(None, description="The look-back period for the SMA in OBV.", alias="smaPeriods"),
    multiplier: Optional[int] = Query(None, description="The multiplier for SuperTrend.", alias="multiplier"),
    tenkan_period: Optional[int] = Query(None, description="The look-back period for the Tenkan line in Ichimoku.", alias="tenkanPeriod"),
    kijun_period: Optional[int] = Query(None, description="The look-back period for the Kijun line in Ichimoku.", alias="kijunPeriod"),
    senkou_period: Optional[int] = Query(None, description="The look-back period for the Senkou span in Ichimoku.", alias="senkouPeriod"),
):
    params = {
        "finance_client": finance_client,
        "symbol": symbol,
        "time_range": time_range,
        "interval": interval,
        "period": period,
        "epoch": epoch,
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

    except TypeError as te:
        param_name = str(te).split("'")[1]
        raise HTTPException(status_code=400, detail=f"Invalid parameter: {param_name} for the {function.name} function.") from te
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to retrieve technical indicators: {str(e)}") from e


@router.get(
    path="/indicators",
    summary="Get latest technical indicators for a symbol",
    description="Returns only the latest values for the requested technical indicators",
    dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))],
    responses={
        200: {
            "description": "The technical analysis summary for the stock.",
            "content": {
                "application/json": {
                    "example": {
                        "SMA(10)": {"SMA": 129.03},
                        "SMA(20)": {"SMA": 131.08},
                        "SMA(50)": {"SMA": 134.95},
                        "SMA(100)": {"SMA": 135.54},
                        "SMA(200)": {"SMA": 124.78},
                        "EMA(10)": {"EMA": 131.93},
                        "EMA(20)": {"EMA": 131.64},
                        "EMA(50)": {"EMA": 133.51},
                        "EMA(100)": {"EMA": 131.7},
                        "EMA(200)": {"EMA": 120.76},
                        "WMA(10)": {"WMA": 125.72},
                        "WMA(20)": {"WMA": 132.3},
                        "WMA(50)": {"WMA": 136.83},
                        "WMA(100)": {"WMA": 135.32},
                        "WMA(200)": {"WMA": 118.59},
                        "VWMA(20)": {"VWMA": 128.17},
                        "RSI(14)": {"RSI": 56.56},
                        "SRSI(3,3,14,14)": {"%K": 92.79, "%D": 81.77},
                        "STOCH %K(14,3,3)": {"%K": 81.25, "%D": 67.41},
                        "CCI(20)": {"CCI": 63.36},
                        "BBANDS(20,2)": {"Upper Band": 149.81, "Middle Band": 131.08, "Lower Band": 112.35},
                        "Aroon(25)": {"Aroon Up": 40.0, "Aroon Down": 64.0},
                        "ADX(14)": {"ADX": 14.43},
                        "MACD(12,26)": {"MACD": -0.53, "Signal": -2.1},
                        "Super Trend": {"Super Trend": 140.25, "Trend": "DOWN"},
                        "Ichimoku Cloud": {
                            "Conversion Line": 127.97,
                            "Base Line": 130.99,
                            "Lagging Span": 138.85,
                            "Leading Span A": 141.74,
                            "Leading Span B": 140.0,
                        },
                    }
                }
            },
        },
        400: {
            "description": "Invalid parameter for the technical indicator.",
            "content": {"application/jsoin": {"example": {"detail": "Invalid parameter: {parameter} for the {function} function."}}},
        },
        404: {
            "description": "Symbol not found",
            "content": {"application/json": {"example": {"detail": "Symbol not found"}}},
        },
        422: {
            "model": ValidationErrorResponse,
            "description": "Validation error of query parameters",
            "content": {
                "application/json": {
                    "example": {
                        "detail": "Invalid request",
                        "errors": {
                            "symbol": ["Field required"],
                            "interval": ["Input should be '1m', '5m', '15m', '30m', '1h', '1d', '1wk', or '1mo'"],
                        },
                    }
                }
            },
        },
    },
)
async def technical_indicators(
    finance_client: FinanceClient,
    symbol: str = Query(..., description="The symbol of the stock to get technical indicators for."),
    interval: Interval = Query(Interval.DAILY, description="The interval to get historical data for."),
    functions: Optional[str] = Query(None, description="Comma-separated list of technical indicators to calculate."),
):
    try:
        indicator_list = [Indicator[ind.strip()] for ind in functions.split(",")] if functions else None
        return await get_technical_indicators(finance_client, symbol, interval, indicator_list)
    except KeyError as ke:
        raise HTTPException(status_code=400, detail=f"Invalid indicator: {str(ke)}") from ke
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to retrieve technical analysis: {str(e)}") from e
