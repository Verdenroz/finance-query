from datetime import date, datetime
from enum import Enum
from typing import Union

from pydantic import Field, BaseModel, AliasChoices, SerializeAsAny
from typing_extensions import Optional, Annotated


class Indicator(Enum):
    SMA = 'SMA'
    EMA = 'EMA'
    WMA = 'WMA'
    VWMA = 'VWMA'
    RSI = 'RSI'
    SRSI = 'SRSI'
    STOCH = 'STOCH'
    CCI = 'CCI'
    OBV = 'OBV'
    BBANDS = 'BBANDS'
    AROON = 'AROON'
    ADX = 'ADX'
    MACD = 'MACD'
    SUPER_TREND = 'SUPERTREND'
    ICHIMOKU = 'ICHIMOKU'


class IndicatorData(BaseModel):
    pass


class SMAData(IndicatorData):
    value: float = Field(
        ..., examples=[30.00], description="Simple Moving Average value", serialization_alias="SMA"
    )

    def to_dict(self):
        return {
            "type": self.type,
            "SMA": str(self.value)
        }


class EMAData(IndicatorData):
    value: float = Field(
        ..., examples=[30.00], description="Exponential Moving Average value", serialization_alias="EMA"
    )

    def to_dict(self):
        return {
            "type": self.type,
            "EMA": str(self.value)
        }


class WMAData(IndicatorData):
    value: float = Field(
        ..., examples=[30.00], description="Weighted Moving Average value", serialization_alias="WMA"
    )

    def to_dict(self):
        return {
            "type": self.type,
            "WMA": str(self.value)
        }


class VWMAData(IndicatorData):
    value: float = Field(
        ..., examples=[30.00], description="Volume Weighted Moving Average value", serialization_alias="VWMA"
    )

    def to_dict(self):
        return {
            "type": self.type,
            "VWMA": str(self.value)
        }


class RSIData(IndicatorData):
    value: float = Field(
        ..., examples=[30.00], description="Relative Strength Index value", serialization_alias="RSI"
    )

    def to_dict(self):
        return {
            "type": self.type,
            "RSI": str(self.value)
        }


class SRSIData(IndicatorData):
    k: float = Field(
        ..., examples=[30.00], description="Stochastic RSI value", serialization_alias="%K"
    )
    d: float = Field(
        ..., examples=[30.00], description="Stochastic RSI Signal value", serialization_alias="%D"
    )

    def to_dict(self):
        return {
            "type": self.type,
            "%K": str(self.k),
            "%D": str(self.d)
        }


class STOCHData(IndicatorData):
    k: float = Field(
        ..., examples=[30.00], description="Stochastic Oscillator %K value", serialization_alias="%K"
    )
    d: float = Field(
        ..., examples=[30.00], description="Stochastic Oscillator %D value", serialization_alias="%D"
    )

    def to_dict(self):
        return {
            "type": self.type,
            "%K": str(self.k),
            "%D": str(self.d)
        }


class CCIData(IndicatorData):
    value: float = Field(
        ..., examples=[30.00], description="Commodity Channel Index value", serialization_alias="CCI"
    )

    def to_dict(self):
        return {
            "type": self.type,
            "CCI": str(self.value)
        }


class MACDData(IndicatorData):
    value: float = Field(
        ..., examples=[30.00], description="Moving Average Convergence Divergence value", serialization_alias="MACD"
    )
    signal: float = Field(
        ..., examples=[30.00], description="MACD Signal value", serialization_alias="Signal"
    )

    def to_dict(self):
        return {
            "type": self.type,
            "MACD": str(self.value),
            "Signal": str(self.signal)
        }


class ADXData(IndicatorData):
    value: float = Field(
        ..., examples=[30.00], description="Average Directional Index value", serialization_alias="ADX"
    )

    def to_dict(self):
        return {
            "type": self.type,
            "ADX": str(self.value)
        }


