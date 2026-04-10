#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
pgrx_home="${PGRX_HOME:-${HOME}/.pgrx}"

export PGHOST="${PGHOST:-/tmp/tqvector_pgrx_home}"
export PGPORT="${PGPORT:-28817}"
export PGDATABASE="${PGDATABASE:-postgres}"
export TQV_PSQL_BIN="${TQV_PSQL_BIN:-${pgrx_home}/17.9/pgrx-install/bin/psql}"

exec "${PYTHON:-python3}" "${repo_root}/scripts/load_real_corpus.py" "$@"
