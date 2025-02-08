from enum import Enum
from typing import Optional

from pydantic import BaseModel, Field, AliasChoices


class TimePeriod(Enum):
    DAY = "1d"
    FIVE_DAYS = "5d"
    ONE_MONTH = "1mo"
    THREE_MONTHS = "3mo"
    SIX_MONTHS = "6mo"
    YTD = "ytd"
    YEAR = "1y"
    TWO_YEARS = "2y"
    FIVE_YEARS = "5y"
    TEN_YEARS = "10y"
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
    open: float = Field(
        default=...,
        examples=[145.00],
        description="Opening price"
    )
    high: float = Field(
        default=...,
        examples=[145.00],
        description="Highest price"
    )
    low: float = Field(
        default=...,
        examples=[145.00],
        description="Lowest price"
    )
    close: float = Field(
        default=...,
        examples=[145.00],
        description="Closing price",
    )
    adj_close: Optional[float] = Field(
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
