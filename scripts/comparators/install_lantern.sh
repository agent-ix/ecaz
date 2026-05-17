#!/usr/bin/env bash
# Install Lantern from source (HNSW + USearch backend).
# Independent: doesn't strictly require pgvector for its own indexes
# (uses real[] internally), but our load script uses pgvector's
# vector(N) type for cross-comparator schema consistency.
set -euo pipefail

COMPARATOR_NAME="lantern"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/_common.sh"

LANTERN_VERSION="${LANTERN_VERSION:-v0.5.0}"
BUILD_DIR="${BUILD_DIR:-$COMPARATORS_BUILD_DIR_DEFAULT}"
PG_CONFIG="${PG_CONFIG:-$PG_CONFIG_DEFAULT}"
FORCE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --build-dir) BUILD_DIR="$2"; shift 2 ;;
    --pg-config) PG_CONFIG="$2"; shift 2 ;;
    --version) LANTERN_VERSION="$2"; shift 2 ;;
    --force) FORCE=1; shift ;;
    -h|--help) sed -n '2,$ s/^# \?//p' "$0" | head -20; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

if [[ $FORCE -eq 0 ]] && comparator_extension_installed lantern; then
  comparator_log "already installed; pass --force to rebuild"
  exit 0
fi

# PG18 compatibility check. PG18 broke vacuum_delay_point() (now
# requires a `bool is_analyze` arg). As of lantern v0.5.0 + master
# (checked 2026-05-17) lantern still calls vacuum_delay_point() with
# no args -- no PG18 port available upstream. Skip with a clear gap
# message unless the operator is targeting an older PG.
PG_MAJOR_NUM="$($PG_CONFIG --version 2>/dev/null | awk '{print $2}' | cut -d. -f1)"
if [[ "$PG_MAJOR_NUM" -ge 18 ]]; then
  comparator_log "GAP: lantern $LANTERN_VERSION does not support PostgreSQL $PG_MAJOR_NUM yet."
  comparator_log "     Upstream calls vacuum_delay_point() with no args; PG18 requires (bool)."
  comparator_log "     No PG18-compatible release as of 2026-05-17."
  comparator_log "     Workarounds: patch src/hnsw/{delete,vacuum,...}.c locally, OR bench"
  comparator_log "     on a separate PG17 instance, OR wait for upstream PG18 support."
  exit 2
fi

comparator_log "building lantern $LANTERN_VERSION (CMake)"
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

if [[ ! -d lantern || $FORCE -eq 1 ]]; then
  rm -rf lantern
  git clone --depth 1 --branch "$LANTERN_VERSION" --recurse-submodules \
    https://github.com/lanterndata/lantern.git
fi

# Lantern v0.5.x layout: CMakeLists.txt for the PG extension lives at
# lantern/lantern_hnsw/CMakeLists.txt, not the repo root (which holds
# the Rust extras crates). Build from the lantern_hnsw subdirectory.
cd lantern/lantern_hnsw
mkdir -p build && cd build
cmake -DCMAKE_BUILD_TYPE=Release -DPG_CONFIG="$PG_CONFIG" ..
make -j"$(nproc)"
sudo make install

comparator_log "installed. Run: psql -c 'CREATE EXTENSION IF NOT EXISTS lantern;'"
