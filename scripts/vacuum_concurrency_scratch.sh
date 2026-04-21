#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  scripts/vacuum_concurrency_scratch.sh [--duration SECONDS] [--socket-dir DIR] [--port PORT]

Runs a scratch-cluster ec_hnsw vacuum harness with concurrent INSERT, graph scan,
and VACUUM workers against the same index.

Prerequisites:
  1. A pg17 scratch cluster is running (`cargo pgrx start pg17`)
  2. The extension is installed with the `pg_test` feature so the harness can
     call `tests.ec_hnsw_debug_scan_result_count(...)`

Defaults:
  duration: 60 seconds
EOF
}

duration=60
socket_dir=""
port=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --duration)
            duration="$2"
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

if ! [[ "$duration" =~ ^[0-9]+$ ]] || (( duration <= 0 )); then
    echo "--duration must be a positive integer number of seconds" >&2
    exit 2
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
scratch_psql="${repo_root}/scripts/pg17_scratch_psql.sh"
scratch_psql_args=()
if [[ -n "${socket_dir}" ]]; then
    scratch_psql_args+=( --socket-dir "${socket_dir}" )
fi
if [[ -n "${port}" ]]; then
    scratch_psql_args+=( --port "${port}" )
fi

run_sql() {
    local database="$1"
    local sql="$2"
    "${scratch_psql}" "${scratch_psql_args[@]}" --db "${database}" --sql "${sql}"
}

read_worker_iterations() {
    local name="$1"
    local logfile="${log_dir}/${name}.log"
    local iterations

    if [[ ! -f "${logfile}" ]]; then
        echo "missing worker log: ${logfile}" >&2
        return 1
    fi

    iterations="$(sed -n 's/^iterations=//p' "${logfile}" | tail -n 1)"
    if ! [[ "${iterations}" =~ ^[0-9]+$ ]]; then
        echo "unexpected iteration count in ${logfile}: ${iterations:-<empty>}" >&2
        return 1
    fi

    printf '%s\n' "${iterations}"
}

table_name="ec_hnsw_vacuum_concurrency"
index_name="${table_name}_idx"
ref_index_name="${table_name}_ref_idx"
harness_db="${table_name}_db"
log_dir="$(mktemp -d "/tmp/${table_name}.XXXXXX")"
end_time="$(( $(date +%s) + duration ))"
worker_pids=()
worker_names=()

cleanup() {
    local status=$?
    for pid in "${worker_pids[@]:-}"; do
        kill "$pid" 2>/dev/null || true
    done
    wait 2>/dev/null || true
    if (( status == 0 )); then
        rm -rf "${log_dir}"
    else
        echo "worker logs kept in ${log_dir}" >&2
    fi
}
trap cleanup EXIT INT TERM

run_sql postgres "SELECT 1" >/dev/null
run_sql postgres "DROP DATABASE IF EXISTS ${harness_db} WITH (FORCE)" >/dev/null
run_sql postgres "CREATE DATABASE ${harness_db}" >/dev/null
run_sql "${harness_db}" "CREATE EXTENSION ecaz" >/dev/null

probe_exists="$(
    run_sql "${harness_db}" "SELECT to_regprocedure('tests.ec_hnsw_debug_scan_result_count(oid,real[])') IS NOT NULL"
)"
if [[ "${probe_exists}" != "t" ]]; then
    cat >&2 <<'EOF'
missing tests.ec_hnsw_debug_scan_result_count(oid, real[])
install a pg_test build into the scratch cluster first, for example:
  cargo pgrx install --release --features 'pg17 pg_test' --no-default-features
EOF
    exit 1
fi

