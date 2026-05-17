#!/usr/bin/env bash
# Install Lantern from source (HNSW + USearch backend).
# Independent: doesn't strictly require pgvector for its own indexes
# (uses real[] internally), but our load script uses pgvector's
# vector(N) type for cross-comparator schema consistency.
set -euo pipefail

COMPARATOR_NAME="lantern"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/_common.sh"

LANTERN_VERSION="${LANTERN_VERSION:-v0.5.4}"
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

comparator_log "building lantern $LANTERN_VERSION (CMake)"
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

if [[ ! -d lantern || $FORCE -eq 1 ]]; then
  rm -rf lantern
  git clone --depth 1 --branch "$LANTERN_VERSION" --recurse-submodules \
    https://github.com/lanterndata/lantern.git
fi

cd lantern
mkdir -p build && cd build
cmake -DCMAKE_BUILD_TYPE=Release -DPG_CONFIG="$PG_CONFIG" ..
make -j"$(nproc)"
sudo make install

comparator_log "installed. Run: psql -c 'CREATE EXTENSION IF NOT EXISTS lantern;'"
