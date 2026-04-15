#!/usr/bin/env bash
# Break down tqvector SQL latency into encode, internal scan, and residual SQL
# overhead on a real-corpus lane.
set -euo pipefail

PSQL_BIN="${TQV_PSQL_BIN:-psql}"

print_help() {
  cat <<'USAGE'
Usage:
  bash scripts/bench_tqvector_sql_overhead_breakdown.sh \
      --corpus-table <table> \
      --query-table <table> \
      --index-name <index> \
      [--bits N] [--seed N] [--ef-search csv] [--query-limit N] \
      [--result-limit K] [--project-attnum N] [--cache-state LABEL] \
      [--warmup-passes N] [--session-mode MODE] [--timing-mode MODE] \
      [--output FILE]

Options:
  --corpus-table  tqvector corpus table to scan. Must expose:
                  - id bigint/int
                  - embedding tqvector
  --query-table   Query table to read. Must expose:
                  - source real[]
  --index-name    Exact tqhnsw index name expected for every measured cell.
  --bits          Quantizer bits for encode_to_tqvector timing. Default: 4.
  --seed          Quantizer seed for encode_to_tqvector timing. Default: 42.
  --ef-search     Comma-separated ef_search list. Default: 40,64,128,320.
  --query-limit   Cap the number of queries per ef_search cell. Default:
                  all rows in --query-table.
  --result-limit  Ordered scan limit used for SQL and limited internal scan
                  profiling. Default: 10.
  --project-attnum
                  Heap attribute number to project during the executor-like
                  slot-fetch profile. Default: 1.
  --cache-state   Free-form label recorded in the stdout banner. Default:
                  unspecified.
  --warmup-passes Number of full query-set warmup passes before timing each
                  ef_search cell. Default: 0.
  --session-mode Session reuse mode for the full SQL timing leg:
                  per-query (default) opens one psql/backend per timed query.
                  per-cell runs all warmup + timed queries for the cell in a
                  single backend session.
  --timing-mode  How to time the full SQL leg:
                  explain (default) uses per-query EXPLAIN (ANALYZE, FORMAT
                  JSON).
                  plain-server runs the plain ordered query and measures it
                  with server-side clock_timestamp() around a MATERIALIZED
                  subquery.
  --output        Append per-cell summary lines to FILE in addition to stdout.
  -h, --help      Show this message and exit.

Notes:
  - Requires tests.tqhnsw_debug_scan_profile_limited(...) and
    tests.tqhnsw_debug_scan_hot_path_profile(...) and
    tests.tqhnsw_debug_scan_heap_fetch_profile(...). On the scratch cluster,
    refresh them with scripts/refresh_adr030_scratch_debug_helpers.sh after
    a new pg_test install.
USAGE
}

CORPUS_TABLE=""
QUERY_TABLE=""
INDEX_NAME=""
BITS="4"
SEED="42"
EF_SEARCH_CSV="40,64,128,320"
QUERY_LIMIT=""
RESULT_LIMIT="10"
PROJECT_ATTNUM="1"
CACHE_STATE="unspecified"
WARMUP_PASSES="0"
SESSION_MODE="per-query"
TIMING_MODE="explain"
OUTPUT_FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --corpus-table)
      CORPUS_TABLE="$2"; shift 2 ;;
    --query-table)
      QUERY_TABLE="$2"; shift 2 ;;
    --index-name)
      INDEX_NAME="$2"; shift 2 ;;
    --bits)
      BITS="$2"; shift 2 ;;
    --seed)
      SEED="$2"; shift 2 ;;
    --ef-search)
      EF_SEARCH_CSV="$2"; shift 2 ;;
    --query-limit)
      QUERY_LIMIT="$2"; shift 2 ;;
    --result-limit)
      RESULT_LIMIT="$2"; shift 2 ;;
    --project-attnum)
      PROJECT_ATTNUM="$2"; shift 2 ;;
    --cache-state)
      CACHE_STATE="$2"; shift 2 ;;
    --warmup-passes)
      WARMUP_PASSES="$2"; shift 2 ;;
    --session-mode)
      SESSION_MODE="$2"; shift 2 ;;
    --timing-mode)
      TIMING_MODE="$2"; shift 2 ;;
    --output)
      OUTPUT_FILE="$2"; shift 2 ;;
    -h|--help)
      print_help; exit 0 ;;
    *)
      echo "unknown argument: $1" >&2
      print_help >&2
      exit 2 ;;
  esac