run_sql "${harness_db}" "
DROP TABLE IF EXISTS ${table_name} CASCADE;
CREATE TABLE ${table_name} (
    id bigserial PRIMARY KEY,
    embedding ecvector NOT NULL
);
INSERT INTO ${table_name} (embedding)
SELECT encode_to_ecvector(
    ARRAY[
        sin((gs * 0.013)::double precision)::real,
        cos((gs * 0.013)::double precision)::real,
        sin((gs * 0.021)::double precision)::real,
        cos((gs * 0.021)::double precision)::real
    ]::real[],
    4,
    42
)
FROM generate_series(1, 2000) AS gs;
CREATE INDEX ${index_name}
ON ${table_name} USING ec_hnsw (embedding ecvector_ip_ops)
WITH (m = 8, ef_construction = 64);
" >/dev/null

insert_worker() {
    local iterations=0
    while (( $(date +%s) < end_time )); do
        run_sql "${harness_db}" "
        INSERT INTO ${table_name} (embedding)
        SELECT encode_to_ecvector(
            ARRAY[
                (random() * 2.0 - 1.0)::real,
                (random() * 2.0 - 1.0)::real,
                (random() * 2.0 - 1.0)::real,
                (random() * 2.0 - 1.0)::real
            ]::real[],
            4,
            42
        )
        FROM generate_series(1, 4);
        " >/dev/null
        iterations=$((iterations + 1))
    done
    echo "iterations=${iterations}"
}

scan_worker() {
    local query_sql="$1"
    local iterations=0
    while (( $(date +%s) < end_time )); do
        local result_count
        result_count="$(
            run_sql "${harness_db}" "SELECT tests.ec_hnsw_debug_scan_result_count('${index_name}'::regclass::oid, ${query_sql})"
        )"
        if ! [[ "${result_count}" =~ ^[0-9]+$ ]] || (( result_count <= 0 )); then
            echo "unexpected ec_hnsw scan result count: ${result_count}" >&2
            return 1
        fi
        iterations=$((iterations + 1))
    done
    echo "iterations=${iterations}"
}

vacuum_worker() {
    local iterations=0
    while (( $(date +%s) < end_time )); do
        run_sql "${harness_db}" "
        DELETE FROM ${table_name}
        WHERE id IN (
            SELECT id
            FROM ${table_name}
            ORDER BY id
            LIMIT 2
        );
        " >/dev/null
        run_sql "${harness_db}" "VACUUM ${table_name}" >/dev/null
        local live_count
        live_count="$(run_sql "${harness_db}" "SELECT count(*) FROM ${table_name}")"
        if ! [[ "${live_count}" =~ ^[0-9]+$ ]] || (( live_count <= 0 )); then
            echo "unexpected live row count after vacuum: ${live_count}" >&2
            return 1
        fi
        iterations=$((iterations + 1))
    done
    echo "iterations=${iterations}"
}

start_worker() {
    local name="$1"
    shift
    (
        set -euo pipefail
        "$@"
    ) >"${log_dir}/${name}.log" 2>&1 &
    worker_pids+=("$!")
    worker_names+=("${name}")
}

start_worker insert insert_worker
start_worker vacuum vacuum_worker
start_worker scan_a scan_worker "ARRAY[1.0, 0.0, 0.5, -1.0]::real[]"
start_worker scan_b scan_worker "ARRAY[0.0, 1.0, -0.5, 0.25]::real[]"

failed=0
for idx in "${!worker_pids[@]}"; do
    if ! wait "${worker_pids[$idx]}"; then
        echo "worker ${worker_names[$idx]} failed" >&2
        cat "${log_dir}/${worker_names[$idx]}.log" >&2
        failed=1
    fi
done

if (( failed != 0 )); then
    exit 1
fi

