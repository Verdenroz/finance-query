from pydantic import BaseModel, Field


class SearchResult(BaseModel):
    name: str = Field(..., example="Apple Inc.", description="The name of the company")
    symbol: str = Field(..., example="AAPL", description="The stock symbol of the company")
    exchange: str = Field(..., example="NASDAQ", description="The exchange the security is traded on")
    type: str = Field(..., example="Equity", description="The type of security")
