.PHONY: $(shell grep -hoE '^[a-zA-Z][a-zA-Z0-9_-]*:' $(MAKEFILE_LIST) | tr -d ':')

# Default target
.DEFAULT_GOAL := help

# Variables
CARGO := cargo
DOCKER := docker
DOCKER_COMPOSE := docker compose
PORT ?= 8000

# Colors
GREEN := $(shell printf '\033[0;32m')
YELLOW := $(shell printf '\033[0;33m')
NC := $(shell printf '\033[0m')

help: ## Show available commands
	@echo "$(GREEN)FinanceQuery Commands$(NC)"
	@echo "===================="
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "$(YELLOW)%-20s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)

mcp: ## Run MCP server (stdio transport, for local development)
	$(CARGO) run -p finance-query-mcp

serve: ## Start development server
	@echo "$(GREEN)Starting server at http://localhost:$(PORT)$(NC)"
	cd server && PORT=$(PORT) $(CARGO) run -p finance-query-server

install-dev: ## Install dev tools and build workspace
	@echo "$(GREEN)Installing dev tools...$(NC)"
	rustup component add rustfmt clippy
	@if ! command -v prek >/dev/null 2>&1; then \
		echo "$(YELLOW)prek not found. Installing via cargo binstall or cargo...$(NC)"; \
		if command -v cargo-binstall >/dev/null 2>&1; then \
			cargo binstall -y prek; \
		else \
			cargo install --locked prek; \
		fi; \
	fi
	@prek install
	@echo "$(GREEN)Building workspace in dev mode...$(NC)"
	$(CARGO) build --workspace
	@echo "$(GREEN)✓ Dev environment ready!$(NC)"

build: ## Build library and server in release mode
	@echo "$(GREEN)Building in release mode...$(NC)"
	$(CARGO) build --release --workspace

test: ## Run ALL tests including network integration tests
	@echo "$(GREEN)Running all tests...$(NC)"
	@echo "$(YELLOW)Note: Some tests make real API calls$(NC)"
	$(CARGO) test --workspace -- --nocapture --include-ignored

test-fast: ## Run only fast tests (excludes network tests)
	@echo "$(GREEN)Running fast tests...$(NC)"
	$(CARGO) test --workspace -- --nocapture

fix: ## Auto-fix formatting and linting issues
	@echo "$(GREEN)Formatting code...$(NC)"
	@$(CARGO) fmt --all
	@echo "$(GREEN)Fixing clippy issues...$(NC)"
	@$(CARGO) clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged
	@echo "$(GREEN)✓ Auto-fix complete!$(NC)"

ci: fix build test soothfast ## Everything CI checks, locally: fix, build, audit, test, then the full soothfast suite
	@echo "$(GREEN)Running cargo-deny...$(NC)"
	@command -v cargo-deny >/dev/null 2>&1 || $(CARGO) install cargo-deny --locked
	@cargo deny check advisories bans licenses sources

# The measured regression gate (`cargo soothfast`) runs natively: per-iteration
# instruction counts via perf_event, plus alloc counts and walltime — no
# valgrind, no container. Install the CLI once with
# `cargo install cargo-soothfast` (or `cargo install --path <soothfast>/cargo-soothfast`
# while it is unpublished).
SOOTHFAST ?= cargo soothfast
BASE ?= origin/master

# =============================================================================
# soothfast: measured regression gate + living docs
# =============================================================================

# dataframe.md/finance.md/ticker.md/tickers.md capture live Yahoo Finance
# data (real prices/timestamps) — their output legitimately differs on every
# run, so they're excluded from the captured-output freshness check below.
CAPTURE_CHECK_PATHS := docs/library/backtesting.md docs/library/commodities.md \
	docs/library/configuration.md docs/library/crypto.md docs/library/economic.md \
	docs/library/error-handling.md docs/library/feeds.md docs/library/filings.md \
	docs/library/forex.md docs/library/futures.md docs/library/getting-started.md \
	docs/library/indicators.md docs/library/indices.md docs/library/providers/alphavantage.md \
	docs/library/providers/coingecko.md docs/library/providers/edgar.md \
	docs/library/providers/fmp.md docs/library/providers/fred.md docs/library/providers/index.md \
	docs/library/providers/polygon.md docs/library/risk.md docs/library/screeners.md \
	docs/library/streaming.md docs/library/translation.md README.md

# Canned recipe (not a target itself) shared by `docs` and `soothfast` so
# regenerating derived pages isn't its own .PHONY entry just to stay DRY
# between the two. The two `coverage docs` calls from earlier revisions were
# merged into one (it accepts --badge alongside its text output in a single
# pass) since it documents the same crate rustdoc already extracted for the
# reference/perf pages above — no need to pay for a third redundant pass.
define REGEN_DOCS_ROUTES
$(SOOTHFAST) docs routes -p finance-query-$(1) --target soothfast-routes --out docs/server
$(SOOTHFAST) spec html -p finance-query-$(1) --target soothfast-routes --out docs/server/api
endef

