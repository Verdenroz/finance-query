from enum import Enum
from typing import Optional

from pydantic import BaseModel, Field, AliasChoices


class Region(Enum):
    UNITED_STATES = "US"
    NORTH_AMERICA = "NA"
    SOUTH_AMERICA = "SA"
    EUROPE = "EU"
    ASIA = "AS"
    AFRICA = "AF"
    MIDDLE_EAST = "ME"
    OCEANIA = "OCE"
    GLOBAL = "global"


class Index(Enum):
    # United States
    GSPC = "snp"                 # S&P 500
    DJI = "djia"                 # Dow Jones Industrial Average
    IXIC = "nasdaq"              # NASDAQ Composite
    NYA = "nyse-composite"       # NYSE Composite
    XAX = "nyse-amex"            # NYSE American Composite
    RUT = "rut"                  # Russell 2000
    VIX = "vix"                  # CBOE Volatility Index

    # North America (excluding US)
    GSPTSE = "tsx-composite"     # Toronto Stock Exchange

    # South America
    BVSP = "ibovespa"            # Brazil Bovespa
    MXX = "ipc-mexico"           # Mexican IPC
    IPSA = "ipsa"                # Chile IPSA
    MERV = "merval"              # Argentina Merval
    IVBX = "ivbx"                # Brazil IVBX
    IBRX_50 = "ibrx-50"          # Brazil IBrX-50

    # Europe
    FTSE = "ftse-100"            # FTSE 100
    GDAXI = "dax"                # German DAX
    FCHI = "cac-40"              # French CAC 40
    STOXX50E = "euro-stoxx-50"   # Euro Stoxx 50
    N100 = "euronext-100"        # Euronext 100
    BFX = "bel-20"               # Belgian BEL 20
    MOEX_ME = "moex"             # Moscow Exchange
    AEX = "aex"                  # Amsterdam Exchange
    IBEX = "ibex-35"             # Spanish IBEX 35
    FTSEMIB = "ftse-mib"         # Italian FTSE MIB
    SSMI = "smi"                 # Swiss Market Index
    PSI = "psi"                  # Portuguese PSI
    ATX = "atx"                  # Austrian ATX
    OMXS30 = "omxs30"            # Stockholm OMX 30
    OMXC25 = "omxc25"            # Copenhagen OMX 25
    WIG20 = "wig20"              # Warsaw WIG 20
    BUX = "budapest-se"          # Budapest Stock Exchange
    IMOEX = "moex-russia"        # Moscow Exchange Russia
    RTSI = "rtsi"                # Russian Trading System

    # Asia
    HSI = "hang-seng"            # Hong Kong Hang Seng
    STI = "sti"                  # Singapore Straits Times
    BSESN = "sensex"             # BSE Sensex (India)
    JKSE = "idx-composite"       # Jakarta Composite
    KLSE = "ftse-bursa"          # FTSE Bursa Malaysia
    KS11 = "kospi"               # Korea KOSPI
    TWII = "twse"                # Taiwan TAIEX
    N225 = "nikkei-225"          # Nikkei 225
    SHANGHAI = "shanghai"        # Shanghai Composite
    SZSE = "szse-component"      # Shenzhen Component
    SET = "set"                  # Thailand SET
    NSEI = "nifty-50"            # NSE Nifty 50 (India)
    CNX200 = "nifty-200"         # NSE Nifty 200
    PSEI = "psei-composite"      # Philippines PSEi Composite
    CHINA_A50 = "china-a50"      # FTSE China A50
    DJSH = "dj-shanghai"         # Dow Jones Shanghai
    INDIAVIX = "india-vix"       # India VIX

    # Africa
    CASE30 = "egx-30"            # Egypt EGX 30
    JN0U_JO = "jse-40"           # FTSE JSE Top 40- USD Net TRI
    FTSEJSE = "ftse-jse"         # FTSE/JSE SA Financials Index
    AFR40 = "afr-40"             # All Africa 40 Rand Index
    RAF40 = "raf-40"             # RAFI 40 Index
    SA40 = "sa-40"               # South Africa Top 40
    ALT15 = "alt-15"             # Alternative 15

    # Middle East
    TA125_TA = "ta-125"          # Tel Aviv 125
    TA35 = "ta-35"               # Tel Aviv 35
    TASI = "tadawul-all-share"   # Tadawul All Share
    TAMAYUZ = "tamayuz"          # Egyptian Tamayuz
    BIST100 = "bist-100"         # Borsa Istanbul 100

    # Oceania
    AXJO = "asx-200"             # ASX 200 (Australia)
    AORD = "all-ordinaries"      # All Ordinaries (Australia)
    NZ50 = "nzx-50"              # NZX 50 (New Zealand)

    # Global/Currency
    DX_Y_NYB = "usd"             # US Dollar Index
    USD_STRD = "msci-europe"     # MSCI Europe USD
    XDB = "gbp"                  # British Pound
    XDE = "euro"                 # Euro
    XDN = "yen"                  # Japanese Yen
    XDA = "australian"           # Australian Dollar
    MSCI_WORLD = "msci-world"    # MSCI World Index
    BUK100P = "cboe-uk-100"      # CBOE UK 100


