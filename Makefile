.PHONY: help serve install install-dev build test lint fix docs docker clean

# Default target
.DEFAULT_GOAL := help

# Variables
CARGO := cargo
MDBOOK := mdbook
DOCKER := docker
PRECOMMIT := pre-commit
PORT ?= 8000

# Colors
GREEN := $(shell printf '\033[0;32m')
YELLOW := $(shell printf '\033[0;33m')
NC := $(shell printf '\033[0m')

help: ## Show available commands
	@echo "$(GREEN)FinanceQuery Commands$(NC)"
	@echo "===================="
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "$(YELLOW)%-15s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)

serve: ## Start development server
	@echo "$(GREEN)Starting server at http://localhost:$(PORT)$(NC)"
	PORT=$(PORT) $(CARGO) run -p finance-query-server

install: ## Build release binary
	@echo "$(GREEN)Building release binary...$(NC)"
	$(CARGO) build --release -p finance-query-server

install-dev: ## Install dev tools and build workspace
	@echo "$(GREEN)Installing dev tools...$(NC)"
	rustup component add rustfmt clippy
	@echo "$(GREEN)Building workspace in dev mode...$(NC)"
	$(CARGO) build --workspace
	@echo "$(GREEN)✓ Dev environment ready! Run 'make lint' before committing.$(NC)"

build: ## Build in release mode
	@echo "$(GREEN)Building server in release mode...$(NC)"
	$(CARGO) build --release -p finance-query-server

test: ## Run all tests
	@echo "$(GREEN)Running tests...$(NC)"
	$(CARGO) test --workspace -- --nocapture

lint: ## Run all pre-commit checks (formatting, linting, compilation, and file checks)
	@echo "$(GREEN)Running all pre-commit checks...$(NC)"
	@prek

fix: ## Auto-fix formatting and linting issues, then verify with pre-commit checks
	@echo "$(GREEN)Formatting code...$(NC)"
	@$(CARGO) fmt --all
	@echo "$(GREEN)Fixing clippy issues...$(NC)"
	@$(CARGO) clippy --workspace --all-targets --all-features --fix --allow-dirty --allow-staged
	@echo "$(GREEN)Running pre-commit checks to verify...$(NC)"
	@prek
	@echo "$(GREEN)✓ Auto-fix complete and verified!$(NC)"

docs: ## Build and serve documentation
	@echo "$(GREEN)Serving docs at http://localhost:3000$(NC)"
	$(MDBOOK) serve --open

docker: ## Build and run Docker container
	@echo "$(GREEN)Building Docker image...$(NC)"
	$(DOCKER) build -t financequery .
	@echo "$(GREEN)Running container on port $(PORT)...$(NC)"
	$(DOCKER) run -p $(PORT):8000 --env-file .env financequery

clean: ## Clean build artifacts and cache
	@echo "$(GREEN)Cleaning build artifacts...$(NC)"
	$(CARGO) clean
	rm -rf book/ target/ Cargo.lock
