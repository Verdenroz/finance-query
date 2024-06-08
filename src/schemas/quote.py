from decimal import Decimal

from pydantic import BaseModel, Field, AliasChoices
from typing_extensions import Optional


class Quote(BaseModel):
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
    after_hours_price: Optional[Decimal] = Field(
        default=None,
        example=145.50,
        description="After hours price of the stock",
        serialization_alias="afterHoursPrice",
        validation_alias=AliasChoices("afterHoursPrice", "after_hours_price")
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
        validation_alias=AliasChoices("percentChange", "percent_change")
    )
    open: Decimal = Field(
        default=...,
        example=144.00,
        description="Opening price of the stock"
    )
    high: Decimal = Field(
        default=...,
        example=146.00,
        description="Highest day price of the stock"
    )
    low: Decimal = Field(
        default=...,
        example=143.00,
        description="Lowest day price of the stock"
    )
    year_high: Decimal = Field(
        default=...,
        example=150.00,
        description="52-week high price of the stock",
        serialization_alias="yearHigh",
        validation_alias=AliasChoices("yearHigh", "year_high")
    )
    year_low: Decimal = Field(
        default=...,
        example=100.00,
        description="52-week low price of the stock",
        serialization_alias="yearLow",
        validation_alias=AliasChoices("yearLow", "year_low")
    )
    volume: int = Field(
        default=...,
        example=1000000,
        description="Volume of the stock")
    avg_volume: int = Field(
        default=...,
        example=2000000,
        description="Average volume of the stock",
        serialization_alias="avgVolume",
        validation_alias=AliasChoices("avgVolume", "avg_volume")
    )
    market_cap: Optional[str] = Field(
        default=None,
        example="2.5T",
        description="Market capitalization of the stock",
        serialization_alias="marketCap",
        validation_alias=AliasChoices("marketCap", "market_cap")
    )
    beta: Optional[Decimal] = Field(
        default=None,
        example=1.23,
        description="Beta of the stock"
    )
    pe: Optional[Decimal] = Field(
        default=None,
        example=30.00,
        description="Price to earnings ratio of the stock"
    )
    eps: Optional[Decimal] = Field(
        default=None,
        example=4.50,
        description="Earnings per share of the stock"
    )
    dividend: Optional[Decimal] = Field(
        default=None,
        example=0.82,
        description="Dividend yield of the stock"
    )
    dividend_yield: Optional[str] = Field(
        default=None,
        example="1.3%",
        description="Dividend yield of the stock",
        serialization_alias="yield",
        validation_alias=AliasChoices("yield", "dividend_yield")
    )
    ex_dividend: Optional[str] = Field(
        default=None, example="Feb 5, 2024",
        description="Ex-dividend date of the stock",
        serialization_alias="exDividend",
        validation_alias=AliasChoices("exDividend", "ex_dividend")
    )
    net_assets: Optional[str] = Field(
        default=None,
        example="10.5B",
        description="Net assets of the stock",
        serialization_alias="netAssets",
        validation_alias=AliasChoices("netAssets", "net_assets")
    )
    nav: Optional[str] = Field(
        default=None,
        example="100.00",
        description="Net asset value of the stock"
    )
    expense_ratio: Optional[str] = Field(
        default=None,
        example="0.05%",
        description="Expense ratio of the stock",
        serialization_alias="expenseRatio",
        validation_alias=AliasChoices("expenseRatio", "expense_ratio")
    )
    earnings_date: Optional[str] = Field(
        default=None,
        example="Apr 23, 2024",
        description="Next earnings date of the stock",
        serialization_alias="earningsDate",
        validation_alias=AliasChoices("earningsDate", "earnings_date")
    )
    sector: Optional[str] = Field(
        default=None,
        example="Technology",
        description="Sector of the company")
    industry: Optional[str] = Field(
        default=None,
        example="Consumer Electronics",
        description="Industry of the company"
    )
    about: Optional[str] = Field(
        default=None,
        example="Apple Inc. designs, manufactures, and markets smartphones, personal computers, "
                "tablets, wearables, and accessories worldwide.",
        description="About the company"
    )
    ytd_return: Optional[str] = Field(
        default=None,
        example="+10.00%",
        description="Year to date return of the company",
        serialization_alias="ytdReturn",
        validation_alias=AliasChoices("ytdReturn", "ytd_return")
    )
    year_return: Optional[str] = Field(
        default=None,
        example="+20.00%",
        description="One year return of the company",
        serialization_alias="yearReturn",
        validation_alias=AliasChoices("yearReturn", "year_return")
    )
    three_year_return: Optional[str] = Field(
        default=None,
        example="+30.00%",
        description="Three year return of the company",
        serialization_alias="threeYearReturn",
        validation_alias=AliasChoices("threeYearReturn", "three_year_return")
    )
    five_year_return: Optional[str] = Field(
        default=None,
        example="+40.00%",
        description="Five year return of the company",
        serialization_alias="fiveYearReturn",
        validation_alias=AliasChoices("fiveYearReturn", "five_year_return")
    )
    logo: Optional[str] = Field(
        default=None,
        example="https://logo.clearbit.com/apple.com",
        description="Company logo"
    )

    def dict(self, *args, **kwargs):
        base_dict = super().dict(*args, **kwargs, by_alias=True)
        return {k: (str(v) if isinstance(v, Decimal) else v) for k, v in base_dict.items() if v is not None}