# Mapping of indices to their regions
INDEX_REGIONS = {
    # United States
    Index.GSPC: Region.UNITED_STATES,
    Index.DJI: Region.UNITED_STATES,
    Index.IXIC: Region.UNITED_STATES,
    Index.NYA: Region.UNITED_STATES,
    Index.XAX: Region.UNITED_STATES,
    Index.RUT: Region.UNITED_STATES,
    Index.VIX: Region.UNITED_STATES,

    # North America (excluding US)
    Index.GSPTSE: Region.NORTH_AMERICA,

    # South America
    Index.BVSP: Region.SOUTH_AMERICA,
    Index.MXX: Region.SOUTH_AMERICA,
    Index.IPSA: Region.SOUTH_AMERICA,
    Index.MERV: Region.SOUTH_AMERICA,
    Index.IVBX: Region.SOUTH_AMERICA,
    Index.IBRX_50: Region.SOUTH_AMERICA,

    # Europe
    Index.FTSE: Region.EUROPE,
    Index.GDAXI: Region.EUROPE,
    Index.FCHI: Region.EUROPE,
    Index.STOXX50E: Region.EUROPE,
    Index.N100: Region.EUROPE,
    Index.BFX: Region.EUROPE,
    Index.MOEX_ME: Region.EUROPE,
    Index.AEX: Region.EUROPE,
    Index.IBEX: Region.EUROPE,
    Index.FTSEMIB: Region.EUROPE,
    Index.SSMI: Region.EUROPE,
    Index.PSI: Region.EUROPE,
    Index.ATX: Region.EUROPE,
    Index.OMXS30: Region.EUROPE,
    Index.OMXC25: Region.EUROPE,
    Index.WIG20: Region.EUROPE,
    Index.BUX: Region.EUROPE,
    Index.IMOEX: Region.EUROPE,
    Index.RTSI: Region.EUROPE,

    # Asia
    Index.HSI: Region.ASIA,
    Index.STI: Region.ASIA,
    Index.BSESN: Region.ASIA,
    Index.JKSE: Region.ASIA,
    Index.KLSE: Region.ASIA,
    Index.KS11: Region.ASIA,
    Index.TWII: Region.ASIA,
    Index.N225: Region.ASIA,
    Index.SHANGHAI: Region.ASIA,
    Index.SZSE: Region.ASIA,
    Index.SET: Region.ASIA,
    Index.NSEI: Region.ASIA,
    Index.CNX200: Region.ASIA,
    Index.PSEI: Region.ASIA,
    Index.CHINA_A50: Region.ASIA,
    Index.DJSH: Region.ASIA,
    Index.INDIAVIX: Region.ASIA,

    # Africa
    Index.CASE30: Region.AFRICA,
    Index.JN0U_JO: Region.AFRICA,
    Index.FTSEJSE: Region.AFRICA,
    Index.AFR40: Region.AFRICA,
    Index.SA40: Region.AFRICA,
    Index.RAF40: Region.AFRICA,
    Index.ALT15: Region.AFRICA,

    # Middle East
    Index.TA125_TA: Region.MIDDLE_EAST,
    Index.TA35: Region.MIDDLE_EAST,
    Index.TASI: Region.MIDDLE_EAST,
    Index.TAMAYUZ: Region.MIDDLE_EAST,
    Index.BIST100: Region.MIDDLE_EAST,

    # Oceania
    Index.AXJO: Region.OCEANIA,
    Index.AORD: Region.OCEANIA,
    Index.NZ50: Region.OCEANIA,

    # Global/Currency
    Index.DX_Y_NYB: Region.GLOBAL,
    Index.USD_STRD: Region.GLOBAL,
    Index.XDB: Region.GLOBAL,
    Index.XDE: Region.GLOBAL,
    Index.XDN: Region.GLOBAL,
    Index.XDA: Region.GLOBAL,
    Index.MSCI_WORLD: Region.GLOBAL,
    Index.BUK100P: Region.GLOBAL,
}


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
