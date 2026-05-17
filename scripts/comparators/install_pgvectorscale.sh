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

# pgvectorscale 0.9.0 (latest) supports PG17/PG18 but pins pgrx 0.16.x.
# cargo-pgrx requires exact version match with the project's pgrx
# dependency, so we install cargo-pgrx 0.16.1 into a parallel cargo
# root and use it just for this build. ecaz's host-wide cargo-pgrx
# 0.17 is unaffected; they coexist as separate binaries.
CARGO_PGRX_0_16_ROOT="${CARGO_PGRX_0_16_ROOT:-/var/lib/pgsql/.cargo-pgrx-0.16}"
CARGO_PGRX_0_16_BIN="$CARGO_PGRX_0_16_ROOT/bin/cargo-pgrx"
if [[ ! -x "$CARGO_PGRX_0_16_BIN" ]]; then
  comparator_log "installing parallel cargo-pgrx 0.16.1 into $CARGO_PGRX_0_16_ROOT"
  cargo install --locked --root "$CARGO_PGRX_0_16_ROOT" cargo-pgrx@0.16.1
fi

comparator_log "cargo-pgrx 0.16 from $CARGO_PGRX_0_16_ROOT/bin:"
"$CARGO_PGRX_0_16_BIN" --version

comparator_log "building pgvectorscale $PGVECTORSCALE_VERSION (Rust + pgrx 0.16)"
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

if [[ ! -d pgvectorscale || $FORCE -eq 1 ]]; then
  rm -rf pgvectorscale
  git clone --depth 1 --branch "$PGVECTORSCALE_VERSION" \
    https://github.com/timescale/pgvectorscale.git
fi

cd pgvectorscale/pgvectorscale
# Use the parallel cargo-pgrx 0.16 explicitly. PATH manipulation
# scoped to this subshell only.
PATH="$CARGO_PGRX_0_16_ROOT/bin:$PATH" \
  cargo pgrx install --release --sudo --pg-config "$PG_CONFIG"

comparator_log "installed. Run: psql -c 'CREATE EXTENSION IF NOT EXISTS vectorscale CASCADE;'"
