from enum import Enum


class TimePeriod(Enum):
    THREE_MONTHS = "3M"
    SIX_MONTHS = "6M"
    YTD = "YTD"
    YEAR = "1Y"
    FIVE_YEARS = "5Y"
    MAX = "max"


class Interval(Enum):
    DAILY = "daily"
    WEEKLY = "weekly"
    MONTHLY = "monthly"
