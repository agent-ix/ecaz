#!/usr/bin/env bash
# Install pgvectorscale from upstream prebuilt zip (no source build).
# Timescale ships per-PG-version + per-arch zips with .so + .sql + .control
# that we just unzip into the local pg_config dirs. Saves the entire
# Rust + pgrx + cargo dance.
#
# Falls back to source build only if --from-source is passed.
set -euo pipefail

COMPARATOR_NAME="pgvectorscale"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/_common.sh"

PGVECTORSCALE_VERSION="${PGVECTORSCALE_VERSION:-0.9.0}"
PG_MAJOR="${PG_MAJOR:-18}"
PG_CONFIG="${PG_CONFIG:-$PG_CONFIG_DEFAULT}"
BUILD_DIR="${BUILD_DIR:-$COMPARATORS_BUILD_DIR_DEFAULT}"
FROM_SOURCE=0
FORCE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --pg-config) PG_CONFIG="$2"; shift 2 ;;
    --version) PGVECTORSCALE_VERSION="$2"; shift 2 ;;
    --pg-major) PG_MAJOR="$2"; shift 2 ;;
    --build-dir) BUILD_DIR="$2"; shift 2 ;;
    --from-source) FROM_SOURCE=1; shift ;;
    --force) FORCE=1; shift ;;
    -h|--help) sed -n '2,$ s/^# \?//p' "$0" | head -15; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

if [[ $FORCE -eq 0 ]] && comparator_extension_installed vectorscale; then
  comparator_log "already installed; pass --force to reinstall"
  exit 0
fi

# Detect arch for the zip name
case "$(uname -m)" in
  aarch64|arm64) ARCH=arm64 ;;
  x86_64|amd64)  ARCH=amd64 ;;
  *) comparator_log "unsupported arch: $(uname -m)"; exit 1 ;;
esac

ZIP_NAME="pgvectorscale-${PGVECTORSCALE_VERSION}-pg${PG_MAJOR}-${ARCH}.zip"
ZIP_URL="https://github.com/timescale/pgvectorscale/releases/download/${PGVECTORSCALE_VERSION}/${ZIP_NAME}"

if [[ $FROM_SOURCE -eq 1 ]]; then
  comparator_log "ERROR: source build path not implemented in this script"
  comparator_log "       (requires parallel cargo-pgrx 0.16 since pgvectorscale pins pgrx 0.16;"
  comparator_log "        use the prebuilt path instead -- it's per-PG-version-specific anyway)"
  exit 2
fi

comparator_log "downloading prebuilt $ZIP_NAME"
mkdir -p "$BUILD_DIR/pgvectorscale-prebuilt"
cd "$BUILD_DIR/pgvectorscale-prebuilt"

if [[ ! -f "$ZIP_NAME" || $FORCE -eq 1 ]]; then
  rm -f "$ZIP_NAME"
  curl -L --fail --silent --show-error -o "$ZIP_NAME" "$ZIP_URL"
fi
unzip -o "$ZIP_NAME" >/dev/null

# The zip contains a .deb package (Debian/Ubuntu native format).
# AL2023 has no dpkg, so extract the .deb manually with ar + tar.
DEB_FILE=$(ls pgvectorscale-postgresql-${PG_MAJOR}_*_Linux_${ARCH}.deb 2>/dev/null | head -1)
[[ -z "$DEB_FILE" ]] && { comparator_log "no matching .deb in zip"; ls; exit 1; }

EXTRACT_DIR="extracted"
rm -rf "$EXTRACT_DIR"
mkdir -p "$EXTRACT_DIR"
cd "$EXTRACT_DIR"
ar x "../$DEB_FILE"
# Debian .deb data.tar is typically compressed: .gz / .xz / .zst.
# AL2023 ships tar with zstd support but the compression has to be
# specified explicitly (auto-detect via --auto-compress doesn't pick
# .zst).
if [[ -f data.tar.zst ]]; then
  unzstd -q data.tar.zst && tar xf data.tar
elif [[ -f data.tar.xz ]]; then
  tar xJf data.tar.xz
elif [[ -f data.tar.gz ]]; then
  tar xzf data.tar.gz
elif [[ -f data.tar ]]; then
  tar xf data.tar
else
  comparator_log "no data.tar.* in extracted .deb"; ls; exit 1
fi
cd ..

PG_PKGLIBDIR="$($PG_CONFIG --pkglibdir)"
PG_SHAREDIR="$($PG_CONFIG --sharedir)"

comparator_log "installing into $PG_PKGLIBDIR and $PG_SHAREDIR/extension"
# Debian convention: files land under .../usr/lib/postgresql/<v>/lib/
# and .../usr/share/postgresql/<v>/extension/ — find them anywhere
# under extracted/.
for so in $(find "$EXTRACT_DIR" -name 'vectorscale*.so' -type f); do
  sudo install -m 0755 "$so" "$PG_PKGLIBDIR/"
done
for ctl in $(find "$EXTRACT_DIR" -name 'vectorscale.control' -type f); do
  sudo install -m 0644 "$ctl" "$PG_SHAREDIR/extension/"
done
for sql in $(find "$EXTRACT_DIR" -name 'vectorscale--*.sql' -type f); do
  sudo install -m 0644 "$sql" "$PG_SHAREDIR/extension/"
done

comparator_log "installed pgvectorscale $PGVECTORSCALE_VERSION (prebuilt) for pg$PG_MAJOR-$ARCH"
comparator_log "Run: psql -c 'CREATE EXTENSION IF NOT EXISTS vectorscale CASCADE;'"
