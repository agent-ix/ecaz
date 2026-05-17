#!/usr/bin/env bash
# Install pgvector from source against the local pg_config.
# Standalone: depends only on _common.sh. Run independently to add
# pgvector without touching other comparators.
set -euo pipefail

COMPARATOR_NAME="pgvector"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/_common.sh"

PGVECTOR_VERSION="${PGVECTOR_VERSION:-master}"
BUILD_DIR="${BUILD_DIR:-$COMPARATORS_BUILD_DIR_DEFAULT}"
PG_CONFIG="${PG_CONFIG:-$PG_CONFIG_DEFAULT}"
FORCE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --build-dir) BUILD_DIR="$2"; shift 2 ;;
    --pg-config) PG_CONFIG="$2"; shift 2 ;;
    --version) PGVECTOR_VERSION="$2"; shift 2 ;;
    --force) FORCE=1; shift ;;
    -h|--help) sed -n '2,$ s/^# \?//p' "$0" | head -20; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

if [[ $FORCE -eq 0 ]] && comparator_extension_installed vector; then
  comparator_log "already installed; pass --force to rebuild"
  exit 0
fi

comparator_log "building $PGVECTOR_VERSION into $($PG_CONFIG --pkglibdir)"
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

if [[ ! -d pgvector || $FORCE -eq 1 ]]; then
  rm -rf pgvector
  git clone --depth 1 --branch "$PGVECTOR_VERSION" \
    https://github.com/pgvector/pgvector.git
fi

cd pgvector
make clean >/dev/null 2>&1 || true
PG_CONFIG="$PG_CONFIG" make -j"$(nproc)"
sudo PG_CONFIG="$PG_CONFIG" make install

comparator_log "installed. Run:  psql -c 'CREATE EXTENSION IF NOT EXISTS vector;'"
