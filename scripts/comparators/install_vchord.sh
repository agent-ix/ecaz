#!/usr/bin/env bash
# Install VectorChord (vchord) — Rust pgrx extension with RaBitQ-on-IVF.
# The most relevant comparator for ecaz's RaBitQ-on-IVF work.
# Requires pgvector to be installed first.
set -euo pipefail

COMPARATOR_NAME="vchord"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/_common.sh"

VCHORD_VERSION="${VCHORD_VERSION:-1.1.1}"
BUILD_DIR="${BUILD_DIR:-$COMPARATORS_BUILD_DIR_DEFAULT}"
PG_CONFIG="${PG_CONFIG:-$PG_CONFIG_DEFAULT}"
FORCE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --build-dir) BUILD_DIR="$2"; shift 2 ;;
    --pg-config) PG_CONFIG="$2"; shift 2 ;;
    --version) VCHORD_VERSION="$2"; shift 2 ;;
    --force) FORCE=1; shift ;;
    -h|--help) sed -n '2,$ s/^# \?//p' "$0" | head -20; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

if ! comparator_extension_installed vector; then
  comparator_log "pgvector must be installed first; run install_pgvector.sh"
  exit 1
fi

if [[ $FORCE -eq 0 ]] && comparator_extension_installed vchord; then
  comparator_log "already installed; pass --force to rebuild"
  exit 0
fi

comparator_log "building VectorChord $VCHORD_VERSION (Rust + pgrx)"
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

if [[ ! -d vectorchord || $FORCE -eq 1 ]]; then
  rm -rf vectorchord
  git clone --depth 1 --branch "$VCHORD_VERSION" \
    https://github.com/tensorchord/VectorChord.git vectorchord
fi

# vchord's simd crate uses aarch64 fp16 intrinsics that require gcc >= 14.
# AL2023's default gcc is 11.5, so install gcc14 if missing and point CC
# at it for the build. gcc14 is a parallel install on AL2023 (does not
# replace the default toolchain).
if ! command -v gcc-14 >/dev/null && ! rpm -q gcc14 >/dev/null 2>&1; then
  comparator_log "installing gcc14 (vchord aarch64_fp16.c requires gcc>=14)"
  sudo dnf install -y gcc14 gcc14-c++
fi
export CC=/usr/bin/gcc-14
export CXX=/usr/bin/g++-14

cd vectorchord
cargo pgrx install --release --sudo --pg-config "$PG_CONFIG"

comparator_log "installed. Run: psql -c 'CREATE EXTENSION IF NOT EXISTS vchord CASCADE;'"
