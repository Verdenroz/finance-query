from datetime import date, datetime
from enum import Enum
from typing import Union

from pydantic import Field, BaseModel, AliasChoices, SerializeAsAny
from typing_extensions import Optional, Annotated


class Indicator(Enum):
    SMA = "SMA"
    EMA = "EMA"
    WMA = "WMA"
    VWMA = "VWMA"
    RSI = "RSI"
    SRSI = "SRSI"
    STOCH = "STOCH"
    CCI = "CCI"
    OBV = "OBV"
    BBANDS = "BBANDS"
    AROON = "AROON"
    ADX = "ADX"
    MACD = "MACD"
    SUPER_TREND = "SUPERTREND"
    ICHIMOKU = "ICHIMOKU"


class IndicatorData(BaseModel):
    """Base class for all technical indicators"""

    def to_dict(self, *args, **kwargs) -> dict:
        return super().model_dump(*args, **kwargs)


class SMAData(IndicatorData):
    value: Optional[float] = Field(
        None, examples=[30.00], description="Simple Moving Average value", serialization_alias="SMA"
    )


class EMAData(IndicatorData):
    value: Optional[float] = Field(
        None, examples=[30.00], description="Exponential Moving Average value", serialization_alias="EMA"
    )


class WMAData(IndicatorData):
    value: Optional[float] = Field(
        None, examples=[30.00], description="Weighted Moving Average value", serialization_alias="WMA"
    )


class VWMAData(IndicatorData):
    value: Optional[float] = Field(
        None, examples=[30.00], description="Volume Weighted Moving Average value", serialization_alias="VWMA"
    )


class RSIData(IndicatorData):
    value: Optional[float] = Field(
        None, examples=[30.00], description="Relative Strength Index value", serialization_alias="RSI"
    )


class SRSIData(IndicatorData):
    k: Optional[float] = Field(None, examples=[30.00], description="Stochastic RSI value", serialization_alias="%K")
    d: Optional[float] = Field(
        None, examples=[30.00], description="Stochastic RSI Signal value", serialization_alias="%D"
    )


class STOCHData(IndicatorData):
    k: Optional[float] = Field(
        None, examples=[30.00], description="Stochastic Oscillator %K value", serialization_alias="%K"
    )
    d: Optional[float] = Field(
        None, examples=[30.00], description="Stochastic Oscillator %D value", serialization_alias="%D"
    )


class CCIData(IndicatorData):
    value: Optional[float] = Field(
        None, examples=[30.00], description="Commodity Channel Index value", serialization_alias="CCI"
    )


class MACDData(IndicatorData):
    value: Optional[float] = Field(
        None, examples=[30.00], description="Moving Average Convergence Divergence value", serialization_alias="MACD"
    )
    signal: Optional[float] = Field(
        None, examples=[30.00], description="MACD Signal value", serialization_alias="Signal"
    )


class ADXData(IndicatorData):
    value: Optional[float] = Field(
        None, examples=[30.00], description="Average Directional Index value", serialization_alias="ADX"
    )


class AROONData(IndicatorData):
    aroon_up: Optional[float] = Field(
        None, examples=[30.00], description="Aroon Up value", serialization_alias="Aroon Up"
    )
    aroon_down: Optional[float] = Field(
        None, examples=[30.00], description="Aroon Down value", serialization_alias="Aroon Down"
    )


class BBANDSData(IndicatorData):
    upper_band: Optional[float] = Field(
        None, examples=[30.00], description="Upper Bollinger Band value", serialization_alias="Upper Band"
    )
    middle_band: Optional[float] = Field(
        None, examples=[30.00], description="Middle Bollinger Band value", serialization_alias="Middle Band"
    )
    lower_band: Optional[float] = Field(
        None, examples=[30.00], description="Lower Bollinger Band value", serialization_alias="Lower Band"
    )


class OBVData(IndicatorData):
    value: Optional[float] = Field(
        None, examples=[30.00], description="On Balance Volume value", serialization_alias="OBV"
    )


class SuperTrendData(IndicatorData):
    value: Optional[float] = Field(
        None, examples=[30.00], description="Super Trend value", serialization_alias="Super Trend"
    )
    trend: str = Field(..., examples=["UP"], description="Trend direction", serialization_alias="Trend")


class IchimokuData(IndicatorData):
    tenkan_sen: Optional[float] = Field(
        None, examples=[30.00], description="Tenkan-sen value", serialization_alias="Conversion Line"
    )
    kijun_sen: Optional[float] = Field(
        None, examples=[30.00], description="Kijun-sen value", serialization_alias="Base Line"
    )
    chikou_span: Optional[float] = Field(
        None, examples=[30.00], description="Chikou Span value", serialization_alias="Lagging Span"
    )
    senkou_span_a: Optional[float] = Field(
        None, examples=[30.00], description="Senkou Span A value", serialization_alias="Leading Span A"
    )
    senkou_span_b: Optional[float] = Field(
        None, examples=[30.00], description="Senkou Span B value", serialization_alias="Leading Span B"
    )


DateType = Annotated[Union[date, datetime, str], Field(description="Date in any format")]


class TechnicalIndicator(BaseModel):
    type: Indicator = Field(default=..., examples=["SMA"], description="The type of technical indicator")
    indicators: dict[DateType, SerializeAsAny[IndicatorData]] = Field(
        default=...,
        serialization_alias="Technical Analysis",
        validation_alias=AliasChoices("Technical Analysis", "indicators"),
        examples=[
            {
                "2021-07-09": {
                    "value": 30.00,
                }
            }
        ],
        description="Dates with indicators for the stock",
    )

    def model_dump(self, *args, **kwargs) -> dict:
        base_dict = super().model_dump(*args, **kwargs)

        # Format the date keys based on their type
        for field_name in ["Technical Analysis", "indicators"]:
            if field_name in base_dict:
                formatted_dict = {}
                for k, v in base_dict[field_name].items():
                    if isinstance(k, datetime):
                        # For datetime, keep full timestamp
                        formatted_dict[k.strftime("%Y-%m-%d %H:%M:%S")] = v
                    elif isinstance(k, date):
                        # For date, use date-only format
                        formatted_dict[k.strftime("%Y-%m-%d")] = v
                    else:
                        # For strings, keep as is
                        formatted_dict[str(k)] = v
                base_dict[field_name] = formatted_dict

        return base_dict
