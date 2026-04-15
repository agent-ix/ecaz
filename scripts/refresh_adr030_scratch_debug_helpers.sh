#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  scripts/refresh_adr030_scratch_debug_helpers.sh [--db DB]

Notes:
  - Refreshes the ADR-030 scratch debug SQL wrappers in the existing scratch DB.
  - Use after installing a new pg_test build when the scratch cluster keeps older
    SQL wrapper signatures around.
EOF
}

db="postgres"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
sql_file="${script_dir}/sql/refresh_adr030_scratch_debug_helpers.sql"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --db)
            db="$2"
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

exec "${script_dir}/pg17_scratch_psql.sh" --db "${db}" --file "${sql_file}"
