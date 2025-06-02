# Contributor Guide

Thank you for your interest in improving FinanceQuery.
This project is open-source under the [MIT license] and
welcomes contributions in the form of bug reports, feature requests, and pull requests.

Here is a list of important resources for contributors:

- [Source Code]
- [Demo]
- [Documentation]
- [Issue Tracker]
- [Code of Conduct]

[mit license]: https://opensource.org/licenses/MIT
[documentation]: https://verdenroz.github.io/finance-query/
[demo]: https://financequery.apidocumentation.com/reference
[source code]: https://github.com/Verdenroz/finance-query
[issue tracker]: https://github.com/Verdenroz/finance-query/issues
[code of conduct]: https://github.com/Verdenroz/finance-query/CODE_OF_CONDUCT.md

## How to report a bug

Report bugs on the [Issue Tracker].

When filing an issue, make sure to answer these questions:

- Which operating system and Python version are you using?
- Which version of this project are you using?
- What did you do?
- What did you expect to see?
- What did you see instead?

The best way to get your bug fixed is to provide a test case,
and/or steps to reproduce the issue.

## How to request a feature

Request features on the [Issue Tracker].

## How to set up your development environment

You need Python 3.11 or newer. We recommend using a virtual environment:

```console
$ python -m venv venv
$ source venv/bin/activate  # On Windows: venv\Scripts\activate
```

### Installation with Poetry (recommended)

Alternatively, you can use [Poetry](https://python-poetry.org/) for dependency management:

```console
$ pip install poetry
$ poetry install --with dev
```

This will install all dependencies defined in the pyproject.toml file, including development dependencies.

To activate the Poetry virtual environment:

```console
$ eval $(poetry env activate)

# Or on Windows:
Invoke-Expression (poetry env activate)
```

### Installation with pip

Install the project dependencies:

```console
$ pip install -r requirements.txt
```

For development, you'll also need the development dependencies:

```console
$ pip install -r requirements/dev.txt
```

### Setting up environment variables

Create a `.env` file in the project root with the following variables:
See the `.env.template` file for an example.

```
# Basic configuration
REDIS_URL=redis://localhost:6379  # Optional, for caching and WebSocket support
USE_SECURITY=True  # Enable rate limiting
BYPASS_CACHE=False  # Set to True to disable caching during development

# Proxy configuration (optional)
USE_PROXY=False
PROXY_URL=
PROXY_TOKEN=
```

## How to test the project

Run the full test suite:

```console
$ pytest
```

You can also run specific test files:

```console
$ pytest tests/test_quotes.py
```

Unit tests are located in the _tests_ directory,
and are written using the [pytest] testing framework.

[pytest]: https://pytest.readthedocs.io/

## Local development

To run the application locally:

```console
$ python -m uvicorn src.main:app --reload
```

This will start the API server at `http://localhost:8000` with automatic reloading enabled.

You can also use Docker:

```console
$ docker build -t finance-query .
$ docker run -p 8000:8000 finance-query
```

## How to submit changes

Open a [pull request] to submit changes to this project.

Your pull request needs to meet the following guidelines for acceptance:

- The test suite must pass without errors and warnings.
- Include unit tests for new functionality.
- If your changes add functionality, update the documentation accordingly.
- Follow the existing code style (Black formatting, isort imports).

Feel free to submit early, though—we can always iterate on this.

To run linting and code formatting checks before committing your change, you can use:

```console
$ pre-commit run --all-files
```

It is recommended to open an issue before starting work on anything.
This will allow a chance to talk it over with the owners and validate your approach.

## Project architecture

Please review the [architecture document](architecture.md) to understand the project's structure before contributing.

[pull request]: https://github.com/Verdenroz/finance-query/pulls
