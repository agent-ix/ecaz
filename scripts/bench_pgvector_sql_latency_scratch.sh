#!/usr/bin/env bash
# Run bench_pgvector_sql_latency.sh against the repo-local pgrx scratch cluster.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
pgrx_home="${PGRX_HOME:-${HOME}/.pgrx}"
socket_dir=""
port=""
database="${PGDATABASE:-postgres}"
forwarded_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --db)
      database="$2"
      shift 2
      ;;
    --socket-dir)
      socket_dir="$2"
      shift 2
      ;;
    --port)
      port="$2"
      shift 2
      ;;
    *)
      forwarded_args+=("$1")
      shift
      ;;
  esac
done

if [[ -n "$socket_dir" ]]; then
  export PGHOST="$socket_dir"
elif [[ -z "${PGHOST:-}" ]]; then
  export PGHOST="$("${script_dir}/resolve_scratch_socket_dir.sh")"
fi
if [[ -n "$port" ]]; then
  export PGPORT="$port"
else
  export PGPORT="${PGPORT:-28817}"
fi
export PGDATABASE="$database"
export TQV_PSQL_BIN="${TQV_PSQL_BIN:-${pgrx_home}/17.9/pgrx-install/bin/psql}"

exec bash "${repo_root}/scripts/bench_pgvector_sql_latency.sh" "${forwarded_args[@]}"