class AROONData(IndicatorData):
    aroon_up: float = Field(
        ..., examples=[30.00], description="Aroon Up value", serialization_alias="Aroon Up"
    )
    aroon_down: float = Field(
        ..., examples=[30.00], description="Aroon Down value", serialization_alias="Aroon Down"
    )

    def to_dict(self):
        return {
            "type": self.type,
            "Aroon Up": str(self.aroon_up),
            "Aroon Down": str(self.aroon_down)
        }


class BBANDSData(IndicatorData):
    upper_band: float = Field(
        ..., examples=[30.00], description="Upper Bollinger Band value", serialization_alias="Upper Band"
    )
    middle_band: float = Field(
        ..., examples=[30.00], description="Middle Bollinger Band value", serialization_alias="Middle Band"
    )
    lower_band: float = Field(
        ..., examples=[30.00], description="Lower Bollinger Band value", serialization_alias="Lower Band"
    )

    def to_dict(self):
        return {
            "type": self.type,
            "Upper Band": str(self.upper_band),
            "Lower Band": str(self.lower_band)
        }


class OBVData(IndicatorData):
    value: float = Field(..., examples=[30.00], description="On Balance Volume value", serialization_alias="OBV")

    def to_dict(self):
        return {
            "type": self.type,
            "OBV": str(self.value)
        }


class SuperTrendData(IndicatorData):
    value: float = Field(..., examples=[30.00], description="Super Trend value", serialization_alias="Super Trend")
    trend: str = Field(..., examples=["UP"], description="Trend direction", serialization_alias="Trend")

    def to_dict(self):
        return {
            "type": self.type,
            "Super Trend": str(self.value),
            "Trend": self.trend
        }


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

    def to_dict(self):
        return {
            "type": self.type,
            "Conversion Line": str(self.tenkan_sen),
            "Base Line": str(self.kijun_sen),
            "Lagging Span": str(self.chikou_span),
            "Leading Span A": str(self.senkou_span_a),
            "Leading Span B": str(self.senkou_span_b)
        }


DateType = Annotated[Union[date, datetime, str], Field(description="Date in any format")]


class Analysis(BaseModel):
    type: Indicator = Field(
        default=...,
        examples=["SMA"],
        description="The type of technical indicator"
    )
    indicators: dict[DateType, SerializeAsAny[IndicatorData]] = Field(
        default=...,
        serialization_alias="Technical Analysis",
        validation_alias=AliasChoices("Technical Analysis", "indicators"),
        examples=[{
            "2021-07-09": {
                "value": 30.00,
            }
        }],
        description="Dates with indicators for the stock"
    )

    def model_dump(self, *args, **kwargs) -> dict:
        base_dict = super().model_dump(*args, **kwargs)

        # Format the date keys based on their type
        if 'Technical Analysis' in base_dict:
            formatted_dict = {}
            for k, v in base_dict['Technical Analysis'].items():
                if isinstance(k, datetime):
                    # For datetime, keep full timestamp
                    formatted_dict[k.strftime('%Y-%m-%d %H:%M:%S')] = v
                elif isinstance(k, date):
                    # For date, use date-only format
                    formatted_dict[k.strftime('%Y-%m-%d')] = v
                else:
                    # For strings, keep as is
                    formatted_dict[str(k)] = v
            base_dict['Technical Analysis'] = formatted_dict

        elif 'indicators' in base_dict:
            formatted_dict = {}
            for k, v in base_dict['indicators'].items():
                if isinstance(k, datetime):
                    formatted_dict[k.strftime('%Y-%m-%d %H:%M:%S')] = v
                elif isinstance(k, date):
                    formatted_dict[k.strftime('%Y-%m-%d')] = v
                else:
                    formatted_dict[str(k)] = v
            base_dict['indicators'] = formatted_dict

        return base_dict


