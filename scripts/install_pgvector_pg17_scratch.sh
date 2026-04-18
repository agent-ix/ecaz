#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGVECTOR_REPO="${PGVECTOR_REPO:-/home/peter/dev_bak/pgvector}"
PGRX_HOME="${PGRX_HOME:-/tmp/tqvector_pgrx_home}"
PG_CONFIG_PATH="${PG_CONFIG_PATH:-/home/peter/.pgrx/17.9/pgrx-install/bin/pg_config}"

if [[ ! -d "${PGVECTOR_REPO}" ]]; then
  echo "[install] pgvector repo not found: ${PGVECTOR_REPO}" >&2
  exit 1
fi

if [[ ! -x "${PG_CONFIG_PATH}" ]]; then
  echo "[install] pg_config not found or not executable: ${PG_CONFIG_PATH}" >&2
  exit 1
fi

echo "[install] repo=${REPO_ROOT}"
echo "[install] pgvector_repo=${PGVECTOR_REPO}"
echo "[install] pgrx_home=${PGRX_HOME}"
echo "[install] pg_config=${PG_CONFIG_PATH}"

make -C "${PGVECTOR_REPO}" PG_CONFIG="${PG_CONFIG_PATH}" install

echo "[install] finished installing pgvector"
