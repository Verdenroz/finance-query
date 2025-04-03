from typing import Optional

from pydantic import BaseModel, Field, AliasChoices


class SimpleQuote(BaseModel):
    symbol: str = Field(
        default=...,
        examples=["AAPL"],
        description="Stock symbol"
    )
    name: str = Field(
        default=...,
        examples=["Apple Inc."],
        description="Company name"
    )
    price: str = Field(
        default=...,
        examples=["145.00"],
        description="Last traded price of the stock"
    )
    pre_market_price: Optional[str] = Field(
        default=None,
        examples=["145.50"],
        description="After hours price of the stock",
        serialization_alias="preMarketPrice",
        validation_alias=AliasChoices("preMarketPrice", "pre_market_price")
    )
    after_hours_price: Optional[str] = Field(
        default=None,
        examples=["145.50"],
        description="After hours price of the stock",
        serialization_alias="afterHoursPrice",
        validation_alias=AliasChoices("afterHoursPrice", "after_hours_price")
    )
    change: str = Field(
        default=...,
        examples=["+1.00"],
        description="Change in the stock price"
    )
    percent_change: str = Field(
        default=...,
        examples=["+0.69%"],
        description="Percentage change in the stock price",
        serialization_alias="percentChange",
        validation_alias=AliasChoices("percentChange", "percent_change")
    )
    logo: Optional[str] = Field(
        default=None,
        examples=["https://img.logo.dev/apple.com?token=pk_Xd1Cdye3QYmCOXzcvxhxyw&retina=true"],
        description="Company logo"
    )

    def dict(self, *args, **kwargs):
        return super().model_dump(*args, **kwargs, by_alias=True, exclude_none=True)
