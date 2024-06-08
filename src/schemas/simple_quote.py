from pydantic import BaseModel, Field, AliasChoices
from decimal import Decimal


class SimpleQuote(BaseModel):
    symbol: str = Field(
        default=...,
        example="AAPL",
        description="Stock symbol"
    )
    name: str = Field(
        default=...,
        example="Apple Inc.",
        description="Company name"
    )
    price: Decimal = Field(
        default=...,
        example=145.00,
        description="Last traded price of the stock"
    )
    change: str = Field(
        default=...,
        example="+1.00",
        description="Change in the stock price"
    )
    percent_change: str = Field(
        default=...,
        example="+0.69%",
        description="Percentage change in the stock price",
        serialization_alias="percentChange",
        validation_alias=AliasChoices("percentChange", "percent_change"))

    def dict(self, *args, **kwargs):
        base_dict = super().dict(*args, **kwargs, exclude_none=True, by_alias=True)
        return {k: (str(v) if isinstance(v, Decimal) else v) for k, v in base_dict.items() if v is not None}
