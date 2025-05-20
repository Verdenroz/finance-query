from fastapi import HTTPException

from utils.dependencies import FinanceClient

from src.models import Quote, SimpleQuote
from src.services.quotes.fetchers import fetch_quotes, fetch_simple_quotes, scrape_quotes, scrape_simple_quotes


async def get_quotes(finance_client: FinanceClient, symbols: list[str]) -> list[Quote]:
    """
    Asynchronously scrapes multiple quotes from a list of symbols and returns a list of Quote objects.

    * Uses the Yahoo Finance API if available, otherwise falls back to scraping.
    * Duplicate symbols should be removed before calling this function.
    * Chunks the symbols to avoid rate limiting.

    :param finance_client: Yahoo Finance client that handles API requests
    :param symbols: List of symbols
    """
    try:
        if quotes := await fetch_quotes(finance_client, symbols):
            return quotes
    except Exception as e:
        print(f"Error with Yahoo Finance API {e}")
        if issubclass(type(e), HTTPException):
            raise e

    # Fallback to scraping if API fails or credentials aren't available
    return await scrape_quotes(symbols)


async def get_simple_quotes(finance_client: FinanceClient, symbols: list[str]) -> list[SimpleQuote]:
    """
    Asynchronously fetches multiple simple quotes from a list of symbols and returns a list of SimpleQuote objects.

    * Uses the Yahoo Finance API if available, otherwise falls back to scraping.
    * Duplicate symbols should be removed before calling this function.
    * Chunks the symbols to avoid rate limiting.

    :param finance_client: Yahoo Finance client that handles API requests
    :param symbols: List of symbols
    """
    try:
        if quotes := await fetch_simple_quotes(finance_client, symbols):
            return quotes
    except Exception as e:
        print(f"Error with Yahoo Finance API {e}")
        if issubclass(type(e), HTTPException):
            raise e

    # Fallback to scraping if API fails or credentials aren't available
    return await scrape_simple_quotes(symbols)
