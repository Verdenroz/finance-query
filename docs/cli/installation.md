# Installation & Quick Start

## Installation Methods

### Pre-built Binaries (Recommended)

The fastest way to get started. Pre-compiled binaries are available for Linux, macOS, and Windows.

**Linux/macOS:**

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Verdenroz/finance-query/releases/latest/download/finance-query-cli-installer.sh | sh
```

**Windows (PowerShell):**

```powershell
powershell -c "irm https://github.com/Verdenroz/finance-query/releases/latest/download/finance-query-cli-installer.ps1 | iex"
```

**macOS (Homebrew):**

```bash
brew install verdenroz/tap/fq
```

### From Cargo

Install from crates.io (requires Rust):

```bash
cargo install finance-query-cli
```

### From Source

Build from source:

```bash
git clone https://github.com/Verdenroz/finance-query
cd finance-query/finance-query-cli
cargo install --path .
```

## Verify Installation

```bash
fq --version
```

## Quick Start

Get a stock quote:

```bash
fq quote AAPL
```

View multiple quotes:

```bash
fq quote AAPL MSFT GOOGL TSLA
```

Stream live prices:

```bash
fq stream AAPL TSLA
```

View an interactive chart:

```bash
fq chart AAPL
```

Launch the dashboard:

```bash
fq dashboard
```

For detailed help, run:

```bash
fq --help
fq <command> --help
```

## Data Storage

The CLI stores alerts, watchlists, and configuration in:

- **Linux/macOS:** `~/.local/share/fq/`
- **Windows:** `%APPDATA%\fq\`
