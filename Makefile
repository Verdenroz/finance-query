.PHONY: help serve install install-dev build test test-fast lint fix audit docs docker docker-compose docker-compose-down clean publish-dry-run \
        prod prod-down prod-logs prod-status prod-build bump bump-cli generate-api-html

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

audit: ## Run security audit on dependencies
	@echo "$(GREEN)Running security audit...$(NC)"
	@command -v cargo-audit >/dev/null 2>&1 || $(CARGO) install cargo-audit
	@$(CARGO) audit

docs: ## Build and serve documentation locally
	@echo "$(GREEN)Serving docs at http://localhost:8080$(NC)"
	uv run mkdocs serve -a localhost:8080 --livereload

docker: ## Build Docker image for v2 server
	@echo "$(GREEN)Building v2 Docker image...$(NC)"
	$(DOCKER) build -f server/Dockerfile -t financequery:v2 .

docker-compose: ## Start both v1 and v2 with Docker Compose
	@echo "$(GREEN)Starting v1 and v2 servers...$(NC)"
	$(DOCKER_COMPOSE) up -d

docker-compose-down: ## Stop Docker Compose services
	$(DOCKER_COMPOSE) down

clean: ## Clean build artifacts
	@echo "$(GREEN)Cleaning build artifacts...$(NC)"
	$(CARGO) clean
	rm -rf target/ site/

publish-dry-run: ## Test publishing to crates.io (dry run)
	@echo "$(GREEN)Testing crates.io publish (dry run)...$(NC)"
	$(CARGO) publish -p finance-query-derive --dry-run
	$(CARGO) publish -p finance-query --dry-run

# =============================================================================
# Production Docker Compose (docker-compose.prod.yml with Caddy)
# =============================================================================

prod: ## Start production stack
	@echo "$(GREEN)Starting production stack...$(NC)"
	$(DOCKER_COMPOSE) -f docker-compose.prod.yml up -d
	@echo "$(GREEN)✓ Running at http://localhost$(NC)"

prod-build: ## Build and start production stack
	@echo "$(GREEN)Building production stack...$(NC)"
	$(DOCKER_COMPOSE) -f docker-compose.prod.yml up -d --build

prod-down: ## Stop production stack
	$(DOCKER_COMPOSE) -f docker-compose.prod.yml down

prod-logs: ## View production logs (use SVC=name for specific service)
	@if [ -n "$(SVC)" ]; then \
		$(DOCKER_COMPOSE) -f docker-compose.prod.yml logs -f $(SVC); \
	else \
		$(DOCKER_COMPOSE) -f docker-compose.prod.yml logs -f; \
	fi

prod-status: ## Check production container status
	$(DOCKER_COMPOSE) -f docker-compose.prod.yml ps

# =============================================================================
# Version bumping
# =============================================================================

bump: ## Bump version for core + server + derive + API specs (usage: make bump VERSION=x.y.z)
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
	@echo "  - finance-query-derive/Cargo.toml"
	@echo "  - server/openapi.yaml"
	@echo "  - server/asyncapi.yaml"
	@echo "  - docs/server/openapi-html/index.html"
	@echo "  - docs/server/asyncapi-html/index.html"

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
