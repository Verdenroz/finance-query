from utils.dependencies import FinanceClient
from utils.retry import retry

from src.models import Quote, SimpleQuote
from src.services.quotes.fetchers import fetch_quotes, fetch_simple_quotes, scrape_quotes, scrape_simple_quotes


@retry(scrape_quotes)
async def get_quotes(finance_client: FinanceClient, symbols: list[str]) -> list[Quote]:
    """
    Asynchronously scrapes multiple quotes from a list of symbols and returns a list of Quote objects.

    * Uses the Yahoo Finance API if available, otherwise falls back to scraping.
    * Duplicate symbols should be removed before calling this function.
    * Chunks the symbols to avoid rate limiting.

    :param finance_client: Yahoo Finance client that handles API requests
    :param symbols: List of symbols
    """
    return await fetch_quotes(finance_client, symbols)


@retry(scrape_simple_quotes)
async def get_simple_quotes(finance_client: FinanceClient, symbols: list[str]) -> list[SimpleQuote]:
    """
    Asynchronously fetches multiple simple quotes from a list of symbols and returns a list of SimpleQuote objects.

    * Uses the Yahoo Finance API if available, otherwise falls back to scraping.
    * Duplicate symbols should be removed before calling this function.
    * Chunks the symbols to avoid rate limiting.

    :param finance_client: Yahoo Finance client that handles API requests
    :param symbols: List of symbols
    """
    return await fetch_simple_quotes(finance_client, symbols)
