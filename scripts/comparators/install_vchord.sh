#!/usr/bin/env bash
# Install VectorChord (vchord) from upstream prebuilt zip.
# Tensorchord ships per-PG-version + per-arch zips containing .so +
# .sql + .control, ready to drop into the local pg_config dirs.
# This avoids the Rust + pgrx + gcc14 build dance from source.
#
# VectorChord is the most relevant comparator for ecaz's RaBitQ-on-IVF
# work (they ship their own RaBitQ implementation).
set -euo pipefail

COMPARATOR_NAME="vchord"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/_common.sh"

VCHORD_VERSION="${VCHORD_VERSION:-1.1.1}"
PG_MAJOR="${PG_MAJOR:-18}"
PG_CONFIG="${PG_CONFIG:-$PG_CONFIG_DEFAULT}"
BUILD_DIR="${BUILD_DIR:-$COMPARATORS_BUILD_DIR_DEFAULT}"
FORCE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --pg-config) PG_CONFIG="$2"; shift 2 ;;
    --version) VCHORD_VERSION="$2"; shift 2 ;;
    --pg-major) PG_MAJOR="$2"; shift 2 ;;
    --build-dir) BUILD_DIR="$2"; shift 2 ;;
    --force) FORCE=1; shift ;;
    -h|--help) sed -n '2,$ s/^# \?//p' "$0" | head -15; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

if [[ $FORCE -eq 0 ]] && comparator_extension_installed vchord; then
  comparator_log "already installed; pass --force to reinstall"
  exit 0
fi

# Detect arch
case "$(uname -m)" in
  aarch64|arm64) ARCH=aarch64-linux-gnu ;;
  x86_64|amd64)  ARCH=x86_64-linux-gnu ;;
  *) comparator_log "unsupported arch: $(uname -m)"; exit 1 ;;
esac

ZIP_NAME="postgresql-${PG_MAJOR}-vchord_${VCHORD_VERSION}_${ARCH}.zip"
ZIP_URL="https://github.com/tensorchord/VectorChord/releases/download/${VCHORD_VERSION}/${ZIP_NAME}"

comparator_log "downloading prebuilt $ZIP_NAME"
mkdir -p "$BUILD_DIR/vchord-prebuilt"
cd "$BUILD_DIR/vchord-prebuilt"

if [[ ! -f "$ZIP_NAME" || $FORCE -eq 1 ]]; then
  rm -f "$ZIP_NAME"
  curl -L --fail --silent --show-error -o "$ZIP_NAME" "$ZIP_URL"
fi
unzip -o "$ZIP_NAME" >/dev/null

PG_PKGLIBDIR="$($PG_CONFIG --pkglibdir)"
PG_SHAREDIR="$($PG_CONFIG --sharedir)"

# The zip layout may be either flat files or a .deb. Handle both.
SO_PATH=$(find . -name 'vchord.so' -type f | head -1)
if [[ -z "$SO_PATH" ]]; then
  # No .so directly — must contain a .deb
  DEB_FILE=$(find . -maxdepth 1 -name '*.deb' | head -1)
  [[ -z "$DEB_FILE" ]] && { comparator_log "no vchord.so and no .deb in zip"; ls; exit 1; }
  comparator_log "zip contains .deb; extracting via ar + tar"
  rm -rf extracted && mkdir extracted && (
    cd extracted
    ar x "../$DEB_FILE"
    if [[ -f data.tar.zst ]]; then unzstd -q data.tar.zst && tar xf data.tar
    elif [[ -f data.tar.xz ]]; then tar xJf data.tar.xz
    elif [[ -f data.tar.gz ]]; then tar xzf data.tar.gz
    elif [[ -f data.tar ]]; then tar xf data.tar
    else echo "no data.tar.* in extracted .deb"; ls; exit 1
    fi
  )
  SO_PATH=$(find extracted -name 'vchord.so' -type f | head -1)
fi
[[ -z "$SO_PATH" ]] && { comparator_log "no vchord.so found anywhere"; exit 1; }

comparator_log "installing into $PG_PKGLIBDIR and $PG_SHAREDIR/extension"
sudo install -m 0755 "$SO_PATH" "$PG_PKGLIBDIR/"
for ctl in $(find . -name 'vchord.control' -type f); do
  sudo install -m 0644 "$ctl" "$PG_SHAREDIR/extension/"
done
for sql in $(find . -name 'vchord--*.sql' -type f); do
  sudo install -m 0644 "$sql" "$PG_SHAREDIR/extension/"
done

comparator_log "installed vchord $VCHORD_VERSION (prebuilt) for pg$PG_MAJOR-$ARCH"
comparator_log "Run: psql -c 'CREATE EXTENSION IF NOT EXISTS vchord CASCADE;'"
