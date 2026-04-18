#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
pgrx_home="${PGRX_HOME:-${HOME}/.pgrx}"
socket_dir=""
port=""
forwarded_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
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
export PGDATABASE="${PGDATABASE:-postgres}"
export TQV_PSQL_BIN="${TQV_PSQL_BIN:-${pgrx_home}/17.9/pgrx-install/bin/psql}"

exec "${PYTHON:-python3}" "${repo_root}/scripts/load_real_corpus.py" "${forwarded_args[@]}"
