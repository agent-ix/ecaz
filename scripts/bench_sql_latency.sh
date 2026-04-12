#!/usr/bin/env bash
# SQL-level latency benchmarks for tqvector HNSW scan.
# Requires: running PostgreSQL with tqvector extension installed.
#
# Two invocation modes:
#
# 1. Synthetic-fixture mode (legacy default). Generates a synthetic corpus,
#    encodes it, builds an index, then runs a single (m, ef_search) sweep.
#    Tunable via the N/DIM/BITS/M/EF_*/RUNS env vars.
#
#       PGDATABASE=tqvector_bench bash scripts/bench_sql_latency.sh
#
# 2. Real-corpus mode (NFR-001 lane). Reuses already-loaded canonical
#    real-corpus tables and indexes produced by scripts/load_real_corpus.py
#    and sweeps (m, ef_search) over an explicit list. Does NOT load any
#    data — load and bench are decoupled, see docs/RECALL_REAL_CORPUS.md.
#
#       scripts/bench_sql_latency.sh --prefix tqhnsw_real_10k \
#           --m 8 --m 16 --ef-search 40,64,100,128,160,200 \
#           --cache-state cold --output /tmp/nfr1_real_10k.summary
set -euo pipefail

PSQL_BIN="${TQV_PSQL_BIN:-psql}"

print_help() {
  cat <<'USAGE'
Usage:
  Synthetic-fixture mode (legacy default; tunables via env vars):
    bash scripts/bench_sql_latency.sh

  Real-corpus mode (reuses preloaded tables and indexes):
    bash scripts/bench_sql_latency.sh --prefix <prefix> [--m N]... \
        [--ef-search csv] [--query-limit N] [--cache-state LABEL] \
        [--warmup-passes N] [--session-mode MODE] [--timing-mode MODE] \
        [--output FILE]

Options (real-corpus mode):
  --prefix       Canonical real-corpus prefix produced by load_real_corpus.py
                 (e.g. tqhnsw_real_10k, tqhnsw_real_50k). The script reads
                 <prefix>_corpus, <prefix>_queries, and <prefix>_m{N}_idx.
  --m            HNSW m value to bench. May be repeated. Default: 8.
                 Each value must already have been built by load_real_corpus.py
                 as <prefix>_m{N}_idx.
  --ef-search    Comma-separated ef_search list. Default: 40,64,100,128,160,200.
  --query-limit  Cap the number of queries per (m, ef_search) cell. Default:
                 all rows in <prefix>_queries.
  --cache-state  Free-form label recorded in the stdout banner (e.g. cold,
                 warm, warm-after-prime). Default: unspecified.
  --warmup-passes
                 Number of full query-set warmup passes to run before timing
                 each (m, ef_search) cell. Default: 0.
  --session-mode Session reuse mode for each (m, ef_search) cell:
                 per-query (default) opens one psql/backend per timed query.
                 per-cell runs all warmup + timed queries for the cell in a
                 single backend session.
  --timing-mode  How to time each measured query:
                 explain (default) uses per-query EXPLAIN (ANALYZE, FORMAT JSON).
                 plain-server runs the plain ordered query and measures it with
                 server-side clock_timestamp() around a MATERIALIZED subquery.
                 cached-plan (per-cell only) measures a temp server function
                 whose ordered-scan plan is reused across the cell.
  --output       Append the per-cell summary to FILE in addition to stdout.
  -h, --help     Show this message and exit.

Environment:
  PGDATABASE / PGHOST / PGPORT / PGUSER  standard libpq variables.
  TQV_PSQL_BIN                           psql client binary (default: psql).
USAGE
}

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------
PREFIX=""
PREFIX_M_LIST=()
EF_SEARCH_CSV=""
QUERY_LIMIT=""
CACHE_STATE="unspecified"
WARMUP_PASSES="0"
SESSION_MODE="per-query"
TIMING_MODE="explain"
OUTPUT_FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      PREFIX="$2"; shift 2 ;;
    --m)
      PREFIX_M_LIST+=("$2"); shift 2 ;;
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