done

show_pg_setting() {
  local name="$1"
  "$PSQL_BIN" -X -A -t -q -c "SHOW ${name};"
}

print_real_corpus_env_banner() {
  local os_name cpu_model ram_total shared_buffers work_mem parallel_workers

  os_name="$(uname -srmo 2>/dev/null || echo unknown)"
  cpu_model="$(
    awk -F: '/model name/ {sub(/^[ \t]+/, "", $2); print $2; exit}' /proc/cpuinfo 2>/dev/null \
      || true
  )"
  ram_total="$(
    awk '/MemTotal:/ {print $2 " " $3; exit}' /proc/meminfo 2>/dev/null || true
  )"
  shared_buffers="$(show_pg_setting shared_buffers)"
  work_mem="$(show_pg_setting work_mem)"
  parallel_workers="$(show_pg_setting max_parallel_workers_per_gather)"

  if [[ -z "$cpu_model" ]]; then
    cpu_model="unknown"
  fi
  if [[ -z "$ram_total" ]]; then
    ram_total="unknown"
  fi

  echo "OS:           $os_name"
  echo "CPU:          $cpu_model"
  echo "RAM:          $ram_total"
  echo "cache state:  $CACHE_STATE"
  echo "shared_buffers: $shared_buffers"
  echo "work_mem:       $work_mem"
  echo "max_parallel_workers_per_gather: $parallel_workers"
}

verify_expected_index_plan() {
  local corpus_table="$1"
  local query_literal="$2"
  local result_limit="$3"
  local ef="$4"
  local expected_index="$5"

  local plan_text
  plan_text="$("$PSQL_BIN" -X -A -t -q <<SQL
SET tqhnsw.ef_search = ${ef};
EXPLAIN
SELECT id FROM ${corpus_table}
ORDER BY embedding <#> '${query_literal}'::real[]
LIMIT ${result_limit};
SQL
)"

  if ! grep -Fq "${expected_index}" <<<"$plan_text"; then
    echo "planner verification failed for ${expected_index} at ef_search=${ef}" >&2
    echo "expected the measured query to use ${expected_index}, but it did not." >&2
    echo "aborting before timing so this run does not record Seq Scan + Sort" >&2
    echo "or the wrong tqhnsw index for the requested corpus." >&2
    echo >&2
    echo "Representative EXPLAIN plan:" >&2
    echo "${plan_text}" >&2
    return 1
  fi

  echo "[verified] planner uses ${expected_index} at ef_search=${ef}" >&2
}

require_regprocedure() {
  local signature="$1"
  local hint="$2"

  local exists
  exists="$("$PSQL_BIN" -X -A -t -q -c "SELECT to_regprocedure('${signature}') IS NOT NULL;")"
  if [[ "$exists" != "t" ]]; then
    echo "required helper ${signature} is missing" >&2
    echo "$hint" >&2
    exit 1
  fi
}

if [[ -z "$CORPUS_TABLE" || -z "$QUERY_TABLE" || -z "$INDEX_NAME" ]]; then
  echo "--corpus-table, --query-table, and --index-name are required" >&2
  exit 2
fi
if [[ ! "$CORPUS_TABLE" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]]; then
  echo "invalid corpus table: $CORPUS_TABLE" >&2
  exit 2
fi
if [[ ! "$QUERY_TABLE" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]]; then
  echo "invalid query table: $QUERY_TABLE" >&2
  exit 2
fi
if [[ ! "$INDEX_NAME" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]]; then
  echo "invalid index name: $INDEX_NAME" >&2
  exit 2
fi
if [[ ! "$BITS" =~ ^[0-9]+$ ]]; then
  echo "invalid bits value: $BITS" >&2
  exit 2
fi
if [[ ! "$SEED" =~ ^-?[0-9]+$ ]]; then
  echo "invalid seed value: $SEED" >&2
  exit 2
fi
if [[ ! "$RESULT_LIMIT" =~ ^[0-9]+$ ]]; then
  echo "invalid result limit: $RESULT_LIMIT" >&2
  exit 2
