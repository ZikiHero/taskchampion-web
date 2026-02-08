# Makefile
.PHONY: help build run test migrate rollback docker-up docker-down clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build the application
	go mod tidy
	CGO_ENABLED=1 go build -o bin/server cmd/server/main.go

run: ## Run the application
	go run cmd/server/main.go

test: ## Run tests
	# CGO_ENABLED=1 -> SQLite wird für Tests benötigt.
	CGO_ENABLED=1 go test -v -race -coverprofile=coverage.out ./...

migrate: ## Run database migrations
	go run cmd/server/main.go -migrate

rollback: ## Rollback last migration
	go run cmd/server/main.go -rollback

docker-up: ## Start docker compose
	podman-compose up -d --build

docker-down: ## Stop docker compose
	podman-compose down

docker-logs: ## Show docker logs
	podman-compose logs -f

clean: ## Clean build artifacts
	rm -rf bin/
	go clean -f cache
	rm -f coverage.out

lint: ## Run linter
	golangci-lint run

.DEFAULT_GOAL := help