define REGEN_DOCS_PAGES_LIB
$(SOOTHFAST) docs reference -p finance-query --baseline base --features full --out docs/reference
$(SOOTHFAST) report render -p finance-query --baseline base --features full --out docs/perf
mv docs/perf/llms.txt docs/llms.txt
cp docs/llms.txt llms.txt
{ echo "# Coverage"; echo; \
  echo "> Generated by \`cargo soothfast coverage docs\`."; echo; \
  echo '```text'; \
  $(SOOTHFAST) coverage docs -p finance-query --features full \
    --badge docs/perf/badges/coverage.json --badge docs/perf/badges/coverage.svg; \
  echo '```'; } > docs/coverage.md
endef


sdk: ## Regenerate the OpenAPI/AsyncAPI specs, MCP tool manifest, and client SDKs from the code
	$(SOOTHFAST) spec gen -p finance-query-server --target soothfast-routes
	$(SOOTHFAST) spec gen -p finance-query-mcp --target soothfast-routes
	$(SOOTHFAST) sdk gen -p finance-query-server --target soothfast-routes

# The reconciliation-status and spec-html pages are generated and gitignored,
# so a fresh checkout has none — run this before `docs`.
# PAGES lets CI regenerate each package's route pages in the job that already
# built its soothfast-routes target, instead of rebuilding both here.
PAGES ?= all

docs-pages: ## Regenerate the derived docs pages (PAGES=all|server|mcp|lib)
ifneq ($(filter all server,$(PAGES)),)
	$(call REGEN_DOCS_ROUTES,server)
endif
ifneq ($(filter all mcp,$(PAGES)),)
	$(call REGEN_DOCS_ROUTES,mcp)
endif
ifneq ($(filter all lib,$(PAGES)),)
	$(REGEN_DOCS_PAGES_LIB)
endif

docs: ## Serve the docs site locally with live reload (run `make docs-pages` first)
	@echo "$(GREEN)Serving docs at http://localhost:8080$(NC)"
	$(SOOTHFAST) docs serve --baseline base --addr localhost:8080

# Everything soothfast: the merge-base regression gate, living-docs checks,
# proto reconciliation, derived-page regen, trend, changelog, and llms.txt
# staleness (the same gates soothfast.yml/deploy.yml run, in one shot).
# `**Measured:**` lines are excluded from the llms.txt diff: those numbers
# are point-in-time and jitter between runs — regression detection is the
# `gate` step below, not this. Every step stays inline rather than becoming
# its own target; run one standalone with `cargo soothfast <cmd>` directly.
soothfast: ## Run every soothfast check + refresh: gate, baseline, docs check/capture/coverage, proto, doc regen, trend, changelog, llms.txt staleness
	@echo "$(GREEN)Running soothfast gate against merge-base of $(BASE)...$(NC)"
	$(SOOTHFAST) gate -p finance-query --features bench-gate --against-ref $(BASE)
	@echo "$(GREEN)Measuring baseline...$(NC)"
	$(SOOTHFAST) measure -p finance-query --features bench-gate --save-baseline base
	@echo "$(GREEN)Checking living docs (binds, claims, generated tests)...$(NC)"
	$(SOOTHFAST) docs check -p finance-query --features full --baseline base docs/library README.md
	@echo "$(GREEN)Verifying captured doc example output...$(NC)"
	$(SOOTHFAST) docs capture -p finance-query --features full --check $(CAPTURE_CHECK_PATHS)
	@echo "$(GREEN)Checking public API doc coverage...$(NC)"
	$(SOOTHFAST) coverage docs -p finance-query --features full --min 95
	@echo "$(GREEN)Reconciling pricing.proto against PricingData...$(NC)"
	$(SOOTHFAST) spec check-proto -p finance-query --proto proto/pricing.proto \
		--message PricingData --source src/streaming/pricing.rs --struct PricingData
	@echo "$(GREEN)Checking generated openapi.yaml/asyncapi.yaml and SDK freshness...$(NC)"
	$(SOOTHFAST) spec gen -p finance-query-server --target soothfast-routes --check
	$(SOOTHFAST) sdk gen -p finance-query-server --target soothfast-routes --check
	@echo "$(GREEN)Checking generated mcp-tools.json freshness...$(NC)"
	$(SOOTHFAST) spec gen -p finance-query-mcp --target soothfast-routes --check
	@echo "$(GREEN)Regenerating derived doc pages...$(NC)"
	@$(MAKE) docs-pages
	@echo "$(GREEN)Appending performance trend point...$(NC)"
	$(SOOTHFAST) trend append -p finance-query --features bench-gate
	$(SOOTHFAST) trend render -p finance-query
	@echo "$(GREEN)Regenerating CHANGELOG.md...$(NC)"
	@PREV=$$(git describe --tags --abbrev=0 2>/dev/null || git rev-list --max-parents=0 HEAD); \
	$(SOOTHFAST) report changelog -p finance-query --baseline base --against-ref $$PREV --features full
	@echo "$(GREEN)Checking llms.txt freshness...$(NC)"
	@rm -rf target/llms-check && mkdir -p target/llms-check
	$(SOOTHFAST) report render -p finance-query --baseline base --features full --out target/llms-check
	@sed '/^\*\*Measured:\*\*/d' llms.txt > target/llms-check/committed.txt
	@sed '/^\*\*Measured:\*\*/d' target/llms-check/llms.txt > target/llms-check/fresh.txt
	@diff -u target/llms-check/committed.txt target/llms-check/fresh.txt || \
		(echo "error: llms.txt is stale — re-run 'make soothfast' and commit the result" >&2 && exit 1)
	@rm -rf target/llms-check
	@echo "llms.txt: up to date"

