#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG="${ECAZ_DYLINT_SELF_TEST_LOG:-$ROOT/crates/ecaz-lints/target/panic_across_ffi.self-test.log}"

mkdir -p "$(dirname "$LOG")"
set +e
DYLINT_RUSTFLAGS="-D ecaz_panic_across_ffi" bash "$ROOT/scripts/run_dylint.sh" \
  --manifest-path "$ROOT/crates/ecaz-lints/fixtures/panic_across_ffi/Cargo.toml" \
  --no-deps >"$LOG" 2>&1
status=$?
set -e

if [ "$status" -eq 0 ]; then
  echo "expected ecaz_panic_across_ffi fixture to fail" >&2
  cat "$LOG" >&2
  exit 1
fi

if ! grep -Eq "ecaz[-_]panic[-_]across[-_]ffi" "$LOG"; then
  echo "expected self-test log to mention ecaz_panic_across_ffi" >&2
  cat "$LOG" >&2
  exit 1
fi

if ! grep -q "unguarded_callback" "$LOG"; then
  echo "expected self-test log to mention unguarded_callback" >&2
  cat "$LOG" >&2
  exit 1
fi

if grep -q "guarded_by_pgrx_helper" "$LOG"; then
  echo "guarded_by_pgrx_helper should not be reported" >&2
  cat "$LOG" >&2
  exit 1
fi

if grep -q "guarded_by_catch_unwind" "$LOG"; then
  echo "guarded_by_catch_unwind should not be reported" >&2
  cat "$LOG" >&2
  exit 1
fi

echo "dylint self-test passed: $LOG"
