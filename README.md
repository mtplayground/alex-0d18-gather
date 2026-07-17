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
cargo build
HOST=0.0.0.0 PORT=8080 cargo run -p gather-api
```

Frontend:

```bash
cd frontend
npm install
npm run dev
```
