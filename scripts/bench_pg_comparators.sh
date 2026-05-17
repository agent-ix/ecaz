#!/usr/bin/env bash
# Latency benchmark for third-party Postgres vector extensions
# (pgvector / pgvectorscale / VectorChord / Lantern), measuring the
# same SELECT-ORDER-BY-LIMIT pattern that `ecaz bench latency` runs
# against ecvector tables -- just adapted to pgvector's column type
# and operators.
#
# Output mirrors the ecaz bench latency output: per-(ext × index)
# table with mean, p50, p95, p99 across N iterations. Writes one
# latency.log per (size × ext × index) and a JSON manifest.
#
# Companion to scripts/run_full_sweep.sh for the ecaz AMs.

set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/bench_pg_comparators.sh --out <dir> --size <label>
    [--db <database>] [--exts "pgv pgvscale vchord lantern"]
    [--iterations N] [--k K]

Defaults:
  iterations = 200, k = 10
  PGHOST=/tmp PGUSER=postgres PGDATABASE=tqvector_bench
  exts = "pgv pgvscale vchord lantern"

Per (ext × index) outputs:
  <out>/<size>/<ext>/<idx>/latency.log    # comfy-table-style result table
  <out>/<size>/<ext>/<idx>/raw.tsv        # iteration_idx<TAB>ms_per_query
  <out>/manifest.json
EOF
}

OUT=""
SIZE=""
DB="${PGDATABASE:-tqvector_bench}"
EXTS="pgv pgvscale vchord lantern"
ITERATIONS=200
K=10

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --size) SIZE="$2"; shift 2 ;;
    --db) DB="$2"; shift 2 ;;
    --exts) EXTS="$2"; shift 2 ;;
    --iterations) ITERATIONS="$2"; shift 2 ;;
    --k) K="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

[[ -z "$OUT" || -z "$SIZE" ]] && { usage; exit 1; }

export PGDATABASE="$DB"
: "${PGHOST:=/tmp}"
: "${PGUSER:=postgres}"
export PGHOST PGUSER

mkdir -p "$OUT"
log() { echo "[bench-comparators] $(date '+%H:%M:%S') $*"; }
MANIFEST="$OUT/manifest.json"

# Run iterations of: SELECT id FROM corpus ORDER BY embedding <op> q LIMIT k
# (pgvector operators: <-> L2, <#> negative inner product, <=> cosine)
# Default to <#> for inner product (matches ecaz's ec_ivf default).
bench_one() {
  local prefix="$1" idx_name="$2" op="$3" outdir="$4"
  mkdir -p "$outdir"
  log "  bench prefix=$prefix idx=$idx_name op=$op"

  # Pull N queries into a file once; iterate by id over them.
  local qfile="$outdir/queries.sql"
  psql -tAc "select id from ${prefix}_queries order by id limit $ITERATIONS;" > "$outdir/query_ids.txt"

  # Generate a SQL script that emits per-query timing
  cat > "$qfile" <<SQL
\\timing on
\\o $outdir/raw.out
SQL
  while read -r qid; do
    [[ -z "$qid" ]] && continue
    cat >> "$qfile" <<SQL
SELECT id FROM ${prefix}_corpus
ORDER BY embedding $op (SELECT embedding FROM ${prefix}_queries WHERE id = $qid)
LIMIT $K;
SQL
  done < "$outdir/query_ids.txt"
  echo "\\o" >> "$qfile"
  echo "\\timing off" >> "$qfile"

  psql -f "$qfile" > "$outdir/run.log" 2>&1 || true

  # Parse timing lines from raw.out -> raw.tsv
  awk '/^Time:/{print NR, $2}' "$outdir/raw.out" > "$outdir/raw.tsv" || true
  local n=$(wc -l < "$outdir/raw.tsv")

  # Compute percentiles from raw.tsv
  python3 - "$outdir/raw.tsv" > "$outdir/latency.log" <<PY
import sys, statistics
path = sys.argv[1]
ms = []
with open(path) as f:
    for line in f:
        parts = line.split()
        if len(parts) >= 2:
            try:
                ms.append(float(parts[1]))
            except ValueError:
                pass
if not ms:
    print("# no samples")
    sys.exit(0)
ms.sort()
def pct(p):
    k = max(0, int(round(p / 100.0 * (len(ms) - 1))))
    return ms[k]
print(f"# iterations: {len(ms)}, k: ${K}")
print(f"# mean: {statistics.mean(ms):.3f} ms")
print(f"# stddev: {statistics.pstdev(ms):.3f} ms")
print(f"# min: {min(ms):.3f} ms")
print(f"# p50: {pct(50):.3f} ms")
print(f"# p95: {pct(95):.3f} ms")
print(f"# p99: {pct(99):.3f} ms")
print(f"# max: {max(ms):.3f} ms")
PY

  log "    wrote $outdir/latency.log ($n samples)"
  printf '  {"size":"%s","ext":"%s","idx":"%s","op":"%s","artifact":"%s","ts":"%s"}\n' \
    "$SIZE" "${prefix##real_${SIZE}_}" "$idx_name" "$op" "$outdir/latency.log" "$(date -u -Iseconds)" \
    >> "$MANIFEST.tmp"
}

