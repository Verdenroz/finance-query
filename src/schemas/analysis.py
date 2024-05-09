from datetime import date
from decimal import Decimal
from enum import Enum

from pydantic import Field, BaseModel, AliasChoices, SerializeAsAny
from typing_extensions import Dict, Optional


class Indicator(Enum):
    SMA = 'sma'
    EMA = 'ema'
    WMA = 'wma'
    VWMA = 'vwma'
    RSI = 'rsi'
    SRSI = 'srsi'
    STOCH = 'stoch'
    CCI = 'cci'
    OBV = 'obv'
    BBANDS = 'bbands'
    AROON = 'aroon'
    ADX = 'adx'
    MACD = 'macd'
    SUPER_TREND = 'supertrend'
    ICHIMOKU = 'ichimoku'


class IndicatorData(BaseModel):
    name: str


class SMAData(IndicatorData):
    name: str = "SMA"
    value: Decimal = Field(
        ..., example=30.00, description="Simple Moving Average value", serialization_alias="SMA"
    )


class EMAData(IndicatorData):
    name: str = "EMA"
    value: Decimal = Field(
        ..., example=30.00, description="Exponential Moving Average value", serialization_alias="EMA"
    )


class WMAData(IndicatorData):
    name: str = "WMA"
    value: Decimal = Field(
        ..., example=30.00, description="Weighted Moving Average value", serialization_alias="WMA"
    )


class VWMAData(IndicatorData):
    name: str = "VWMA"
    value: Decimal = Field(
        ..., example=30.00, description="Volume Weighted Moving Average value", serialization_alias="VWAP"
    )


class RSIData(IndicatorData):
    name: str = "RSI"
    value: Decimal = Field(
        ..., example=30.00, description="Relative Strength Index value", serialization_alias="RSI"
    )


class SRSIData(IndicatorData):
    name: str = "SRSI"
    k: Decimal = Field(
        ..., example=30.00, description="Stochastic RSI value", serialization_alias="%K"
    )
    d: Decimal = Field(
        ..., example=30.00, description="Stochastic RSI Signal value", serialization_alias="%D"
    )


class STOCHData(IndicatorData):
    name: str = "STOCH"
    k: Decimal = Field(
        ..., example=30.00, description="Stochastic Oscillator %K value", serialization_alias="%K"
    )
    d: Decimal = Field(
        ..., example=30.00, description="Stochastic Oscillator %D value", serialization_alias="%D"
    )


class CCIData(IndicatorData):
    name: str = "CCI"
    value: Decimal = Field(
        ..., example=30.00, description="Commodity Channel Index value", serialization_alias="CCI"
    )


class MACDData(IndicatorData):
    name: str = "MACD"
    value: Decimal = Field(
        ..., example=30.00, description="Moving Average Convergence Divergence value", serialization_alias="MACD"
    )
    signal: Decimal = Field(
        ..., example=30.00, description="MACD Signal value", serialization_alias="Signal"
    )


class ADXData(IndicatorData):
    name: str = "ADX"
    value: Decimal = Field(
        ..., example=30.00, description="Average Directional Index value", serialization_alias="ADX"
    )


class AROONData(IndicatorData):
    name: str = "AROON"
    aroon_up: Decimal = Field(
        ..., example=30.00, description="Aroon Up value", serialization_alias="Aroon Up"
    )
    aroon_down: Decimal = Field(
        ..., example=30.00, description="Aroon Down value", serialization_alias="Aroon Down"
    )


class BBANDSData(IndicatorData):
    name: str = "BBANDS"
    upper_band: Decimal = Field(
        ..., example=30.00, description="Upper Bollinger Band value", serialization_alias="Upper Band"
    )
    lower_band: Decimal = Field(
        ..., example=30.00, description="Lower Bollinger Band value", serialization_alias="Lower Band"
    )


class OBVData(IndicatorData):
    name: str = "OBV"
    value: Decimal = Field(..., example=30.00, description="On Balance Volume value", serialization_alias="OBV")


class SuperTrendData(IndicatorData):
    name: str = "Super Trend"
    value: Decimal = Field(..., example=30.00, description="Super Trend value", serialization_alias="Super Trend")
    trend: str = Field(..., example="UP", description="Trend direction", serialization_alias="Trend")


