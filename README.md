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

Copy `.env.example` to your local environment manager and replace placeholder
values with the provisioned secrets. Runtime secrets such as `.env.production`
must stay out of Git.

Backend:

```bash
set -a
source /workspace/.env.production
set +a
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

Configuration is centralized in `backend/src/config.rs`. Object storage reads
the exact `OBJECT_STORAGE_*` names provisioned by myClawTeam, auth reads the
`MCTAI_AUTH_*` service variables, email reads the optional `MCTAI_EMAIL_*`
proxy variables, and `JWT_SECRET` is treated as legacy compatibility only.
The object storage client in `backend/src/storage.rs` prepends
`OBJECT_STORAGE_PREFIX` to every S3 object key before put, delete, or presigned
read operations.
The email proxy client in `backend/src/email.rs` posts to `MCTAI_EMAIL_URL`
with `MCTAI_EMAIL_APP_TOKEN` as a bearer token and returns a skipped outcome
when the proxy is not configured.
User persistence starts with the `users` table migration and typed Rust models
in `backend/src/models/user.rs`.
Password hashing uses Argon2 in `backend/src/auth/password.rs`. Session setup
uses verified myClawTeam `mctai_session` claims to upsert users without issuing
an additional app JWT.
Registration starts at `POST /api/auth/register` with an email address and
optional frontend `return_to` path. The endpoint sends a myClawTeam auth link
through the central email proxy when configured, and never returns or creates an
app-issued JWT.
Login starts at `POST /api/auth/login` and returns a myClawTeam auth URL. The
platform sets the persistent `mctai_session` cookie after the browser completes
that flow.
Google sign-in starts at `GET /api/auth/google`, which redirects to the same
central myClawTeam auth service instead of handling Google client secrets in
this app. Verified `mctai_session` claims are linked to local users through the
existing user upsert helper.
Authenticated API routes use `backend/src/auth/middleware.rs` to validate the
`mctai_session` cookie against the myClawTeam JWKS endpoint, then upsert the
local user record. `GET /api/auth/me` is the initial protected route.
Password reset starts at `POST /api/auth/password-reset/request`, which emails a
central myClawTeam auth recovery link. `POST /api/auth/password-reset/confirm`
returns the same central-auth handoff because this app does not own password
credentials.

Frontend:

```bash
cd frontend
npm install
npm run dev
```

## Self-Hosted Directory Deployment

This repository includes bare file/directory deployment scripts. They do not
use Docker or a CI/CD pipeline.

```bash
scripts/build-self-hosted.sh
cp /workspace/.env.production dist/self-hosted/.env.production
dist/self-hosted/run.sh
```

`scripts/build-self-hosted.sh` builds the Vite frontend and release Rust API,
then writes a package to `dist/self-hosted/`. `run.sh` loads
`.env.production` from that package when present, defaults to
`HOST=0.0.0.0` and `PORT=8080`, and starts `bin/gather-api`.
