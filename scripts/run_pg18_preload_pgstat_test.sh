#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  scripts/run_pg18_preload_pgstat_test.sh [--port PORT]

Starts a repo-local PostgreSQL 18 cluster under `target/` with
`shared_preload_libraries = 'ecaz'`, creates the extension, runs an
`ec_hnsw` scan in one backend, and verifies from another backend that:

  - the preload setting is active
  - the planner snapshot no longer reports the PG18 pgstat blocker
  - `ecaz_stats()` exposes shared counter deltas across backends

Prerequisite:
  - the current build is already installed into the local pgrx PG18 tree
    (for example via `cargo pgrx test pg18` or `cargo pgrx install`)
EOF
}

port="28818"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --port)
            port="$2"
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

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
pgrx_home="${PGRX_HOME:-${HOME}/.pgrx}"

pg_config="$(find "${pgrx_home}" -maxdepth 5 -path '*/18*/pgrx-install/bin/pg_config' -print | sort -V | tail -n 1)"
if [[ -z "${pg_config}" ]]; then
    echo "could not find a PG18 pgrx pg_config under ${pgrx_home}" >&2
    exit 1
fi

bin_dir="$(dirname "${pg_config}")"
install_root="$(cd -- "${bin_dir}/.." && pwd)"
initdb_bin="${bin_dir}/initdb"
pg_ctl_bin="${bin_dir}/pg_ctl"
psql_bin="${bin_dir}/psql"

control_file="${install_root}/share/postgresql/extension/ecaz.control"
library_file="${install_root}/lib/postgresql/ecaz.so"
if [[ ! -f "${control_file}" || ! -f "${library_file}" ]]; then
    echo "ecaz is not installed in the local PG18 pgrx tree at ${install_root}" >&2
    echo "run \`cargo pgrx test pg18\` or \`cargo pgrx install --features 'pg18 pg_test' --no-default-features\` first" >&2
    exit 1
fi

cluster_root="${repo_root}/target/pg18-preload-pgstat"
data_dir="${cluster_root}/data"
log_file="${cluster_root}/postgres.log"
database="postgres"
host="127.0.0.1"

mkdir -p "${cluster_root}"

stop_cluster() {
    if [[ -f "${data_dir}/PG_VERSION" ]] && "${pg_ctl_bin}" -D "${data_dir}" status >/dev/null 2>&1; then
        "${pg_ctl_bin}" -D "${data_dir}" -m fast -w stop >/dev/null
    fi
}

trap stop_cluster EXIT

if [[ ! -f "${data_dir}/PG_VERSION" ]]; then
    "${initdb_bin}" -D "${data_dir}" -A trust -U postgres >/dev/null
fi

stop_cluster

selected_port=""
for port_offset in $(seq 0 9); do
    candidate_port="$((port + port_offset))"
    : > "${log_file}"
    if "${pg_ctl_bin}" \
        -D "${data_dir}" \
        -l "${log_file}" \
        -o "-p ${candidate_port} -c listen_addresses=${host} -c shared_preload_libraries=ecaz" \
        -w start >/dev/null 2>/dev/null; then
        selected_port="${candidate_port}"
        break
    fi

    if ! grep -q "Address already in use" "${log_file}"; then
        cat "${log_file}" >&2
        exit 1
    fi
done

if [[ -z "${selected_port}" ]]; then
    echo "could not find a free local port starting at ${port}" >&2
    exit 1
fi

psql_base=(
    "${psql_bin}"
    -X
    -U postgres
    -h "${host}"
    -p "${selected_port}"
    -d "${database}"
    -v ON_ERROR_STOP=1
    -At
    -F $'\t'
)

run_sql() {
    "${psql_base[@]}" -c "$1"
}

show_sql() {
    local sql="$1"
    printf '[pg18-preload] sql: %s\n' "${sql}"
    run_sql "${sql}"
}

assert_eq() {
    local actual="$1"
    local expected="$2"
    local message="$3"
    if [[ "${actual}" != "${expected}" ]]; then
        printf 'assertion failed: %s\nexpected: %s\nactual:   %s\n' "${message}" "${expected}" "${actual}" >&2
        exit 1
    fi
}

assert_contains() {
    local haystack="$1"
    local needle="$2"
    local message="$3"
    if [[ "${haystack}" != *"${needle}"* ]]; then
        printf 'assertion failed: %s\nexpected substring: %s\nactual: %s\n' "${message}" "${needle}" "${haystack}" >&2
        exit 1
    fi
}

assert_gt() {
    local lhs="$1"
    local rhs="$2"
    local message="$3"
    if (( lhs <= rhs )); then
        printf 'assertion failed: %s\nexpected %s > %s\n' "${message}" "${lhs}" "${rhs}" >&2
        exit 1
    fi
}

preload_setting="$(run_sql "SHOW shared_preload_libraries")"
assert_contains "${preload_setting}" "ecaz" "shared_preload_libraries should include ecaz"

show_sql "
DROP TABLE IF EXISTS pg18_preload_pgstat_fixture CASCADE;
DROP EXTENSION IF EXISTS ecaz CASCADE;
CREATE EXTENSION ecaz;
CREATE TABLE pg18_preload_pgstat_fixture (id bigint primary key, embedding ecvector);
INSERT INTO pg18_preload_pgstat_fixture VALUES
  (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
  (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42)),
  (3, encode_to_ecvector(ARRAY[0.5, 0.5, -0.5, 1.0], 4, 42));
CREATE INDEX pg18_preload_pgstat_fixture_idx ON pg18_preload_pgstat_fixture USING ec_hnsw (embedding ecvector_ip_ops);
" >/dev/null

planner_snapshot="$(
    run_sql "
    SELECT pg18_diagnostics_surface_ready, next_pg18_blocker
    FROM ec_hnsw_planner_integration_snapshot('pg18_preload_pgstat_fixture_idx'::regclass)
"
)"
IFS=$'\t' read -r diagnostics_ready next_pg18_blocker <<< "${planner_snapshot}"
assert_eq "${diagnostics_ready}" "t" "planner snapshot should report PG18 diagnostics surface ready under preload"
assert_eq "${next_pg18_blocker}" "no merged PG18 blocker remains on main" "planner snapshot should clear the preload-only blocker under preload"

baseline_stats="$(run_sql "SELECT total_scans_started, total_distance_calcs FROM ecaz_stats()")"
IFS=$'\t' read -r baseline_scans baseline_distance <<< "${baseline_stats}"

show_sql "
SET enable_seqscan = off;
SELECT id
FROM pg18_preload_pgstat_fixture
ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.5, -1.0]::real[]
LIMIT 1
" >/dev/null

shared_stats="$(run_sql "SELECT total_scans_started, total_distance_calcs FROM ecaz_stats()")"
IFS=$'\t' read -r shared_scans shared_distance <<< "${shared_stats}"

assert_gt "${shared_scans}" "${baseline_scans}" "observer backend should see shared scan count increase"
assert_gt "${shared_distance}" "${baseline_distance}" "observer backend should see shared distance calculations increase"

printf '[pg18-preload] shared_preload_libraries=%s\n' "${preload_setting}"
printf '[pg18-preload] baseline_scans=%s baseline_distance_calcs=%s\n' "${baseline_scans}" "${baseline_distance}"
printf '[pg18-preload] shared_scans=%s shared_distance_calcs=%s\n' "${shared_scans}" "${shared_distance}"
printf '[pg18-preload] preload-aware PG18 shared pgstat validation passed\n'
