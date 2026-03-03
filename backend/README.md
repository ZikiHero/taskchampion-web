# Taskchampion Web Backend

The backend service for Taskchampion Web, built with Go and the Gin web framework. It serves as the orchestrator between the frontend and the Taskchampion middleware, handling authentication, data mapping, and business logic.

## Features

- **RESTful API**: Built with [Gin](https://gin-gonic.com/).
- **Database Integration**: Uses [GORM](https://gorm.io/) with support for PostgreSQL (production) and SQLite (development/testing).
- **Authentication**: JWT-based authentication.
- **Taskwarrior Integration**: Communicates with the Rust middleware to manage tasks.
- **Migration System**: Built-in database migration support.
- **Dockerized**: Optimized multi-stage build.

## Prerequisites

- [Go](https://go.dev/) 1.25+
- [Make](https://www.gnu.org/software/make/)
- [Docker](https://www.docker.com/) (optional)

## Getting Started

### Local Development

1. **Install dependencies**:
   ```bash
   go mod download
   ```

2. **Run database migrations**:
   ```bash
   make migrate
   ```

3. **Run the server**:
   ```bash
   make run
   ```
   The server will start on `http://localhost:8080` (default port inside the container, mapped to `8090` in docker-compose).

### Available Commands

| Command            | Description                                      |
|--------------------|--------------------------------------------------|
| `make build`       | Build the binary to `bin/server`                 |
| `make run`         | Run the application                              |
| `make test`        | Run tests (requires CGO for SQLite)              |
| `make migrate`     | Run database migrations                          |
| `make rollback`    | Rollback the last migration                      |
| `make docker-up`   | Start the stack using podman-compose/docker-compose |
| `make docker-down` | Stop the stack                                   |
| `make clean`       | Remove build artifacts                           |

## API Endpoints (Selection)

The backend exposes several endpoints, including:

- `GET /api/tasks`: List tasks
- `GET /api/tasks/:uuid`: Get task details
- `PUT /api/tasks/:uuid`: Update a task
- `GET /api/tags`: List all tags
- `GET /api/projects`: List all projects
- `POST /api/auth/login`: User authentication

## Configuration

Configuration is handled via environment variables and the `.env` file in the project root. Key variables include:

- `DB_HOST`, `DB_USER`, `DB_PASSWORD`, `DB_NAME`, `DB_PORT`: Database connection details.
- `JWT_SECRET`: Secret key for signing tokens.
- `MIDDLEWARE_URL`: URL of the Taskchampion middleware.

## Project Structure

- `cmd/server`: Entry point of the application.
- `internal/application`: Application services and business logic.
- `internal/domain`: Domain models and repository interfaces.
- `internal/infrastructure`: HTTP handlers, router, and repository implementations (GORM, Taskwarrior).
- `pkg/`: Reusable utility packages.