class SummaryAnalysis(BaseModel):
    symbol: str = Field(
        default=...,
        description="Stock symbol",
        examples=["NVDA"]
    )
    sma_10: Optional[SMAData] = Field(
        default=None,
        description="10-day Simple Moving Average",
        serialization_alias="SMA(10)",
    )
    sma_20: Optional[SMAData] = Field(
        default=None,
        description="20-day Simple Moving Average",
        serialization_alias="SMA(20)",
    )
    sma_50: Optional[SMAData] = Field(
        default=None,
        description="50-day Simple Moving Average",
        serialization_alias="SMA(50)"
    )
    sma_100: Optional[SMAData] = Field(
        default=None,
        description="100-day Simple Moving Average",
        serialization_alias="SMA(100)"
    )
    sma_200: Optional[SMAData] = Field(
        default=None,
        description="200-day Simple Moving Average",
        serialization_alias="SMA(200)"
    )
    ema_10: Optional[EMAData] = Field(
        default=None,
        description="10-day Exponential Moving Average",
        serialization_alias="EMA(10)"
    )
    ema_20: Optional[EMAData] = Field(
        default=None,
        description="20-day Exponential Moving Average",
        serialization_alias="EMA(20)"
    )
    ema_50: Optional[EMAData] = Field(
        default=None,
        description="50-day Exponential Moving Average",
        serialization_alias="EMA(50)"
    )
    ema_100: Optional[EMAData] = Field(
        default=None,
        description="100-day Exponential Moving Average",
        serialization_alias="EMA(100)"
    )
    ema_200: Optional[EMAData] = Field(
        default=None,
        description="200-day Exponential Moving Average",
        serialization_alias="EMA(200)"
    )
    wma_10: Optional[WMAData] = Field(
        default=None,
        description="10-day Weighted Moving Average",
        serialization_alias="WMA(10)"
    )
    wma_20: Optional[WMAData] = Field(
        default=None,
        description="20-day Weighted Moving Average",
        serialization_alias="WMA(20)"
    )
    wma_50: Optional[WMAData] = Field(
        default=None,
        description="50-day Weighted Moving Average",
        serialization_alias="WMA(50)"
    )
    wma_100: Optional[WMAData] = Field(
        default=None,
        description="100-day Weighted Moving Average",
        serialization_alias="WMA(100)"
    )
    wma_200: Optional[WMAData] = Field(
        default=None,
        description="200-day Weighted Moving Average",
        serialization_alias="WMA(200)"
    )
    vwma: Optional[VWMAData] = Field(
        default=None,
        description="20-day Volume Weighted Moving Average",
        serialization_alias="VWMA(20)"
    )
    rsi: Optional[RSIData] = Field(
        default=None,
        description="14-day Relative Strength Index",
        serialization_alias="RSI(14)"
    )
    srsi: Optional[SRSIData] = Field(
        default=None,
        description="14-day Stochastic RSI",
        serialization_alias="SRSI(3,3,14,14)"
    )
    cci: Optional[CCIData] = Field(
        default=None,
        description="20-day Commodity Channel Index",
        serialization_alias="CCI(20)"
    )
    adx: Optional[ADXData] = Field(
        default=None,
        description="14-day Average Directional Index",
        serialization_alias="ADX(14)"
    )
    macd: Optional[MACDData] = Field(
        default=None,
        description="Moving Average Convergence Divergence",
        serialization_alias="MACD(12,26)"
    )
    stoch: Optional[STOCHData] = Field(
        default=None,
        description="Stochastic Oscillator",
        serialization_alias="STOCH %K(14,3,3)"
    )
    aroon: Optional[AROONData] = Field(
        default=None,
        description="25-day Aroon Indicator",
        serialization_alias="Aroon(25)"
    )
    bbands: Optional[BBANDSData] = Field(
        default=None,
        description="Bollinger Bands",
        serialization_alias="BBANDS(20,2)"
    )
    supertrend: Optional[SuperTrendData] = Field(
        default=None,
        description="Super Trend",
        serialization_alias="Super Trend"
    )
    ichimoku: Optional[IchimokuData] = Field(
        default=None,
        description="Ichimoku Cloud",
        serialization_alias="Ichimoku Cloud"
    )
