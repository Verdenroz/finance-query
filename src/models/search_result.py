from enum import Enum

from pydantic import BaseModel, Field

"""
 Stocks, ETFs, and Trusts are all types of securities, but they can be called different things depending on the context.
    For example:

    stock -> equity
    trust -> mutual funds
"""


class Type(Enum):
    STOCK = "stock"
    ETF = "etf"
    TRUST = "trust"


class SearchResult(BaseModel):
    name: str = Field(default=..., examples=["Apple Inc."], description="The name of the company")
    symbol: str = Field(default=..., examples=["AAPL"], description="The stock symbol of the company")
    exchange: str = Field(default=..., examples=["NASDAQ"], description="The exchange the security is traded on")
    type: str = Field(default=..., examples=["stock"], description="The type of security")
