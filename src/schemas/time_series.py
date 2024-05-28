from enum import Enum

from pydantic import BaseModel, Field
from decimal import Decimal

from typing_extensions import Dict


class TimePeriod(Enum):
    DAY = "1d"
    FIVE_DAYS = "5d"
    ONE_MONTH = "1mo"
    THREE_MONTHS = "3mo"
    SIX_MONTHS = "6mo"
    YTD = "YTD"
    YEAR = "1Y"
    FIVE_YEARS = "5Y"
    TEN_YEARS = "10Y"
    MAX = "max"


class Interval(Enum):
    FIFTEEN_MINUTES = "15m"
    THIRTY_MINUTES = "30m"
    ONE_HOUR = "1h"
    DAILY = "1d"
    WEEKLY = "1wk"
    MONTHLY = "1mo"
    QUARTERLY = "3mo"


class HistoricalData(BaseModel):
    open: Decimal = Field(..., example=145.00, description="Opening price")
    high: Decimal = Field(..., example=145.00, description="Highest price")
    low: Decimal = Field(..., example=145.00, description="Lowest price")
    adj_close: Decimal = Field(..., example=145.00, description="Adjusted closing price",
                               serialization_alias="adjClose")
    volume: int = Field(..., example=1000000, description="Volume traded")


class TimeSeries(BaseModel):
    history: Dict[str, HistoricalData] = Field(
        ...,
        serialization_alias="Historical Data",
        example={
            "2021-07-09": {
                "open": 145.00,
                "high": 145.00,
                "low": 145.00,
                "adjClose": 145.00,
                "volume": 1000000
            }
        }, description="Dates with historical data for the stock")
