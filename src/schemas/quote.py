from decimal import Decimal

from pydantic import BaseModel, Field, AliasChoices
from typing_extensions import Optional


class Quote(BaseModel):
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
    price: Decimal = Field(
        default=...,
        examples=[145.00],
        description="Last traded price of the stock"
    )
    after_hours_price: Optional[Decimal] = Field(
        default=None,
        examples=[145.50],
        description="After hours price of the stock",
        serialization_alias="afterHoursPrice",
        validation_alias=AliasChoices("afterHoursPrice", "after_hours_price")
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
        validation_alias=AliasChoices("percentChange", "percent_change")
    )
    open: Optional[Decimal] = Field(
        default=None,
        examples=[144.00],
        description="Opening price of the stock"
    )
    high: Optional[Decimal] = Field(
        default=None,
        examples=[146.00],
        description="Highest day price of the stock"
    )
    low: Optional[Decimal] = Field(
        default=None,
        examples=[143.00],
        description="Lowest day price of the stock"
    )
    year_high: Optional[Decimal] = Field(
        default=None,
        examples=[150.00],
        description="52-week high price of the stock",
        serialization_alias="yearHigh",
        validation_alias=AliasChoices("yearHigh", "year_high")
    )
    year_low: Optional[Decimal] = Field(
        default=None,
        examples=[100.00],
        description="52-week low price of the stock",
        serialization_alias="yearLow",
        validation_alias=AliasChoices("yearLow", "year_low")
    )
    volume: Optional[int] = Field(
        default=None,
        examples=[1000000],
        description="Volume of the stock")
    avg_volume: Optional[int] = Field(
        default=None,
        examples=[2000000],
        description="Average volume of the stock",
        serialization_alias="avgVolume",
        validation_alias=AliasChoices("avgVolume", "avg_volume")
    )
    market_cap: Optional[str] = Field(
        default=None,
        examples=["2.5T"],
        description="Market capitalization of the stock",
        serialization_alias="marketCap",
        validation_alias=AliasChoices("marketCap", "market_cap")
    )
    beta: Optional[str] = Field(
        default=None,
        examples=[1.23],
        description="Beta of the stock"
    )
    pe: Optional[str] = Field(
        default=None,
        examples=["30.00"],
        description="Price to earnings ratio of the stock"
    )
    eps: Optional[str] = Field(
        default=None,
        examples=["4.50"],
        description="Earnings per share of the stock"
    )
    dividend: Optional[str] = Field(
        default=None,
        examples=["0.82"],
        description="Dividend yield of the stock"
    )
    dividend_yield: Optional[str] = Field(
        default=None,
        examples=["1.3%"],
        description="Dividend yield of the stock",
        serialization_alias="yield",
        validation_alias=AliasChoices("yield", "dividend_yield")
    )
    ex_dividend: Optional[str] = Field(
        default=None, examples=["Feb 5, 2024"],
        description="Ex-dividend date of the stock",
        serialization_alias="exDividend",
        validation_alias=AliasChoices("exDividend", "ex_dividend")
    )
    net_assets: Optional[str] = Field(
        default=None,
        examples=["10.5B"],
        description="Net assets of the stock",
        serialization_alias="netAssets",
        validation_alias=AliasChoices("netAssets", "net_assets")
    )
    nav: Optional[str] = Field(
        default=None,
        examples=["100.00"],
        description="Net asset value of the stock"
    )
    expense_ratio: Optional[str] = Field(
        default=None,
        examples=["0.05%"],
        description="Expense ratio of the stock",
        serialization_alias="expenseRatio",
        validation_alias=AliasChoices("expenseRatio", "expense_ratio")
    )
    category: Optional[str] = Field(
        default=None,
        examples=["Large Growth"],
        description="Category of the fund"
    )
    last_capital_gain: Optional[str] = Field(
        default=None,
        examples=["10.00"],
        description="Last capital gain of the fund",
        serialization_alias="lastCapitalGain",
        validation_alias=AliasChoices("lastCapitalGain", "last_capital_gain")
    )
    morningstar_rating: Optional[str] = Field(
        default=None,
        examples=["★★"],
        description="Morningstar rating of the fund",
        serialization_alias="morningstarRating",
        validation_alias=AliasChoices("morningstarRating", "morningstar_rating")
    )
    morningstar_risk_rating: Optional[str] = Field(
        default=None,
        examples=["Low"],
        description="Morningstar risk rating of the fund",
        serialization_alias="morningstarRiskRating",
        validation_alias=AliasChoices("morningstarRiskRating", "morningstar_risk_rating")
    )
    holdings_turnover: Optional[str] = Field(
        default=None,
        examples=["5.00%"],
        description="Holdings turnover of the fund",
        serialization_alias="holdingsTurnover",
        validation_alias=AliasChoices("holdingsTurnover", "holdings_turnover")
    )
    earnings_date: Optional[str] = Field(
        default=None,
        examples=["Apr 23, 2024"],
        description="Next earnings date of the stock",
        serialization_alias="earningsDate",
        validation_alias=AliasChoices("earningsDate", "earnings_date")
    )
    last_dividend: Optional[str] = Field(
        default=None,
        examples=["0.82"],
        description="Last dividend of the fund",
        serialization_alias="lastDividend",
        validation_alias=AliasChoices("lastDividend", "last_dividend")
    )
    inception_date: Optional[str] = Field(
        default=None,
        examples=["Jan 1, 2020"],
        description="Inception date of the fund",
        serialization_alias="inceptionDate",
        validation_alias=AliasChoices("inceptionDate", "inception_date")
    )
    sector: Optional[str] = Field(
        default=None,
        examples=["Technology"],
        description="Sector of the company")
    industry: Optional[str] = Field(
        default=None,
        examples=["Consumer Electronics"],
        description="Industry of the company"
    )
    about: Optional[str] = Field(
        default=None,
        examples=["Apple Inc. designs, manufactures, and markets smartphones, personal computers, "
                  "tablets, wearables, and accessories worldwide."],
        description="About the company"
    )
    employees: Optional[str] = Field(
        default=None,
        examples=["150,000"],
        description="Number of employees in the company"
    )
    ytd_return: Optional[str] = Field(
        default=None,
        examples=["+10.00%"],
        description="Year to date return of the company",
        serialization_alias="ytdReturn",
        validation_alias=AliasChoices("ytdReturn", "ytd_return")
    )
    year_return: Optional[str] = Field(
        default=None,
        examples=["+20.00%"],
        description="One year return of the company",
        serialization_alias="yearReturn",
        validation_alias=AliasChoices("yearReturn", "year_return")
    )
    three_year_return: Optional[str] = Field(
        default=None,
        examples=["+30.00%"],
        description="Three year return of the company",
        serialization_alias="threeYearReturn",
        validation_alias=AliasChoices("threeYearReturn", "three_year_return")
    )
    five_year_return: Optional[str] = Field(
        default=None,
        examples=["+40.00%"],
        description="Five year return of the company",
        serialization_alias="fiveYearReturn",
        validation_alias=AliasChoices("fiveYearReturn", "five_year_return")
    )
    logo: Optional[str] = Field(
        default=None,
        examples=["https://logo.clearbit.com/apple.com"],
        description="Company logo"
    )

    def dict(self, *args, **kwargs):
        base_dict = super().model_dump(*args, **kwargs, by_alias=True, exclude_none=True)
        return {k: (str(v) if isinstance(v, Decimal) else v) for k, v in base_dict.items() if v is not None}
