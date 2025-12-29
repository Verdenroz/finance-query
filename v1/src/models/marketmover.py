from enum import Enum

from pydantic import AliasChoices, BaseModel, Field


class MoverCount(Enum):
    TWENTY_FIVE = "25"
    FIFTY = "50"
    HUNDRED = "100"


class MarketMover(BaseModel):
    symbol: str = Field(default=..., examples=["AAPL"], description="Stock symbol")
    name: str = Field(default=..., examples=["Apple Inc."], description="Company name")
    price: str = Field(default=..., examples=["145.86"], description="Last traded price of the stock")
    change: str = Field(default=..., examples=["+1.00"], description="Change in the stock price")
    percent_change: str = Field(
        default=...,
        examples=["+0.69%"],
        description="Percentage change in the stock price",
        serialization_alias="percentChange",
        validation_alias=AliasChoices("percentChange", "percent_change"),
    )