fi
if [[ ! "$PROJECT_ATTNUM" =~ ^[0-9]+$ ]]; then
  echo "invalid project attnum: $PROJECT_ATTNUM" >&2
  exit 2
fi
if [[ ! "$WARMUP_PASSES" =~ ^[0-9]+$ ]]; then
  echo "invalid warmup pass count: $WARMUP_PASSES" >&2
  exit 2
fi
if [[ "$SESSION_MODE" != "per-query" && "$SESSION_MODE" != "per-cell" ]]; then
  echo "invalid session mode: $SESSION_MODE (expected per-query or per-cell)" >&2
  exit 2
fi
if [[ "$TIMING_MODE" != "explain" && "$TIMING_MODE" != "plain-server" ]]; then
  echo "invalid timing mode: $TIMING_MODE (expected explain or plain-server)" >&2
  exit 2
fi

IFS=',' read -r -a ef_list <<< "$EF_SEARCH_CSV"
for ef in "${ef_list[@]}"; do
  if [[ ! "$ef" =~ ^[0-9]+$ ]]; then
    echo "invalid ef_search value: $ef" >&2
    exit 2
  fi
done

exists="$("$PSQL_BIN" -X -A -t -q -c "SELECT to_regclass('${INDEX_NAME}') IS NOT NULL;")"
if [[ "$exists" != "t" ]]; then
  echo "index ${INDEX_NAME} not found" >&2
  exit 1
fi

require_regprocedure \
  "tests.tqhnsw_debug_scan_profile_limited(oid,real[],integer)" \
  "run scripts/refresh_adr030_scratch_debug_helpers.sh after installing a new pg_test build"
require_regprocedure \
  "tests.tqhnsw_debug_scan_hot_path_profile(oid,real[])" \
  "run scripts/refresh_adr030_scratch_debug_helpers.sh after installing a new pg_test build"
require_regprocedure \
  "tests.tqhnsw_debug_scan_heap_fetch_profile(oid,real[],integer,integer)" \
  "run scripts/refresh_adr030_scratch_debug_helpers.sh after installing a new pg_test build"

echo "=== tqvector SQL overhead breakdown ==="
echo "Database:     ${PGDATABASE:-(libpq default)}"
echo "Corpus table: $CORPUS_TABLE"
echo "Query table:  $QUERY_TABLE"
echo "index name:   $INDEX_NAME"
echo "bits:         $BITS"
echo "seed:         $SEED"
echo "ef_search:    $EF_SEARCH_CSV"
echo "result limit: $RESULT_LIMIT"
echo "project attnum: $PROJECT_ATTNUM"
if [[ -n "$QUERY_LIMIT" ]]; then
  echo "query limit:  $QUERY_LIMIT"
fi
if [[ -n "$OUTPUT_FILE" ]]; then
  echo "summary file: $OUTPUT_FILE"
fi
if [[ "$WARMUP_PASSES" != "0" ]]; then
  echo "warmup passes: $WARMUP_PASSES"
fi
echo "session mode: $SESSION_MODE"
echo "timing mode:  $TIMING_MODE"
print_real_corpus_env_banner
echo

query_count="$("$PSQL_BIN" -X -A -t -q -c "SELECT count(*) FROM ${QUERY_TABLE};")"
if [[ -z "$query_count" || "$query_count" == "0" ]]; then
  echo "no queries found in ${QUERY_TABLE}" >&2
  exit 1
fi
if [[ -n "$QUERY_LIMIT" ]]; then
  if (( QUERY_LIMIT < query_count )); then
    query_count="$QUERY_LIMIT"
  fi
fi

queries_tsv="$(mktemp -t tqv_overhead_queries.XXXXXX.tsv)"
sql_cell_file="$(mktemp -t tqv_overhead_sql_cell.XXXXXX.sql)"
sql_results_file="$(mktemp -t tqv_overhead_sql.XXXXXX.txt)"
encode_results_file="$(mktemp -t tqv_overhead_encode.XXXXXX.txt)"
profile_rows_file="$(mktemp -t tqv_overhead_profile.XXXXXX.tsv)"
hot_path_rows_file="$(mktemp -t tqv_overhead_hot.XXXXXX.tsv)"
heap_fetch_rows_file="$(mktemp -t tqv_overhead_heap.XXXXXX.tsv)"
trap 'rm -f "$queries_tsv" "$sql_cell_file" "$sql_results_file" "$encode_results_file" "$profile_rows_file" "$hot_path_rows_file" "$heap_fetch_rows_file"' EXIT

