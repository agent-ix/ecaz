#!/usr/bin/env bash
# Install pgvectorscale (Timescale's StreamingDiskANN + SBQ) from source.
# Requires pgvector to be installed already.
set -euo pipefail

COMPARATOR_NAME="pgvectorscale"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/_common.sh"

PGVECTORSCALE_VERSION="${PGVECTORSCALE_VERSION:-0.9.0}"
BUILD_DIR="${BUILD_DIR:-$COMPARATORS_BUILD_DIR_DEFAULT}"
PG_CONFIG="${PG_CONFIG:-$PG_CONFIG_DEFAULT}"
FORCE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --build-dir) BUILD_DIR="$2"; shift 2 ;;
    --pg-config) PG_CONFIG="$2"; shift 2 ;;
    --version) PGVECTORSCALE_VERSION="$2"; shift 2 ;;
    --force) FORCE=1; shift ;;
    -h|--help) sed -n '2,$ s/^# \?//p' "$0" | head -20; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

if ! comparator_extension_installed vector; then
  comparator_log "pgvector must be installed first; run install_pgvector.sh"
  exit 1
fi

if [[ $FORCE -eq 0 ]] && comparator_extension_installed vectorscale; then
  comparator_log "already installed; pass --force to rebuild"
  exit 0
fi

# pgvectorscale 0.9.0 (latest as of 2026-05-17) pins pgrx 0.16.1
# while ecaz's host-wide cargo-pgrx is 0.17. cargo-pgrx requires
# exact version match. Until pgvectorscale ships a pgrx-0.17 release,
# building it requires installing a parallel cargo-pgrx 0.16.1
# (e.g. into ~/.cargo/bin-pgrx-0.16/) and switching PATH for this
# build. Skipping for now and emitting a clear gap message.
INSTALLED_CARGO_PGRX="$(cargo pgrx --version 2>/dev/null | head -1 || true)"
if [[ "$INSTALLED_CARGO_PGRX" != *"0.16"* ]]; then
  comparator_log "GAP: pgvectorscale $PGVECTORSCALE_VERSION requires cargo-pgrx 0.16.x"
  comparator_log "     local cargo-pgrx is: ${INSTALLED_CARGO_PGRX:-not installed}"
  comparator_log "     install a parallel 0.16 toolchain first:"
  comparator_log "       cargo install --locked --root /var/lib/pgsql/.cargo-pgrx-0.16 cargo-pgrx@0.16.1"
  comparator_log "       PATH=/var/lib/pgsql/.cargo-pgrx-0.16/bin:\$PATH $0"
  exit 2
fi

comparator_log "building pgvectorscale $PGVECTORSCALE_VERSION (Rust + pgrx)"
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

if [[ ! -d pgvectorscale || $FORCE -eq 1 ]]; then
  rm -rf pgvectorscale
  git clone --depth 1 --branch "$PGVECTORSCALE_VERSION" \
    https://github.com/timescale/pgvectorscale.git
fi

cd pgvectorscale/pgvectorscale
cargo pgrx install --release --sudo --pg-config "$PG_CONFIG"

comparator_log "installed. Run: psql -c 'CREATE EXTENSION IF NOT EXISTS vectorscale CASCADE;'"
