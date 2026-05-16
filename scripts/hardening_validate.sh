#!/usr/bin/env bash
set -euo pipefail

status=0
retired_pattern='rudra|flux|loom|shuttle'

for retired in hardening/rudra hardening/flux hardening/loom hardening/shuttle; do
  if [ -f "$retired/Cargo.toml" ] || [ -f "$retired/src/lib.rs" ]; then
    echo "synthetic hardening lane still present: $retired" >&2
    status=1
  fi
  if [ -n "$(git ls-files "$retired/")" ]; then
    echo "retired synthetic lane has tracked files: $retired" >&2
    status=1
  fi
done

for manifest in hardening/*/Cargo.toml; do
  lane_dir="$(dirname "$manifest")"
  src_file="$lane_dir/src/lib.rs"
  if [ ! -f "$src_file" ]; then
    continue
  fi
  if ! grep -Eq '#\[path = "\.\./\.\./\.\./src/' "$src_file"; then
    echo "hardening lane does not import real src/ code: $lane_dir" >&2
    status=1
  fi
done

if grep -Eq '^(rudra|flux|loom|shuttle):' Makefile; then
  echo "retired synthetic lane target still present in Makefile" >&2
  status=1
fi

for aggregate in hardening-local hardening-nightly-local; do
  if awk -v aggregate="$aggregate" -v retired_pattern="$retired_pattern" '
    $0 ~ "^" aggregate ":" &&
      $0 ~ "(^|[[:space:]:])(" retired_pattern ")([[:space:]]|$)" {
        found = 1
      }
    END { exit found ? 0 : 1 }
  ' Makefile; then
    echo "$aggregate still depends on retired synthetic lanes" >&2
    status=1
  fi
done

if ! awk '
  /^test:/ { in_test = 1; next }
  /^[[:alnum:]_.-]+:/ { in_test = 0 }
  in_test && /(^|[[:space:]])cargo[[:space:]]+test([[:space:]]|$)/ { found = 1 }
  END { exit found ? 0 : 1 }
' Makefile; then
  echo "make test must run cargo test directly" >&2
  status=1
fi

if awk '
  /^test:/ { in_test = 1; next }
  /^[[:alnum:]_.-]+:/ { in_test = 0 }
  in_test && /test-hardening-local/ { found = 1 }
  END { exit found ? 0 : 1 }
' Makefile; then
  echo "make test must not be narrowed to test-hardening-local" >&2
  status=1
fi

if [ ! -f docs/hardening-governance.md ]; then
  echo "missing docs/hardening-governance.md" >&2
  status=1
fi

exit "$status"