# ---------------------------------------------------------------------------
# Real-corpus mode
# ---------------------------------------------------------------------------
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

  local plan_text
  plan_text="$("$PSQL_BIN" -X -A -t -q <<SQL
SET tqhnsw.ef_search = ${ef};
EXPLAIN
SELECT id FROM ${corpus_table}
ORDER BY embedding <#> '${query_literal}'::real[]
LIMIT ${k};
SQL
)"

  if ! grep -Fq "${expected_index}" <<<"$plan_text"; then
    echo "planner verification failed for ${expected_index} at ef_search=${ef}" >&2
    echo "expected the measured query to use ${expected_index}, but it did not." >&2
    echo "aborting before timing so this run does not record Seq Scan + Sort" >&2
    echo "or the wrong tqhnsw index for the requested m value." >&2
    echo >&2
    echo "Representative EXPLAIN plan:" >&2
    echo "${plan_text}" >&2
    return 1
  fi

  echo "[verified] planner uses ${expected_index} at ef_search=${ef}" >&2
}

run_real_corpus_bench() {
  local prefix="$1"
  if [[ ! "$prefix" =~ ^[a-zA-Z_][a-zA-Z0-9_]*$ ]]; then
    echo "invalid prefix: $prefix" >&2
    exit 2
  fi
  if [[ ${#PREFIX_M_LIST[@]} -eq 0 ]]; then
    PREFIX_M_LIST=(8)
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
  if [[ "$TIMING_MODE" != "explain" && "$TIMING_MODE" != "plain-server" && "$TIMING_MODE" != "cached-plan" ]]; then
    echo "invalid timing mode: $TIMING_MODE (expected explain, plain-server, or cached-plan)" >&2
    exit 2
  fi
  if [[ "$TIMING_MODE" == "cached-plan" && "$SESSION_MODE" != "per-cell" ]]; then
    echo "timing mode cached-plan requires --session-mode per-cell" >&2
    exit 2
  fi
  IFS=',' read -r -a ef_list <<< "$EF_SEARCH_CSV"
  for ef in "${ef_list[@]}"; do
    if [[ ! "$ef" =~ ^[0-9]+$ ]]; then
      echo "invalid ef_search value: $ef" >&2
      exit 2
    fi
  done
  for m in "${PREFIX_M_LIST[@]}"; do
    if [[ ! "$m" =~ ^[0-9]+$ ]]; then
      echo "invalid m value: $m" >&2
      exit 2
    fi
  done

  local corpus_table="${prefix}_corpus"
  local query_table="${prefix}_queries"
  local k="${K:-10}"

  echo "=== tqvector SQL latency (real corpus) ==="
  echo "Database:     ${PGDATABASE:-(libpq default)}"
  echo "Corpus table: $corpus_table"
  echo "Query table:  $query_table"
  echo "m values:     ${PREFIX_M_LIST[*]}"
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
  query_count=$("$PSQL_BIN" -X -A -t -q -c "SELECT count(*) FROM ${query_table};")
  if [[ -z "$query_count" || "$query_count" == "0" ]]; then
    echo "no queries found in ${query_table}; did the loader run?" >&2
    exit 1
  fi
  if [[ -n "$QUERY_LIMIT" ]]; then
    if (( QUERY_LIMIT < query_count )); then
      query_count="$QUERY_LIMIT"
    fi
  fi

  local queries_tsv
  queries_tsv="$(mktemp -t tqv_queries.XXXXXX.tsv)"
  local cell_sql
  cell_sql="$(mktemp -t tqv_latency_cell.XXXXXX.sql)"
  trap 'rm -f "$queries_tsv" "$results_file" "$cell_sql"' EXIT

  local query_select="SELECT source FROM ${query_table} ORDER BY id"
  if [[ -n "$QUERY_LIMIT" ]]; then
    query_select="${query_select} LIMIT ${QUERY_LIMIT}"
  fi
  "$PSQL_BIN" -X -A -t -q -c "${query_select};" > "$queries_tsv"
  echo "queries available: $query_count"
  if grep -n "'" "$queries_tsv" >/dev/null; then
    echo "unexpected single quote in query literal output from ${query_table}" >&2
    exit 2
  fi
  local probe_query=""
  while IFS= read -r probe_query; do
    [[ -n "$probe_query" ]] && break
  done < "$queries_tsv"
  if [[ -z "$probe_query" ]]; then
    echo "no probe query found in ${query_table}; did the loader run?" >&2
    exit 1
  fi

  local results_file
  results_file="$(mktemp -t tqv_latency_real.XXXXXX.txt)"

  for m in "${PREFIX_M_LIST[@]}"; do
    local index_name="${prefix}_m${m}_idx"
    local exists
    exists=$("$PSQL_BIN" -X -A -t -q -c "SELECT to_regclass('${index_name}') IS NOT NULL;")
    if [[ "$exists" != "t" ]]; then
      echo "index ${index_name} not found; build it with scripts/load_real_corpus.py --m ${m}" >&2
      exit 1
    fi
    if [[ -n "${TQV_REQUIRE_INDEX_NAME:-}" && "${TQV_REQUIRE_INDEX_NAME}" != "${index_name}" ]]; then
      echo "verified bench expected index ${TQV_REQUIRE_INDEX_NAME}, but current cell resolves to ${index_name}" >&2
      echo "invoke the verified launcher separately for each m value." >&2
      exit 2
    fi
    for ef in "${ef_list[@]}"; do
      echo "--- m=${m} ef_search=${ef} ---"
      if [[ -n "${TQV_REQUIRE_INDEX_NAME:-}" ]]; then
        verify_expected_index_plan \
          "${corpus_table}" \
          "${probe_query}" \
          "${k}" \
          "${ef}" \
          "${TQV_REQUIRE_INDEX_NAME}"
      fi
      : > "$results_file"
      local wall_start
      wall_start="$(date +%s.%N)"
      local warmup_pass
      if [[ "$SESSION_MODE" == "per-cell" ]]; then
        : > "$cell_sql"
        if [[ "$TIMING_MODE" == "cached-plan" ]]; then
          cat >> "$cell_sql" <<SQL
SET tqhnsw.ef_search = ${ef};
CREATE OR REPLACE FUNCTION pg_temp.tqv_latency_cached_plan(input_query real[])
RETURNS double precision
LANGUAGE plpgsql
AS \$tqv\$
DECLARE
  started timestamptz;
  finished timestamptz;
BEGIN
  started := clock_timestamp();
  PERFORM id FROM ${corpus_table}
  ORDER BY embedding <#> input_query
  LIMIT ${k};
  finished := clock_timestamp();
  RETURN extract(epoch FROM (finished - started)) * 1000.0;
END
\$tqv\$;
\o /dev/null
SELECT pg_temp.tqv_latency_cached_plan('${probe_query}'::real[]);
\o
SQL
        fi
        if (( WARMUP_PASSES > 0 )); then
          printf '\\o /dev/null\n' >> "$cell_sql"
          for ((warmup_pass = 1; warmup_pass <= WARMUP_PASSES; warmup_pass++)); do
            echo "[warmup] m=${m} ef_search=${ef} pass ${warmup_pass}/${WARMUP_PASSES}" >&2
            while IFS= read -r query_line; do
              [[ -z "$query_line" ]] && continue
              if [[ "$TIMING_MODE" == "cached-plan" ]]; then
                cat >> "$cell_sql" <<SQL
SELECT pg_temp.tqv_latency_cached_plan('${query_line}'::real[]);
SQL
              else
                cat >> "$cell_sql" <<SQL
SET tqhnsw.ef_search = ${ef};
SELECT id FROM ${corpus_table}
ORDER BY embedding <#> '${query_line}'::real[]
LIMIT ${k};
SQL
              fi
            done < "$queries_tsv"
          done
          printf '\\o\n' >> "$cell_sql"
        fi
        while IFS= read -r query_line; do
          [[ -z "$query_line" ]] && continue
          if [[ "$TIMING_MODE" == "explain" ]]; then
            cat >> "$cell_sql" <<SQL
SET tqhnsw.ef_search = ${ef};
EXPLAIN (ANALYZE, TIMING, FORMAT JSON)
SELECT id FROM ${corpus_table}
ORDER BY embedding <#> '${query_line}'::real[]
LIMIT ${k};
SQL
          elif [[ "$TIMING_MODE" == "plain-server" ]]; then
            cat >> "$cell_sql" <<SQL
SET tqhnsw.ef_search = ${ef};
WITH started AS (
  SELECT clock_timestamp() AS t0
),
measured AS MATERIALIZED (
  SELECT id FROM ${corpus_table}
  ORDER BY embedding <#> '${query_line}'::real[]
  LIMIT ${k}
),
finished AS (
  SELECT clock_timestamp() AS t1, count(*) AS rows_seen FROM measured
)
SELECT extract(epoch FROM (finished.t1 - started.t0)) * 1000.0
FROM started, finished;
SQL
          else
            cat >> "$cell_sql" <<SQL
SELECT pg_temp.tqv_latency_cached_plan('${query_line}'::real[]);
SQL
          fi
        done < "$queries_tsv"
        "$PSQL_BIN" -X -A -t -q -f "$cell_sql" > "$results_file"
      else
        if (( WARMUP_PASSES > 0 )); then
          for ((warmup_pass = 1; warmup_pass <= WARMUP_PASSES; warmup_pass++)); do
            echo "[warmup] m=${m} ef_search=${ef} pass ${warmup_pass}/${WARMUP_PASSES}" >&2
            while IFS= read -r query_line; do
              [[ -z "$query_line" ]] && continue
              "$PSQL_BIN" -X -A -t -q > /dev/null <<SQL
SET tqhnsw.ef_search = ${ef};
SELECT id FROM ${corpus_table}
ORDER BY embedding <#> '${query_line}'::real[]
LIMIT ${k};
SQL
            done < "$queries_tsv"
          done
        fi
        while IFS= read -r query_line; do
          [[ -z "$query_line" ]] && continue
          if [[ "$TIMING_MODE" == "explain" ]]; then
            "$PSQL_BIN" -X -A -t -q <<SQL >> "$results_file"
SET tqhnsw.ef_search = ${ef};
EXPLAIN (ANALYZE, TIMING, FORMAT JSON)
SELECT id FROM ${corpus_table}
ORDER BY embedding <#> '${query_line}'::real[]
LIMIT ${k};
SQL
          else
            "$PSQL_BIN" -X -A -t -q <<SQL >> "$results_file"
SET tqhnsw.ef_search = ${ef};
WITH started AS (
  SELECT clock_timestamp() AS t0
),
measured AS MATERIALIZED (
  SELECT id FROM ${corpus_table}
  ORDER BY embedding <#> '${query_line}'::real[]
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

      python3 - "$results_file" "$m" "$ef" "$wall_start" "$wall_end" "$OUTPUT_FILE" "$TIMING_MODE" <<'PY'
import json
import statistics
import sys

results_path, m_str, ef_str, wall_start_str, wall_end_str, output_path, timing_mode = sys.argv[1:]

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
    print(f"  no per-query timings parsed", file=sys.stderr)
    sys.exit(2)

times_ms.sort()
n = len(times_ms)


def pct(p: float) -> float:
    if n == 0:
        return float("nan")
    rank = max(0, min(n - 1, int(round(p * (n - 1)))))
    return times_ms[rank]


wall_seconds = max(1e-9, float(wall_end_str) - float(wall_start_str))
sum_ms = sum(times_ms)
server_qps = (1000.0 * n / sum_ms) if sum_ms > 0 else float("inf")

summary = {
    "m": int(m_str),
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
    f"m={summary['m']:<3} ef_search={summary['ef_search']:<4} "
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
  done

  rm -f "$queries_tsv" "$results_file"
  trap - EXIT
}

if [[ -n "$PREFIX" ]]; then
  run_real_corpus_bench "$PREFIX"
  exit 0
fi

# ---------------------------------------------------------------------------
# Synthetic-fixture mode (unchanged legacy path)
# ---------------------------------------------------------------------------
PGDATABASE="${PGDATABASE:-tqvector_bench}"
N="${N:-50000}"
DIM="${DIM:-1536}"
BITS="${BITS:-4}"
M="${M:-8}"
EF_CONSTRUCTION="${EF_CONSTRUCTION:-128}"
EF_SEARCH="${EF_SEARCH:-40}"
K="${K:-10}"
RUNS="${RUNS:-100}"
SEED="${SEED:-42}"

echo "=== tqvector SQL Latency Benchmark ==="
echo "Database: $PGDATABASE"
echo "Corpus: $N vectors, dim=$DIM, bits=$BITS"
echo "Index: m=$M, ef_construction=$EF_CONSTRUCTION"
echo "Query: top-$K, ef_search=$EF_SEARCH, runs=$RUNS"
echo ""

# Step 1: Setup tables
echo "[1/5] Creating tables and loading data..."
psql "$PGDATABASE" -q <<SQL
DROP TABLE IF EXISTS bench_encoded CASCADE;
DROP TABLE IF EXISTS bench_vectors CASCADE;
CREATE TABLE bench_vectors (id int, embedding real[]);
SQL

python3 scripts/gen_synthetic_data.py --n "$N" --dim "$DIM" --seed "$SEED" \
  | psql "$PGDATABASE" -q -c "COPY bench_vectors (id, embedding) FROM STDIN WITH (FORMAT csv)"

# Step 2: Encode
echo "[2/5] Encoding vectors..."
psql "$PGDATABASE" -q <<SQL
CREATE TABLE bench_encoded AS
SELECT id, encode_to_tqvector(embedding, $BITS, $SEED) AS vec
FROM bench_vectors;
SQL

# Step 3: Build index
echo "[3/5] Building HNSW index..."
psql "$PGDATABASE" -q <<SQL
CREATE INDEX bench_idx ON bench_encoded
USING tqhnsw (vec tqvector_ip_ops)
WITH (m = $M, ef_construction = $EF_CONSTRUCTION);
SQL

# Step 4: Generate queries
echo "[4/5] Running $RUNS queries..."
python3 scripts/gen_synthetic_data.py --n "$RUNS" --dim "$DIM" --seed 999 --format query > /tmp/tq_queries.csv

psql "$PGDATABASE" -q -c "SET tqhnsw.ef_search = $EF_SEARCH;"

RESULTS_FILE="/tmp/tq_latency_results.txt"
> "$RESULTS_FILE"

while IFS= read -r query_line; do
  psql "$PGDATABASE" -t -A <<SQL >> "$RESULTS_FILE"
SET tqhnsw.ef_search = $EF_SEARCH;
EXPLAIN (ANALYZE, TIMING, FORMAT JSON)
SELECT id FROM bench_encoded
ORDER BY vec <#> ARRAY[$query_line]::real[]
LIMIT $K;
SQL
done < /tmp/tq_queries.csv

# Step 5: Report
echo "[5/5] Results:"
python3 -c "
import json, statistics

times = []
with open('$RESULTS_FILE') as f:
    content = f.read()
    # Parse JSON fragments
    depth = 0
    start = None
    for i, c in enumerate(content):
        if c == '[' and depth == 0:
            start = i
        if c == '[': depth += 1
        if c == ']': depth -= 1
        if depth == 0 and start is not None:
            try:
                plan = json.loads(content[start:i+1])
                times.append(plan[0]['Execution Time'])
            except (json.JSONDecodeError, KeyError, IndexError):
                pass
            start = None

if not times:
    print('No results parsed.')
else:
    times.sort()
    n = len(times)
    print(f'Queries: {n}')
    print(f'p50: {times[n//2]:.3f} ms')
    print(f'p99: {times[int(n*0.99)]:.3f} ms')
    print(f'mean: {statistics.mean(times):.3f} ms')
    print(f'stddev: {statistics.stdev(times):.3f} ms')
    print(f'min: {min(times):.3f} ms')
    print(f'max: {max(times):.3f} ms')
"
