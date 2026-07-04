# HMCTS Task Manager API

A task management REST API built with Rust, Axum, and PostgreSQL, following hexagonal (ports-and-adapters) architecture.

## Tech Stack

- **Rust** (edition 2021)
- **Axum** 0.8 – HTTP framework
- **SQLx** 0.8 – async PostgreSQL driver with compile-time query checks and migrations
- **PostgreSQL** 17 – database
- **Tokio** – async runtime
- **docker compose** – local PostgreSQL instance

## Project Structure

```
src/
├── main.rs                            # Composition root: DB pool, migrations, wiring
├── domain/
│   └── task.rs                        # Core entity (Task, Status, ValidationError)
├── application/
│   ├── ports/
│   │   ├── primary.rs                 # Inbound port: TaskUseCase
│   │   └── secondary.rs               # Outbound port: TaskRepository, RepositoryError
│   └── task_service.rs                # Implements inbound port, depends on outbound port
├── adapters/
│   ├── api/
│   │   ├── controllers.rs             # HTTP handlers (inbound adapter)
│   │   ├── routes.rs                  # Axum router setup
│   │   └── error.rs                   # HTTP error responses
│   └── persistence/
│       └── task_repository.rs         # PostgreSQL adapter (implements outbound port)
scripts/
└── init-dbs.sh                        # Creates the `hmcts_dev` database on container start
migrations/
└── 20250101000000_create_tasks_table.sql
```

### Architecture

```
Inbound Adapter (HTTP)  ──►  Primary Port (TaskUseCase)  ──►  Application Service
                                                                │
                                                                ▼
                                                     Secondary Port (TaskRepository)
                                                                │
                                                                ▼
                                                     Outbound Adapter (PostgreSQL)
```

- **Domain** (`domain/`) — pure Rust with zero external dependencies. Defines entities and validation.
- **Application** (`application/`) — contains ports (interfaces at the application boundary) and the service that implements the inbound port.
- **Adapters** (`adapters/`) — inbound (HTTP API) and outbound (PostgreSQL persistence) implementations of the ports.
- The composition root (`main.rs`) is the only place that knows about concrete implementations.

## API Endpoints

| Method   | Path            | Description                |
|----------|-----------------|----------------------------|
| `GET`    | `/health`       | Health check               |
| `POST`   | `/tasks`        | Create a task              |
| `GET`    | `/tasks`        | List all tasks             |
| `GET`    | `/tasks/{id}`   | Get a single task          |
| `PATCH`  | `/tasks/{id}`   | Update task (status, title, description) |
| `DELETE` | `/tasks/{id}`   | Delete a task              |

### Task schema

```json
{
  "id": "uuid",
  "title": "string",
  "description": "string | null",
  "status": "pending | in-progress | completed",
  "dueDate": "ISO-8601 string | null",
  "createdAt": "ISO-8601 string",
  "updatedAt": "ISO-8601 string"
}
```

### Create a task

```
POST /tasks
Content-Type: application/json

{
  "title": "Do the thing",
  "description": "Optional details",
  "dueDate": "2026-06-01T12:00:00Z"
}
```

### Update a task

```
PATCH /tasks/{id}
Content-Type: application/json

{ "status": "in-progress", "title": "Updated title", "description": "Updated description" }
```

All fields (`status`, `title`, `description`) are optional — only provided fields are updated.
Valid statuses: `pending`, `in-progress`, `completed`.

## Docker Setup

### Prerequisites

- Docker and docker compose

### Start PostgreSQL

```bash
docker compose up -d
```

This starts a PostgreSQL 17 container with:
- User: `hmcts`
- Password: `hmcts`
- Database: `hmcts` (default) + `hmcts_dev` (created by init script)
- Port: `5432`

### Run the application

```bash
cargo run
```

The server starts on `http://0.0.0.0:3000`.

### Run tests

```bash
# Requires a running PostgreSQL instance (docker compose up -d)
cargo test
```

## Environment Variables

| Variable        | Default                                              |
|-----------------|------------------------------------------------------|
| `DATABASE_URL`  | `postgres://hmcts:hmcts@localhost:5432/hmcts_dev`    |

A `.env` file is provided with the default value.
