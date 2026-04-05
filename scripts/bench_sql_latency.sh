#!/usr/bin/env bash
# SQL-level latency benchmarks for tqvector HNSW scan.
# Requires: running PostgreSQL with tqvector extension installed.
# Usage: PGDATABASE=tqvector_bench bash scripts/bench_sql_latency.sh
set -euo pipefail

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
