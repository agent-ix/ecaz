#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "usage: $0 <expected-passed-count> <cargo-test-args...>" >&2
  exit 2
fi

expected_count="$1"
shift

output_file="$(mktemp)"
summary_file="$(mktemp)"
trap 'rm -f "$output_file" "$summary_file"' EXIT

set +e
cargo test "$@" 2>&1 | tee "$output_file"
cargo_status="${PIPESTATUS[0]}"
set -e

if [[ "$cargo_status" -ne 0 ]]; then
  exit "$cargo_status"
fi

sed -n 's/^test result: ok\. \([0-9][0-9]*\) passed;.*/\1/p' \
  "$output_file" >"$summary_file"
summary_count="$(wc -l <"$summary_file" | tr -d ' ')"
if [[ "$summary_count" -eq 0 ]]; then
  echo "counted cargo test: missing successful test-result summary" >&2
  exit 1
fi
if [[ "$summary_count" -ne 1 ]]; then
  echo \
    "counted cargo test: expected exactly one test-result summary, observed ${summary_count}" \
    >&2
  exit 1
fi
IFS= read -r summary <"$summary_file"
if [[ "$summary" -ne "$expected_count" ]]; then
  echo \
    "counted cargo test: expected ${expected_count} passed tests, observed ${summary}" \
    >&2
  exit 1
fi

echo "counted cargo test: observed expected ${expected_count} passed tests"