run_sql "${harness_db}" "VACUUM (ANALYZE) ${table_name}" >/dev/null
final_live_rows="$(run_sql "${harness_db}" "SELECT count(*) FROM ${table_name}")"
final_live_elements="$(
    run_sql "${harness_db}" "SELECT total_live_nodes FROM ec_hnsw_index_admin_snapshot('${index_name}'::regclass)"
)"
final_reachable_live_elements="$(
    run_sql "${harness_db}" "SELECT tests.ec_hnsw_debug_reachable_live_element_count('${index_name}'::regclass::oid)"
)"
final_scan_result_count="$(
    run_sql "${harness_db}" "SELECT tests.ec_hnsw_debug_scan_result_count('${index_name}'::regclass::oid, ARRAY[1.0, 0.0, 0.5, -1.0]::real[])"
)"
run_sql "${harness_db}" "DROP INDEX IF EXISTS ${ref_index_name}" >/dev/null
run_sql "${harness_db}" "
CREATE INDEX ${ref_index_name}
ON ${table_name} USING ec_hnsw (embedding ecvector_ip_ops)
WITH (m = 8, ef_construction = 64);
" >/dev/null
reference_live_elements="$(
    run_sql "${harness_db}" "SELECT total_live_nodes FROM ec_hnsw_index_admin_snapshot('${ref_index_name}'::regclass)"
)"
reference_reachable_live_elements="$(
    run_sql "${harness_db}" "SELECT tests.ec_hnsw_debug_reachable_live_element_count('${ref_index_name}'::regclass::oid)"
)"
if ! [[ "${final_live_rows}" =~ ^[0-9]+$ ]]; then
    echo "unexpected final live row count: ${final_live_rows}" >&2
    exit 1
fi
if ! [[ "${final_live_elements}" =~ ^[0-9]+$ ]]; then
    echo "unexpected final live element count: ${final_live_elements}" >&2
    exit 1
fi
if ! [[ "${final_reachable_live_elements}" =~ ^[0-9]+$ ]]; then
    echo "unexpected final reachable live element count: ${final_reachable_live_elements}" >&2
    exit 1
fi
if ! [[ "${final_scan_result_count}" =~ ^[0-9]+$ ]]; then
    echo "unexpected final scan result count: ${final_scan_result_count}" >&2
    exit 1
fi
if ! [[ "${reference_live_elements}" =~ ^[0-9]+$ ]]; then
    echo "unexpected reference live element count: ${reference_live_elements}" >&2
    exit 1
fi
if ! [[ "${reference_reachable_live_elements}" =~ ^[0-9]+$ ]]; then
    echo "unexpected reference reachable live element count: ${reference_reachable_live_elements}" >&2
    exit 1
fi
if (( final_live_elements != reference_live_elements )); then
    echo "final_live_elements (${final_live_elements}) != reference_live_elements (${reference_live_elements})" >&2
    exit 1
fi
if (( final_scan_result_count <= 0 )); then
    echo "unexpected final scan result count: ${final_scan_result_count}" >&2
    exit 1
fi
if (( reference_reachable_live_elements <= 0 )); then
    echo "unexpected reference reachable live element count: ${reference_reachable_live_elements}" >&2
    exit 1
fi
if (( final_reachable_live_elements * 100 < reference_reachable_live_elements * 90 )); then
    echo "final reachable live elements (${final_reachable_live_elements}) fell below 90% of rebuilt reference (${reference_reachable_live_elements})" >&2
    exit 1
fi

insert_iterations="$(read_worker_iterations insert)"
vacuum_iterations="$(read_worker_iterations vacuum)"
scan_a_iterations="$(read_worker_iterations scan_a)"
scan_b_iterations="$(read_worker_iterations scan_b)"
reachable_vs_reference_percent="$(( final_reachable_live_elements * 100 / reference_reachable_live_elements ))"

echo "vacuum concurrency harness passed"
echo "duration_seconds=${duration}"
echo "insert_worker_iterations=${insert_iterations}"
echo "vacuum_worker_iterations=${vacuum_iterations}"
echo "scan_a_worker_iterations=${scan_a_iterations}"
echo "scan_b_worker_iterations=${scan_b_iterations}"
echo "final_live_rows=${final_live_rows}"
echo "final_live_elements=${final_live_elements}"
echo "final_reachable_live_elements=${final_reachable_live_elements}"
echo "reference_reachable_live_elements=${reference_reachable_live_elements}"
echo "reachable_vs_reference_percent=${reachable_vs_reference_percent}"
echo "final_scan_result_count=${final_scan_result_count}"
