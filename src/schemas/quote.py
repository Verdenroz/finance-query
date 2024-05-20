from pydantic import BaseModel, Field
from decimal import Decimal

from typing_extensions import Optional

from .stock import Stock
from .news import News


class Quote(BaseModel):
    symbol: str = Field(..., example="AAPL", description="Stock symbol")
    name: str = Field(..., example="Apple Inc.", description="Company name")
    price: Decimal = Field(..., example=145.00, description="Last traded price of the stock")
    after_hours_price: Optional[Decimal] = Field(None, example=145.50, description="After hours price of the stock")
    change: str = Field(..., example="+1.00", description="Change in the stock price")
    percent_change: str = Field(..., example="+0.69%", description="Percentage change in the stock price")
    open: Decimal = Field(..., example=144.00, description="Opening price of the stock")
    high: Decimal = Field(..., example=146.00, description="Highest day price of the stock")
    low: Decimal = Field(..., example=143.00, description="Lowest day price of the stock")
    year_high: Decimal = Field(..., example=150.00, description="52-week high price of the stock")
    year_low: Decimal = Field(..., example=100.00, description="52-week low price of the stock")
    volume: int = Field(..., example=1000000, description="Volume of the stock")
    avg_volume: int = Field(..., example=2000000, description="Average volume of the stock")
    market_cap: Optional[str] = Field(None, example="2.5T", description="Market capitalization of the stock")
    beta: Optional[Decimal] = Field(None, example=1.23, description="Beta of the stock")
    pe: Optional[Decimal] = Field(None, example=30.00, description="Price to earnings ratio of the stock")
    eps: Optional[Decimal] = Field(None, example=4.50, description="Earnings per share of the stock")
    dividend: Optional[Decimal] = Field(None, example=0.82, description="Dividend yield of the stock")
    ex_dividend: Optional[str] = Field(None, example="Feb 5, 2024", description="Ex-dividend date of the stock")
    earnings_date: Optional[str] = Field(None, example="Apr 23, 2024", description="Next earnings date of the stock")
    sector: Optional[str] = Field(None, example="Technology", description="Sector of the company")
    industry: Optional[str] = Field(None, example="Consumer Electronics", description="Industry of the company")
    about: Optional[str] = Field(None,
                       example="Apple Inc. designs, manufactures, and markets smartphones, personal computers, "
                               "tablets, wearables, and accessories worldwide.",
                       description="About the company")
    logo: Optional[str] = Field(None, example="https://logo.clearbit.com/apple.com", description="Company logo")


    def dict(self, *args, **kwargs):
        return {
            "symbol": self.symbol,
            "name": self.name,
            "price": str(self.price),
            "after_hours_price": str(self.after_hours_price) if self.after_hours_price else None,
            "change": self.change,
            "percent_change": self.percent_change,
            "open": str(self.open),
            "high": str(self.high),
            "low": str(self.low),
            "year_high": str(self.year_high),
            "year_low": str(self.year_low),
            "volume": self.volume,
            "avg_volume": self.avg_volume,
            "market_cap": self.market_cap,
            "beta": str(self.beta) if self.beta else None,
            "pe": str(self.pe) if self.pe else None,
            "eps": str(self.eps) if self.eps else None,
            "dividend": str(self.dividend) if self.dividend else None,
            "ex_dividend": self.ex_dividend,
            "earnings_date": self.earnings_date,
            "sector": self.sector,
            "industry": self.industry,
            "about": self.about,
            "logo": self.logo
        }