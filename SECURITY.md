# Security Policy

FinanceQuery is a Rust financial-data library (`finance-query` on crates.io) plus
a server, CLI (`fq`), and MCP server built on top of it. This policy covers all
of those components.

## Supported Versions

Security fixes land on the latest released line only. If you are on an older
version, the fix is to upgrade.

| Component                   | Version          | Supported          |
| --------------------------- | ---------------- | ------------------ |
| `finance-query` (library)   | 2.6.x (latest)   | :white_check_mark: |
| `finance-query` (library)   | < 2.6            | :x:                |
| `fq` (CLI)                  | 0.3.x (latest)   | :white_check_mark: |
| Server / MCP server         | latest `master` + published image | :white_check_mark: |
| v1 (legacy Python, `v1/`)   | all              | :x: (unmaintained) |

The v1 Python implementation is preserved for compatibility and is **not**
actively maintained. We still apply dependency security updates to it while it
remains deployed, but its application code receives no security fixes — treat it
as end-of-life and migrate to v2.

## Reporting a Vulnerability

**Please do not report security issues through public GitHub issues, discussions,
or pull requests.**

Report privately through GitHub's built-in flow (preferred):

1. Go to the [**Security** tab](https://github.com/Verdenroz/finance-query/security/advisories)
   → **Report a vulnerability**, or open
   <https://github.com/Verdenroz/finance-query/security/advisories/new> directly.
2. Describe the issue, affected component/version, and impact.

If you cannot use GitHub Private Vulnerability Reporting, email
**harveytseng2@gmail.com** with `SECURITY` in the subject.

Please include, where possible:

- The affected component and version (library / server / CLI / MCP).
- A description of the vulnerability and its impact.
- Steps to reproduce or a proof of concept.
- Any suggested remediation.

## What to Expect

This is a small, volunteer-maintained project, so timelines are best-effort:

- **Acknowledgement** within 3 business days.
- **Initial assessment** (accepted / needs-info / declined, with reasoning)
  within 7 days.
- For accepted reports: we coordinate a fix and a patched release, publish a
  GitHub Security Advisory, and request a CVE through GitHub where warranted.
- We credit reporters in the advisory unless you ask to remain anonymous.
- We ask for coordinated disclosure — please give us a reasonable window
  (target: 90 days) before any public disclosure.

## Scope

In scope:

- The `finance-query` library and its published crate.
- The server (`server/`), CLI (`finance-query-cli/`), and MCP server
  (`finance-query-mcp/`).
- The build/release supply chain (CI workflows, published Docker images).

Out of scope:

- The legacy v1 Python implementation (`v1/`) — see Supported Versions above.
- Vulnerabilities in upstream data providers (Yahoo Finance, FMP, Polygon,
  Alpha Vantage, FRED, CoinGecko, SEC EDGAR) — report those to the provider.
- Issues requiring a pre-compromised host, malicious local environment, or
  physical access.
- Rate-limit / quota exhaustion against third-party provider APIs.
- Reports generated solely by automated scanners without a demonstrated,
  exploitable impact.

Thank you for helping keep FinanceQuery and its users safe.
