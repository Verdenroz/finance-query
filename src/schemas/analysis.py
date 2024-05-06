from datetime import date
from decimal import Decimal
from enum import Enum

from pydantic import Field, BaseModel
from typing_extensions import Dict, Union, Optional


class Indicator(Enum):
    SMA = 'sma'
    EMA = 'ema'
    WMA = 'wma'
    VWAP = 'vwap'
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


class SMAData(BaseModel):
    name: str = "SMA"
    value: Decimal = Field(
        ..., example=30.00, description="Simple Moving Average value", serialization_alias="SMA"
    )


class EMAData(BaseModel):
    name: str = "EMA"
    value: Decimal = Field(
        ..., example=30.00, description="Exponential Moving Average value", serialization_alias="EMA"
    )


class WMAData(BaseModel):
    name: str = "WMA"
    value: Decimal = Field(
        ..., example=30.00, description="Weighted Moving Average value", serialization_alias="WMA"
    )


class VWAPData(BaseModel):
    name: str = "VWAP"
    value: Decimal = Field(
        ..., example=30.00, description="Volume Weighted Average Price value", serialization_alias="VWAP"
    )


class RSIData(BaseModel):
    name: str = "RSI"
    value: Decimal = Field(
        ..., example=30.00, description="Relative Strength Index value", serialization_alias="RSI"
    )


class SRSIData(BaseModel):
    name: str = "SRSI"
    k: Decimal = Field(
        ..., example=30.00, description="Stochastic RSI value", serialization_alias="%K"
    )
    d: Decimal = Field(
        ..., example=30.00, description="Stochastic RSI Signal value", serialization_alias="%D"
    )


class STOCHData(BaseModel):
    name: str = "STOCH"
    k: Decimal = Field(
        ..., example=30.00, description="Stochastic Oscillator %K value", serialization_alias="%K"
    )
    d: Decimal = Field(
        ..., example=30.00, description="Stochastic Oscillator %D value", serialization_alias="%D"
    )


class CCIData(BaseModel):
    name: str = "CCI"
    value: Decimal = Field(
        ..., example=30.00, description="Commodity Channel Index value", serialization_alias="CCI"
    )


class MACDData(BaseModel):
    name: str = "MACD"
    value: Decimal = Field(
        ..., example=30.00, description="Moving Average Convergence Divergence value", serialization_alias="MACD"
    )
    signal: Decimal = Field(
        ..., example=30.00, description="MACD Signal value", serialization_alias="Signal"
    )


class ADXData(BaseModel):
    name: str = "ADX"
    value: Decimal = Field(
        ..., example=30.00, description="Average Directional Index value", serialization_alias="ADX"
    )


class AROONData(BaseModel):
    name: str = "AROON"
    aroon_up: Decimal = Field(
        ..., example=30.00, description="Aroon Up value", serialization_alias="Aroon Up"
    )
    aroon_down: Decimal = Field(
        ..., example=30.00, description="Aroon Down value", serialization_alias="Aroon Down"
    )


class BBANDSData(BaseModel):
    name: str = "BBANDS"
    upper_band: Decimal = Field(
        ..., example=30.00, description="Upper Bollinger Band value", serialization_alias="Upper Band"
    )
    lower_band: Decimal = Field(
        ..., example=30.00, description="Lower Bollinger Band value", serialization_alias="Lower Band"
    )


class OBVData(BaseModel):
    name: str = "OBV"
    value: Decimal = Field(..., example=30.00, description="On Balance Volume value", serialization_alias="OBV")


class SuperTrendData(BaseModel):
    name: str = "Super Trend"
    value: Decimal = Field(..., example=30.00, description="Super Trend value", serialization_alias="Super Trend")
    trend: str = Field(..., example="UP", description="Trend direction", serialization_alias="Trend")


class IchimokuData(BaseModel):
    name: str = "Ichimoku"
    tenkan_sen: Optional[Decimal] = Field(
        None, example=30.00, description="Tenkan-sen value", serialization_alias="Conversion Line"
    )
    kijun_sen: Optional[Decimal] = Field(
        None, example=30.00, description="Kijun-sen value", serialization_alias="Base Line"
    )
    senkou_span_a: Optional[Decimal] = Field(
        None, example=30.00, description="Senkou Span A value", serialization_alias="Leading Span A"
    )
    senkou_span_b: Optional[Decimal] = Field(
        None, example=30.00, description="Senkou Span B value", serialization_alias="Leading Span B"
    )
    chikou_span: Optional[Decimal] = Field(
        None, example=30.00, description="Chikou Span value", serialization_alias="Lagging Span"
    )


IndicatorData = Union[
    SMAData, EMAData, WMAData, VWAPData, RSIData, SRSIData, MACDData, STOCHData, ADXData,
    CCIData, AROONData, BBANDSData, OBVData, SuperTrendData, IchimokuData
]


class Analysis(BaseModel):
    indicators: Dict[date, IndicatorData] = Field(
        ...,
        serialization_alias="Technical Analysis",
        example={
            "2021-07-09": {
                "name: str": "SMA",
                "value": 30.00,
            }
        },
        description="Dates with indicators for the stock")
