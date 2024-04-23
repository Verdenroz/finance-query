from selenium.common import NoSuchElementException
from datetime import datetime

from src.schemas import TimeSeries
from src.utils import TimePeriod, Interval

from selenium import webdriver
from selenium.webdriver.chrome.service import Service
from selenium.webdriver.common.by import By
from selenium.webdriver.chrome.options import Options
from webdriver_manager.chrome import ChromeDriverManager
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC


def select_time_period(driver, time_period):
    # Map time periods to the corresponding button values
    time_period_values = {
        TimePeriod.THREE_MONTHS: '3_M',
        TimePeriod.SIX_MONTHS: '6_M',
        TimePeriod.YTD: 'YTD',
        TimePeriod.YEAR: '1_Y',
        TimePeriod.FIVE_YEARS: '5_Y',
        TimePeriod.MAX: 'MAX'
    }
    # Click the dialog to make the buttons visible
    dialog = driver.find_element(By.CSS_SELECTOR, 'button.tertiary-btn.fin-size-small.menuBtn.rounded.svelte-1ndj15j')
    dialog.click()
    # Wait until the button with the appropriate value is present and then click it
    WebDriverWait(driver, 10).until(EC.presence_of_element_located((By.CSS_SELECTOR,
                                                                    f'button.tertiary-btn.fin-size-small.tw-w-full.tw-justify-center.rounded.svelte-1ndj15j[value="{time_period_values[time_period]}"]'))).click()


def select_interval(driver, interval):
    # Map intervals to the corresponding item data-values
    interval_values = {
        Interval.DAILY: '1d',
        Interval.WEEKLY: '1wk',
        Interval.MONTHLY: '1mo'
    }
    # Click the dialog to make the items visible
    dialog = driver.find_element(By.CSS_SELECTOR,
                                 'button.tertiary-btn.fin-size-small.menuBtn.tw-justify-center.rounded.rightAlign.svelte-1ndj15j')
    dialog.click()
    # Wait until the item with the appropriate data-value is present and then click it
    WebDriverWait(driver, 10).until(
        EC.presence_of_element_located((By.CSS_SELECTOR, f'div.itm[data-value="{interval_values[interval]}"]'))).click()


def scrape_historical(symbol: str, time: TimePeriod, interval: Interval):
    # Setup webdriver
    webdriver_service = Service(ChromeDriverManager().install())
    options = Options()
    options.add_argument("--headless")  # Ensure GUI is off for Docker

    # Disable images
    chrome_prefs = {"profile.managed_default_content_settings.images": 2}
    options.add_experimental_option("prefs", chrome_prefs)

    driver = webdriver.Chrome(service=webdriver_service, options=options)
    import time as t
    try:
        start = t.time()
        driver.get(f'https://finance.yahoo.com/quote/{symbol}/history')

        # Select the time period and interval
        select_time_period(driver, time)
        select_interval(driver, interval)

        # Wait for the data to load and then scrape it
        WebDriverWait(driver, 10).until(EC.presence_of_element_located((By.CSS_SELECTOR, 'table.svelte-ewueuo')))
        middle = t.time()
        print(f"Time taken to load data: {middle - start} seconds")
        data = {}
        row_index = 0
        while True:
            try:
                row = driver.find_element(By.CSS_SELECTOR, f'table.svelte-ewueuo tbody tr:nth-child({row_index + 1})')
                cells = row.find_elements(By.TAG_NAME, 'td')
                date = cells[0].text
                try:
                    open_price = float(cells[1].text)
                except ValueError:
                    # If cells[1] cannot be converted to a float, skip to the next row
                    row_index += 1
                    continue
                if date == '':
                    date = datetime.now().strftime("%b %d, %Y")
                data[date] = {
                    'open': open_price,
                    'high': float(cells[2].text),
                    'low': float(cells[3].text),
                    'adj_close': float(cells[5].text),
                    'volume': int(cells[6].text.replace(',', ''))
                }
                row_index += 1
            except NoSuchElementException:
                # No more rows
                break
        end = t.time()
        print(f"Time taken to scrape data: {end - start} seconds")
        return TimeSeries(history=data)
    finally:
        driver.quit()
