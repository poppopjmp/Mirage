.PHONY: help build test clean dev-up dev-down docker-build

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build all services
	cargo build --release

test: ## Run all tests
	cargo test

clean: ## Clean build artifacts
	cargo clean
	docker system prune -f

dev-up: ## Start development environment
	docker-compose -f docker-compose.dev.yml up -d

dev-down: ## Stop development environment
	docker-compose -f docker-compose.dev.yml down

docker-build: ## Build all Docker images
	docker-compose build

init-db: ## Initialize databases
	@echo "Creating databases..."
	docker-compose exec postgres psql -U mirage -d mirage -f /docker-entrypoint-initdb.d/init-db.sql

migrate: ## Run database migrations
	@for service in auth-service user-management-service scan-orchestration-service; do \
		echo "Running migrations for $$service..."; \
		cd services/$$service && sqlx migrate run && cd ../..; \
	done

fmt: ## Format code
	cargo fmt --all

lint: ## Run linter
	cargo clippy --all-targets --all-features -- -D warnings

check: fmt lint test ## Run all checks