from datetime import datetime, time, date, timedelta
from enum import Enum
from typing import Optional

import pytz


class MarketStatus(str, Enum):
    OPEN = "Open"
    CLOSED = "Closed"
    EARLY_CLOSE = "Early Close"


class MarketSchedule:
    def __init__(self):
        self.year = datetime.now().year

        # Regular trading hours (Eastern Time)
        self.regular_open = time(9, 30)  # 9:30 AM ET
        self.regular_close = time(16, 0)  # 4:00 PM ET
        self.early_close_time = time(13, 0)  # 1:00 PM ET

        # Calculate holidays for the specified year
        self._calculate_holidays()

    def _get_nth_weekday_of_month(self, year: int, month: int, weekday: int, n: int) -> date:
        """Get the nth occurrence of a weekday in a given month.

        Args:
            year: The year
            month: The month (1-12)
            weekday: The day of the week (0=Monday, 6=Sunday)
            n: The occurrence (1=first, 2=second, etc.)
        """
        first_day = date(year, month, 1)
        first_weekday = first_day.weekday()

        # Calculate days until first occurrence
        days_until = (weekday - first_weekday) % 7
        # Add (n-1) weeks to get to the nth occurrence
        target_day = first_day + timedelta(days=days_until + (n - 1) * 7)
        return target_day

    def _get_good_friday(self, year: int) -> date:
        """Calculate Good Friday for a given year using Butcher's Algorithm."""
        a = year % 19
        b = year // 100
        c = year % 100
        d = b // 4
        e = b % 4
        f = (b + 8) // 25
        g = (b - f + 1) // 3
        h = (19 * a + b - d - g + 15) % 30
        i = c // 4
        k = c % 4
        l = (32 + 2 * e + 2 * i - h - k) % 7
        m = (a + 11 * h + 22 * l) // 451

        month = (h + l - 7 * m + 114) // 31
        day = ((h + l - 7 * m + 114) % 31) + 1

        # Easter Sunday
        easter = date(year, month, day)
        # Good Friday is two days before Easter
        return easter - timedelta(days=2)

    def _calculate_holidays(self):
        """Calculate all market holidays for the year."""
        self.full_holidays = {
            # Fixed date holidays
            date(self.year, 1, 1): "New Year's Day",
            date(self.year, 6, 19): "Juneteenth",
            date(self.year, 7, 4): "Independence Day",
            date(self.year, 12, 25): "Christmas Day",

            # Nth weekday of month holidays
            self._get_nth_weekday_of_month(self.year, 1, 0, 3): "Martin Luther King Jr. Day",  # 3rd Monday in January
            self._get_nth_weekday_of_month(self.year, 2, 0, 3): "Presidents Day",  # 3rd Monday in February
            self._get_nth_weekday_of_month(self.year, 5, 0, -1): "Memorial Day",  # Last Monday in May
            self._get_nth_weekday_of_month(self.year, 9, 0, 1): "Labor Day",  # 1st Monday in September
            self._get_nth_weekday_of_month(self.year, 11, 4, 4): "Thanksgiving Day",  # 4th Thursday in November

            # Special calculations
            self._get_good_friday(self.year): "Good Friday",
        }

        # Calculate early closure dates
        self.early_close_dates = {
            date(self.year, 7, 3): "July 3rd",  # Day before Independence Day
            self._get_nth_weekday_of_month(self.year, 11, 4, 4) + timedelta(days=1): "Black Friday",
            # Day after Thanksgiving
            date(self.year, 12, 24): "Christmas Eve",
        }

        # Handle weekend adjustments for fixed-date holidays
        self._adjust_weekend_holidays()

    def _adjust_weekend_holidays(self):
        """Adjust fixed-date holidays that fall on weekends."""
        weekend_adjustments = {}
        weekend_removals = []

        for holiday_date, holiday_name in self.full_holidays.items():
            # Skip holidays that are already calculated to avoid weekends
            if holiday_name in ["Martin Luther King Jr. Day", "Presidents Day",
                                "Memorial Day", "Labor Day", "Thanksgiving Day",
                                "Good Friday"]:
                continue

            weekday = holiday_date.weekday()
            if weekday == 5:  # Saturday
                weekend_adjustments[holiday_date - timedelta(days=1)] = holiday_name
                weekend_removals.append(holiday_date)
            elif weekday == 6:  # Sunday
                weekend_adjustments[holiday_date + timedelta(days=1)] = holiday_name
                weekend_removals.append(holiday_date)

        # Update the holidays dict
        for date_to_remove in weekend_removals:
            del self.full_holidays[date_to_remove]
        self.full_holidays.update(weekend_adjustments)

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