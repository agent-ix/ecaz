#!/usr/bin/env bash
# Minimal test runner for scripts/tests/*.py.
#
# This is the "no harness yet" wrapper: it just invokes each Python test
# module directly with the repo root on PYTHONPATH. When a real Python test
# harness lands, this can be replaced with a single pytest/unittest command.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../.." && pwd)"

cd "$REPO_ROOT"

status=0
for test_file in "$HERE"/test_*.py; do
    echo "[scripts/tests] running $test_file"
    if ! python3 "$test_file"; then
        status=1
    fi
done

exit "$status"
