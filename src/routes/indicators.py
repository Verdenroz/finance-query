from fastapi import APIRouter, Security, Query, HTTPException, Response
from fastapi.security import APIKeyHeader
from typing_extensions import Optional

from src.schemas.analysis import Indicator, Analysis
from src.schemas.time_series import Interval
from src.services.indicators import (get_sma, get_ema, get_wma, get_vwma, get_rsi, get_srsi, get_stoch, get_cci,
                                     get_macd, get_adx, get_aroon, get_bbands, get_obv, get_super_trend, get_ichimoku)
from src.services.indicators.get_summary_analysis import get_summary_analysis

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


@router.get("/indicators",
            summary="Returns technical indicators for a stock",
            response_model=Analysis,
            description="Get requested technical indicators for a stock.",
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
async def get_technical_indicators(
        response: Response,
        function: Indicator = Query(..., description="The technical indicator to get."),
        symbol: str = Query(..., description="The symbol of the stock to get technical indicators for."),
        interval: Optional[Interval] = Query(
            Interval.DAILY,
            description="The interval to get historical data for. Available values: 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo.")
        ,
        period: Optional[int] = Query(None, description="The look-back period for the technical indicators."),
        stoch_period: Optional[int] = Query(None, description="The stochastic look-back period for STOCH and SRSI."),
        signal_period: Optional[int] = Query(None, description="The signal period for MACD, STOCH, and SRSI."),
        smooth: Optional[int] = Query(None, description="The smoothing period for STOCH and SRSI."),
        fast_period: Optional[int] = Query(None, description="The fast period for MACD."),
        slow_period: Optional[int] = Query(None, description="The slow period for MACD."),
        std_dev: Optional[int] = Query(None, description="The standard deviation for Bollinger Bands."),
        sma_periods: Optional[int] = Query(None, description="The look-back period for the SMA in OBV."),
        multiplier: Optional[int] = Query(None, description="The multiplier for SuperTrend."),
        tenkan_period: Optional[int] = Query(None, description="The look-back period for the Tenkan line in Ichimoku."),
        kijun_period: Optional[int] = Query(None, description="The look-back period for the Kijun line in Ichimoku."),
        senkou_period: Optional[int] = Query(None, description="The look-back period for the Senkou span in Ichimoku."),
):
    response.headers["Access-Control-Allow-Origin"] = "*"
    params = {
        "symbol": symbol,
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
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Internal Server Error: {str(e)}")


@router.get("/analysis",
            summary="Returns technical indicators for a stock",
            description="Get requested technical indicators for a stock."
                        "per minute.",
            dependencies=[Security(APIKeyHeader(name="x-api-key", auto_error=False))])
async def get_technical_analysis(
        response: Response,
        symbol: str = Query(..., description="The symbol of the stock to get technical indicators for."),
        interval: Interval = Query(Interval.DAILY, description="The interval to get historical data for."),
):
    if not symbol:
        raise HTTPException(status_code=400, detail="Symbol parameter is required")

    response.headers["Access-Control-Allow-Origin"] = "*"
    return await get_summary_analysis(symbol, interval)
