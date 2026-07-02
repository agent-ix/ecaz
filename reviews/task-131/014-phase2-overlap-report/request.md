# Task 131 Packet 014: Phase 2 Overlap Report

## Scope

This packet pivots away from the Phase 1 heap-pruning matrix per reviewer feedback and adds a first-class report table for Phase 2 production read overlap:

- `Production read phase overlap` summarizes candidate-vs-heap timing by `nprobe`.
- `heap_start_minus_candidate_done_*` reports the minimum heap start minus the latest candidate completion for each query. Negative values are the evidence shape needed for candidate-phase barrier removal.
- `fast_heap_lead_*` reports the spread between the first and last heap completion, which is useful but weaker evidence than candidate-phase overlap.
- The suite table parser now resets headers between named report sections, fixing a same-width adjacent table failure mode exposed by the new overlap table.

## Code Under Review

- `bef2d1062ed1d9c17cbc215e87a212f2275938f5` (`task 131 report production phase overlap`)

## Evidence

Artifacts are under `reviews/task-131/014-phase2-overlap-report/artifacts/`.

- `bench-suite/spire-pipeline.log` emits the new overlap table.
- `bench-suite/results.jsonl` was regenerated with the fixed suite parser and now keeps overlap rows separate from timeline rows.
- `phase13e-static-remote-placement.log` records the local four-instance PG18 harness run.
- Focused validation:
  - `cargo-test-ecaz-cli-production-read-overlap.log`
  - `cargo-test-ecaz-cli-suite-table-parser.log`
  - `cargo-build-ecaz-cli.log`

Key harness lines:

```text
bench_suite_summary=passed|reviews/task-131/014-phase2-overlap-report/artifacts/bench-suite/phase13e-local-spire-pipeline-suite.json|reviews/task-131/014-phase2-overlap-report/artifacts/bench-suite/suite-manifest.json|reviews/task-131/014-phase2-overlap-report/artifacts/bench-suite/results.jsonl
production_timeline_rows=1|2|candidate_receive|0|13|13|6|ready|none;1|3|candidate_receive|0|13|13|3|ready|none;1|4|candidate_receive|0|13|13|3|ready|none;1|2|heap_receive|13|635|621|6|ready|none;1|3|heap_receive|13|26|12|3|ready|none;1|4|heap_receive|13|26|12|3|ready|none;
production_timeline_summary=3|3|635|26|0
SPIRE Phase 13e static remote placement PG18 fixture passed
```

Key overlap row:

```text
nprobe=3 queries=1 complete_timeline_queries=1 candidate_rows_sum=3 heap_rows_sum=3 heap_started_before_all_candidates_done=0 heap_start_minus_candidate_done_min=0 ms heap_start_minus_candidate_done_p50=0 ms heap_start_minus_candidate_done_max=0 ms
```

## Readout

This checkpoint does not claim streaming top-k is implemented, and it does not claim candidate-phase overlap. It gives the next Phase 2 work a stable local/reportable metric: once heap dispatch starts before all candidate receive phases complete, `heap_start_minus_candidate_done_*` should go negative and `heap_started_before_all_candidates_done` should increment.
