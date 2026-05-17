# SPIRE Active Scan Heap Rerank

## Checkpoint

- Code commit: `2b2bbd97` (`Add SPIRE heap rerank for active scans`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: live active-epoch heap rerank

## Summary

This checkpoint replaces the temporary quantized-score passthrough in live
`ec_spire` scans with heap-backed rerank for the resolved candidate window:

- Active relation-backed scans now resolve the heap relation and executor
  snapshot during `amrescan`.
- The rerank callback fetches each candidate heap row, reads the indexed column,
  and computes inner product against the validated ORDER BY query.
- Heap `ecvector` rows are decoded from their raw float payload.
- Heap `tqvector` rows are decoded through the product quantizer before scoring,
  matching the source-vector path used during SPIRE build collection.
- The populated `ecvector` PG18 test continues to cover active scan behavior and
  relation-backed diagnostics.
- A new populated `tqvector_spire_ip_ops` PG18 test covers the heap-rerank
  `tqvector` decode branch.

PQ-FastScan scorer binding remains deferred until grouped-PQ model metadata is
persisted for SPIRE.

## Changed Files

- `src/am/ec_spire/scan.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_tqvector_populated_build_scans_with_heap_rerank --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `197 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is not a recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- Insert-after-build, delete/vacuum cleanup, PQ-FastScan scorer binding, and
  SQL/admin diagnostics remain open.
