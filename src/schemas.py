from pydantic import BaseModel, Field
from decimal import Decimal

class Quote(BaseModel):
    symbol: str = Field(..., example="AAPL", description="Stock symbol")
    name: str = Field(..., example="Apple Inc.", description="Company name")
    price: Decimal = Field(..., example=145.00, description="Last traded price of the stock")
    change: str = Field(..., example="+1.00", description="Change in the stock price")
    percent_change: str = Field(..., example="+0.69%", description="Percentage change in the stock price")
    open: Decimal = Field(..., example=144.00, description="Opening price of the stock")
    high: Decimal = Field(..., example=146.00, description="Highest price of the stock")
    low: Decimal = Field(..., example=143.00, description="Lowest price of the stock")
    volume: int = Field(..., example=1000000, description="Volume of the stock")
    avg_volume: int = Field(..., example=2000000, description="Average volume of the stock")
    market_cap: str = Field(..., example="2.5T", description="Market capitalization of the stock")
    pe_ratio: Decimal = Field(..., example=30.00, description="Price to earnings ratio of the stock")
    dividend: Decimal = Field(..., example=0.82, description="Dividend yield of the stock")

class Index(BaseModel):
    name: str = Field(..., example="S&P 500", description="Name of the index")
    value: Decimal = Field(..., example=4300.00, description="Current value of the index")
    change: str = Field(..., example="+10.00", description="Change in the index value")
    percent_change: str = Field(..., example="+0.23%", description="Percentage change in the index value")

class MarketMover(BaseModel):
    symbol: str = Field(..., example="AAPL", description="Stock symbol")
    name: str = Field(..., example="Apple Inc.", description="Company name")
    price: Decimal = Field(..., example=145.00, description="Last traded price of the stock")
    change: str = Field(..., example="+1.00", description="Change in the stock price")
    percent_change: str = Field(..., example="+0.69%", description="Percentage change in the stock price")