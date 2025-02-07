from decimal import Decimal

from pydantic import BaseModel, Field, AliasChoices


class Index(BaseModel):
    name: str = Field(
        default=...,
        examples=["S&P 500"],
        description="Name of the index"
    )
    value: Decimal = Field(
        default=...,
        examples=[4300.00],
        description="Current value of the index"
    )
    change: str = Field(
        default=...,
        examples=["+10.00"],
        description="Change in the index value"
    )
    percent_change: str = Field(
        default=...,
        examples=["+0.23%"],
        description="Percentage change in the index value",
        serialization_alias="percentChange",
        validation_alias=AliasChoices("percentChange", "percent_change"))

    def dict(self, *args, **kwargs):
        base_dict = super().model_dump(*args, **kwargs, exclude_none=True, by_alias=True)
        return {k: (str(v) if isinstance(v, Decimal) else v) for k, v in base_dict.items() if v is not None}