class IchimokuData(IndicatorData):
    name: str = "Ichimoku"
    tenkan_sen: Optional[Decimal] = Field(
        None, example=30.00, description="Tenkan-sen value", serialization_alias="Conversion Line"
    )
    kijun_sen: Optional[Decimal] = Field(
        None, example=30.00, description="Kijun-sen value", serialization_alias="Base Line"
    )
    chikou_span: Optional[Decimal] = Field(
        None, example=30.00, description="Chikou Span value", serialization_alias="Lagging Span"
    )
    senkou_span_a: Optional[Decimal] = Field(
        None, example=30.00, description="Senkou Span A value", serialization_alias="Leading Span A"
    )
    senkou_span_b: Optional[Decimal] = Field(
        None, example=30.00, description="Senkou Span B value", serialization_alias="Leading Span B"
    )


class Analysis(BaseModel):
    indicators: Dict[date, SerializeAsAny[IndicatorData]] = Field(
        ...,
        serialization_alias="Technical Analysis",
        validation_alias=AliasChoices("Technical Analysis", "indicators"),
        example={
            "2021-07-09": {
                "name: str": "SMA",
                "value": 30.00,
            }
        },
        description="Dates with indicators for the stock")


class SummaryAnalysis(BaseModel):
    symbol: str = Field(..., example="AAPL", description="Stock symbol")

    sma_10: Optional[Decimal] = Field(None, description="10-day Simple Moving Average", serialization_alias="SMA(10)")

    sma_20: Optional[Decimal] = Field(None, description="20-day Simple Moving Average", serialization_alias="SMA(20)")

    sma_50: Optional[Decimal] = Field(None, description="50-day Simple Moving Average", serialization_alias="SMA(50)")
    sma_100: Optional[Decimal] = Field(None, description="100-day Simple Moving Average",
                                       serialization_alias="SMA(100)")

    sma_200: Optional[Decimal] = Field(None, description="200-day Simple Moving Average",
                                       serialization_alias="SMA(200)")

    ema_10: Optional[float] = Field(None, description="10-day Exponential Moving Average",
                                    serialization_alias="EMA(10)")

    ema_20: Optional[float] = Field(None, description="20-day Exponential Moving Average",
                                    serialization_alias="EMA(20)")

    ema_50: Optional[float] = Field(None, description="50-day Exponential Moving Average",
                                    serialization_alias="EMA(50)")

    ema_100: Optional[float] = Field(None, description="100-day Exponential Moving Average",
                                     serialization_alias="EMA(100)")

    ema_200: Optional[float] = Field(None, description="200-day Exponential Moving Average",
                                     serialization_alias="EMA(200)")

    wma_10: Optional[float] = Field(None, description="10-day Weighted Moving Average", serialization_alias="WMA(10)")

    wma_20: Optional[float] = Field(None, description="20-day Weighted Moving Average", serialization_alias="WMA(20)")

    wma_50: Optional[float] = Field(None, description="50-day Weighted Moving Average", serialization_alias="WMA(50)")
    wma_100: Optional[float] = Field(None, description="100-day Weighted Moving Average",
                                     serialization_alias="WMA(100)")

    wma_200: Optional[float] = Field(None, description="200-day Weighted Moving Average",
                                     serialization_alias="WMA(200)")

    vwma_20: Optional[float] = Field(None, description="20-day Volume Weighted Moving Average",
                                     serialization_alias="VWAP(20)")

    rsi_14: Optional[float] = Field(None, description="14-day Relative Strength Index", serialization_alias="RSI(14)")

    srsi_14: Optional[float] = Field(None, description="14-day Stochastic RSI", serialization_alias="SRSI(14)")

    cci_20: Optional[float] = Field(None, description="20-day Commodity Channel Index", serialization_alias="CCI(20)")

    adx_14: Optional[float] = Field(None, description="14-day Average Directional Index",
                                    serialization_alias="ADX(14)")

    macd_12_26: Optional[float] = Field(None, description="Moving Average Convergence Divergence",
                                        serialization_alias="MACD(12,26)")

    stoch_3_3_14_14: Optional[float] = Field(None, description="Stochastic Oscillator",
                                             serialization_alias="STOCH(3,3,14,14)")

    obv: Optional[float] = Field(None, description="On Balance Volume", serialization_alias="OBV")

    aroon_25: Optional[dict] = Field(None, description="25-day Aroon Indicator", serialization_alias="Aroon(25)")

    bbands_20_2: Optional[dict] = Field(None, description="Bollinger Bands", serialization_alias="BBANDS(20,2)")

    supertrend: Optional[dict] = Field(None, description="Super Trend", serialization_alias="Super Trend")

    ichimoku: Optional[dict] = Field(None, description="Ichimoku Cloud", serialization_alias="Ichimoku Cloud")