probe: ## Run data-quality probes (probes.toml) against a locally launched release server
	cargo build --release -p finance-query-server
	$(SOOTHFAST) spec probe -p finance-query-server

probe-accept: ## Re-lock probes.lock after a deliberate response change
	cargo build --release -p finance-query-server
	$(SOOTHFAST) spec probe -p finance-query-server --accept

# =============================================================================
# Production Docker Compose
# =============================================================================

# docker-compose.prod.yml is an overlay — both -f flags are required together.
PROD_COMPOSE := $(DOCKER_COMPOSE) -f docker-compose.yml -f docker-compose.prod.yml

prod: ## Start production stack (docker compose -f ... -f docker-compose.prod.yml for down/logs/ps)
	@echo "$(GREEN)Starting production stack...$(NC)"
	$(PROD_COMPOSE) up -d --build
	@echo "$(GREEN)✓ Running at http://localhost$(NC)"

# =============================================================================
# Version bumping
# =============================================================================

bump: ## Bump version for core + server + mcp + derive + API specs (usage: make bump VERSION=x.y.z)
ifndef VERSION
	$(error VERSION is required. Usage: make bump VERSION=x.y.z)
endif
	@echo "$(GREEN)Bumping version to $(VERSION)...$(NC)"
	@# Update root Cargo.toml package version
	@sed -i 's/^version = "[^"]*"/version = "$(VERSION)"/' Cargo.toml
	@# Update finance-query-derive dependency version in root Cargo.toml
	@sed -i 's/finance-query-derive = { version = "[^"]*"/finance-query-derive = { version = "$(VERSION)"/' Cargo.toml
	@# Update server Cargo.toml
	@sed -i 's/^version = "[^"]*"/version = "$(VERSION)"/' server/Cargo.toml
	@# Update MCP Cargo.toml (lockstepped with core/server, not independently versioned)
	@sed -i 's/^version = "[^"]*"/version = "$(VERSION)"/' finance-query-mcp/Cargo.toml
	@# Update derive Cargo.toml
	@sed -i 's/^version = "[^"]*"/version = "$(VERSION)"/' finance-query-derive/Cargo.toml
	@# Update server OpenAPI version
	@sed -i 's/^  version: [0-9.]*$$/  version: $(VERSION)/' server/openapi.yaml
	@# Update server AsyncAPI version
	@sed -i 's/^  version: [0-9.]*$$/  version: $(VERSION)/' server/asyncapi.yaml
	@echo "$(GREEN)Regenerating API docs HTML...$(NC)"
	@$(SOOTHFAST) spec html -p finance-query-server --target soothfast-routes --out docs/server/api
	@$(SOOTHFAST) spec html -p finance-query-mcp --target soothfast-routes --out docs/server/api
	@echo "$(GREEN)✓ Version bumped to $(VERSION)$(NC)"
	@echo "$(YELLOW)Updated files:$(NC)"
	@echo "  - Cargo.toml"
	@echo "  - server/Cargo.toml"
	@echo "  - finance-query-mcp/Cargo.toml"
	@echo "  - finance-query-derive/Cargo.toml"
	@echo "  - server/openapi.yaml"
	@echo "  - server/asyncapi.yaml"
	@echo "  - docs/server/api/openapi.md"
	@echo "  - docs/server/api/asyncapi.md"
	@echo "  - docs/server/api/mcp-tools.md"
	@echo "$(YELLOW)Remember to also hand-edit:$(NC)"
	@echo "  - CHANGELOG.md (library)"
	@echo "  - server/CHANGELOG.md"
	@echo "  - finance-query-mcp/CHANGELOG.md"

bump-cli: ## Bump version for CLI only (usage: make bump-cli VERSION=x.y.z)
ifndef VERSION
	$(error VERSION is required. Usage: make bump-cli VERSION=x.y.z)
endif
	@echo "$(GREEN)Bumping CLI version to $(VERSION)...$(NC)"
	@sed -i 's/^version = "[^"]*"/version = "$(VERSION)"/' finance-query-cli/Cargo.toml
	@echo "$(GREEN)✓ CLI version bumped to $(VERSION)$(NC)"
	@echo "$(YELLOW)Updated files:$(NC)"
	@echo "  - finance-query-cli/Cargo.toml"
