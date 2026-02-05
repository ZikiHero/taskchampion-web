# Taskchampion Middleware service

A lightweight Rust-based middleware service that provides a REST API for [Taskchampion](https://github.com/GothenburgBitFactory/taskchampion). This service allows you to manage tasks through standard HTTP endpoints and automatically synchronizes with a Taskchampion server.

## Features

- **REST API**: CRUD operations for tasks (Create, Read, Update, Delete).
- **End-to-End Encryption**: Securely synchronizes data. All encryption/decryption happens in the middleware; the sync server never sees your keys.
- **Argon2 Key Derivation**: Derives high-entropy encryption keys from a user-provided password.
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

2. **Configure environment variables**:
   Required variables must be set for the service to start:
   - `TC_CLIENT_ID`: A valid UUID for your Taskchampion client.
   - `TC_ENCRYPTION_PASSWORD`: Password used to derive the encryption key.
   - `TC_SYNC_SERVER_URL`: URL of the Taskchampion sync server.

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
  -d '{
    "description": "Buy milk",
    "tags": ["shopping", "home"],
    "priority": "H",
    "due": "2025-12-31T23:59:59Z"
  }'
```

### Example: Get a Task

```bash
curl -X GET http://localhost:3001/tasks/<uuid>
```

### Example: Update a Task

```bash
curl -X PUT http://localhost:3001/tasks/<uuid> \
  -H "Content-Type: application/json" \
  -d '{
    "status": "completed"
  }'
```

## Docker

### Build the Image

The Dockerfile uses `cargo-chef` for optimized dependency caching.

```bash
docker build -t taskchampion-middleware .
```

### Run the Container

```bash
docker run -p 3001:3001 \
  -v $(pwd)/data:/app/data \
  taskchampion-middleware  
```

## Configuration

The following environment variables can be used to configure the service:

### Required

| Variable | Description |
|----------|-------------|
| `TC_CLIENT_ID` | Your Taskchampion Client ID (UUID) |
| `TC_ENCRYPTION_PASSWORD` | Password for end-to-end encryption |
| `TC_SYNC_SERVER_URL` | URL of the remote sync server |

### Optional

| Variable | Description | Default |
|----------|-------------|---------|
| `TC_DB_PATH` | Path to the SQLite database file | `./local-cache.db` |
| `TC_AUTO_SYNC` | Enable auto-sync after write operations | `true` |
| `RUST_LOG` | Logging level (e.g., `info`, `debug`, `error`) | `info` |

## Architecture Notes

- **Thread Safety**: The service uses `Arc<Mutex<...>>` to share the Taskchampion Replica and Server between Axum handlers.
- **Async Handling**: The service uses the `current_thread` Tokio runtime flavor to safely handle synchronization in an async environment, as the Taskchampion `Server` trait implementation often involves non-`Send` components.
- **Security**: Encryption keys are derived using Argon2 with a fixed salt, ensuring that the same password consistently produces the same 32-byte key for decryption.
