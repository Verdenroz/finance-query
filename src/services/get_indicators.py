from stock_indicators import indicators, Quote
from typing_extensions import List, OrderedDict

from src.schemas.analysis import (
    SMAData, EMAData, WMAData, VWAPData, RSIData, SRSIData, MACDData, STOCHData, ADXData, CCIData, AROONData,
    BBANDSData, Indicator, Analysis, OBVData, SuperTrendData, IchimokuData
)
from src.schemas.time_series import TimePeriod, Interval
from src.services.get_historical import get_historical_quotes









async def get_indicators(symbol: str, function: Indicator, period: int = 14):
    return "Not implemented yet."
