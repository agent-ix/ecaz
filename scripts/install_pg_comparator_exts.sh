#!/usr/bin/env bash
# Install third-party PostgreSQL vector-search extensions from source
# alongside ecaz, so we can benchmark them on the same corpus + same
# hardware as ecaz's ec_hnsw / ec_ivf / ec_diskann.
#
# Per AGENTS.md / [[feedback_benches_must_be_repeatable]]:
# every bench-host setup step lives in a checked-in script, not
# operator shell history. Run this once per host (idempotent).
#
# Installs (defaults can be skipped with --skip-<name>):
#   - pgvector              (vector(N) column type, HNSW + IVFFlat indexes)
#   - pgvectorscale         (StreamingDiskANN + SBQ quantization)
#   - vchord (VectorChord)  (RaBitQ-on-IVF, Rust extension)
#   - lantern               (HNSW + USearch backend)
#
# Requires:
#   - postgresql<N>-server-devel (pg_config available)
#   - cargo + cargo-pgrx (for vchord)
#   - cmake + clang + make + git (for pgvectorscale, lantern)
#
# All compiled extensions land in `pg_config --pkglibdir`.

set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/install_pg_comparator_exts.sh [--build-dir DIR] [--skip-pgvector]
    [--skip-pgvectorscale] [--skip-vchord] [--skip-lantern]
    [--pg-config PATH]

Options:
  --build-dir DIR    Where to clone+build (default: /var/lib/pgsql/build/exts).
                     Should be on a volume with > 4 GB free.
  --pg-config PATH   Override pg_config (default: /usr/bin/pg_config).
  --skip-<name>      Skip that extension. Use the upstream name.

Environment:
  PGVECTOR_VERSION       (default: master -- needed for PG18 ABI)
  PGVECTORSCALE_VERSION  (default: 0.4.0)
  VCHORD_VERSION         (default: main)
  LANTERN_VERSION        (default: v0.5.4)
EOF
}

BUILD_DIR="/var/lib/pgsql/build/exts"
PG_CONFIG="/usr/bin/pg_config"
SKIP_PGVECTOR=0
SKIP_PGVECTORSCALE=0
SKIP_VCHORD=0
SKIP_LANTERN=0

PGVECTOR_VERSION="${PGVECTOR_VERSION:-master}"
PGVECTORSCALE_VERSION="${PGVECTORSCALE_VERSION:-0.4.0}"
VCHORD_VERSION="${VCHORD_VERSION:-main}"
LANTERN_VERSION="${LANTERN_VERSION:-v0.5.4}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --build-dir) BUILD_DIR="$2"; shift 2 ;;
    --pg-config) PG_CONFIG="$2"; shift 2 ;;
    --skip-pgvector) SKIP_PGVECTOR=1; shift ;;
    --skip-pgvectorscale) SKIP_PGVECTORSCALE=1; shift ;;
    --skip-vchord) SKIP_VCHORD=1; shift ;;
    --skip-lantern) SKIP_LANTERN=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

log() { echo "[install-comparators] $(date '+%H:%M:%S') $*"; }

log "build dir: $BUILD_DIR"
log "pg_config: $PG_CONFIG"
"$PG_CONFIG" --version

sudo install -d -o "$(whoami)" -g "$(whoami)" "$BUILD_DIR"
cd "$BUILD_DIR"

PG_PKGLIBDIR="$($PG_CONFIG --pkglibdir)"
PG_SHAREDIR="$($PG_CONFIG --sharedir)"

extension_installed() {
  ls "$PG_SHAREDIR/extension/$1.control" >/dev/null 2>&1
}

# --- pgvector ---
if [[ $SKIP_PGVECTOR -eq 0 ]]; then
  if extension_installed vector; then
    log "pgvector already installed; skipping"
  else
    log "building pgvector $PGVECTOR_VERSION"
    if [[ ! -d "$BUILD_DIR/pgvector" ]]; then
      git clone --depth 1 --branch "$PGVECTOR_VERSION" \
        https://github.com/pgvector/pgvector.git
    fi
    (
      cd pgvector
      make clean >/dev/null 2>&1 || true
      PG_CONFIG="$PG_CONFIG" make -j"$(nproc)"
      sudo PG_CONFIG="$PG_CONFIG" make install
    )
    log "pgvector installed"
  fi
fi

# --- pgvectorscale ---
if [[ $SKIP_PGVECTORSCALE -eq 0 ]]; then
  if extension_installed vectorscale; then
    log "pgvectorscale already installed; skipping"
  else
    log "building pgvectorscale $PGVECTORSCALE_VERSION (Rust + pgrx)"
    if [[ ! -d "$BUILD_DIR/pgvectorscale" ]]; then
      git clone --depth 1 --branch "$PGVECTORSCALE_VERSION" \
        https://github.com/timescale/pgvectorscale.git
    fi
    (
      cd pgvectorscale/pgvectorscale
      # pgvectorscale requires pgrx; same toolchain as ecaz.
      # Build is heavy (~5-10 min) but only needed once per host.
      cargo pgrx install --release --sudo --pg-config "$PG_CONFIG"
    )
    log "pgvectorscale installed"
  fi
fi

# --- vchord (VectorChord) ---
if [[ $SKIP_VCHORD -eq 0 ]]; then
  if extension_installed vchord; then
    log "vchord already installed; skipping"
  else
    log "building vchord (VectorChord) $VCHORD_VERSION (Rust + pgrx)"
    if [[ ! -d "$BUILD_DIR/vectorchord" ]]; then
      git clone --depth 1 --branch "$VCHORD_VERSION" \
        https://github.com/tensorchord/VectorChord.git vectorchord
    fi
    (
      cd vectorchord
      cargo pgrx install --release --sudo --pg-config "$PG_CONFIG"
    )
    log "vchord installed"
  fi
fi

# --- lantern ---
if [[ $SKIP_LANTERN -eq 0 ]]; then
  if extension_installed lantern; then
    log "lantern already installed; skipping"
  else
    log "building lantern $LANTERN_VERSION"
    if [[ ! -d "$BUILD_DIR/lantern" ]]; then
      git clone --depth 1 --branch "$LANTERN_VERSION" --recurse-submodules \
        https://github.com/lanterndata/lantern.git
    fi
    (
      cd lantern
      mkdir -p build && cd build
      cmake -DCMAKE_BUILD_TYPE=Release \
            -DPG_CONFIG="$PG_CONFIG" \
            ..
      make -j"$(nproc)"
      sudo make install
    )
    log "lantern installed"
  fi
fi

log "installed extension controls:"
ls "$PG_SHAREDIR/extension/" | grep -E "^(vector|vectorscale|vchord|lantern)\." || echo "  (none of the comparator extensions installed)"

log "done. Reload postgres if any extension just installed a new .so:"
log "  sudo systemctl restart postgresql"
