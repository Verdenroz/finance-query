from pydantic import BaseModel, Field, AliasChoices


class Index(BaseModel):
    name: str = Field(
        default=...,
        examples=["S&P 500"],
        description="Name of the index"
    )
    value: float = Field(
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
        return super().model_dump(*args, **kwargs, exclude_none=True, by_alias=True)
