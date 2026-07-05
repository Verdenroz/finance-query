.PHONY: help serve install install-dev build test test-fast lint fix audit bench baseline docs clean publish-dry-run \
        prod prod-down prod-logs prod-status prod-build bump bump-cli generate-api-html generate-mcp-html mcp mcp-http

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

mcp-http: ## Run MCP server (HTTP streaming transport, for VPS — binds to MCP_ADDR, default 0.0.0.0:3000)
	MCP_TRANSPORT=http $(CARGO) run -p finance-query-mcp

serve: ## Start development server
	@echo "$(GREEN)Starting server at http://localhost:$(PORT)$(NC)"
	cd server && PORT=$(PORT) $(CARGO) run -p finance-query-server

install: ## Build release binary
	@echo "$(GREEN)Building release binary...$(NC)"
	$(CARGO) build --release -p finance-query-server

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
	@echo "$(GREEN)Setting up Python environment for docs...$(NC)"
	@if command -v uv >/dev/null 2>&1; then \
		uv sync; \
	else \
		if ! [ -d .venv ]; then python3 -m venv .venv; fi; \
		.venv/bin/pip install -q --upgrade pip; \
		.venv/bin/pip install -q -e .; \
	fi
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

lint: ## Run all pre-commit checks
	@echo "$(GREEN)Running pre-commit checks...$(NC)"
	@prek

fix: ## Auto-fix formatting and linting issues
	@echo "$(GREEN)Formatting code...$(NC)"
	@$(CARGO) fmt --all
	@echo "$(GREEN)Fixing clippy issues...$(NC)"
	@$(CARGO) clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged
	@echo "$(GREEN)✓ Auto-fix complete!$(NC)"

audit: ## Run the same dependency policy check as CI (advisories, bans, licenses, sources)
	@echo "$(GREEN)Running cargo-deny...$(NC)"
	@command -v cargo-deny >/dev/null 2>&1 || $(CARGO) install cargo-deny --locked
	@cargo deny check advisories bans licenses sources

bench: ## Run criterion wall-clock benchmarks (local profiling, not a CI gate)
	@echo "$(GREEN)Running criterion benchmarks...$(NC)"
	$(CARGO) bench --features full \
		--bench indicators --bench backtesting --bench ticker --bench tickers \
		--bench finance --bench providers --bench risk --bench stream \
		--bench serde --bench dataframe --bench feeds

baseline: ARGS ?= --save-baseline=base
baseline: ## Run the regression gate in a Debian container; saves/updates baseline "base" (ARGS="--baseline=base" to compare without overwriting)
	@echo "$(GREEN)Running regression gate...$(NC)"
	$(DOCKER) run --rm --security-opt seccomp=unconfined \
		-v "$(CURDIR)":/app -w /app \
		-v fq-bench-cargo:/usr/local/cargo/registry \
		-v fq-bench-cargobin:/usr/local/cargo/bin \
		-v fq-bench-target:/app/target \
		rust:bookworm bash -c 'apt-get update -qq && apt-get install -y -qq valgrind >/dev/null && \
		(command -v iai-callgrind-runner || cargo install iai-callgrind-runner --version 0.16.1 --locked) && \
		cargo bench --bench regression --features bench-gate -- $(ARGS)'

docs: ## Build and serve documentation locally
	@echo "$(GREEN)Serving docs at http://localhost:8080$(NC)"
	uv run mkdocs serve -a localhost:8080 --livereload

clean: ## Clean build artifacts
	@echo "$(GREEN)Cleaning build artifacts...$(NC)"
	$(CARGO) clean
	rm -rf target/ site/

publish-dry-run: ## Test publishing to crates.io (dry run)
	@echo "$(GREEN)Testing crates.io publish (dry run)...$(NC)"
	$(CARGO) publish -p finance-query-derive --dry-run
	$(CARGO) publish -p finance-query --dry-run

# =============================================================================
# Production Docker Compose
# =============================================================================

# docker-compose.prod.yml is an overlay — both -f flags are required together.
PROD_COMPOSE := $(DOCKER_COMPOSE) -f docker-compose.yml -f docker-compose.prod.yml

prod: ## Start production stack
	@echo "$(GREEN)Starting production stack...$(NC)"
	$(PROD_COMPOSE) up -d
	@echo "$(GREEN)✓ Running at http://localhost$(NC)"

