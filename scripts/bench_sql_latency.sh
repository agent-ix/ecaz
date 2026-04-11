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
#           --output /tmp/nfr1_real_10k.txt
set -euo pipefail

PSQL_BIN="${TQV_PSQL_BIN:-psql}"

print_help() {
  cat <<'USAGE'
Usage:
  Synthetic-fixture mode (legacy default; tunables via env vars):
    bash scripts/bench_sql_latency.sh

  Real-corpus mode (reuses preloaded tables and indexes):
    bash scripts/bench_sql_latency.sh --prefix <prefix> [--m N]... \
        [--ef-search csv] [--query-limit N] [--output FILE]

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
print_real_corpus_env_banner() {
  local shared_buffers
  local work_mem
  local max_parallel_workers
  local host_uname
  local host_cpu
  local host_ram

  shared_buffers=$("$PSQL_BIN" -X -A -t -q -c "SHOW shared_buffers;")
  work_mem=$("$PSQL_BIN" -X -A -t -q -c "SHOW work_mem;")
  max_parallel_workers=$("$PSQL_BIN" -X -A -t -q -c "SHOW max_parallel_workers_per_gather;")
  host_uname="$(uname -a)"
  host_cpu="$(grep -m1 'model name' /proc/cpuinfo 2>/dev/null | cut -d: -f2- | sed 's/^ *//')"
  host_ram="$(grep -m1 '^MemTotal:' /proc/meminfo 2>/dev/null | awk '{print $2 " " $3}')"

  echo "shared_buffers: ${shared_buffers:-unknown}"
  echo "work_mem: ${work_mem:-unknown}"
  echo "max_parallel_workers_per_gather: ${max_parallel_workers:-unknown}"
  echo "host_uname: ${host_uname:-unknown}"
  if [[ -n "$host_cpu" ]]; then
    echo "host_cpu: ${host_cpu}"
  fi
  if [[ -n "$host_ram" ]]; then
    echo "host_ram: ${host_ram}"
  fi
  echo "cache_state: operator-supplied; script does not warm cache"
  echo
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
  echo
  print_real_corpus_env_banner

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
  echo "queries available: $query_count"

  local queries_tsv
  queries_tsv="$(mktemp -t tqv_queries.XXXXXX.tsv)"
  local results_file=""
  trap 'rm -f "$queries_tsv" "$results_file"' EXIT

  local query_select="SELECT source FROM ${query_table} ORDER BY id"
  if [[ -n "$QUERY_LIMIT" ]]; then
    query_select="${query_select} LIMIT ${QUERY_LIMIT}"
  fi
  "$PSQL_BIN" -X -A -t -q -c "${query_select};" > "$queries_tsv"
  if LC_ALL=C grep -q "'" "$queries_tsv"; then
    echo "query literals from ${query_table} contain a single quote; refusing to inline unsafe SQL" >&2
    exit 1
  fi

  results_file="$(mktemp -t tqv_latency_real.XXXXXX.txt)"

  for m in "${PREFIX_M_LIST[@]}"; do
    local index_name="${prefix}_m${m}_idx"
    local exists
    exists=$("$PSQL_BIN" -X -A -t -q -c "SELECT to_regclass('${index_name}') IS NOT NULL;")
    if [[ "$exists" != "t" ]]; then
      echo "index ${index_name} not found; build it with scripts/load_real_corpus.py --m ${m}" >&2
      exit 1
    fi
    for ef in "${ef_list[@]}"; do
      echo "--- m=${m} ef_search=${ef} ---"
      : > "$results_file"
      local wall_start
      wall_start="$(date +%s.%N)"
      while IFS= read -r query_line; do
        [[ -z "$query_line" ]] && continue
        "$PSQL_BIN" -X -A -t -q <<SQL >> "$results_file"
SET tqhnsw.ef_search = ${ef};
EXPLAIN (ANALYZE, TIMING, FORMAT JSON)
SELECT id FROM ${corpus_table}
ORDER BY embedding <#> '${query_line}'::real[]
LIMIT ${k};
SQL
      done < "$queries_tsv"
      local wall_end
      wall_end="$(date +%s.%N)"

      python3 - "$results_file" "$m" "$ef" "$wall_start" "$wall_end" "$OUTPUT_FILE" <<'PY'
import json
import statistics
import sys

results_path, m_str, ef_str, wall_start_str, wall_end_str, output_path = sys.argv[1:]

times_ms = []
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
server_seconds = max(1e-9, sum(times_ms) / 1000.0)
server_qps = n / server_seconds

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
    f"wall={summary['wall_seconds']:.3f}s "
    f"server_qps={summary['server_qps']:.2f}"
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
