from pydantic import BaseModel, Field
from decimal import Decimal

from typing_extensions import Dict


class HistoricalData(BaseModel):
    open: Decimal = Field(..., example=145.00, description="Opening price")
    high: Decimal = Field(..., example=145.00, description="Highest price")
    low: Decimal = Field(..., example=145.00, description="Lowest price")
    adj_close: Decimal = Field(..., example=145.00, description="Adjusted closing price")
    volume: int = Field(..., example=1000000, description="Volume traded")


class TimeSeries(BaseModel):
    history: Dict[str, HistoricalData] = Field(..., example={
        "2021-07-09": {
            "open": 145.00,
            "high": 145.00,
            "low": 145.00,
            "adj_close": 145.00,
            "volume": 1000000
        }
    }, description="Dates with historical data for the stock")
