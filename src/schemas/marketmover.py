from pydantic import BaseModel, Field, AliasChoices


class MarketMover(BaseModel):
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
        examples=["145.86"],
        description="Last traded price of the stock"
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
        validation_alias=AliasChoices("percentChange", "percent_change"))

    def dict(self, *args, **kwargs):
        return super().model_dump(*args, **kwargs, exclude_none=True, by_alias=True)
