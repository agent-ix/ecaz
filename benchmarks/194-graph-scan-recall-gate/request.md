# Review Request: Graph Scan Recall Gate

Commit: `d372b7c`

## Summary

- add an in-tree A4 recall harness that measures the live graph-first scan path against brute-force fp32 top-k truth
- leave behind a persistent ordered-result regression test for graph-first scan output ordering
- record the corrected A4 outcome on `main`: Recall@10 fails badly, with `1.7%` at `m=8, ef=128`

## What changed

- added a pg-test/debug helper that returns graph-first emitted heap tids together with their operator-facing scores
- added `test_tqhnsw_graph_first_scan_emits_distance_sorted_scores()` so A4 leaves behind durable ordered-result coverage
- added an A4-only recall harness and SQL-callable report/probe surfaces in `src/lib.rs`:
  - `tests.tqhnsw_graph_scan_recall_gate_report()`
  - `tests.tqhnsw_graph_scan_recall_probe(m, ef_search, query_index)`
- measured the required A4 configs over a built `10k x 1536-dim x 4-bit` synthetic corpus and updated planning/status docs with the result:
  - `(m=8, ef=40)`: `1.1%`
  - `(m=8, ef=128)`: `1.7%` (`FAIL`, gate requires `>= 89%`)
  - `(m=8, ef=200)`: `2.4%`
  - `(m=16, ef=200)`: `3.5%`

## Why

- A4 requires evidence from the live graph-first scan runtime, not planner-selected SQL behavior.
- The first harness draft used `UNLOGGED` tables and produced zero emitted tuples; that was corrected before accepting any recall result.
- With the corrected regular-table harness, the graph path emits thousands of tuples and prefills the first result, but its top-10 still diverges sharply from both brute-force fp32 truth and the exact `tqvector` ordering on the same table.
- That makes the current A4 failure evidence point at graph traversal/runtime behavior rather than a pure scorer-vs-fp32 gap.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether the new ordered-result regression test is the right durable A4 coverage to keep on `main`
- whether the report/probe surfaces cleanly demonstrate that the corrected recall failure is in graph traversal/runtime behavior
- whether the A4 plan/status updates accurately describe the gate as failing and blocking downstream work
