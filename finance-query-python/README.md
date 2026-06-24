# finance-query-py

Python bindings for [finance-query](https://github.com/Verdenroz/finance-query) — a fast,
type-safe Rust library for financial data.

## Install

`pip install finance-query-py`

## Quickstart

```python
import asyncio
from finance_query import Ticker

async def main():
    ticker = await Ticker.new("AAPL")
    quote = await ticker.quote()
    print(quote.current_price)

asyncio.run(main())
```

See the [full documentation](https://finance-query.readthedocs.io/python/).

## Development

```bash
# Build the extension into the local venv
make develop

# Run the test suite (network tests skipped by default)
make test

# Regenerate type stubs (semi-automatic; async sigs are hand-polished)
make stubs
```

Local network tests need access to Yahoo Finance. Use `pytest -m network` to run them; they're deselected by default.

## Releasing

1. Bump the version in `Cargo.toml` and `pyproject.toml` to match the parent
   `finance-query` crate version. They should always agree.
2. Tag and push:
   ```bash
   VERSION=$(cargo pkgid -p finance-query-python | cut -d# -f2)
   git tag "finance-query-py-v${VERSION}"
   git push origin --tags
   ```
3. CI (`.github/workflows/python-wheels.yml`) builds 5 wheels (linux x86_64,
   linux aarch64, macos x86_64, macos aarch64, windows x86_64) plus an sdist
   and publishes to PyPI via trusted publishing.

### Trusted publishing setup (one-time)

Before the first release tag will succeed, configure PyPI trusted publishing:

1. Sign in to PyPI as the project owner.
2. Visit https://pypi.org/manage/account/publishing/ → "Add a new pending publisher".
3. Enter:
   - PyPI Project Name: `finance-query-py`
   - Owner: `Verdenroz`
   - Repository name: `finance-query`
   - Workflow name: `python-wheels.yml`
   - Environment name: `pypi`
4. Save. The workflow's `publish` job will now upload to PyPI without needing an API token.
