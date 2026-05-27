#!/usr/bin/env bash
# Run an AWS verification pass with teardown on exit and a detached timeout guard.

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"

log_line() {
  local log_file="$1"
  shift
  printf '[%s] %s\n' "$(date -u +'%Y-%m-%dT%H:%M:%SZ')" "$*" | tee -a "$log_file"
}

teardown_once() {
  local aws_dir="$1"
  local artifact_dir="$2"
  local log_file="$3"
  local lock_dir="$4"

  if ! mkdir "$lock_dir" 2>/dev/null; then
    log_line "$log_file" "teardown already claimed by another watchdog/cleanup path"
    return 0
  fi

  log_line "$log_file" "starting teardown"
  make -C "$aws_dir" ARTIFACT_DIR="$artifact_dir" teardown >>"$log_file" 2>&1 || {
    log_line "$log_file" "teardown command failed; run cleanup-residue before the next AWS pass"
    return 1
  }

  make -C "$aws_dir" preflight-state >>"$log_file" 2>&1 || {
    log_line "$log_file" "post-teardown state preflight failed"
    return 1
  }
  log_line "$log_file" "teardown complete and Terraform state is clean"
}

if [[ "${1:-}" == "--watchdog" ]]; then
  main_pid="${2:?usage: run-pass-with-watchdog.sh --watchdog <main-pid> <timeout-seconds> <aws-dir> <artifact-dir> <log-file> <done-file> <lock-dir>}"
  timeout_seconds="${3:?usage: run-pass-with-watchdog.sh --watchdog <main-pid> <timeout-seconds> <aws-dir> <artifact-dir> <log-file> <done-file> <lock-dir>}"
  aws_dir="${4:?usage: run-pass-with-watchdog.sh --watchdog <main-pid> <timeout-seconds> <aws-dir> <artifact-dir> <log-file> <done-file> <lock-dir>}"
  artifact_dir="${5:?usage: run-pass-with-watchdog.sh --watchdog <main-pid> <timeout-seconds> <aws-dir> <artifact-dir> <log-file> <done-file> <lock-dir>}"
  log_file="${6:?usage: run-pass-with-watchdog.sh --watchdog <main-pid> <timeout-seconds> <aws-dir> <artifact-dir> <log-file> <done-file> <lock-dir>}"
  done_file="${7:?usage: run-pass-with-watchdog.sh --watchdog <main-pid> <timeout-seconds> <aws-dir> <artifact-dir> <log-file> <done-file> <lock-dir>}"
  lock_dir="${8:?usage: run-pass-with-watchdog.sh --watchdog <main-pid> <timeout-seconds> <aws-dir> <artifact-dir> <log-file> <done-file> <lock-dir>}"

  sleep "$timeout_seconds"
  if [[ -f "$done_file" ]]; then
    exit 0
  fi

  log_line "$log_file" "timeout reached after ${timeout_seconds}s; tearing down AWS resources"
  teardown_once "$aws_dir" "$artifact_dir" "$log_file" "$lock_dir" || true
  kill -TERM "$main_pid" 2>/dev/null || true
  exit 0
fi

target="${1:?usage: run-pass-with-watchdog.sh <make-target> <artifact-dir>}"
artifact_dir="${2:?usage: run-pass-with-watchdog.sh <make-target> <artifact-dir>}"
aws_dir="${SPIRE_AWS_DIR:-$repo_root/infra/spire-aws}"

case "$target" in
  pass-correctness-body|pass-representative-body|pass-representative-performance-body)
    "$repo_root/scripts/spire-aws/confirm-provision.sh"
    ;;
esac

case "$target" in
  pass-correctness-body)
    default_timeout=7200
    ;;
  pass-representative-body)
    default_timeout=14400
    ;;
  pass-representative-performance-body)
    default_timeout=14400
    ;;
  *)
    default_timeout=7200
    ;;
esac

timeout_seconds="${SPIRE_AWS_PASS_TIMEOUT_SECONDS:-$default_timeout}"
if [[ ! "$timeout_seconds" =~ ^[0-9]+$ ]] || ((timeout_seconds <= 0)); then
  printf 'ERROR: SPIRE_AWS_PASS_TIMEOUT_SECONDS must be a positive integer, got: %s\n' "$timeout_seconds" >&2
  exit 2
fi

mkdir -p "$artifact_dir"
log_file="$artifact_dir/aws-pass-watchdog.log"
done_file="$artifact_dir/.aws-pass-watchdog.done"
lock_dir="$artifact_dir/.aws-pass-teardown.lock"
rm -f "$done_file"
rm -rf "$lock_dir"

log_line "$log_file" "starting $target with timeout=${timeout_seconds}s"

watchdog_pid=""
if command -v setsid >/dev/null 2>&1; then
  setsid bash "$0" --watchdog "$$" "$timeout_seconds" "$aws_dir" "$artifact_dir" "$log_file" "$done_file" "$lock_dir" >/dev/null 2>&1 &
else
  nohup bash "$0" --watchdog "$$" "$timeout_seconds" "$aws_dir" "$artifact_dir" "$log_file" "$done_file" "$lock_dir" >/dev/null 2>&1 &
fi
watchdog_pid="$!"

cleanup() {
  local rc=$?
  trap - EXIT INT TERM HUP
  touch "$done_file"
  if [[ -n "$watchdog_pid" ]]; then
    kill "$watchdog_pid" 2>/dev/null || true
  fi
  teardown_once "$aws_dir" "$artifact_dir" "$log_file" "$lock_dir" || rc=$?
  log_line "$log_file" "$target exiting with status $rc"
  exit "$rc"
}
trap cleanup EXIT INT TERM HUP

make -C "$aws_dir" ARTIFACT_DIR="$artifact_dir" "$target"