query_select="SELECT source FROM ${QUERY_TABLE} ORDER BY id"
if [[ -n "$QUERY_LIMIT" ]]; then
  query_select="${query_select} LIMIT ${QUERY_LIMIT}"
fi
"$PSQL_BIN" -X -A -t -q -c "${query_select};" > "$queries_tsv"
echo "queries available: $query_count"
if grep -n "'" "$queries_tsv" >/dev/null; then
  echo "unexpected single quote in query literal output from ${QUERY_TABLE}" >&2
  exit 2
fi

probe_query=""
while IFS= read -r probe_query; do
  [[ -n "$probe_query" ]] && break
done < "$queries_tsv"
if [[ -z "$probe_query" ]]; then
  echo "no probe query found in ${QUERY_TABLE}" >&2
  exit 1
fi

for ef in "${ef_list[@]}"; do
  echo "--- ef_search=${ef} ---"
  verify_expected_index_plan "$CORPUS_TABLE" "$probe_query" "$RESULT_LIMIT" "$ef" "$INDEX_NAME"

  : > "$sql_results_file"
  : > "$sql_cell_file"
  : > "$encode_results_file"

  if [[ "$SESSION_MODE" == "per-cell" ]]; then
    if (( WARMUP_PASSES > 0 )); then
      printf '\\o /dev/null\n' >> "$sql_cell_file"
      for ((warmup_pass = 1; warmup_pass <= WARMUP_PASSES; warmup_pass++)); do
        echo "[warmup] ef_search=${ef} pass ${warmup_pass}/${WARMUP_PASSES}" >&2
        while IFS= read -r query_line; do
          [[ -z "$query_line" ]] && continue
          cat >> "$sql_cell_file" <<SQL
SET tqhnsw.ef_search = ${ef};
SELECT id FROM ${CORPUS_TABLE}
ORDER BY embedding <#> '${query_line}'::real[]
LIMIT ${RESULT_LIMIT};
SQL
        done < "$queries_tsv"
      done
      printf '\\o\n' >> "$sql_cell_file"
    fi

    while IFS= read -r query_line; do
      [[ -z "$query_line" ]] && continue
      if [[ "$TIMING_MODE" == "explain" ]]; then
        cat >> "$sql_cell_file" <<SQL
SET tqhnsw.ef_search = ${ef};
EXPLAIN (ANALYZE, TIMING, FORMAT JSON)
SELECT id FROM ${CORPUS_TABLE}
ORDER BY embedding <#> '${query_line}'::real[]
LIMIT ${RESULT_LIMIT};
SQL
      else
        cat >> "$sql_cell_file" <<SQL
SET tqhnsw.ef_search = ${ef};
WITH started AS (
  SELECT clock_timestamp() AS t0
),
measured AS MATERIALIZED (
  SELECT id FROM ${CORPUS_TABLE}
  ORDER BY embedding <#> '${query_line}'::real[]
  LIMIT ${RESULT_LIMIT}
),
finished AS (
  SELECT clock_timestamp() AS t1, count(*) AS rows_seen FROM measured
)
SELECT extract(epoch FROM (finished.t1 - started.t0)) * 1000.0
FROM started, finished;
SQL
      fi
    done < "$queries_tsv"
    "$PSQL_BIN" -X -A -t -q -f "$sql_cell_file" > "$sql_results_file"
  else
    if (( WARMUP_PASSES > 0 )); then
      for ((warmup_pass = 1; warmup_pass <= WARMUP_PASSES; warmup_pass++)); do
        echo "[warmup] ef_search=${ef} pass ${warmup_pass}/${WARMUP_PASSES}" >&2
        while IFS= read -r query_line; do
          [[ -z "$query_line" ]] && continue
          "$PSQL_BIN" -X -A -t -q > /dev/null <<SQL
