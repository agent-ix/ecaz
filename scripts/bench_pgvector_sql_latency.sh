#!/usr/bin/env bash
# SQL-level latency benchmarks for pgvector HNSW over a real corpus.
#
# This script mirrors the real-corpus lane of scripts/bench_sql_latency.sh but
# targets pgvector's vector type and hnsw.ef_search GUC instead of tqvector's
# tqhnsw access method.
set -euo pipefail

PSQL_BIN="${TQV_PSQL_BIN:-psql}"

print_help() {
  cat <<'USAGE'
Usage:
  bash scripts/bench_pgvector_sql_latency.sh \
      --corpus-table <table> \
      --query-table <table> \
      --index-name <index> \
      [--dim N] [--ef-search csv] [--query-limit N] \
      [--cache-state LABEL] [--warmup-passes N] \
      [--session-mode MODE] [--timing-mode MODE] [--output FILE]

Options:
  --corpus-table Explicit corpus table/view to scan. Must expose:
                 - id bigint/int
                 - embedding vector(dim)
  --query-table  Query table to read. Must expose:
                 - source real[]
  --index-name   Exact pgvector HNSW index name expected for every measured
                 cell. The script aborts before timing if the planner picks a
                 different plan.
  --dim          Vector dimension used for query casts. Default: 1536.
  --ef-search    Comma-separated ef_search list. Default:
                 40,64,100,128,160,200.
  --query-limit  Cap the number of queries per ef_search cell. Default:
                 all rows in --query-table.
  --cache-state  Free-form label recorded in the stdout banner. Default:
                 unspecified.
  --warmup-passes
                 Number of full query-set warmup passes before timing each
                 ef_search cell. Default: 0.
  --session-mode Session reuse mode for each ef_search cell:
                 per-query (default) opens one psql/backend per timed query.
                 per-cell runs all warmup + timed queries for the cell in a
                 single backend session.
  --timing-mode  How to time each measured query:
                 explain (default) uses per-query EXPLAIN (ANALYZE, FORMAT
                 JSON).
                 plain-server runs the plain ordered query and measures it
                 with server-side clock_timestamp() around a MATERIALIZED
                 subquery.
  --output       Append the per-cell summary to FILE in addition to stdout.
  -h, --help     Show this message and exit.

Environment:
  PGDATABASE / PGHOST / PGPORT / PGUSER  standard libpq variables.
  TQV_PSQL_BIN                           psql client binary (default: psql).
USAGE
}

CORPUS_TABLE=""
QUERY_TABLE=""
INDEX_NAME=""
DIM="1536"
EF_SEARCH_CSV=""
QUERY_LIMIT=""
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
    --dim)
      DIM="$2"; shift 2 ;;
    --ef-search)
      EF_SEARCH_CSV="$2"; shift 2 ;;
    --query-limit)
      QUERY_LIMIT="$2"; shift 2 ;;
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
  local k="$3"
  local ef="$4"
  local expected_index="$5"
  local dim="$6"

  local plan_text
  plan_text="$("$PSQL_BIN" -X -A -t -q <<SQL
SET hnsw.ef_search = ${ef};
EXPLAIN
SELECT id FROM ${corpus_table}
ORDER BY embedding <#> '${query_literal}'::real[]::vector(${dim})
LIMIT ${k};
SQL
)"

  if ! grep -Fq "${expected_index}" <<<"$plan_text"; then
    echo "planner verification failed for ${expected_index} at ef_search=${ef}" >&2
    echo "expected the measured query to use ${expected_index}, but it did not." >&2
    echo "aborting before timing so this run does not record Seq Scan + Sort" >&2
    echo "or the wrong pgvector index for the requested corpus." >&2
    echo >&2
    echo "Representative EXPLAIN plan:" >&2
    echo "${plan_text}" >&2
    return 1
  fi

  echo "[verified] planner uses ${expected_index} at ef_search=${ef}" >&2
}

