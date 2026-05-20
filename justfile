# Justfile for taskchampion-web project

set shell := ["bash", "-c"]

backend_dir := "backend"
frontend_dir := "frontend"
middleware_dir := "middleware"

# Image names
backend_image := "ghcr.io/dhufe/tcweb-backend:dev"
frontend_image := "ghcr.io/dhufe/tcweb-frontend:dev"
middleware_image := "ghcr.io/dhufe/tcweb-middleware:dev"

# Default recipe: show help
default:
    @just --list

# --- Backend Recipes ---

# Build the backend
build-backend:
    cd {{backend_dir}} && go build -o bin/server cmd/server/main.go

# Run the backend
run-backend:
    cd {{backend_dir}} && go run cmd/server/main.go

# Run backend tests
test-backend:
    cd {{backend_dir}} && CGO_ENABLED=1 gotestsum \
        --junitfile report.xml \
        --format testname \
        -- -coverprofile=cover.out ./...

# Run database migrations
migrate:
    cd {{backend_dir}} && go run cmd/server/main.go -migrate

# Rollback last migration
rollback:
    cd {{backend_dir}} && go run cmd/server/main.go -rollback

# Run backend linter
lint-backend:
    cd {{backend_dir}} && golangci-lint run \
        --output.text.path=stdout \
        --output.text.colors=false \
        --output.text.print-issued-lines=false \
        --output.code-climate.path=gl-code-quality-report.json

# Clean backend build artifacts
clean-backend:
    rm -rf {{backend_dir}}/bin/
    rm -f {{backend_dir}}/coverage.out
    rm -f {{backend_dir}}/cover.out
    rm -f {{backend_dir}}/report.xml

# --- Frontend Recipes ---

# Install frontend dependencies
install-frontend:
    cd {{frontend_dir}} && npm install

# Build the frontend
build-frontend:
    cd {{frontend_dir}} && npm ci && npm run build -- --configuration production --output-path=dist

# Run frontend development server
run-frontend:
    cd {{frontend_dir}} && npm run start

# Run frontend tests
test-frontend:
    cd {{frontend_dir}} && npx vitest run

# Run frontend linter (if applicable, using generic npm lint if it exists)
lint-frontend:
    cd {{frontend_dir}} && if grep -q '"lint":' package.json; then npm run lint; else echo "No lint script in frontend/package.json"; fi

# Clean frontend artifacts
clean-frontend:
    rm -rf {{frontend_dir}}/dist/

# --- Middleware Recipes ---

# Build the middleware
build-middleware:
    cd {{middleware_dir}} && cargo build --release

# Run the middleware
run-middleware:
    cd {{middleware_dir}} && cargo run

# Run middleware tests
test-middleware:
    cd {{middleware_dir}} && cargo test

# Run middleware linter
lint-middleware:
    cd {{middleware_dir}} && cargo clippy -- -D warnings

# Clean middleware build artifacts
clean-middleware:
    cd {{middleware_dir}} && cargo clean

# --- Container Recipes ---

# Remove container images
clean-images:
    podman rmi {{backend_image}} {{frontend_image}} {{middleware_image}} || true

# Remove volumes and containers
clean-volumes:
    podman-compose down -v

# Clean all container-related artifacts
clean-containers: clean-images clean-volumes

# --- Infrastructure Recipes ---

# Start podman compose
up:
    podman-compose -f docker-compose-dev.yml up -d --build

# Start only the backend
up-backend:
    podman-compose -f docker-compose-dev.yml up -d --build tcweb-backend

# Start only the middleware
up-middleware:
    podman-compose -f docker-compose-dev.yml up -d --build sync-middleware

# Start only the frontend
up-frontend:
    podman-compose -f docker-compose-dev.yml up -d --build tcweb-frontend

# Stop podman compose
down:
    podman-compose down

# Show podman logs
logs:
    podman-compose logs -f

# --- Combined Recipes ---

# Build all components
build-all: build-backend build-frontend build-middleware

# Run all tests
test-all: test-backend test-frontend test-middleware

# Clean everything
clean-all: clean-backend clean-frontend clean-middleware clean-containers

# Alias for backward compatibility or convenience
backend: run-backend
frontend: run-frontend
middleware: run-middleware
build: build-backend
run: run-backend
test: test-backend
lint: lint-backend
clean: clean-backend