SET tqhnsw.ef_search = ${ef};
SELECT id FROM ${CORPUS_TABLE}
ORDER BY embedding <#> '${query_line}'::real[]
LIMIT ${RESULT_LIMIT};
SQL
        done < "$queries_tsv"
      done
    fi

    while IFS= read -r query_line; do
      [[ -z "$query_line" ]] && continue
      if [[ "$TIMING_MODE" == "explain" ]]; then
        "$PSQL_BIN" -X -A -t -q <<SQL >> "$sql_results_file"
SET tqhnsw.ef_search = ${ef};
EXPLAIN (ANALYZE, TIMING, FORMAT JSON)
SELECT id FROM ${CORPUS_TABLE}
ORDER BY embedding <#> '${query_line}'::real[]
LIMIT ${RESULT_LIMIT};
SQL
      else
        "$PSQL_BIN" -X -A -t -q <<SQL >> "$sql_results_file"
SET tqhnsw.ef_search = ${ef};
WITH started AS (
  SELECT clock_timestamp() AS t0
),
measured AS MATERIALIZED (
  SELECT id FROM ${CORPUS_TABLE}
  ORDER BY embedding <#> '${query_line}'::real[]
  LIMIT ${RESULT_LIMIT}
),
finished AS (
  SELECT clock_timestamp() AS t1, count(*) AS rows_seen FROM measured
)
SELECT extract(epoch FROM (finished.t1 - started.t0)) * 1000.0
FROM started, finished;
SQL
      fi
    done < "$queries_tsv"
  fi

  while IFS= read -r query_line; do
    [[ -z "$query_line" ]] && continue
    "$PSQL_BIN" -X -A -t -q <<SQL >> "$encode_results_file"
WITH started AS (
  SELECT clock_timestamp() AS t0
),
measured AS MATERIALIZED (
  SELECT encode_to_tqvector('${query_line}'::real[], ${BITS}, ${SEED}) AS vec
),
finished AS (
  SELECT clock_timestamp() AS t1, count(*) AS rows_seen FROM measured
)
SELECT extract(epoch FROM (finished.t1 - started.t0)) * 1000.0
FROM started, finished;
SQL
  done < "$queries_tsv"

  "$PSQL_BIN" -X -A -F $'\t' -t -q <<SQL > "$profile_rows_file"
SET tqhnsw.ef_search = ${ef};
SELECT
  p.rescan_elapsed_us,
  p.emit_elapsed_us,
  p.total_elapsed_us,
  p.result_count,
  p.total_heap_tids_returned
FROM (
  SELECT source FROM ${QUERY_TABLE} ORDER BY id
  LIMIT ${query_count}
) AS q
CROSS JOIN LATERAL tests.tqhnsw_debug_scan_profile_limited(
  '${INDEX_NAME}'::regclass::oid,
  q.source,
  ${RESULT_LIMIT}
) AS p;
SQL

  "$PSQL_BIN" -X -A -F $'\t' -t -q <<SQL > "$hot_path_rows_file"
SET tqhnsw.ef_search = ${ef};
SELECT
  p.rescan_amrescan_total_elapsed_us,
  p.rescan_query_decode_elapsed_us,
  p.rescan_prepare_query_elapsed_us,
  p.rescan_frontier_consume_elapsed_us,
  p.rescan_graph_result_materialize_elapsed_us,
  p.candidate_score_elapsed_us
FROM (
  SELECT source FROM ${QUERY_TABLE} ORDER BY id
  LIMIT ${query_count}
) AS q
CROSS JOIN LATERAL tests.tqhnsw_debug_scan_hot_path_profile(
  '${INDEX_NAME}'::regclass::oid,
  q.source
) AS p;
SQL

  "$PSQL_BIN" -X -A -F $'\t' -t -q <<SQL > "$heap_fetch_rows_file"
SET tqhnsw.ef_search = ${ef};
SELECT
  p.rescan_elapsed_us,
  p.emit_elapsed_us,
  p.total_elapsed_us,
  p.slot_fetch_elapsed_us,
  p.projection_elapsed_us,
  p.result_count,
  p.slot_fetch_count,
  p.projected_count
