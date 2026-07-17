# Gather

Gather is an event coordination application. This repository is organized as a
Rust/Axum backend and a React/Tailwind frontend so the remaining implementation
issues can add authentication, profiles, events, invitations, RSVP, comments,
activity feeds, email, scheduling, tests, and deployment incrementally.

## Structure

- `backend/` - Axum API server. It currently exposes `/` and `/api/health`.
- `frontend/` - Vite React app styled with Tailwind.
- `.plan` - Architecture and issue dependency notes for the full project.

## Development

Backend:

```bash
export DATABASE_URL=$(cat /workspace/.database_url)
cargo build
HOST=0.0.0.0 PORT=8080 cargo run -p gather-api
```

The API uses SQLx with PostgreSQL. Migrations live in `migrations/` and are
embedded into the backend binary with `sqlx::migrate!()`, then applied at
startup before the server begins accepting traffic.

Optional database tuning environment variables:

- `DATABASE_MAX_CONNECTIONS` - SQLx pool size, default `5`.
- `DATABASE_ACQUIRE_TIMEOUT_SECONDS` - pool acquire timeout, default `10`.
- `DATABASE_SSL_MODE` - optional SQLx PostgreSQL SSL mode override. When unset,
  SQLx uses the mode from `DATABASE_URL`. Supported values are `disable`,
  `prefer`, `require`, `verify-ca`, and `verify-full`.

Frontend:

```bash
cd frontend
npm install
npm run dev
```
