from pydantic import BaseModel, Field


class SearchResult(BaseModel):
    name: str = Field(
        default=...,
        example="Apple Inc.",
        description="The name of the company"
    )
    symbol: str = Field(
        default=...,
        example="AAPL",
        description="The stock symbol of the company"
    )
    exchange: str = Field(
        default=...,
        example="NASDAQ",
        description="The exchange the security is traded on"
    )
    type: str = Field(
        default=...,
        example="Equity",
        description="The type of security"
    )
