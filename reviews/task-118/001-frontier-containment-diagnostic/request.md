# Review Request: HNSW Frontier Containment Diagnostic

Task: `plan/tasks/118-hnsw-quantized-recall-attribution.md`
Code commit: `a2b1654e0fea655dc28e6f4ddb740529786139d1`

## Summary

This first Task 118 slice adds a `pg_test` HNSW frontier-containment diagnostic
without changing production scan behavior.

- Adds `debug_gettuple_frontier_containment_report`, which captures the
  visible HNSW frontier immediately after `amrescan` and before any final
  `amgettuple` output is consumed.
- Exposes SQL-visible
  `ec_hnsw_graph_scan_recall_frontier_containment(...)` for external recall
  fixtures.
- Reports requested `ef_search`, visited/frontier/emitted counts, heap/source
  rerank counts, quantized rerank counts, truth top-10/top-100 containment,
  frontier row ids, frontier approximate scores, exact f32 scores, approximate
  ranks, exact ranks, and final emitted row ids.

## Validation

- `artifacts/cargo-check-pg18-pgtest.log`
  - `cargo check --no-default-features --features pg18,pg_test`
  - Passes.

## Notes

This is plumbing for Phase 1 and Phase 2 evidence. It does not yet add the
suite-runner step or the 10k/50k/100k measurement matrix required for final
Task 118 closeout.
