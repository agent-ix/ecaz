#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "usage: $0 <expected-passed-count> <cargo-test-args...>" >&2
  exit 2
fi

expected_count="$1"
shift

output_file="$(mktemp)"
trap 'rm -f "$output_file"' EXIT

set +e
cargo test "$@" 2>&1 | tee "$output_file"
cargo_status="${PIPESTATUS[0]}"
set -e

if [[ "$cargo_status" -ne 0 ]]; then
  exit "$cargo_status"
fi

summary="$(
  sed -n 's/^test result: ok\. \([0-9][0-9]*\) passed;.*/\1/p' "$output_file" |
    tail -n 1
)"
if [[ -z "$summary" ]]; then
  echo "counted cargo test: missing successful test-result summary" >&2
  exit 1
fi
if [[ "$summary" -ne "$expected_count" ]]; then
  echo \
    "counted cargo test: expected ${expected_count} passed tests, observed ${summary}" \
    >&2
  exit 1
fi

echo "counted cargo test: observed expected ${expected_count} passed tests"
