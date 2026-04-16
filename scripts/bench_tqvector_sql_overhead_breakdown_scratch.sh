#!/usr/bin/env bash
# Run bench_tqvector_sql_overhead_breakdown.sh against the repo-local pgrx
# scratch cluster.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
pgrx_home="${PGRX_HOME:-${HOME}/.pgrx}"

if [[ -z "${PGHOST:-}" ]]; then
  export PGHOST="$("${script_dir}/resolve_scratch_socket_dir.sh")"
fi
export PGPORT="${PGPORT:-28817}"
export PGDATABASE="${PGDATABASE:-postgres}"
export TQV_PSQL_BIN="${TQV_PSQL_BIN:-${pgrx_home}/17.9/pgrx-install/bin/psql}"

exec bash "${repo_root}/scripts/bench_tqvector_sql_overhead_breakdown.sh" "$@"
