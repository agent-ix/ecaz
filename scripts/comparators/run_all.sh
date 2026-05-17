#!/usr/bin/env bash
# Thin orchestrator: runs install + load + bench for the named
# comparators in sequence. Use the per-comparator scripts directly if
# you want to add or rerun just one. This is provided for "set it up
# from scratch and bench everything" convenience only.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

usage() {
  cat <<EOF
Usage:
  scripts/comparators/run_all.sh --out <dir> --size <S> --dim <N>
    --corpus-file <tsv> --queries-file <tsv>
    [--db <database>] [--exts "pgvector pgvectorscale vchord lantern"]
    [--phases "install load bench"]

  --exts selects which comparators run (each is its own per-ext script).
  --phases selects which phases run per comparator. Skip "install"
  if extensions are already installed.

Comparators in order:
  pgvector       -> install_pgvector.sh, load_pgvector.sh, bench_pgvector.sh
  pgvectorscale  -> install_pgvectorscale.sh, load_pgvectorscale.sh, bench_pgvectorscale.sh
  vchord         -> install_vchord.sh,        load_vchord.sh,        bench_vchord.sh
  lantern        -> install_lantern.sh,       load_lantern.sh,       bench_lantern.sh
EOF
}

OUT="" SIZE="" DIM="" CORPUS="" QUERIES=""
DB="${PGDATABASE:-tqvector_bench}"
EXTS="pgvector pgvectorscale vchord lantern"
PHASES="install load bench"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --size) SIZE="$2"; shift 2 ;;
    --dim) DIM="$2"; shift 2 ;;
    --corpus-file) CORPUS="$2"; shift 2 ;;
    --queries-file) QUERIES="$2"; shift 2 ;;
    --db) DB="$2"; shift 2 ;;
    --exts) EXTS="$2"; shift 2 ;;
    --phases) PHASES="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

for phase in install load bench; do
  if ! [[ " $PHASES " == *" $phase "* ]]; then continue; fi
  for ext in $EXTS; do
    case "$phase:$ext" in
      install:pgvector|install:pgvectorscale|install:vchord|install:lantern)
        "$SCRIPT_DIR/install_${ext}.sh"
        ;;
      load:pgvector)
        "$SCRIPT_DIR/load_pgvector.sh" --size "$SIZE" --dim "$DIM" \
          --corpus-file "$CORPUS" --queries-file "$QUERIES" --db "$DB"
        ;;
      load:pgvectorscale)
        "$SCRIPT_DIR/load_pgvectorscale.sh" --size "$SIZE" --dim "$DIM" \
          --corpus-file "$CORPUS" --queries-file "$QUERIES" --db "$DB"
        ;;
      load:vchord)
        "$SCRIPT_DIR/load_vchord.sh" --size "$SIZE" --dim "$DIM" \
          --corpus-file "$CORPUS" --queries-file "$QUERIES" --db "$DB"
        ;;
      load:lantern)
        "$SCRIPT_DIR/load_lantern.sh" --size "$SIZE" --dim "$DIM" \
          --corpus-file "$CORPUS" --queries-file "$QUERIES" --db "$DB"
        ;;
      bench:*)
        # bench script names are bench_<ext>.sh
        "$SCRIPT_DIR/bench_${ext}.sh" --out "$OUT" --size "$SIZE" --db "$DB"
        ;;
    esac
  done
done
