#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "$(basename "${SCRIPT_DIR}")" == "scripts" ]]; then
  ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
  DEFAULT_APP_DIR="${ROOT_DIR}/dist/self-hosted"
else
  ROOT_DIR="${SCRIPT_DIR}"
  DEFAULT_APP_DIR="${SCRIPT_DIR}"
fi

APP_DIR="${APP_DIR:-"${DEFAULT_APP_DIR}"}"
BIN_PATH="${BIN_PATH:-"${APP_DIR}/bin/gather-api"}"

if [[ -z "${ENV_FILE:-}" ]]; then
  if [[ -f "${APP_DIR}/.env.production" ]]; then
    ENV_FILE="${APP_DIR}/.env.production"
  elif [[ -f "${ROOT_DIR}/.env.production" ]]; then
    ENV_FILE="${ROOT_DIR}/.env.production"
  else
    ENV_FILE=""
  fi
fi

if [[ -n "${ENV_FILE}" ]]; then
  if [[ ! -f "${ENV_FILE}" ]]; then
    echo "ENV_FILE does not exist: ${ENV_FILE}" >&2
    exit 1
  fi
  set -a
  # shellcheck disable=SC1090
  source "${ENV_FILE}"
  set +a
else
  echo "No env file found; relying on current process environment" >&2
fi

if [[ ! -x "${BIN_PATH}" ]]; then
  echo "backend binary is not executable: ${BIN_PATH}" >&2
  echo "Run scripts/build-self-hosted.sh first or set BIN_PATH." >&2
  exit 1
fi

export HOST="${HOST:-0.0.0.0}"
export PORT="${PORT:-8080}"

if [[ -z "${FRONTEND_DIST_DIR:-}" && -f "${APP_DIR}/frontend/index.html" ]]; then
  export FRONTEND_DIST_DIR="${APP_DIR}/frontend"
fi

if [[ -n "${FRONTEND_DIST_DIR:-}" && ! -f "${FRONTEND_DIST_DIR}/index.html" ]]; then
  echo "FRONTEND_DIST_DIR must contain index.html: ${FRONTEND_DIST_DIR}" >&2
  exit 1
fi

echo "Starting Gather API on ${HOST}:${PORT}"
exec "${BIN_PATH}"
