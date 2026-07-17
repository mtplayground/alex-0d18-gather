#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUT_DIR="${OUT_DIR:-"${ROOT_DIR}/dist/self-hosted"}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

find_cargo() {
  if [[ -n "${CARGO_BIN:-}" ]]; then
    echo "${CARGO_BIN}"
    return
  fi
  if command -v cargo >/dev/null 2>&1; then
    command -v cargo
    return
  fi
  local rustup_root="${RUSTUP_HOME:-/usr/local/rustup}"
  local candidate
  candidate="$(find "${rustup_root}/toolchains" -path '*/bin/cargo' -type f 2>/dev/null | sort -V | tail -n 1 || true)"
  if [[ -n "${candidate}" ]]; then
    echo "${candidate}"
    return
  fi
  echo "cargo"
}

CARGO_BIN="$(find_cargo)"
NPM_BIN="${NPM_BIN:-npm}"

if [[ ! -x "${CARGO_BIN}" ]] && ! command -v "${CARGO_BIN}" >/dev/null 2>&1; then
  echo "missing required command: ${CARGO_BIN}" >&2
  exit 1
fi
if [[ -z "${RUSTC:-}" && -x "$(dirname "${CARGO_BIN}")/rustc" ]]; then
  export RUSTC="$(dirname "${CARGO_BIN}")/rustc"
fi
require_command "${NPM_BIN}"

if [[ -z "${OUT_DIR}" || "${OUT_DIR}" == "/" ]]; then
  echo "refusing to use unsafe OUT_DIR=${OUT_DIR}" >&2
  exit 1
fi

echo "Building Gather frontend"
(
  cd "${ROOT_DIR}/frontend"
  if [[ -f package-lock.json ]]; then
    "${NPM_BIN}" ci
  else
    "${NPM_BIN}" install
  fi
  "${NPM_BIN}" run build
)

echo "Building Gather backend"
(
  cd "${ROOT_DIR}"
  "${CARGO_BIN}" build --release -p gather-api
)

echo "Packaging self-hosted directory: ${OUT_DIR}"
rm -rf "${OUT_DIR}"
mkdir -p "${OUT_DIR}/bin" "${OUT_DIR}/frontend" "${OUT_DIR}/scripts"

cp "${ROOT_DIR}/target/release/gather-api" "${OUT_DIR}/bin/gather-api"
cp -R "${ROOT_DIR}/frontend/dist/." "${OUT_DIR}/frontend/"
cp -R "${ROOT_DIR}/migrations" "${OUT_DIR}/migrations"
cp "${ROOT_DIR}/.env.example" "${OUT_DIR}/.env.example"
cp "${ROOT_DIR}/README.md" "${OUT_DIR}/README.md"
cp "${ROOT_DIR}/scripts/run-self-hosted.sh" "${OUT_DIR}/run.sh"
chmod +x "${OUT_DIR}/bin/gather-api" "${OUT_DIR}/run.sh"

if command -v git >/dev/null 2>&1 && git -C "${ROOT_DIR}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  git -C "${ROOT_DIR}" rev-parse HEAD >"${OUT_DIR}/VERSION"
fi

echo "Self-hosted package ready at ${OUT_DIR}"
echo "Copy .env.production into that directory, then run: ${OUT_DIR}/run.sh"
