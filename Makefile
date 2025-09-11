.PHONY: help serve install install-dev build test lint docs docker clean

# Default target
.DEFAULT_GOAL := help

# Variables
UV := uv
PYTHON := python3
MKDOCS := mkdocs
DOCKER := docker

# Colors
GREEN := \033[0;32m
YELLOW := \033[0;33m
NC := \033[0m

help: ## Show available commands
	@echo "$(GREEN)FinanceQuery Commands$(NC)"
	@echo "===================="
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "$(YELLOW)%-15s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)

serve: ## Start development server
	@echo "$(GREEN)Starting server at http://localhost:8000$(NC)"
	$(PYTHON) -m uvicorn src.main:app --reload --host 0.0.0.0 --port 8000

install: ## Install production dependencies
	@echo "$(GREEN)Installing production dependencies...$(NC)"
	$(UV) sync
	$(PYTHON) setup.py build_ext --inplace

install-dev: ## Install dev dependencies + pre-commit hooks
	@echo "$(GREEN)Installing dev dependencies...$(NC)"
	$(UV) sync --all-groups
	$(PYTHON) setup.py build_ext --inplace
	pre-commit install

build: ## Build Cython extensions
	@echo "$(GREEN)Building Cython extensions...$(NC)"
	$(PYTHON) setup.py build_ext --inplace

test: ## Run tests with coverage
	@echo "$(GREEN)Running tests with coverage...$(NC)"
	pytest

lint: ## Run linting and formatting
	@echo "$(GREEN)Running linting and formatting...$(NC)"
	pre-commit run --all-files

docs: ## Build and serve documentation
	@echo "$(GREEN)Serving docs at http://localhost:8001$(NC)"
	$(MKDOCS) serve --dev-addr=0.0.0.0:8001

docker: ## Build and run Docker container
	@echo "$(GREEN)Building and running Docker container...$(NC)"
	$(DOCKER) build -t financequery .
	$(DOCKER) run -p 8000:8000 financequery

clean: ## Clean build artifacts and cache
	@echo "$(GREEN)Cleaning build artifacts...$(NC)"
	rm -rf build/ dist/ *.egg-info/ .pytest_cache/ .coverage htmlcov/ .ruff_cache/
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete
	find . -type f -name "*.c" -path "*/services/indicators/core/*" -delete
	find . -type f -name "*.so" -path "*/services/indicators/core/*" -delete
	find . -type f -name "*.pyd" -path "*/services/indicators/core/*" -delete
