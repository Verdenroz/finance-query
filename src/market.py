from datetime import datetime, time, date
from enum import Enum
from typing import Optional

import pytz


class MarketStatus(str, Enum):
    OPEN = "Open"
    CLOSED = "Closed"
    EARLY_CLOSE = "Early Close"


class MarketSchedule:
    def __init__(self):
        # Regular trading hours (Eastern Time)
        self.regular_open = time(9, 30)  # 9:30 AM ET
        self.regular_close = time(16, 0)  # 4:00 PM ET
        self.early_close_time = time(13, 0)  # 1:00 PM ET

        # 2024 Full Holiday Closures
        self.full_holidays = {
            date(2024, 1, 1): "New Year's Day",
            date(2024, 1, 15): "Martin Luther King Jr. Day",
            date(2024, 2, 19): "Presidents Day",
            date(2024, 3, 29): "Good Friday",
            date(2024, 5, 27): "Memorial Day",
            date(2024, 6, 19): "Juneteenth",
            date(2024, 7, 4): "Independence Day",
            date(2024, 9, 2): "Labor Day",
            date(2024, 11, 28): "Thanksgiving Day",
            date(2024, 12, 25): "Christmas Day",
        }

        # 2024 Early Closures (1:00 PM ET)
        self.early_close_dates = {
            date(2024, 7, 3): "July 3rd",
            date(2024, 11, 29): "Black Friday",
            date(2024, 12, 24): "Christmas Eve",
        }

    def get_market_status(self) -> tuple[MarketStatus, Optional[str]]:
        et_tz = pytz.timezone('America/New_York')
        current_et = datetime.now(et_tz)
        current_date = current_et.date()
        current_time = current_et.time()

        # Check if it's a weekend
        if current_et.weekday() >= 5:  # 5 is Saturday, 6 is Sunday
            return MarketStatus.CLOSED, "Weekend"

        # Check if it's a holiday
        if current_date in self.full_holidays:
            return MarketStatus.CLOSED, f"Holiday: {self.full_holidays[current_date]}"

        # Check if it's an early closure day
        if current_date in self.early_close_dates:
            if current_time < self.regular_open:
                return MarketStatus.CLOSED, "Pre-market"
            elif current_time >= self.early_close_time:
                return MarketStatus.CLOSED, f"Early Close: {self.early_close_dates[current_date]}"
            else:
                return MarketStatus.EARLY_CLOSE, f"Early Close Day: {self.early_close_dates[current_date]}"

        # Regular trading day logic
        if current_time < self.regular_open:
            return MarketStatus.CLOSED, "Pre-market"
        elif current_time >= self.regular_close:
            return MarketStatus.CLOSED, "After-hours"
        else:
            return MarketStatus.OPEN, "Regular trading hours"