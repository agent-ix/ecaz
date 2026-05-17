# Shared latency-bench helper for comparator scripts.
# Source after _common.sh.
#
# Provides:
#   comparator_bench_latency --prefix <p> --op <op> --outdir <d>
#                            --iterations <N> --k <K>

# Runs N latency-timed SELECT-ORDER-BY-LIMIT queries against
# <prefix>_corpus using a query vector pulled from <prefix>_queries.
# Same query pattern that `ecaz bench latency` uses, adapted to
# pgvector's operator types so the same harness works across
# pgvector, pgvectorscale, vchord, and lantern.
comparator_bench_latency() {
  local prefix="" op="" outdir="" iters=200 k=10
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --prefix) prefix="$2"; shift 2 ;;
      --op) op="$2"; shift 2 ;;
      --outdir) outdir="$2"; shift 2 ;;
      --iterations) iters="$2"; shift 2 ;;
      --k) k="$2"; shift 2 ;;
      *) echo "comparator_bench_latency: unknown arg $1" >&2; return 1 ;;
    esac
  done
  [[ -z "$prefix" || -z "$op" || -z "$outdir" ]] && {
    echo "comparator_bench_latency requires --prefix --op --outdir" >&2
    return 1
  }

  mkdir -p "$outdir"
  comparator_log "  bench prefix=$prefix op=$op iters=$iters k=$k"

  local qfile="$outdir/queries.sql"
  local rawout="$outdir/raw.out"

  psql -tAc "select id from ${prefix}_queries order by id limit $iters;" \
    > "$outdir/query_ids.txt"

  {
    echo "\\timing on"
    echo "\\o $rawout"
    while read -r qid; do
      [[ -z "$qid" ]] && continue
      echo "SELECT id FROM ${prefix}_corpus ORDER BY embedding $op (SELECT embedding FROM ${prefix}_queries WHERE id = $qid) LIMIT $k;"
    done < "$outdir/query_ids.txt"
    echo "\\o"
    echo "\\timing off"
  } > "$qfile"

  psql -q -f "$qfile" > "$outdir/run.log" 2>&1 || true

  awk '/^Time:/{print NR, $2}' "$rawout" > "$outdir/raw.tsv"
  local n
  n=$(wc -l < "$outdir/raw.tsv")

  python3 - "$outdir/raw.tsv" "$k" > "$outdir/latency.log" <<'PY'
import sys, statistics
path, k = sys.argv[1], sys.argv[2]
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
    print("# no samples"); sys.exit(0)
ms.sort()
def pct(p):
    return ms[max(0, int(round(p / 100.0 * (len(ms) - 1))))]
print(f"# iterations: {len(ms)}, k: {k}")
print(f"# mean: {statistics.mean(ms):.3f} ms")
print(f"# stddev: {statistics.pstdev(ms):.3f} ms")
print(f"# min: {min(ms):.3f} ms")
print(f"# p50: {pct(50):.3f} ms")
print(f"# p95: {pct(95):.3f} ms")
print(f"# p99: {pct(99):.3f} ms")
print(f"# max: {max(ms):.3f} ms")
PY

  comparator_log "    wrote $outdir/latency.log ($n samples)"
}
