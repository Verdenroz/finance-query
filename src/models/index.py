from enum import Enum
from typing import Optional

from pydantic import BaseModel, Field, AliasChoices


class Index(Enum):
    GSPC = "snp"
    DJI = "djia"
    IXIC = "nasdaq"
    NYA = "nyse-composite"
    XAX = "nyse-amex"
    BUK100P = "cboe-uk-100"
    RUT = "rut"
    VIX = "vix"
    FTSE = "ftse-100"
    GDAXI = "dax"
    FCHI = "cac-40"
    STOXX50E = "euro-stoxx-50"
    N100 = "euronext-100"
    BFX = "bel-20"
    MOEX_ME = "moex"
    HSI = "hang-seng"
    STI = "sti"
    AXJO = "asx-200"
    AORD = "all-ordinaries"
    BSESN = "sensex"
    JKSE = "idx-composite"
    KLSE = "ftse-bursa"
    NZ50 = "nzx-50"
    KS11 = "kospi"
    TWII = "twse"
    GSPTSE = "tsx-composite"
    BVSP = "ibovespa"
    MXX = "ipc-mexico"
    IPSA = "ipsa"
    MERV = "merval"
    TA125_TA = "ta-125"
    CASE30 = "egx-30"
    JN0U_JO = "top-40-usd"
    DX_Y_NYB = "usd-index"
    USD_STRD = "msci-europe"
    XDB = "gbp-index"
    XDE = "euro-index"
    SS = "sse-composite"
    N225 = "nikkei-225"
    XDN = "yen"
    XDA = "australian-dollar"


class MarketIndex(BaseModel):
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
    five_days_return: Optional[str] = Field(
        default=None,
        examples=["-19.35%"],
        description="Five days return of the company",
        serialization_alias="fiveDaysReturn",
        validation_alias=AliasChoices("fiveDaysReturn", "five_days_return")
    )
    one_month_return: Optional[str] = Field(
        default=None,
        examples=["-28.48%"],
        description="One month return of the company",
        serialization_alias="oneMonthReturn",
        validation_alias=AliasChoices("oneMonthReturn", "one_month_return")
    )
    three_month_return: Optional[str] = Field(
        default=None,
        examples=["-14.02%"],
        description="Three month return of the company",
        serialization_alias="threeMonthReturn",
        validation_alias=AliasChoices("threeMonthReturn", "three_month_return")
    )
    six_month_return: Optional[str] = Field(
        default=None,
        examples=["36.39%"],
        description="Six month return of the company",
        serialization_alias="sixMonthReturn",
        validation_alias=AliasChoices("sixMonthReturn", "six_month_return")
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
    ten_year_return: Optional[str] = Field(
        default=None,
        examples=["2,005.31%"],
        description="Ten year return of the company",
        serialization_alias="tenYearReturn",
        validation_alias=AliasChoices("tenYearReturn", "ten_year_return")
    )
    max_return: Optional[str] = Field(
        default=None,
        examples=["22,857.89%"],
        description="Maximum return of the company",
        serialization_alias="maxReturn",
        validation_alias=AliasChoices("maxReturn", "max_return")
    )

    def dict(self, *args, **kwargs):
        return super().model_dump(*args, **kwargs, exclude_none=True, by_alias=True)
