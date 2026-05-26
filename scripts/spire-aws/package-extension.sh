#!/usr/bin/env bash
# Build the ecaz PG18 extension bundle expected by bootstrap-node.sh.
# Args:
#   $1  Artifact directory for package outputs and logs
#   $2  Output tarball path, defaulting to $ARTIFACT_DIR/ecaz-latest.tar.gz

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$REPO_ROOT"

ARTIFACT_DIR="${1:?artifact directory required}"
TARBALL_PATH="${2:-$ARTIFACT_DIR/ecaz-latest.tar.gz}"
PG_CONFIG="${PG_CONFIG:-/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config}"
PG_FEATURE="${PG_FEATURE:-pg18}"

PACKAGE_DIR="$ARTIFACT_DIR/ecaz-pgrx-package"
STAGE_DIR="$ARTIFACT_DIR/ecaz-bootstrap-package"
SOURCE_DIR="$ARTIFACT_DIR/ecaz-source-build"
LOG_FILE="$ARTIFACT_DIR/package-extension.log"
SOURCE_TARBALL="$ARTIFACT_DIR/ecaz-source.tar.gz"

mkdir -p "$ARTIFACT_DIR"

if [[ ! -x "$PG_CONFIG" ]]; then
  echo "ERROR: PG_CONFIG does not point to an executable: $PG_CONFIG" >&2
  exit 2
fi

rm -rf "$PACKAGE_DIR" "$STAGE_DIR" "$SOURCE_DIR"
mkdir -p "$PACKAGE_DIR" "$STAGE_DIR/lib" "$STAGE_DIR/extension"

cargo pgrx package \
  --package ecaz \
  --no-default-features \
  --features "$PG_FEATURE" \
  --pg-config "$PG_CONFIG" \
  --out-dir "$PACKAGE_DIR" \
  > "$LOG_FILE" 2>&1

mapfile -t so_files < <(find "$PACKAGE_DIR" -type f -name 'ecaz*.so' | sort)
mapfile -t control_files < <(find "$PACKAGE_DIR" -type f -name 'ecaz.control' | sort)
mapfile -t sql_files < <(find "$PACKAGE_DIR" -type f -name 'ecaz--*.sql' | sort)

if ((${#so_files[@]} != 1)); then
  echo "ERROR: expected exactly one ecaz shared library, found ${#so_files[@]}" >&2
  printf '%s\n' "${so_files[@]}" >&2
  exit 2
fi
if ((${#control_files[@]} != 1)); then
  echo "ERROR: expected exactly one ecaz.control, found ${#control_files[@]}" >&2
  printf '%s\n' "${control_files[@]}" >&2
  exit 2
fi
if ((${#sql_files[@]} == 0)); then
  echo "ERROR: expected at least one ecaz extension SQL file" >&2
  exit 2
fi

cp "${so_files[0]}" "$STAGE_DIR/lib/"
cp "${control_files[0]}" "$STAGE_DIR/extension/"
cp "${sql_files[@]}" "$STAGE_DIR/extension/"
tar -C "$STAGE_DIR" -czf "$TARBALL_PATH" lib extension

mkdir -p "$SOURCE_DIR"
git ls-files -z | tar --null -T - -cf - | tar -C "$SOURCE_DIR" -xf -
(
  cd "$SOURCE_DIR"
  cargo vendor vendor >> .cargo/config.toml 2>> "$LOG_FILE"
)
tar -C "$SOURCE_DIR" -czf "$SOURCE_TARBALL" .

{
  echo "pg_config=$PG_CONFIG"
  echo "pg_feature=$PG_FEATURE"
  echo "package_dir=$PACKAGE_DIR"
  echo "tarball=$TARBALL_PATH"
  echo "source_tarball=$SOURCE_TARBALL"
  echo "shared_library=${so_files[0]}"
  echo "control_file=${control_files[0]}"
  printf 'sql_file=%s\n' "${sql_files[@]}"
} >> "$LOG_FILE"
