# Taskchampion Web

A modern web-based task management system for [Taskwarrior](https://taskwarrior.org/) and [Taskchampion](https://github.com/GothenburgBitFactory/taskchampion). This project provides a full-stack environment to manage your tasks with a clean web interface, a robust Go backend, and a specialized Rust middleware for Taskchampion synchronization.

## Architecture

The project is composed of several specialized services:

- **Frontend**: An Angular-based web application providing a user-friendly task management interface.
- **Backend**: A Go-based API server that handles business logic, user management, and communicates with the middleware.
- **Middleware**: A Rust-based service that interacts directly with Taskchampion, providing RESTful access and handling end-to-end encryption and synchronization.
- **Sync Server**: A Taskchampion synchronization server for backing up and syncing tasks across multiple clients.
- **Database**: PostgreSQL is used for persistent storage.

## Project Structure

```text
.
├── backend/       # Go (Gin) API Server
├── frontend/      # Angular Web Application
├── middleware/    # Rust Taskchampion Middleware
└── docs/          # Project documentation
```

## Getting Started

### Prerequisites

- [Docker](https://www.docker.com/) and [Docker Compose](https://docs.docker.com/compose/) (or Podman/podman-compose)
- An `.env` file based on `env-sample`

### Quick Start

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd taskchampion-web
   ```

2. **Configure environment**:
   ```bash
   cp env-sample .env
   # Edit .env with your configuration
   ```

3. **Start the services**:
   ```bash
   docker-compose up -d
   ```
   Or using the Makefile in the backend directory:
   ```bash
   cd backend && make docker-up
   ```

4. **Access the application**:
   - Frontend: `http://localhost:4200`
   - Backend API: `http://localhost:8090`
   - Middleware API: `http://localhost:3001`

## Development

Each component has its own detailed README with development instructions:

- [Backend Documentation](./backend/README.md)
- [Frontend Documentation](./frontend/README.md)
- [Middleware Documentation](./middleware/README.md)

## License

See the LICENSE files in individual component directories.
