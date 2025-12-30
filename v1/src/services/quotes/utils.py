from concurrent.futures import ThreadPoolExecutor
from datetime import datetime

import psutil
from lxml import etree, html

from src.utils.dependencies import get_logo
from src.utils.logging import get_logger

# Initialize thread pool and logger
thread_pool = ThreadPoolExecutor(max_workers=psutil.cpu_count(logical=True) * 2)
logger = get_logger(__name__)


def get_adaptive_chunk_size() -> int:
    """Calculate adaptive chunk size based on system resources."""
    cpu_count = psutil.cpu_count()
    memory_info = psutil.virtual_memory()
    available_memory = memory_info.available

    base_chunk_size = 10
    chunk_size = base_chunk_size * cpu_count * (available_memory // (512 * 1024 * 1024))
    return max(base_chunk_size, min(chunk_size, 100))


def is_within_pre_market_time(pre_market_time: int) -> bool:
    return int(datetime.now().timestamp()) <= pre_market_time


def is_within_post_market_time(post_market_time: int) -> bool:
    return int(datetime.now().timestamp()) >= post_market_time


def format_date(date_string: str) -> str | None:
    if not date_string:
        return None
    try:
        date = datetime.strptime(date_string, "%Y-%m-%d")
        return date.strftime("%b %d, %Y")
    except (ValueError, TypeError):
        return None


def format_percent(value) -> str | None:
    """
    Accepts either the usual Yahoo dict **or** a bare float / int.
    Converts 0.1234 → '12.34%' and 12.34 → '12.34%'.
    """
    if value is None:
        return None

    # Yahoo’s structured form: {"raw": 0.1234, "fmt": "12.34%"}
    if isinstance(value, dict):
        raw = value.get("raw")
        if raw is None:
            return None
        # raw is a *fraction* (0.1234) → multiply by 100
        return f"{raw * 100:.2f}%"

    # Bare numeric from quote endpoint (already in percent units)
    if isinstance(value, int | float):
        return f"{value:.2f}%"

    # Already a string
    return str(value)


def format_change(value: str) -> str:
    if value and value[0] not in {"-", "+"}:
        return f"+{value}"
    return value


def get_fmt(obj, key) -> str | None:
    if not obj or not isinstance(obj, dict):
        return None
    value = obj.get(key, {})
    if isinstance(value, dict):
        value = value.get("fmt", None)
    return value


def get_raw(obj, key) -> str | None:
    if not obj or not isinstance(obj, dict):
        return None
    value = obj.get(key, {})
    if isinstance(value, dict):
        value = value.get("raw", None)
    return value


def get_morningstar_risk_rating(raw_risk: int) -> str | None:
    risk_mapping = {
        1: "Below Average",
        2: "Average",
        3: "Above Average",
        4: "High",
    }
    return risk_mapping.get(raw_risk, None)


def parse_tree(html_content: str) -> etree.ElementTree:
    """
    Parse HTML content in a separate thread to avoid blocking the event loop.
    """
    return html.fromstring(html_content)


async def _scrape_price_data(tree: etree.ElementTree) -> tuple:
    """
    Scrape the price data from the HTML content using XPath and format the data.

    :param tree: The parsed HTML tree
    :return: Regular price, change, percent change, and post price as a tuple
    """
    try:
        # XPath expressions
        price_xpath = "//span[@data-testid='qsp-price']/text()"
        change_xpath = "//span[@data-testid='qsp-price-change']/text()"
        percent_change_xpath = "//span[@data-testid='qsp-price-change-percent']/text()"
        post_price_xpath = "//fin-streamer[@data-testid='qsp-post-price']/@data-value"
        pre_price_xpath = "//fin-streamer[@data-testid='qsp-pre-price']/@data-value"

        # Extract values
        regular_price = tree.xpath(price_xpath)
        regular_change = tree.xpath(change_xpath)
        regular_percent_change = tree.xpath(percent_change_xpath)
        post_price = tree.xpath(post_price_xpath)
        pre_price = tree.xpath(pre_price_xpath)

        # Format values
        regular_price = regular_price[0].strip() if regular_price else None
        regular_change = regular_change[0].strip() if regular_change else None
        regular_percent_change = regular_percent_change[0].strip().replace("(", "").replace(")", "") if regular_percent_change else None
        post_price = post_price[0].strip() if post_price else None
        pre_price = pre_price[0].strip() if pre_price else None

        return regular_price, regular_change, regular_percent_change, pre_price, post_price

    except Exception as e:
        logger.error("Failed to scrape prices", extra={"error": str(e)}, exc_info=True)
        return None, None, None, None, None


async def _scrape_general_info(tree: etree.ElementTree):
    """
    Scrape misc. info from the tree object and formats the data

    :param tree: The parsed HTML tree
    :return: A tuple of the scraped data
    """
    try:
        ul_xpath = './/div[@data-testid="quote-statistics"]/ul'
        list_items_xpath = ".//li"
        label_xpath = './/span[contains(@class, "label")]/text()'
        value_xpath = './/span[contains(@class, "value")]/fin-streamer/@data-value | .//span[contains(@class, "value")]/text()'

        ul_element = tree.xpath(ul_xpath)
        if not ul_element:
            return {}

        ul_element = ul_element[0]
        list_items = ul_element.xpath(list_items_xpath)

        # Extract all data in one pass
        data = {}
        for item in list_items:
            label = item.xpath(label_xpath)[0].strip()
            value_elements = item.xpath(value_xpath)
            value = value_elements[0].strip() if value_elements else None
            data[label] = value

        # Process the extracted data
        days_range = data.get("Day's Range", "")
        low, high = days_range.split(" - ") if " - " in days_range else (None, None)

        fifty_two_week_range = data.get("52 Week Range", "")
        year_low, year_high = fifty_two_week_range.split(" - ") if " - " in fifty_two_week_range else (None, None)

        volume_str = data.get("Volume", "")
        avg_volume_str = data.get("Avg. Volume", "")

        volume = int(volume_str.replace(",", "")) if volume_str and volume_str.replace(",", "").isdigit() else None
        avg_volume = int(avg_volume_str.replace(",", "")) if avg_volume_str and avg_volume_str.replace(",", "").isdigit() else None

        forward_dividend_yield = data.get("Forward Dividend & Yield", "")
        if forward_dividend_yield and any(char.isdigit() for char in forward_dividend_yield):
            dividend, yield_percent = forward_dividend_yield.replace("(", "").replace(")", "").split()
        else:
            dividend, yield_percent = None, data.get("Yield")

        return {
            "open": data.get("Open"),
            "high": high,
            "low": low,
            "year_high": year_high,
            "year_low": year_low,
            "volume": volume,
            "avg_volume": avg_volume,
            "market_cap": data.get("Market Cap (intraday)"),
            "beta": data.get("Beta (5Y Monthly)"),
            "pe": data.get("PE Ratio (TTM)"),
            "eps": data.get("EPS (TTM)"),
            "earnings_date": data.get("Earnings Date"),
            "dividend": dividend,
            "dividend_yield": yield_percent,
            "ex_dividend": data.get("Ex-Dividend Date") if data.get("Ex-Dividend Date") != "--" else None,
            "net_assets": data.get("Net Assets"),
            "nav": data.get("NAV"),
            "expense_ratio": data.get("Expense Ratio (net)"),
            "category": data.get("Category"),
            "last_capital_gain": data.get("Last Cap Gain"),
            "morningstar_rating": data.get("Morningstar Rating", "").split()[0] if data.get("Morningstar Rating") else None,
            "morningstar_risk_rating": data.get("Morningstar Risk Rating"),
            "holdings_turnover": data.get("Holdings Turnover"),
            "last_dividend": data.get("Last Dividend"),
            "inception_date": data.get("Inception Date"),
        }

    except Exception as e:
        logger.error("Failed to scrape general info", extra={"error": str(e)}, exc_info=True)
        return {}


async def _scrape_company_info(tree: etree.ElementTree, symbol: str):
    """
    Scrape the sector and industry data from the tree object

    :return: sector, industry, about, employees, logo as a tuple
    """
    try:
        container_xpath = "/html/body/div[2]/main/section/section/section/article/section[2]/div/div/div[2]/div"
        xpaths = {
            "about": './/div[contains(@class, "description")]/p/text()',
            "website": './/div[contains(@class, "description")]/a[contains(@data-ylk, "business-url")]/@href',
            "sector": './/div[contains(@class, "infoSection")][h3[text()="Sector"]]/p/a/text()',
            "industry": './/div[contains(@class, "infoSection")][h3[text()="Industry"]]/p/a/text()',
            "employees": './/div[contains(@class, "infoSection")][h3[text()="Full Time Employees"]]/p/text()',
        }

        container_element = tree.xpath(container_xpath)
        if not container_element:
            return {}

        container_element = container_element[0]
        results = {}

        for key, xpath in xpaths.items():
            elements = container_element.xpath(xpath)
            results[key] = elements[0].strip() if elements else None

        # Get logo asynchronously if website exists, but don't block if it fails
        logo = None
        if results.get("website"):
            try:
                logo = await get_logo(symbol=symbol, url=results["website"])
            except Exception as e:
                logger.debug(f"Logo fetch failed for {symbol}: {str(e)}")

        return {
            "sector": results["sector"],
            "industry": results["industry"],
            "about": results["about"],
            "employees": results["employees"],
            "logo": logo,
        }

    except Exception as e:
        logger.error("Failed to scrape company info", extra={"error": str(e)}, exc_info=True)
        return {}


async def _scrape_performance(tree: etree.ElementTree):
    """
    Scrape the performance data from the parsed HTML tree using XPath.

    :param tree: Parsed HTML tree
    :return: YTD, 1 year, 3 year, and 5 year returns as a tuple
    """
    try:
        container_xpath = "/html/body/div[2]/main/section/section/section/article/section[5]"
        return_xpaths = {
            "ytd_return": './/section[1]//div[contains(@class, "perf")]/text()',
            "year_return": './/section[2]//div[contains(@class, "perf")]/text()',
            "three_year_return": './/section[3]//div[contains(@class, "perf")]/text()',
            "five_year_return": './/section[4]//div[contains(@class, "perf")]/text()',
        }

        container_element = tree.xpath(container_xpath)
        if not container_element:
            return {}

        container_element = container_element[0]
        results = {}

        for key, xpath in return_xpaths.items():
            elements = container_element.xpath(xpath)
            results[key] = elements[0].strip() if elements else None

        return results

    except Exception as e:
        logger.error("Failed to scrape performance data", extra={"error": str(e)}, exc_info=True)
        return {}
