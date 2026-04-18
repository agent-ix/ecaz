#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  scripts/pg17_scratch_psql.sh [--db DB] [--raw] --sql "SELECT 1"
  scripts/pg17_scratch_psql.sh [--db DB] [--raw] --file path/to/query.sql
  scripts/pg17_scratch_psql.sh [--db DB] [--raw]

Defaults:
  socket dir: /tmp/tqvector_pgrx_home
  port:       28817
  database:   postgres
  mode:       aligned-off, tuples-only, tab-separated, ON_ERROR_STOP=1

Environment overrides:
  TQV_PG_SOCKET_DIR
  TQV_PG_PORT
  TQV_PG_DATABASE
  TQV_PSQL_BIN
  PGHOST
EOF
}

port="${TQV_PG_PORT:-28817}"
database="${TQV_PG_DATABASE:-postgres}"
pgrx_home="${PGRX_HOME:-${HOME}/.pgrx}"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
psql_bin="${TQV_PSQL_BIN:-${pgrx_home}/17.9/pgrx-install/bin/psql}"
raw_mode=0
sql=""
sql_file=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --db)
            database="$2"
            shift 2
            ;;
        --raw)
            raw_mode=1
            shift
            ;;
        --sql)
            sql="$2"
            shift 2
            ;;
        --file)
            sql_file="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ -n "$sql" && -n "$sql_file" ]]; then
    echo "--sql and --file are mutually exclusive" >&2
    exit 2
fi

if [[ -n "${TQV_PG_SOCKET_DIR:-}" ]]; then
    socket_dir="${TQV_PG_SOCKET_DIR}"
elif [[ -n "${PGHOST:-}" ]]; then
    socket_dir="${PGHOST}"
else
    socket_dir="$("${script_dir}/resolve_scratch_socket_dir.sh")"
fi

psql_args=(
    -h "$socket_dir"
    -p "$port"
    -d "$database"
    -v ON_ERROR_STOP=1
)

if [[ "$raw_mode" -eq 0 ]]; then
    psql_args+=(-At -F $'\t')
fi

if [[ -n "$sql" ]]; then
    exec "$psql_bin" "${psql_args[@]}" -c "$sql"
fi

if [[ -n "$sql_file" ]]; then
    exec "$psql_bin" "${psql_args[@]}" -f "$sql_file"
fi

exec "$psql_bin" "${psql_args[@]}"
