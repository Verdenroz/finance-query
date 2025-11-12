.PHONY: help serve install install-dev build test lint docs docker docker-aws clean

# Default target
.DEFAULT_GOAL := help

# Variables
UV := uv
PYTHON := python3
MKDOCS := mkdocs
DOCKER := docker

# Colors
GREEN := $(shell printf '\033[0;32m')
YELLOW := $(shell printf '\033[0;33m')
NC := $(shell printf '\033[0m')

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

docker-aws: ## Build and test AWS Lambda Docker image
	@echo "$(GREEN)Building AWS Lambda Docker image...$(NC)"
	$(DOCKER) build -f Dockerfile.aws -t financequery-lambda .
	@echo "$(GREEN)Starting Lambda container in background...$(NC)"
	$(DOCKER) run -d --name financequery-lambda-test -p 9000:8080 financequery-lambda
	@echo "$(YELLOW)Waiting for Lambda to be ready...$(NC)"
	@sleep 5
	@echo "$(GREEN)Testing /ping endpoint...$(NC)"
	@curl -s -X POST "http://localhost:9000/2015-03-31/functions/function/invocations" \
		-H "Content-Type: application/json" \
		-d '{"resource":"/ping","path":"/ping","httpMethod":"GET","headers":{"Accept":"*/*","Host":"localhost:9000"},"requestContext":{"requestId":"test-request-id","accountId":"123456789012","stage":"prod","identity":{"sourceIp":"127.0.0.1"}},"queryStringParameters":null,"pathParameters":null,"stageVariables":null,"body":null,"isBase64Encoded":false}' \
		| grep -q '"statusCode": 200' && echo "$(GREEN)✓ /ping endpoint working$(NC)" || (echo "$(YELLOW)✗ /ping endpoint failed$(NC)" && exit 1)
	@echo "$(GREEN)Testing /health endpoint...$(NC)"
	@curl -s -X POST "http://localhost:9000/2015-03-31/functions/function/invocations" \
		-H "Content-Type: application/json" \
		-d '{"resource":"/health","path":"/health","httpMethod":"GET","headers":{"Accept":"*/*","Host":"localhost:9000"},"requestContext":{"requestId":"test-request-id","accountId":"123456789012","stage":"prod","identity":{"sourceIp":"127.0.0.1"}},"queryStringParameters":null,"pathParameters":null,"stageVariables":null,"body":null,"isBase64Encoded":false}' \
		| grep -q '"statusCode": 200' && echo "$(GREEN)✓ /health endpoint working$(NC)" || (echo "$(YELLOW)✗ /health endpoint failed$(NC)" && exit 1)
	@echo "$(GREEN)All tests passed! Cleaning up...$(NC)"
	@$(DOCKER) stop financequery-lambda-test > /dev/null
	@$(DOCKER) rm financequery-lambda-test > /dev/null
	@echo "$(GREEN)AWS Lambda Docker image test complete!$(NC)"

clean: ## Clean build artifacts and cache
	@echo "$(GREEN)Cleaning build artifacts...$(NC)"
	rm -rf build/ dist/ *.egg-info/ .pytest_cache/ .coverage htmlcov/ .ruff_cache/
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete
	find . -type f -name "*.c" -path "*/services/indicators/core/*" -delete
	find . -type f -name "*.so" -path "*/services/indicators/core/*" -delete
	find . -type f -name "*.pyd" -path "*/services/indicators/core/*" -delete