run_real_corpus_bench() {
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
  if [[ ! "$DIM" =~ ^[0-9]+$ ]]; then
    echo "invalid dimension: $DIM" >&2
    exit 2
  fi
  if [[ -z "$EF_SEARCH_CSV" ]]; then
    EF_SEARCH_CSV="40,64,100,128,160,200"
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

  local exists
  exists=$("$PSQL_BIN" -X -A -t -q -c "SELECT to_regclass('${INDEX_NAME}') IS NOT NULL;")
  if [[ "$exists" != "t" ]]; then
    echo "index ${INDEX_NAME} not found" >&2
    exit 1
  fi

  local k="${K:-10}"
  echo "=== pgvector SQL latency (real corpus) ==="
  echo "Database:     ${PGDATABASE:-(libpq default)}"
  echo "Corpus table: $CORPUS_TABLE"
  echo "Query table:  $QUERY_TABLE"
  echo "index name:   $INDEX_NAME"
  echo "dimension:    $DIM"
  echo "ef_search:    $EF_SEARCH_CSV"
  echo "top-k:        $k"
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

  local query_count
  query_count=$("$PSQL_BIN" -X -A -t -q -c "SELECT count(*) FROM ${QUERY_TABLE};")
  if [[ -z "$query_count" || "$query_count" == "0" ]]; then
    echo "no queries found in ${QUERY_TABLE}" >&2
    exit 1
  fi
  if [[ -n "$QUERY_LIMIT" ]]; then
    if (( QUERY_LIMIT < query_count )); then
      query_count="$QUERY_LIMIT"
    fi
  fi

  local queries_tsv
  queries_tsv="$(mktemp -t tqv_pgvector_queries.XXXXXX.tsv)"
  local cell_sql
  cell_sql="$(mktemp -t tqv_pgvector_latency_cell.XXXXXX.sql)"
  local results_file
  results_file="$(mktemp -t tqv_pgvector_latency_real.XXXXXX.txt)"
  trap 'rm -f "$queries_tsv" "$cell_sql" "$results_file"' EXIT

  local query_select="SELECT source FROM ${QUERY_TABLE} ORDER BY id"
  if [[ -n "$QUERY_LIMIT" ]]; then
    query_select="${query_select} LIMIT ${QUERY_LIMIT}"
  fi
  "$PSQL_BIN" -X -A -t -q -c "${query_select};" > "$queries_tsv"
  echo "queries available: $query_count"
  if grep -n "'" "$queries_tsv" >/dev/null; then
    echo "unexpected single quote in query literal output from ${QUERY_TABLE}" >&2
    exit 2
  fi

  local probe_query=""
  while IFS= read -r probe_query; do
    [[ -n "$probe_query" ]] && break
  done < "$queries_tsv"
  if [[ -z "$probe_query" ]]; then
    echo "no probe query found in ${QUERY_TABLE}" >&2
    exit 1
  fi

  for ef in "${ef_list[@]}"; do
    echo "--- ef_search=${ef} ---"
    verify_expected_index_plan "$CORPUS_TABLE" "$probe_query" "$k" "$ef" "$INDEX_NAME" "$DIM"
    : > "$results_file"

    local wall_start
    wall_start="$(date +%s.%N)"
    local warmup_pass

    if [[ "$SESSION_MODE" == "per-cell" ]]; then
      : > "$cell_sql"
      if (( WARMUP_PASSES > 0 )); then
        printf '\\o /dev/null\n' >> "$cell_sql"
        for ((warmup_pass = 1; warmup_pass <= WARMUP_PASSES; warmup_pass++)); do
          echo "[warmup] ef_search=${ef} pass ${warmup_pass}/${WARMUP_PASSES}" >&2
          while IFS= read -r query_line; do
            [[ -z "$query_line" ]] && continue
            cat >> "$cell_sql" <<SQL
SET hnsw.ef_search = ${ef};
SELECT id FROM ${CORPUS_TABLE}
ORDER BY embedding <#> '${query_line}'::real[]::vector(${DIM})
LIMIT ${k};
SQL
          done < "$queries_tsv"
        done
        printf '\\o\n' >> "$cell_sql"
      fi

      while IFS= read -r query_line; do
        [[ -z "$query_line" ]] && continue
        if [[ "$TIMING_MODE" == "explain" ]]; then
          cat >> "$cell_sql" <<SQL
SET hnsw.ef_search = ${ef};
EXPLAIN (ANALYZE, TIMING, FORMAT JSON)
SELECT id FROM ${CORPUS_TABLE}
ORDER BY embedding <#> '${query_line}'::real[]::vector(${DIM})
LIMIT ${k};
SQL
        else
          cat >> "$cell_sql" <<SQL
SET hnsw.ef_search = ${ef};
WITH started AS (
  SELECT clock_timestamp() AS t0
),
measured AS MATERIALIZED (
  SELECT id FROM ${CORPUS_TABLE}
  ORDER BY embedding <#> '${query_line}'::real[]::vector(${DIM})
  LIMIT ${k}
),
finished AS (
  SELECT clock_timestamp() AS t1, count(*) AS rows_seen FROM measured
)
SELECT extract(epoch FROM (finished.t1 - started.t0)) * 1000.0
FROM started, finished;
SQL
        fi
      done < "$queries_tsv"
      "$PSQL_BIN" -X -A -t -q -f "$cell_sql" > "$results_file"
    else
      if (( WARMUP_PASSES > 0 )); then
        for ((warmup_pass = 1; warmup_pass <= WARMUP_PASSES; warmup_pass++)); do
          echo "[warmup] ef_search=${ef} pass ${warmup_pass}/${WARMUP_PASSES}" >&2
          while IFS= read -r query_line; do
            [[ -z "$query_line" ]] && continue
            "$PSQL_BIN" -X -A -t -q <<SQL > /dev/null
SET hnsw.ef_search = ${ef};
SELECT id FROM ${CORPUS_TABLE}
ORDER BY embedding <#> '${query_line}'::real[]::vector(${DIM})
LIMIT ${k};
SQL
          done < "$queries_tsv"
        done
      fi

      while IFS= read -r query_line; do
        [[ -z "$query_line" ]] && continue
        if [[ "$TIMING_MODE" == "explain" ]]; then
          "$PSQL_BIN" -X -A -t -q <<SQL >> "$results_file"
SET hnsw.ef_search = ${ef};
EXPLAIN (ANALYZE, TIMING, FORMAT JSON)
SELECT id FROM ${CORPUS_TABLE}
ORDER BY embedding <#> '${query_line}'::real[]::vector(${DIM})
LIMIT ${k};
SQL
        else
          "$PSQL_BIN" -X -A -t -q <<SQL >> "$results_file"
SET hnsw.ef_search = ${ef};
WITH started AS (
  SELECT clock_timestamp() AS t0
),
measured AS MATERIALIZED (
  SELECT id FROM ${CORPUS_TABLE}
  ORDER BY embedding <#> '${query_line}'::real[]::vector(${DIM})
  LIMIT ${k}
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

    local wall_end
    wall_end="$(date +%s.%N)"

    python3 - "$results_file" "$ef" "$wall_start" "$wall_end" "$OUTPUT_FILE" "$TIMING_MODE" <<'PY'
import json
import statistics
import sys

results_path, ef_str, wall_start_str, wall_end_str, output_path, timing_mode = sys.argv[1:]

times_ms = []
if timing_mode == "explain":
    with open(results_path, "r", encoding="utf-8") as fh:
        content = fh.read()
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
else:
    with open(results_path, "r", encoding="utf-8") as fh:
        for raw_line in fh:
            line = raw_line.strip()
            if not line:
                continue
            try:
                times_ms.append(float(line))
            except ValueError:
                pass

if not times_ms:
    print("  no per-query timings parsed", file=sys.stderr)
    sys.exit(2)

if timing_mode != "explain":
    negative_times = [value for value in times_ms if value < 0]
    if negative_times:
        print(
            "  invalid negative per-query timings parsed for "
            f"{timing_mode}: count={len(negative_times)} "
            f"min={min(negative_times):.3f}ms; rerun this cell",
            file=sys.stderr,
        )
        sys.exit(2)

times_ms.sort()
n = len(times_ms)


def pct(p: float) -> float:
    rank = max(0, min(n - 1, int(round(p * (n - 1)))))
    return times_ms[rank]


wall_seconds = max(1e-9, float(wall_end_str) - float(wall_start_str))
sum_ms = sum(times_ms)
server_qps = (1000.0 * n / sum_ms) if sum_ms > 0 else float("inf")

summary = {
    "ef_search": int(ef_str),
    "queries": n,
    "p50_ms": pct(0.50),
    "p95_ms": pct(0.95),
    "p99_ms": pct(0.99),
    "mean_ms": statistics.fmean(times_ms),
    "min_ms": times_ms[0],
    "max_ms": times_ms[-1],
    "wall_seconds": wall_seconds,
    "server_qps": server_qps,
}

line = (
    f"ef_search={summary['ef_search']:<4} "
    f"n={summary['queries']:<5} "
    f"p50={summary['p50_ms']:.3f}ms "
    f"p95={summary['p95_ms']:.3f}ms "
    f"p99={summary['p99_ms']:.3f}ms "
    f"mean={summary['mean_ms']:.3f}ms "
    f"min={summary['min_ms']:.3f}ms "
    f"max={summary['max_ms']:.3f}ms "
    f"server_qps={summary['server_qps']:.2f} "
    f"wall={summary['wall_seconds']:.2f}s"
)
print(line)
if output_path:
    with open(output_path, "a", encoding="utf-8") as fh:
        fh.write(line + "\n")
PY
  done

  rm -f "$queries_tsv" "$cell_sql" "$results_file"
  trap - EXIT
}

run_real_corpus_bench
