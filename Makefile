.PHONY: help serve install install-dev build test test-fast lint fix audit docs docker docker-compose docker-compose-down clean publish-dry-run

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
	@command -v mkdocs >/dev/null 2>&1 || pip install mkdocs-material
	mkdocs serve -a localhost:8080

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
