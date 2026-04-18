#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  scripts/install_adr030_pg17_pg_test.sh \
      [--pgrx-home /tmp/tqvector_pgrx_home] \
      [--pg-config /home/peter/.pgrx/17.9/pgrx-install/bin/pg_config]

Notes:
  - Installs the pg17 `pg_test` build used by ADR-030 scratch diagnostics.
  - Forces the pg17 `pg_config` path so `cargo pgrx install` does not fall back
    to the system PostgreSQL build.
EOF
}

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"

pgrx_home="${PGRX_HOME:-/tmp/tqvector_pgrx_home}"
pg_config="/home/peter/.pgrx/17.9/pgrx-install/bin/pg_config"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --pgrx-home)
            pgrx_home="$2"
            shift 2
            ;;
        --pg-config)
            pg_config="$2"
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

printf '[install] repo=%s\n' "${repo_root}"
printf '[install] pgrx_home=%s\n' "${pgrx_home}"
printf '[install] pg_config=%s\n' "${pg_config}"

cd "${repo_root}"
exec /bin/bash -lc "PGRX_HOME='${pgrx_home}' cargo pgrx install --pg-config '${pg_config}' --release --features 'pg17 pg_test' --no-default-features"
