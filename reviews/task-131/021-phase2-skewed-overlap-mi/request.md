# Task 131 Review Request: Phase 2 Skewed Overlap Smoke

## Scope

This packet records skewed four-instance PG18 evidence for Task 131 Phase 2 candidate-to-heap streaming.

No new heap-side Phase 0/1 work is included. No code changes are included in this packet; current production transport already uses the default streaming branch when `ec_spire.remote_search_global_pre_heap_merge` is off. This run verifies that behavior under a deliberately slow candidate worker.

## Evidence

Artifacts live under `reviews/task-131/021-phase2-skewed-overlap-mi/artifacts/`.

Command:

```sh
ECAZ_BIN=/home/peter/dev/ecaz/target/debug/ecaz \
  scripts/run_spire_phase13e_static_remote_placement_pg18.sh \
  --artifact-dir reviews/task-131/021-phase2-skewed-overlap-mi/artifacts \
  --run-id task131-skew-overlap-021a \
  --fixture-rows 12 \
  --bench-top-k 6 \
  --bench-queries-limit 1 \
  --bench-sweep 3 \
  --slow-candidate-node2-ms 250
```

Harness result from `artifacts/phase13e-static-remote-placement.log`:

```text
placement_summary=2:1,3:1,4:1
profile_summary=ready|3|3|3|3|6
bench_suite_summary=passed|reviews/task-131/021-phase2-skewed-overlap-mi/artifacts/bench-suite/phase13e-local-spire-pipeline-suite.json|reviews/task-131/021-phase2-skewed-overlap-mi/artifacts/bench-suite/suite-manifest.json|reviews/task-131/021-phase2-skewed-overlap-mi/artifacts/bench-suite/results.jsonl
production_timeline_summary=3|3|643|26|0|266|13|1
degraded_profile_summary=degraded_ready|3|2|2|2|1|0|0|6|none
SPIRE Phase 13e static remote placement PG18 fixture passed
```

Suite row from `artifacts/bench-suite/results.jsonl`:

```text
nprobe=3 queries=1 recall@k=1.0000 latency_p50=312.077 ms
heap_started_before_all_candidates_done=1 fast_heap_before_slowest_heap=1
heap_start_minus_candidate_done_p50=-252 ms fast_heap_lead_p50=252 ms
```

Per-node timeline:

```text
node_id=2 candidate_receive 0ms -> 266ms
node_id=3 candidate_receive 0ms -> 13ms
node_id=4 candidate_receive 0ms -> 13ms
node_id=3 heap_receive 13ms -> 26ms
node_id=4 heap_receive 13ms -> 26ms
node_id=2 heap_receive 266ms -> 643ms
```

## Interpretation

This is the concrete Phase 2 signal missing from the earlier normal-fixture smoke: heap receive for fast workers starts while the slow worker is still in candidate receive. The overlap report captures that as `heap_started_before_all_candidates_done=1` and a negative `heap_start_minus_candidate_done_p50`.

Strict and degraded harness checks both completed in this fixture, and recall for the sampled production read remained `1.0000`.

## Limits

This is not a closeout packet. It is a narrow skewed-fixture smoke proving the barrier is absent in the current default production read path. Task 131 still needs the Phase 3 streaming threshold feedback prototype or a measured rejection of it, plus final promote/iterate/shelve decisions.
