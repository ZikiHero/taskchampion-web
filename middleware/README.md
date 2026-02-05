# Taskchampion Middleware Service

A lightweight Rust-based middleware service that provides a REST API for [Taskchampion](https://github.com/GothenburgBitFactory/taskchampion). This service allows you to manage tasks through standard HTTP endpoints and automatically synchronizes with a Taskchampion server.

## Features

- **REST API**: CRUD operations for tasks (Create, Read, Update, Delete).
- **Automatic Sync**: Synchronizes with a Taskchampion server after every write operation (if enabled).
- **Manual Sync**: Dedicated endpoint to trigger synchronization on demand.
- **SQLite Backend**: Uses a local SQLite database for caching and task storage.
- **Dockerized**: Easy deployment using a multi-stage optimized Dockerfile.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (Edition 2024)
- [Docker](https://docs.docker.com/) (optional, for containerized deployment)

## Getting Started

### Local Development

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd taskchampion-web/middleware
   ```

2. **Configure environment variables** (optional):
   - `TC_SERVER_DIR`: Path to the Taskchampion server directory (default: `./server-data`).
   - `AUTO_SYNC`: Enable/disable auto-sync after writes (default: `true`).

3. **Run the service**:
   ```bash
   cargo run
   ```
   The server will start on `http://0.0.0.0:3001`.

## API Documentation

### Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET`  | `/health` | Service health check |
| `GET`  | `/tasks` | List all tasks (excluding deleted) |
| `POST` | `/tasks` | Create a new task |
| `GET`  | `/tasks/:uuid` | Get details of a specific task |
| `PUT`  | `/tasks/:uuid` | Update an existing task |
| `DELETE`| `/tasks/:uuid` | Delete a task |
| `POST` | `/sync` | Manually trigger synchronization |

### Example: Create a Task

```bash
curl -X POST http://localhost:3001/tasks \
  -H "Content-Type: application/json" \
  -d '{"description": "Buy milk", "tags": ["shopping"]}'
```

## Docker

### Build the Image

The Dockerfile uses `cargo-chef` for optimized dependency caching.

```bash
docker build -t taskchampion-service .
```

### Run the Container

```bash
docker run -p 3001:3001 \
  -e TC_SERVER_DIR=/app/data \
  -v $(pwd)/data:/app/data \
  taskchampion-service
```

## Configuration

The following environment variables can be used to configure the service:

| Variable | Description | Default |
|----------|-------------|---------|
| `TC_SERVER_DIR` | Directory for Taskchampion server data | `./server-data` |
| `AUTO_SYNC` | Enable auto-sync after write operations | `true` |
| `RUST_LOG` | Logging level (e.g., `info`, `debug`, `error`) | `info` |

## Architecture Notes

- **Thread Safety**: The service uses `Arc<Mutex<...>>` to share the Taskchampion Replica and Server between Axum handlers.
- **Async Handling**: Since Taskchampion's `Server` trait is not `Send`, the service uses a custom `UnsafeSendFuture` wrapper and the `current_thread` Tokio runtime flavor to safely handle synchronization in an async environment.
