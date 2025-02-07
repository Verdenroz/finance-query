from fastapi import HTTPException

from src.models import Quote, SimpleQuote
from src.services.quotes import fetch_quotes, scrape_quotes, fetch_simple_quotes, scrape_simple_quotes


async def get_quotes(symbols: list[str], cookies: str, crumb: str) -> list[Quote]:
    """
    Asynchronously scrapes multiple quotes from a list of symbols and returns a list of Quote objects.

    * Fallback to scraping if cookies and crumb are not available or on error.
    * Duplicate symbols should be removed before calling this function.
    * Chunks the symbols to avoid rate limiting.

    :param symbols: List of symbols
    :param cookies: Authentication cookies
    :param crumb: Authentication crumb
    """
    try:
        if cookies and crumb:
            return await fetch_quotes(symbols, cookies, crumb)
    except HTTPException:
        raise  # Re-raise the exception
    except ValueError as e:
        print(f"Error with Yahoo Finance credentials: {e}")
    except Exception as e:
        print(f"Error fetching quotes from API: {e}")

    # Fallback to scraping if API fails or credentials aren't available
    return await scrape_quotes(symbols)


async def get_simple_quotes(symbols: list[str], cookies: str, crumb: str) -> list[SimpleQuote]:
    """
    Asynchronously fetches multiple simple quotes from a list of symbols and returns a list of SimpleQuote objects.

    * Fallback to scraping if cookies and crumb are not available or on error.
    * Duplicate symbols should be removed before calling this function.
    * Chunks the symbols to avoid rate limiting.

    :param symbols: List of symbols
    :param cookies: Authentication cookies
    :param crumb: Authentication crumb
    """
    try:
        if cookies and crumb:
            return await fetch_simple_quotes(symbols, cookies, crumb)
    except HTTPException:
        raise  # Re-raise the exception
    except ValueError as e:
        print(f"Error with Yahoo Finance credentials: {e}")
    except Exception as e:
        print(f"Error fetching quotes from API: {e}")

    # Fallback to scraping if API fails or credentials aren't available
    return await scrape_simple_quotes(symbols)
