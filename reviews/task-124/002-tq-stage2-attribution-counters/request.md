# Review Request: Task 124 TQ Stage-2 Attribution Counters

## Summary

This continues Task 124 after the initial engine slice. It adds benchmark-visible attribution counters for the TurboQuant stage-2 pipeline, without changing the scan result contract.

New `IvfExplainCounters` fields:

- `TQ Stage2 Candidate Rows`
- `TQ Stage2 Rows Scored`
- `TQ Stage2 Rows Retained`
- `TQ Stage2 Payload Bytes Scored`
- `TQ Stage2 Final Exact Rows`
- `TQ Stage2 Final Source Bytes Read`

These are exposed through the existing IVF EXPLAIN/profile property surface used by `ecaz bench suite`.

## Code Changes

- `rerank_probe_candidates_source_side` and `rerank_probe_candidates_index_side` now return local pass stats while preserving the existing generic rerank counters.
- The Task 124 TQ branch records stage-2 candidate/scored/retained rows and compact payload bytes after the index-side TQ pass.
- The final exact/source f32 pass records final exact row count and final source bytes read.
- The PG debug snapshot exposes the new counters for focused tests.
- The existing Task 124 PG test now asserts both generic counters and dedicated stage-2/final attribution counters.

## Validation

- `cargo test -p ecaz am::common::explain`
  - `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 2214 filtered out`
- `cargo test -p ecaz am::ec_ivf::scan`
  - `test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 2194 filtered out`
- `cargo pgrx test pg18 test_ec_ivf_tq_stage2_final_exact_width_bounds_heap_reads`
  - `test tests::pg_test_ec_ivf_tq_stage2_final_exact_width_bounds_heap_reads ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2222 filtered out`

Logs and manifest are under `artifacts/`.

## Remaining Task 124 Work

This is still not a closeout. The next required step is the 10k / 50k / 100k `ecaz bench suite` A/B matrix with width 25, comparing the in-engine TQ stage-2 path against the current RaBitQ + f32 product baseline and source/f32 context.
