# Task 131 Review Request: Phase 3 Threshold Profile Multi-Instance Smoke

## Scope

This packet validates the Phase 3 scan-time diagnostic path on the local four-instance PG18 SPIRE fixture after the reviewer directive to stop expanding the Phase 0/1 heap-pruning work.

It does not add more heap-side matrix evidence and does not claim a streaming top-k latency win. The purpose is narrower: confirm that the candidate-derived global threshold profile is visible from a real coordinator-to-remote production read run, and record whether the selected remote lists expose sound bounds that a future early-stop rule could use.

## Evidence

Artifacts live under `reviews/task-131/020-phase3-threshold-profile-mi-smoke/artifacts/`.

Command:

```sh
ECAZ_BIN=/home/peter/dev/ecaz/target/debug/ecaz \
  scripts/run_spire_phase13e_static_remote_placement_pg18.sh \
  --artifact-dir reviews/task-131/020-phase3-threshold-profile-mi-smoke/artifacts \
  --run-id task131-threshold-mi-020a \
  --fixture-rows 12 \
  --bench-top-k 6 \
  --bench-queries-limit 1 \
  --bench-sweep 3
```

Harness result from `artifacts/phase13e-static-remote-placement.log`:

```text
placement_summary=2:1,3:1,4:1
profile_summary=ready|3|3|3|3|6
bench_suite_summary=passed|reviews/task-131/020-phase3-threshold-profile-mi-smoke/artifacts/bench-suite/phase13e-local-spire-pipeline-suite.json|reviews/task-131/020-phase3-threshold-profile-mi-smoke/artifacts/bench-suite/suite-manifest.json|reviews/task-131/020-phase3-threshold-profile-mi-smoke/artifacts/bench-suite/results.jsonl
production_timeline_summary=3|3|1566|26|0|13|13|0
degraded_profile_summary=degraded_ready|3|2|2|2|1|0|0|6|none
SPIRE Phase 13e static remote placement PG18 fixture passed
```

Suite result rows from `artifacts/bench-suite/results.jsonl` show the production read stayed healthy:

```text
nprobe=3 queries=1 recall@k=1.0000 latency_p50=57.686 ms
status=ready selected_pid_sum=3 remote_pid_sum=3 returned_sum=6 strict_fail_sum=0
```

The new candidate-derived threshold profile rows are present for all three remotes:

```text
node_id=2 threshold_score_count=1 threshold_score_min=-0.627586 sound_bound_available_sum=0 sound_bound_missing_sum=1 threshold_block_available_sum=0 threshold_block_skipped_sum=0 threshold_row_available_sum=0 threshold_row_skipped_sum=0
node_id=3 threshold_score_count=1 threshold_score_min=-0.627586 sound_bound_available_sum=0 sound_bound_missing_sum=1 threshold_block_available_sum=0 threshold_block_skipped_sum=0 threshold_row_available_sum=0 threshold_row_skipped_sum=0
node_id=4 threshold_score_count=1 threshold_score_min=-0.627586 sound_bound_available_sum=0 sound_bound_missing_sum=1 threshold_block_available_sum=0 threshold_block_skipped_sum=0 threshold_row_available_sum=0 threshold_row_skipped_sum=0
```

## Interpretation

This confirms the diagnostic plumbing works in the intended multi-instance shape: the coordinator derives a compact-score kth threshold and fans it out to each selected remote for scan-time profiling.

The fixture also reports `sound_bound_available_sum=0` and `sound_bound_missing_sum=1` on every selected remote. That means this particular static fixture cannot prove any selected list or block is safely skippable under the threshold. It is useful as a smoke test of the reporting path, not as evidence that early stop is possible or beneficial.

## Next Work

The next implementation slice should stay on the reviewer-directed Phase 3 path: run the threshold profile against a surface with real leaf/block summary bounds, then use those facts to decide whether to add the first gated scan-time skip rule. The dormant heap-side merge-before-heap path should not receive additional matrix work.
