#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACT_DIR="${ARTIFACT_DIR:-$ROOT_DIR/reviews/task-30/987-spire-phase13e-local-gates/artifacts}"
SUITE="${SUITE:-core}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
SKIP_INSTALL=0
INCLUDE_DOCKER_TLS=0
INSTALLED=0

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_phase13e_local_gates_pg18.sh [options]

Options:
  --artifact-dir DIR    Store all gate logs under DIR.
  --include-docker-tls Include the Docker TLS transport gate.
  --run-id ID          Run id recorded in summary output.
  --skip-install       Skip the one-time cargo pgrx install gate.
  --suite SUITE        Gate suite: core, extended, or all. Default: core.
  -h, --help           Show this help.

Core gates cover Phase 13e remote placement, distributed CustomScan read,
coordinator insert/readback, and transport overlap. Extended gates add the
Stage E local fault matrix.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir)
      ARTIFACT_DIR="$2"
      shift 2
      ;;
    --include-docker-tls)
      INCLUDE_DOCKER_TLS=1
      shift
      ;;
    --run-id)
      RUN_ID="$2"
      shift 2
      ;;
    --skip-install)
      SKIP_INSTALL=1
      shift
      ;;
    --suite)
      SUITE="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ "$SUITE" != "core" && "$SUITE" != "extended" && "$SUITE" != "all" ]]; then
  echo "unsupported --suite: $SUITE" >&2
  usage >&2
  exit 2
fi

mkdir -p "$ARTIFACT_DIR"
SUMMARY="$ARTIFACT_DIR/phase13e-local-gates-summary.tsv"
: > "$SUMMARY"
printf "run_id\tgate\tstatus\tartifact_dir\n" >> "$SUMMARY"

next_install_arg=()
prepare_install_arg() {
  next_install_arg=()
  if [[ "$SKIP_INSTALL" == "1" || "$INSTALLED" == "1" ]]; then
    next_install_arg=(--skip-install)
  else
    INSTALLED=1
  fi
}

run_gate() {
  local gate="$1"
  shift
  local gate_dir="$ARTIFACT_DIR/$gate"
  mkdir -p "$gate_dir"
  echo "gate_start=$gate"
  if "$@" --artifact-dir "$gate_dir"; then
    printf "%s\t%s\tpass\t%s\n" "$RUN_ID" "$gate" "$gate_dir" >> "$SUMMARY"
    echo "gate_pass=$gate"
  else
    local status=$?
    printf "%s\t%s\tfail:%s\t%s\n" "$RUN_ID" "$gate" "$status" "$gate_dir" >> "$SUMMARY"
    echo "gate_fail=$gate status=$status" >&2
    exit "$status"
  fi
}

run_core() {
  prepare_install_arg
  run_gate phase13e-static-remote-placement \
    bash "$ROOT_DIR/scripts/run_spire_phase13e_static_remote_placement_pg18.sh" \
    "${next_install_arg[@]}" \
    --coord-port 39600 --remote1-port 39601 --remote2-port 39602 --remote3-port 39603

  prepare_install_arg
  run_gate multicluster-customscan-read \
    bash "$ROOT_DIR/scripts/run_spire_multicluster_customscan_read_pg18.sh" \
    "${next_install_arg[@]}" \
    --coord-port 39610 --remote-port 39611

  prepare_install_arg
  run_gate insert-read-after-customscan-helper \
    bash "$ROOT_DIR/scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh" \
    "${next_install_arg[@]}" \
    --coord-port 39612 --remote-port 39613 --insert-mode helper

  prepare_install_arg
  run_gate insert-read-after-customscan-trigger \
    bash "$ROOT_DIR/scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh" \
    "${next_install_arg[@]}" \
    --coord-port 39614 --remote-port 39615 --insert-mode trigger

  prepare_install_arg
  run_gate transport-overlap \
    bash "$ROOT_DIR/scripts/run_spire_multicluster_transport_overlap_pg18.sh" \
    "${next_install_arg[@]}" \
    --coord-port 39620 --remote-fast-port 39621 --remote-slow-port 39622

  if [[ "$INCLUDE_DOCKER_TLS" == "1" ]]; then
    prepare_install_arg
    run_gate remote-tls-docker \
      bash "$ROOT_DIR/scripts/run_spire_remote_tls_docker_pg18.sh" \
      "${next_install_arg[@]}" \
      --coord-port 39630 --remote-port 39631
  fi
}

run_extended() {
  local case_name

  for case_name in epoch_mismatch version_skew; do
    prepare_install_arg
    run_gate "stage-e-predispatch-$case_name" \
      bash "$ROOT_DIR/scripts/run_spire_multicluster_stage_e_predispatch_fault_pg18.sh" \
      "${next_install_arg[@]}" \
      --case "$case_name" --coord-port 39640 --remote-ready-port 39641
  done

  for case_name in fingerprint_mismatch missing_or_reindexed_remote_index; do
    prepare_install_arg
    run_gate "stage-e-candidate-$case_name" \
      bash "$ROOT_DIR/scripts/run_spire_multicluster_stage_e_candidate_receive_fault_pg18.sh" \
      "${next_install_arg[@]}" \
      --case "$case_name" --coord-port 39642 --remote-ready-port 39643
  done

  prepare_install_arg
  run_gate stage-e-network-partition \
    bash "$ROOT_DIR/scripts/run_spire_multicluster_stage_e_network_partition_pg18.sh" \
    "${next_install_arg[@]}" \
    --coord-port 39644 --remote-ready-port 39645

  for case_name in connection_reset_mid_batch local_cancel local_statement_timeout remote_backend_termination remote_oom remote_statement_timeout; do
    prepare_install_arg
    run_gate "stage-e-transport-$case_name" \
      bash "$ROOT_DIR/scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh" \
      "${next_install_arg[@]}" \
      --case "$case_name" --coord-port 39646 --remote-ready-port 39647
  done

  for case_name in create_index_concurrently_missing_descriptor create_index_concurrently_new_descriptor drop_remote_index_before_fanout drop_remote_index_in_flight reindex_remote_index_before_fanout reindex_remote_index_in_flight; do
    prepare_install_arg
    run_gate "stage-e-lifecycle-$case_name" \
      bash "$ROOT_DIR/scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh" \
      "${next_install_arg[@]}" \
      --case "$case_name" --coord-port 39648 --remote-ready-port 39649
  done
}

case "$SUITE" in
  core)
    run_core
    ;;
  extended)
    run_extended
    ;;
  all)
    run_core
    run_extended
    ;;
esac

echo "phase13e_local_gates_passed suite=$SUITE summary=$SUMMARY"