FROM (
  SELECT source FROM ${QUERY_TABLE} ORDER BY id
  LIMIT ${query_count}
) AS q
CROSS JOIN LATERAL tests.tqhnsw_debug_scan_heap_fetch_profile(
  '${INDEX_NAME}'::regclass::oid,
  q.source,
  ${RESULT_LIMIT},
  ${PROJECT_ATTNUM}
) AS p;
SQL

  python3 - \
    "$sql_results_file" \
    "$encode_results_file" \
    "$profile_rows_file" \
    "$hot_path_rows_file" \
    "$heap_fetch_rows_file" \
    "$ef" \
    "$OUTPUT_FILE" \
    "$TIMING_MODE" <<'PY'
import json
import statistics
import sys


(
    sql_results_path,
    encode_results_path,
    profile_rows_path,
    hot_path_rows_path,
    heap_fetch_rows_path,
    ef_str,
    output_path,
    timing_mode,
) = sys.argv[1:]


def parse_explain_times(path: str) -> list[float]:
    with open(path, "r", encoding="utf-8") as fh:
        content = fh.read()
    times_ms: list[float] = []
    depth = 0
    start = None
    for i, c in enumerate(content):
        if c == "[" and depth == 0:
            start = i
        if c == "[":
            depth += 1
        if c == "]":
            depth -= 1
            if depth == 0 and start is not None:
                try:
                    plan = json.loads(content[start : i + 1])
                    times_ms.append(float(plan[0]["Execution Time"]))
                except (json.JSONDecodeError, KeyError, IndexError, ValueError):
                    pass
                start = None
    return times_ms


def parse_float_lines(path: str) -> list[float]:
    values: list[float] = []
    with open(path, "r", encoding="utf-8") as fh:
        for raw_line in fh:
            line = raw_line.strip()
            if not line:
                continue
            values.append(float(line))
    return values


def parse_tsv_rows(path: str, width: int) -> list[list[float]]:
    rows: list[list[float]] = []
    with open(path, "r", encoding="utf-8") as fh:
        for raw_line in fh:
            line = raw_line.strip()
            if not line:
                continue
            parts = line.split("\t")
            if len(parts) != width:
                raise SystemExit(f"expected {width} tab-separated fields, got {len(parts)} in: {line}")
            rows.append([float(value) for value in parts])
    return rows


if timing_mode == "explain":
    sql_times_ms = parse_explain_times(sql_results_path)
else:
    sql_times_ms = parse_float_lines(sql_results_path)
encode_times_ms = parse_float_lines(encode_results_path)
profile_rows = parse_tsv_rows(profile_rows_path, 5)
hot_path_rows = parse_tsv_rows(hot_path_rows_path, 6)
heap_fetch_rows = parse_tsv_rows(heap_fetch_rows_path, 8)

if not sql_times_ms:
    raise SystemExit("no SQL timings parsed")
if not encode_times_ms:
    raise SystemExit("no encode timings parsed")
if not profile_rows:
    raise SystemExit("no limited scan-profile rows parsed")
if not hot_path_rows:
    raise SystemExit("no hot-path rows parsed")
if not heap_fetch_rows:
    raise SystemExit("no heap-fetch rows parsed")

expected_n = len(sql_times_ms)
if (
    len(encode_times_ms) != expected_n
    or len(profile_rows) != expected_n
    or len(hot_path_rows) != expected_n
    or len(heap_fetch_rows) != expected_n
):
    raise SystemExit(
        "measurement row-count mismatch: "
        f"sql={len(sql_times_ms)} encode={len(encode_times_ms)} "
        f"profile={len(profile_rows)} hot={len(hot_path_rows)} "
        f"heap={len(heap_fetch_rows)}"
    )


def mean(values: list[float]) -> float:
    return statistics.fmean(values)


def mean_column(rows: list[list[float]], index: int, *, scale: float = 1.0) -> float:
    return mean([row[index] / scale for row in rows])


