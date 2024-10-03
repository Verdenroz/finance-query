from decimal import Decimal
from enum import Enum
from typing import Optional

from pydantic import BaseModel, Field, AliasChoices


class TimePeriod(Enum):
    DAY = "1d"
    FIVE_DAYS = "5d"
    SEVEN_DAYS = "7d"
    ONE_MONTH = "1mo"
    THREE_MONTHS = "3mo"
    SIX_MONTHS = "6mo"
    YTD = "YTD"
    YEAR = "1Y"
    FIVE_YEARS = "5Y"
    TEN_YEARS = "10Y"
    MAX = "max"


class Interval(Enum):
    ONE_MINUTE = "1m"
    FIVE_MINUTES = "5m"
    FIFTEEN_MINUTES = "15m"
    THIRTY_MINUTES = "30m"
    ONE_HOUR = "1h"
    DAILY = "1d"
    WEEKLY = "1wk"
    MONTHLY = "1mo"
    QUARTERLY = "3mo"


class HistoricalData(BaseModel):
    open: Decimal = Field(
        default=...,
        examples=[145.00],
        description="Opening price"
    )
    high: Decimal = Field(
        default=...,
        examples=[145.00],
        description="Highest price"
    )
    low: Decimal = Field(
        default=...,
        examples=[145.00],
        description="Lowest price"
    )
    close: Decimal = Field(
        default=...,
        examples=[145.00],
        description="Closing price",
    )
    adj_close: Optional[Decimal] = Field(
        default=None,
        examples=[145.00],
        description="Adjusted closing price",
        serialization_alias="adjClose",
        validation_alias=AliasChoices("adjClose", "adj_close")
    )
    volume: int = Field(
        default=...,
        examples=[1000000],
        description="Volume traded"
    )


class TimeSeries(BaseModel):
    history: dict[str, HistoricalData] = Field(
        default=...,
        serialization_alias="Historical Data",
        validation_alias=AliasChoices("Historical Data", "history"),
        examples=[{
            "2021-07-09": {
                "open": 145.00,
                "high": 145.00,
                "low": 145.00,
                "adjClose": 145.00,
                "volume": 1000000
            }
        }], description="Dates with historical data for the stock"
    )
