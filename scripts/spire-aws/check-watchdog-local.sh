#!/usr/bin/env bash
# Local self-check that the AWS pass watchdog honors teardown policy and does
# not leave its timeout sleep running after the main pass exits.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$REPO_ROOT"

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/spire-watchdog-local.XXXXXX")"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

aws_dir="$work_dir/aws"
artifact_dir="$work_dir/artifacts"
timeout_seconds=35791
mkdir -p "$aws_dir" "$artifact_dir"

cat > "$aws_dir/Makefile" <<'MAKE'
.PHONY: pass-representative-performance-body pass-correctness-body teardown preflight-state

pass-representative-performance-body:
	@printf 'body ran\n'

pass-correctness-body:
	@printf 'body failed\n'
	@exit 42

teardown:
	@printf 'teardown ran\n'

preflight-state:
	@printf 'state clean\n'
MAKE

SPIRE_AWS_DIR="$aws_dir" \
SPIRE_AWS_CONFIRM_PROVISION=yes \
SPIRE_AWS_TEARDOWN_ON_EXIT=always \
SPIRE_AWS_PASS_TIMEOUT_SECONDS="$timeout_seconds" \
scripts/spire-aws/run-pass-with-watchdog.sh \
  pass-representative-performance-body \
  "$artifact_dir"

if ! grep -q 'teardown complete and Terraform state is clean' "$artifact_dir/aws-pass-watchdog.log"; then
  printf 'ERROR: watchdog wrapper did not run teardown\n' >&2
  cat "$artifact_dir/aws-pass-watchdog.log" >&2
  exit 1
fi

if ! [[ -f "$artifact_dir/.aws-pass-watchdog.done" ]]; then
  printf 'ERROR: watchdog done marker was not written\n' >&2
  exit 1
fi

if pgrep -af "run-pass-with-watchdog.sh --watchdog .*${artifact_dir}" >/dev/null 2>&1; then
  printf 'ERROR: watchdog process still running for %s\n' "$artifact_dir" >&2
  pgrep -af "run-pass-with-watchdog.sh --watchdog .*${artifact_dir}" >&2 || true
  exit 1
fi

if pgrep -af "sleep ${timeout_seconds}" >/dev/null 2>&1; then
  printf 'ERROR: watchdog sleep still running for timeout %s\n' "$timeout_seconds" >&2
  pgrep -af "sleep ${timeout_seconds}" >&2 || true
  exit 1
fi

failure_artifact_dir="$work_dir/failure-artifacts"
mkdir -p "$failure_artifact_dir"
set +e
SPIRE_AWS_DIR="$aws_dir" \
SPIRE_AWS_CONFIRM_PROVISION=yes \
SPIRE_AWS_PASS_TIMEOUT_SECONDS="$timeout_seconds" \
scripts/spire-aws/run-pass-with-watchdog.sh \
  pass-correctness-body \
  "$failure_artifact_dir"
failure_status=$?
set -e

if [[ "$failure_status" -ne 2 ]]; then
  printf 'ERROR: expected make failure status 2, got %s\n' "$failure_status" >&2
  cat "$failure_artifact_dir/aws-pass-watchdog.log" >&2
  exit 1
fi

if ! grep -q 'preserving AWS resources for in-place diagnosis' "$failure_artifact_dir/aws-pass-watchdog.log"; then
  printf 'ERROR: watchdog wrapper did not preserve resources on failure\n' >&2
  cat "$failure_artifact_dir/aws-pass-watchdog.log" >&2
  exit 1
fi

if grep -q 'starting teardown' "$failure_artifact_dir/aws-pass-watchdog.log"; then
  printf 'ERROR: watchdog wrapper unexpectedly ran teardown on failure\n' >&2
  cat "$failure_artifact_dir/aws-pass-watchdog.log" >&2
  exit 1
fi

cat "$artifact_dir/aws-pass-watchdog.log"
cat "$failure_artifact_dir/aws-pass-watchdog.log"
printf 'SPIRE AWS watchdog local self-check passed\n'