sql_mean_ms = mean(sql_times_ms)
encode_mean_ms = mean(encode_times_ms)
internal_rescan_mean_ms = mean_column(profile_rows, 0, scale=1000.0)
internal_emit_mean_ms = mean_column(profile_rows, 1, scale=1000.0)
internal_total_mean_ms = mean_column(profile_rows, 2, scale=1000.0)
limited_result_count_mean = mean_column(profile_rows, 3)
limited_heap_tids_mean = mean_column(profile_rows, 4)
hot_amrescan_mean_ms = mean_column(hot_path_rows, 0, scale=1000.0)
query_decode_mean_ms = mean_column(hot_path_rows, 1, scale=1000.0)
prepare_query_mean_ms = mean_column(hot_path_rows, 2, scale=1000.0)
frontier_consume_mean_ms = mean_column(hot_path_rows, 3, scale=1000.0)
graph_materialize_mean_ms = mean_column(hot_path_rows, 4, scale=1000.0)
candidate_score_mean_ms = mean_column(hot_path_rows, 5, scale=1000.0)
slot_fetch_rescan_mean_ms = mean_column(heap_fetch_rows, 0, scale=1000.0)
slot_fetch_emit_mean_ms = mean_column(heap_fetch_rows, 1, scale=1000.0)
executor_like_total_mean_ms = mean_column(heap_fetch_rows, 2, scale=1000.0)
slot_fetch_total_mean_ms = mean_column(heap_fetch_rows, 3, scale=1000.0)
projection_mean_ms = mean_column(heap_fetch_rows, 4, scale=1000.0)
slot_fetch_result_count_mean = mean_column(heap_fetch_rows, 5)
slot_fetch_count_mean = mean_column(heap_fetch_rows, 6)
projected_count_mean = mean_column(heap_fetch_rows, 7)
executor_like_over_internal_ms = executor_like_total_mean_ms - internal_total_mean_ms
residual_sql_over_internal_ms = sql_mean_ms - internal_total_mean_ms
residual_sql_over_executor_like_ms = sql_mean_ms - executor_like_total_mean_ms
residual_after_encode_ms = residual_sql_over_internal_ms - encode_mean_ms

line = (
    f"ef_search={int(ef_str):<4} "
    f"n={expected_n:<5} "
    f"sql_mean={sql_mean_ms:.3f}ms "
    f"encode_mean={encode_mean_ms:.3f}ms "
    f"internal_total_mean={internal_total_mean_ms:.3f}ms "
    f"internal_rescan_mean={internal_rescan_mean_ms:.3f}ms "
    f"internal_emit_mean={internal_emit_mean_ms:.3f}ms "
    f"hot_amrescan_mean={hot_amrescan_mean_ms:.3f}ms "
    f"query_decode_mean={query_decode_mean_ms:.3f}ms "
    f"prepare_query_mean={prepare_query_mean_ms:.3f}ms "
    f"frontier_mean={frontier_consume_mean_ms:.3f}ms "
    f"graph_materialize_mean={graph_materialize_mean_ms:.3f}ms "
    f"candidate_score_mean={candidate_score_mean_ms:.3f}ms "
    f"executor_like_total_mean={executor_like_total_mean_ms:.3f}ms "
    f"slot_fetch_total_mean={slot_fetch_total_mean_ms:.3f}ms "
    f"projection_mean={projection_mean_ms:.3f}ms "
    f"slot_fetch_rescan_mean={slot_fetch_rescan_mean_ms:.3f}ms "
    f"slot_fetch_emit_mean={slot_fetch_emit_mean_ms:.3f}ms "
    f"limited_results_mean={limited_result_count_mean:.2f} "
    f"limited_heap_tids_mean={limited_heap_tids_mean:.2f} "
    f"slot_fetch_results_mean={slot_fetch_result_count_mean:.2f} "
    f"slot_fetch_count_mean={slot_fetch_count_mean:.2f} "
    f"projected_count_mean={projected_count_mean:.2f} "
    f"executor_like_over_internal={executor_like_over_internal_ms:.3f}ms "
    f"residual_sql_over_internal={residual_sql_over_internal_ms:.3f}ms "
    f"residual_sql_over_executor_like={residual_sql_over_executor_like_ms:.3f}ms "
    f"residual_after_encode={residual_after_encode_ms:.3f}ms"
)
print(line)
if output_path:
    with open(output_path, "a", encoding="utf-8") as fh:
        fh.write(line + "\n")
PY
done

rm -f "$queries_tsv" "$sql_cell_file" "$sql_results_file" "$encode_results_file" "$profile_rows_file" "$hot_path_rows_file" "$heap_fetch_rows_file"
trap - EXIT