prod-build: ## Build and start production stack
	@echo "$(GREEN)Building production stack...$(NC)"
	$(PROD_COMPOSE) up -d --build

prod-down: ## Stop production stack
	$(PROD_COMPOSE) down

prod-logs: ## View production logs (use SVC=name for specific service)
	@if [ -n "$(SVC)" ]; then \
		$(PROD_COMPOSE) logs -f $(SVC); \
	else \
		$(PROD_COMPOSE) logs -f; \
	fi

prod-status: ## Check production container status
	$(PROD_COMPOSE) ps

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
	@$(MAKE) -s generate-api-html
	@echo "$(GREEN)✓ Version bumped to $(VERSION)$(NC)"
	@echo "$(YELLOW)Updated files:$(NC)"
	@echo "  - Cargo.toml"
	@echo "  - server/Cargo.toml"
	@echo "  - finance-query-mcp/Cargo.toml"
	@echo "  - finance-query-derive/Cargo.toml"
	@echo "  - server/openapi.yaml"
	@echo "  - server/asyncapi.yaml"
	@echo "  - docs/server/openapi-html/index.html"
	@echo "  - docs/server/asyncapi-html/index.html"
	@echo "  - docs/server/mcp-html/index.html"
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


generate-api-html: ## Regenerate OpenAPI and AsyncAPI HTML docs from server specs
	@echo "$(GREEN)Generating OpenAPI HTML...$(NC)"
	@python3 -c '\
import yaml, json; \
spec = json.dumps(yaml.safe_load(open("server/openapi.yaml")), indent=2); \
html = """<!DOCTYPE html>\n\
<html lang="en">\n\
<head>\n\
  <meta charset="UTF-8">\n\
  <meta name="viewport" content="width=device-width, initial-scale=1.0">\n\
  <title>Finance Query API</title>\n\
  <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">\n\
  <style>\n\
    html { box-sizing: border-box; overflow-y: scroll; }\n\
    *, *:before, *:after { box-sizing: inherit; }\n\
    body { margin: 0; background: #fafafa; }\n\
    .swagger-ui .topbar { display: none; }\n\
  </style>\n\
</head>\n\
<body>\n\
  <div id="swagger-ui"></div>\n\
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>\n\
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-standalone-preset.js"></script>\n\
  <script>\n\
    window.onload = function() {\n\
      window.ui = SwaggerUIBundle({\n\
        spec: """ + spec + """,\n\
        dom_id: "#swagger-ui",\n\
        presets: [SwaggerUIBundle.presets.apis, SwaggerUIStandalonePreset],\n\
        layout: "StandaloneLayout"\n\
      });\n\
    };\n\
  </script>\n\
</body>\n\
</html>"""; \
open("docs/server/openapi-html/index.html", "w").write(html)'
	@echo "$(GREEN)Generating AsyncAPI HTML...$(NC)"
	@npx -y @asyncapi/generator@2 server/asyncapi.yaml @asyncapi/html-template -o docs/server/asyncapi-html -p singleFile=true --force-write
	@echo "$(GREEN)Generating MCP tools HTML...$(NC)"
	@$(MAKE) -s generate-mcp-html

generate-mcp-html: ## Generate MCP tools reference HTML from live server
	@$(CARGO) build -p finance-query-mcp --quiet
	@mkdir -p docs/server/mcp-html; \
	MCP_TRANSPORT=http MCP_ADDR=127.0.0.1:13337 target/debug/fq-mcp 2>/dev/null & \
	MCP_PID=$$!; \
	for i in $$(seq 1 15); do sleep 1; curl -sf http://127.0.0.1:13337/health >/dev/null 2>&1 && break; done; \
	curl -s -X POST http://127.0.0.1:13337/ \
	    -H 'Content-Type: application/json' \
	    -H 'Accept: application/json, text/event-stream' \
	    -d '{"jsonrpc":"2.0","method":"tools/list","id":1}' \
	| python3 docs/server/generate-mcp-html.py > docs/server/mcp-html/index.html; \
	STATUS=$$?; kill $$MCP_PID 2>/dev/null; exit $$STATUS
	@echo "$(GREEN)✓ MCP tools HTML written to docs/server/mcp-html/index.html$(NC)"
