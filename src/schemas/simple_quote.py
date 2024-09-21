from decimal import Decimal
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
    price: Decimal = Field(
        default=...,
        examples=[145.00],
        description="Last traded price of the stock"
    ),
    after_hours_price: Optional[Decimal] = Field(
        default=None,
        examples=[145.50],
        description="After hours price of the stock",
        serialization_alias="afterHoursPrice",
        validation_alias=AliasChoices("afterHoursPrice", "after_hours_price")
    ),
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
        validation_alias=AliasChoices("percentChange", "percent_change"))
    logo: Optional[str] = Field(
        default=None,
        examples=["https://logo.clearbit.com/apple.com"],
        description="Company logo"
    )

    def dict(self, *args, **kwargs):
        base_dict = super().model_dump(*args, **kwargs, exclude_none=True, by_alias=True)
        return {k: (str(v) if isinstance(v, Decimal) else v) for k, v in base_dict.items() if v is not None}