: > "$MANIFEST.tmp"
{
  echo '{'
  printf '  "suite": "bench-comparators",\n'
  printf '  "started_utc": "%s",\n' "$(date -u -Iseconds)"
  printf '  "host": "%s",\n' "$(uname -n)"
  printf '  "db": "%s",\n' "$DB"
  printf '  "size": "%s",\n' "$SIZE"
  printf '  "exts": "%s",\n' "$EXTS"
  printf '  "iterations": %d,\n' "$ITERATIONS"
  printf '  "k": %d,\n' "$K"
  printf '  "steps": [\n'
} > "$MANIFEST"

for ext in $EXTS; do
  prefix="real_${SIZE}_${ext}"
  # Skip if corpus table missing
  if ! psql -tAc "select 1 from pg_tables where tablename='${prefix}_corpus';" | grep -q 1; then
    log "$prefix corpus missing; skipping"
    continue
  fi
  case "$ext" in
    pgv)
      # HNSW + IVFFlat. Swap-on-pass dance: drop the other.
      for idx_kind in hnsw ivfflat; do
        idx="${prefix}_${idx_kind}_idx"
        other_kind=$([[ "$idx_kind" == "hnsw" ]] && echo ivfflat || echo hnsw)
        other_idx="${prefix}_${other_kind}_idx"
        if psql -tAc "select 1 from pg_indexes where indexname='$other_idx';" | grep -q 1; then
          psql -c "DROP INDEX $other_idx;" >> "$OUT/index-events.log" 2>&1
        fi
        if ! psql -tAc "select 1 from pg_indexes where indexname='$idx';" | grep -q 1; then
          log "rebuilding $idx from cached config"
          case "$idx_kind" in
            hnsw) psql -c "CREATE INDEX $idx ON ${prefix}_corpus USING hnsw (embedding vector_ip_ops) WITH (m = 16, ef_construction = 64);" ;;
            ivfflat) psql -c "CREATE INDEX $idx ON ${prefix}_corpus USING ivfflat (embedding vector_ip_ops) WITH (lists = 1024);" ;;
          esac
        fi
        bench_one "$prefix" "$idx_kind" "<#>" "$OUT/${SIZE}/pgv/$idx_kind"
      done
      ;;
    pgvscale)
      bench_one "$prefix" "diskann" "<=>" "$OUT/${SIZE}/pgvscale/diskann"
      ;;
    vchord)
      bench_one "$prefix" "rabitq" "<#>" "$OUT/${SIZE}/vchord/rabitq"
      ;;
    lantern)
      bench_one "$prefix" "hnsw" "<=>" "$OUT/${SIZE}/lantern/hnsw"
      ;;
  esac
done

{
  sed '$!s/$/,/' "$MANIFEST.tmp"
  echo '  ],'
  printf '  "finished_utc": "%s"\n' "$(date -u -Iseconds)"
  echo '}'
} >> "$MANIFEST"
rm -f "$MANIFEST.tmp"

log "comparator bench done; manifest at $MANIFEST"
