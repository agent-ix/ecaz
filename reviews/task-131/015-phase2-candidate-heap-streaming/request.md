# Task 131 Packet 015: Phase 2 Candidate-To-Heap Streaming

## Scope

This packet implements the Phase 2 enabling change: the production session-reuse path no longer waits for all compact candidate receives before starting heap receive. When a worker's candidate session becomes ready, its heap receive future is launched immediately.

Strict/degraded state application is still conservative:

- final executor state applies all candidate results before heap results;
- local cancellation stops launching additional heap futures;
- strict candidate failure stops launching additional heap futures, while already-started heap futures drain for cleanup/accounting;
- degraded candidate failures continue to skip failed workers and launch heap only for ready sessions;
- the retired global pre-heap merge GUC path remains barriered, because that explicit-subset heap surface is the Phase 1 dead end called out by reviewer feedback and should not shape the streaming path.

The harness now has an opt-in `--slow-candidate-node2-ms` mode that shadows node 2's `ec_spire_remote_search` through conninfo `search_path`, sleeps in compact candidate receive, and asserts that heap starts before the slow candidate finishes.

## Code Under Review

- `7f4c19026a4df7ca22a6a0a796fb5694d121a205` (`task 131 stream heap after candidate readiness`)

## Evidence

Artifacts are under `reviews/task-131/015-phase2-candidate-heap-streaming/artifacts/`.

Local four-instance PG18 harness:

```text
slow_candidate_node2_ms=750
bench_suite_summary=passed|reviews/task-131/015-phase2-candidate-heap-streaming/artifacts/bench-suite/phase13e-local-spire-pipeline-suite.json|reviews/task-131/015-phase2-candidate-heap-streaming/artifacts/bench-suite/suite-manifest.json|reviews/task-131/015-phase2-candidate-heap-streaming/artifacts/bench-suite/results.jsonl
production_timeline_rows=1|2|candidate_receive|0|766|766|6|ready|none;1|3|candidate_receive|0|13|13|3|ready|none;1|4|candidate_receive|0|13|13|3|ready|none;1|2|heap_receive|766|779|13|6|ready|none;1|3|heap_receive|13|26|12|3|ready|none;1|4|heap_receive|13|26|12|3|ready|none;
production_timeline_summary=3|3|779|26|0|766|13|1
SPIRE Phase 13e static remote placement PG18 fixture passed
```

The summary's last three fields show node 2 candidate completion at `766 ms`, earliest heap start at `13 ms`, and `heap_started_before_slow_candidate=1`.

Suite overlap row:

```text
heap_started_before_all_candidates_done=1
heap_start_minus_candidate_done_min=-751 ms
heap_start_minus_candidate_done_p50=-751 ms
heap_start_minus_candidate_done_max=-751 ms
fast_heap_before_slowest_heap=1
fast_heap_lead_p50=752 ms
```

Focused validation:

- `cargo-check-lib.log`: `cargo check --lib` passed.
- `cargo-test-lib-production-executor.log`: `cargo test --lib production_executor` passed, `42 passed; 0 failed`.

## Readout

This packet proves the Phase 2 structural behavior on a skewed local fixture: fast workers can begin and finish heap receive while a slow worker is still producing compact candidates. It does not implement Phase 3 global threshold feedback, and it does not claim a 10k/50k/100k latency win. The next task-131 slice should use this streaming structure to prototype threshold updates and scan-time early stop, gated by the Phase 0 finding that the current smoke has no sound per-list bound (`sound_bound_available_sum=0`).
