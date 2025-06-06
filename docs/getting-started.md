# Getting Started with FinanceQuery

This guide will help you set up and run FinanceQuery on your system.

## Requirements

Before getting started with FinanceQuery, ensure your system meets the following requirements:

### System Requirements

- **Python**: 3.11 or higher (Python 3.12 recommended)
- **Operating System**: Linux, macOS, or Windows
- **Memory**: Minimum 512MB RAM (1GB+ recommended for production)

FinanceQuery requires several system-level dependencies for optimal performance:

=== "Linux (Ubuntu/Debian)"
    ```bash
    sudo apt-get update && sudo apt-get install -y \
        ca-certificates \
        libkrb5-dev \
        libicu-dev \
        zlib1g-dev \
        openssl \
        gcc \
        python3-dev
    ```

=== "macOS"
    ```bash
    # Using Homebrew
    brew install openssl krb5 icu4c
    ```

=== "Windows"
    ```powershell
    # Required components
    - Visual Studio Build Tools or Visual Studio Community (for C++ compilation)
    - OpenSSL (typically included with Python installation)
    ```

### Dependency Management

FinanceQuery uses Poetry for dependency management. A `requirements.txt` file is also provided for environments where Poetry isn't available.

```bash
# Using Poetry (recommended)
poetry install

# Using pip
pip install -r requirements.txt
```

### Optional Requirements

#### Redis (Recommended for Production)

- **Redis** server for caching and improved performance
- Installation: [Redis Quick Start Guide](https://redis.io/docs/getting-started/)

#### Proxy Support (Optional)

- For enhanced data fetching reliability in production environments
- Configure via environment variables (see Configuration section)

## Quick Start

### Installation

Create and activate a virtual environment and then install the dependencies:

```bash
# Clone the project
git clone https://github.com/Verdenroz/finance-query.git
cd finance-query

# Install dependencies
poetry install

# Cythonize files for performance
python setup.py build_ext --inplace

# Start the server
uvicorn src.main:app --reload
```

## Testing the API

Once the server is running, you can access:

- API Documentation: [http://localhost:8000/docs](http://localhost:8000/docs)
- Alternative Documentation: [http://localhost:8000/redoc](http://localhost:8000/redoc)

### Making Your First Request

Try getting stock information using curl:

```bash
curl -X GET "http://localhost:8000/v1/simple-quotes?symbols=AAPL"
```

Or using Python:

```python
import requests

response = requests.get("http://localhost:8000/v1/simple-quotes", params={"symbols": "AAPL"})
print(response.json())
```
