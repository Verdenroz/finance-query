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

You need Python 3.11 or newer. We recommend using a virtual environment.

### Quick Setup with Makefile (Recommended)

For the fastest setup, use the provided Makefile:

```console
$ make install-dev
```

This will:
- Install all dependencies using `uv`
- Build required Cython extensions for technical indicators
- Set up pre-commit hooks automatically

Then you can use these commands for development:

```console
$ make help        # Show all available commands
$ make build       # Build Cython extensions (required for technical indicators)
$ make serve       # Start development server at http://localhost:8000
$ make test        # Run tests with coverage
$ make lint        # Run linting and formatting (pre-commit hooks)
$ make docs        # Serve documentation at http://localhost:8001
$ make clean       # Clean build artifacts
```

### Manual Setup

If you prefer manual setup, you'll need [uv](https://docs.astral.sh/uv/) for dependency management:

```console
$ pip install uv
$ uv sync --all-groups
$ python setup.py build_ext --inplace  # Required for technical indicators
$ pre-commit install
```

### Legacy Setup with pip

For environments without `uv`, you can still use pip:

```console
$ python -m venv venv
$ source venv/bin/activate  # On Windows: venv\Scripts\activate
$ pip install -r requirements.txt
$ pip install -r requirements/dev.txt
$ python setup.py build_ext --inplace  # Required for technical indicators
```

### Setting up environment variables

Create a `.env` file in the project root with the following variables:
See the `.env.template` file for an example.

```
# Basic configuration
REDIS_URL=redis://localhost:6379  # Optional, for caching and WebSocket support
USE_SECURITY=True  # Enable rate limiting and API key authentication
ADMIN_API_KEY=your-admin-key-here  # Admin key that bypasses rate limits
BYPASS_CACHE=False  # Set to True to disable caching during development

# Proxy configuration (optional, recommended for production)
USE_PROXY=False
PROXY_URL=
PROXY_TOKEN=  # For whitelisting IPs in proxy service

# Algolia search (optional, uses default public credentials)
ALGOLIA_APP_ID=ZTZOECLXBC
ALGOLIA_API_KEY=a3882d6ec31c0b1063ede94374616d8a

# Logging configuration
LOG_LEVEL=DEBUG  # DEBUG, INFO, WARNING, ERROR, CRITICAL
LOG_FORMAT=text  # json or text
PERFORMANCE_THRESHOLD_MS=2000  # Slow operation warning threshold

# Logo fetching configuration
DISABLE_LOGO_FETCHING=false  # Set to true to disable logo fetching
LOGO_TIMEOUT_SECONDS=1  # Timeout for logo requests
LOGO_CIRCUIT_BREAKER_THRESHOLD=5  # Failures before circuit breaker opens
LOGO_CIRCUIT_BREAKER_TIMEOUT=300  # Circuit breaker timeout in seconds
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

### Using Makefile (Recommended)

```console
$ make serve
```

### Manual Development Server

To run the application locally:

```console
$ python -m uvicorn src.main:app --reload
```

This will start the API server at `http://localhost:8000` with automatic reloading enabled.

### Docker Development

You can also use Docker:

```console
$ make docker
# Or manually:
$ docker build -t finance-query .
$ docker run -p 8000:8000 finance-query
```

#### Docker with Environment Variables

**Build-time configuration** (baked into image):
```console
$ docker build \
  --build-arg LOG_LEVEL=DEBUG \
  --build-arg LOG_FORMAT=text \
  --build-arg DISABLE_LOGO_FETCHING=true \
  --build-arg LOGO_TIMEOUT_SECONDS=2 \
  -t finance-query .
```

**Runtime configuration** (can be changed when running):
```console
$ docker run -p 8000:8000 \
  -e LOG_LEVEL=DEBUG \
  -e LOG_FORMAT=text \
  -e REDIS_URL=redis://host.docker.internal:6379 \
  -e USE_SECURITY=true \
  -e ADMIN_API_KEY=your-admin-key \
  -e USE_PROXY=true \
  -e PROXY_URL=http://proxy:8080 \
  -e PROXY_TOKEN=your-proxy-token \
  finance-query
```

**Docker Compose example**:
```yaml
version: '3.8'
services:
  api:
    build: .
    ports:
      - "8000:8000"
    environment:
      - LOG_LEVEL=INFO
      - LOG_FORMAT=json
      - REDIS_URL=redis://redis:6379
      - USE_SECURITY=true
      - ADMIN_API_KEY=your-admin-key
      - DISABLE_LOGO_FETCHING=false
      - LOGO_TIMEOUT_SECONDS=1
  redis:
    image: redis:alpine
    ports:
      - "6379:6379"
```

## How to submit changes

Open a [pull request] to submit changes to this project.

Your pull request needs to meet the following guidelines for acceptance:

- The test suite must pass without errors and warnings.
- Include unit tests for new functionality.
- If your changes add functionality, update the documentation accordingly.
- Follow the existing code style (Ruff formatting and linting).

Feel free to submit early, though—we can always iterate on this.

### Code Quality Checks

#### Using Makefile (Recommended)

```console
$ make lint
```

This runs all pre-commit hooks including:
- Ruff linting with auto-fixes
- Ruff formatting
- TOML/YAML validation
- Trailing whitespace removal

#### Manual Code Quality

To run linting and code formatting checks before committing your change:

```console
$ pre-commit run --all-files
```

It is recommended to open an issue before starting work on anything.
This will allow a chance to talk it over with the owners and validate your approach.

### Branch Workflow

FinanceQuery follows a structured branch workflow:

1. **Feature branches**: Create a branch for your feature or bugfix. Branch names should be descriptive and follow this format: `feat/your-feature-name` or `fix/issue-description`.

2. **Staging branch**: All feature branches must be merged into the `staging` branch first for integration testing.

3. **Master branch**: The `master` branch contains production-ready code. Pull requests to `master` are only accepted from the `staging` branch and are automatically restricted by our CI workflow.

This workflow ensures that code in the master branch has been properly reviewed and tested in staging before deployment to production.

```
feature/your-feature --> staging --> master
```

Please do not attempt to merge feature branches directly to master as these pull requests will be automatically rejected.

## Project architecture

Please review the [architecture document](architecture.md) to understand the project's structure before contributing.

[pull request]: https://github.com/Verdenroz/finance-query/pulls
