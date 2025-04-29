from datetime import datetime, time

import pytest
import pytz
from freezegun import freeze_time

from src.market import MarketSchedule, MarketStatus


def test_market_hours_endpoint(test_client):
    """Test the /hours endpoint"""
    response = test_client.get("/hours")
    assert response.status_code == 200

    data = response.json()
    assert "status" in data
    assert "reason" in data
    assert "timestamp" in data

    # Verify timestamp format and timezone
    timestamp = datetime.fromisoformat(data["timestamp"].replace("Z", "+00:00"))

    # Convert to pytz timezone for comparison
    utc_timestamp = timestamp.astimezone(pytz.UTC)
    assert utc_timestamp.tzinfo == pytz.UTC


class TestMarketSchedule:
    """Test suite for MarketSchedule class"""

    def test_init(self):
        """Test MarketSchedule initialization"""
        schedule = MarketSchedule()
        assert schedule.regular_open == time(9, 30)
        assert schedule.regular_close == time(16, 0)
        assert schedule.early_close_time == time(13, 0)
        assert isinstance(schedule.full_holidays, dict)
        assert isinstance(schedule.early_close_dates, dict)

    @pytest.mark.parametrize(
        "test_datetime,expected_status,expected_reason",
        [
            # Weekend tests
            ("2024-03-23 12:00:00", MarketStatus.CLOSED, "Weekend"),  # Saturday
            ("2024-03-24 12:00:00", MarketStatus.CLOSED, "Weekend"),  # Sunday
            # Regular trading day tests
            ("2024-03-22 09:29:00", MarketStatus.CLOSED, "Pre-market"),  # Before open
            ("2024-03-22 09:30:00", MarketStatus.OPEN, "Regular trading hours"),  # At open
            ("2024-03-22 12:00:00", MarketStatus.OPEN, "Regular trading hours"),  # During day
            ("2024-03-22 16:00:00", MarketStatus.CLOSED, "After-hours"),  # At close
            ("2024-03-22 16:01:00", MarketStatus.CLOSED, "After-hours"),  # After close
            # Holiday tests
            ("2024-01-01 12:00:00", MarketStatus.CLOSED, "Holiday: New Year's Day"),
            ("2024-12-25 12:00:00", MarketStatus.CLOSED, "Holiday: Christmas Day"),
            # Early close day tests
            ("2024-12-24 09:29:00", MarketStatus.CLOSED, "Pre-market"),  # Christmas Eve before open
            ("2024-12-24 10:00:00", MarketStatus.EARLY_CLOSE, "Early Close Day: Christmas Eve"),  # During trading
            ("2024-12-24 13:00:00", MarketStatus.CLOSED, "Early Close: Christmas Eve"),  # After early close
        ],
    )
    def test_market_status(self, test_datetime, expected_status, expected_reason):
        """Test market status determination for various scenarios"""
        dt = datetime.fromisoformat(test_datetime)
        et_tz = pytz.timezone("America/New_York")

        # Create a localized datetime in Eastern Time
        localized_dt = et_tz.localize(dt)

        with freeze_time(localized_dt):
            schedule = MarketSchedule()
            status, reason = schedule.get_market_status()
            assert status == expected_status
            assert reason == expected_reason

    def test_holiday_calculations(self):
        """Test holiday date calculations"""
        # Freeze time to 2024 for consistent holiday calculations
        with freeze_time("2024-01-01"):
            schedule = MarketSchedule()
            # Test specific 2024 holidays
            assert datetime(2024, 1, 1).date() in schedule.full_holidays  # New Year's
            assert datetime(2024, 1, 15).date() in schedule.full_holidays  # MLK Day
            assert datetime(2024, 2, 19).date() in schedule.full_holidays  # Presidents Day
            assert datetime(2024, 5, 27).date() in schedule.full_holidays  # Memorial Day
            assert datetime(2024, 6, 19).date() in schedule.full_holidays  # Juneteenth
            assert datetime(2024, 7, 4).date() in schedule.full_holidays  # Independence Day
            assert datetime(2024, 9, 2).date() in schedule.full_holidays  # Labor Day
            assert datetime(2024, 11, 28).date() in schedule.full_holidays  # Thanksgiving

    def test_early_close_dates(self):
        """Test early close date calculations"""
        with freeze_time("2024-01-01"):
            schedule = MarketSchedule()

            # Test 2024 early close dates
            assert datetime(2024, 7, 3).date() in schedule.early_close_dates  # July 3rd
            assert datetime(2024, 11, 29).date() in schedule.early_close_dates  # Black Friday
            assert datetime(2024, 12, 24).date() in schedule.early_close_dates  # Christmas Eve

    def test_weekend_adjustments(self):
        """Test that fixed‑date holidays falling on weekends get observed on the correct weekday"""

        # Independence Day 2026 falls on Saturday → observed Friday, July 3,2026
        with freeze_time("2026-07-03"):
            schedule = MarketSchedule()
            original = datetime(2026, 7, 4).date()
            adjusted = datetime(2026, 7, 3).date()

            # The 4th itself should be removed...
            assert original not in schedule.full_holidays
            # ...and the 3rd should carry the "Independence Day" label
            assert adjusted in schedule.full_holidays
            assert schedule.full_holidays[adjusted] == "Independence Day"

        # New Year's Day 2023 falls on Sunday → observed Monday, January 2,2023
        with freeze_time("2023-01-01"):
            schedule = MarketSchedule()
            original = datetime(2023, 1, 1).date()
            adjusted = datetime(2023, 1, 2).date()

            # The 1st itself should be removed...
            assert original not in schedule.full_holidays
            # ...and the 2nd should carry the "New Year's Day" label
            assert adjusted in schedule.full_holidays
            assert schedule.full_holidays[adjusted] == "New Year's Day"
