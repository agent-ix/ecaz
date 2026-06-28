# Artifact Manifest

Head SHA: `e77b68649` before local attribution-counter edits.
Task bucket: `reviews/task-124/`
Packet path: `reviews/task-124/002-tq-stage2-attribution-counters/`
Timestamp: `2026-06-28T18:59:54Z`
Lane / fixture / storage format / rerank mode: local dev, focused unit + PG18 tests, `ec_ivf` `coarse_rerank`, `rerank_format = 'turboquant'`, `rerank_placement = 'index'`, `stage2_final_rerank_width = 3` for runtime fixture.
Isolation: focused one-index-per-test-table surfaces; no shared benchmark tables.

## Artifacts

- `cargo-test-common-explain.log`
  - Command: `cargo test -p ecaz am::common::explain > reviews/task-124/002-tq-stage2-attribution-counters/artifacts/cargo-test-common-explain.log 2>&1`
  - Key result: `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 2214 filtered out`.
  - Coverage: `IvfExplainCounters` records and renders the new Task 124 EXPLAIN properties.

- `cargo-test-ec-ivf-scan.log`
  - Command: `cargo test -p ecaz am::ec_ivf::scan > reviews/task-124/002-tq-stage2-attribution-counters/artifacts/cargo-test-ec-ivf-scan.log 2>&1`
  - Key result: `test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 2194 filtered out`.

- `cargo-pgrx-test-pg18-tq-stage2-final-counters.log`
  - Command: `cargo pgrx test pg18 test_ec_ivf_tq_stage2_final_exact_width_bounds_heap_reads > reviews/task-124/002-tq-stage2-attribution-counters/artifacts/cargo-pgrx-test-pg18-tq-stage2-final-counters.log 2>&1`
  - Key result: `test tests::pg_test_ec_ivf_tq_stage2_final_exact_width_bounds_heap_reads ... ok`; `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2222 filtered out`.
  - Runtime fixture: one-list `coarse_rerank` index with `rerank_format='turboquant'`, `rerank_placement='index'`, `rerank_width=8`, `stage2_final_rerank_width=3`.
  - Assertion summary: new attribution counters report 8 TQ stage-2 candidate/scored rows, 3 retained rows, TQ stage-2 payload bytes equal the generic payload scorer bytes, 3 final exact rows, and final source bytes equal the generic source bytes read.

## Benchmark Status

No `ecaz bench suite` benchmark evidence is included in this packet. This is a Phase 3 counter/attribution checkpoint only. Task 124 remains open pending the required 10k / 50k / 100k A/B matrix and a promote/iterate/shelve decision.
