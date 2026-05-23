# Contributing to FinanceQuery

Thank you for your interest in contributing to FinanceQuery. This document covers everything you need to get started.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Reporting Bugs](#reporting-bugs)
- [Reporting Security Vulnerabilities](#reporting-security-vulnerabilities)
- [Suggesting Features](#suggesting-features)
- [Development Setup](#development-setup)
- [Building and Testing](#building-and-testing)
- [Code Style](#code-style)
- [Submitting a Pull Request](#submitting-a-pull-request)
- [Commit Message Format](#commit-message-format)
- [Release Process](#release-process)

---

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating you agree to abide by its terms. Report violations to **harveytseng2@gmail.com**.

---

## Reporting Bugs

Before filing a bug, search [existing issues](https://github.com/Verdenroz/finance-query/issues) to avoid duplicates.

Open a [new issue](https://github.com/Verdenroz/finance-query/issues/new/choose) and include:

- The component affected (library / server / CLI / MCP server).
- The version (`cargo metadata`, `fq --version`, or the crate version from `Cargo.toml`).
- Steps to reproduce, including a minimal code example where possible.
- Actual vs. expected behavior.
- Rust version (`rustc --version`) and OS.

---

## Reporting Security Vulnerabilities

**Do not open a public issue for security vulnerabilities.** See [SECURITY.md](SECURITY.md) for the private disclosure process.

---

## Suggesting Features

Open a [new issue](https://github.com/Verdenroz/finance-query/issues/new/choose) with the `enhancement` label. Describe:

- The problem you are trying to solve.
- Your proposed solution and any alternatives you considered.
- Whether you are willing to implement it (helps prioritize).

For large changes (new providers, new public API surface, architectural shifts), open an issue for discussion before writing code — this avoids wasted effort if the direction does not fit the project.

---

## Development Setup

**Prerequisites:**

- Rust stable toolchain — install via [rustup](https://rustup.rs/)
- `cargo` (comes with Rust)
- Docker (optional, for testing the server image)
- Python ≥ 3.8 + pip (optional, for the MkDocs documentation site)

**Clone and build:**

```bash
git clone https://github.com/Verdenroz/finance-query.git
cd finance-query
cargo build
```

**Optional API keys** (set in your shell environment for integration tests):

| Variable | Provider |
|----------|---------|
| `FMP_API_KEY` | Financial Modeling Prep |
| `POLYGON_API_KEY` | Polygon.io |
| `ALPHAVANTAGE_API_KEY` | Alpha Vantage |
| `FRED_API_KEY` | FRED |
| `EDGAR_EMAIL` | SEC EDGAR (any valid email) |

Yahoo Finance and CoinGecko require no keys.

---

## Building and Testing

```bash
make help              # list all available targets
make lint              # fmt + clippy + check (same as CI)
make fix               # auto-fix formatting and clippy issues
make test-fast         # unit tests only (no network)
make test              # all tests including network integration tests
make audit             # cargo-deny supply-chain audit
make docs              # build and serve MkDocs docs at localhost:8080
```

Or directly with Cargo:

```bash
cargo test -p finance-query                      # library unit tests
cargo test -- --ignored                          # network integration tests
cargo test --doc --all-features                  # doctests
cargo clippy --all-targets --all-features -- -D warnings
```

**Test categories:**

- **Unit tests** — fast, no network, run by default.
- **Integration tests** — marked `#[ignore = "requires network access"]`, make real API calls, require provider env vars.
- **Doctests** — `no_run` examples in the public API; compiled with `--all-features`.

All tests must pass before a PR is merged.

---

## Code Style

- Run `cargo fmt --all` before committing. CI enforces formatting.
- Run `cargo clippy --all-targets --all-features -- -D warnings`. All warnings are errors in CI.
- Public API items must have doc comments (`///`). See existing items for tone and style.
- Every `rust` code block in `docs/library/*.md` must have a corresponding test in `tests/doc_<filename>.rs`.
- No `unwrap()` or `expect()` in library code outside of tests — use `?` and `FinanceError`.
- Prefer small, focused commits. One logical change per commit.

---

## Submitting a Pull Request

1. **Fork** the repository and create a branch from `master`:
   ```bash
   git checkout -b feat/my-feature
   ```
2. Make your changes, including tests for any new behaviour.
3. Run `make lint` and `make test-fast` locally. Fix any issues.
4. Push your branch and open a PR against `master`.
5. Fill in the PR description: what changed, why, and how you tested it.
6. Address any review comments. All CI checks must pass before merge.

PRs that add features or fix bugs should include tests. PRs that only touch documentation or CI do not require new tests.

For the `finance-query` library, breaking API changes require a semver major bump and a CHANGELOG entry. Discuss breaking changes in an issue first.

---

## Commit Message Format

```
<type>: <short description>

<optional body — explain the why, not the what>
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`, `ci`, `security`

Examples:
```
feat: add Polygon.io chart adapter
fix: handle empty response from Yahoo fundamentals endpoint
security: pin GitHub Actions to commit SHAs
```

---

## Release Process

Releases are managed by the project maintainer. If you believe a bug fix warrants a release, comment on the relevant issue.

Library releases (`finance-query` crate) follow semver. CLI releases (`fq`) are versioned independently. Both are published automatically via CI when a version tag is pushed.
